use super::{AiApiError, AiApiResult, parse_json_response};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::time::Duration;
use tokio::time::{Instant, sleep};

const BAILIAN_BEIJING_BASE_URL: &str = "https://dashscope.aliyuncs.com/api/v1";
const BAILIAN_SINGAPORE_BASE_URL: &str = "https://dashscope-intl.aliyuncs.com/api/v1";
const MAX_BAILIAN_SEED: u32 = 2_147_483_647;
pub const BAILIAN_DEFAULT_IMAGE_MODEL: &str = "wan2.7-image-pro";

pub type BailianObject = BTreeMap<String, Value>;

pub fn bailian_image_data_url(mime_type: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> String {
    format!(
        "data:{};base64,{}",
        mime_type.as_ref(),
        STANDARD.encode(bytes)
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BailianRegion {
    Beijing,
    Singapore,
}

impl BailianRegion {
    pub fn base_url(self) -> &'static str {
        match self {
            Self::Beijing => BAILIAN_BEIJING_BASE_URL,
            Self::Singapore => BAILIAN_SINGAPORE_BASE_URL,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BailianConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Option<Duration>,
}

impl BailianConfig {
    pub fn new(api_key: impl Into<String>, region: BailianRegion) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: region.base_url().to_string(),
            timeout: Some(Duration::from_secs(180)),
        }
    }

    pub fn from_env() -> AiApiResult<Self> {
        let api_key =
            env::var("DASHSCOPE_API_KEY").map_err(|_| AiApiError::MissingEnvironmentVariable {
                name: "DASHSCOPE_API_KEY",
            })?;
        let region = match env::var("DASHSCOPE_REGION").ok().as_deref() {
            Some("singapore") | Some("intl") | Some("ap-southeast-1") => BailianRegion::Singapore,
            _ => BailianRegion::Beijing,
        };
        let mut config = Self::new(api_key, region);
        if let Ok(base_url) = env::var("DASHSCOPE_BASE_URL") {
            config.base_url = base_url;
        }
        Ok(config)
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    fn validate(&self) -> AiApiResult<()> {
        if self.api_key.trim().is_empty() {
            return Err(AiApiError::InvalidConfig(
                "Bailian API key cannot be empty".to_string(),
            ));
        }
        if self.base_url.trim().is_empty() {
            return Err(AiApiError::InvalidConfig(
                "Bailian base URL cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BailianClient {
    http: reqwest::Client,
    config: BailianConfig,
}

impl BailianClient {
    pub fn new(config: BailianConfig) -> AiApiResult<Self> {
        config.validate()?;
        let mut builder = reqwest::Client::builder();
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }
        Ok(Self {
            http: builder.build()?,
            config,
        })
    }

    pub fn from_env() -> AiApiResult<Self> {
        Self::new(BailianConfig::from_env()?)
    }

    pub async fn generate_image_sync(
        &self,
        request: &BailianImageGenerationRequest,
    ) -> AiApiResult<BailianImageGenerationResponse> {
        request.validate()?;
        let response = self
            .request(
                Method::POST,
                "/services/aigc/multimodal-generation/generation",
            )
            .json(request)
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn create_image_task(
        &self,
        request: &BailianImageGenerationRequest,
    ) -> AiApiResult<BailianImageGenerationResponse> {
        request.validate()?;
        let response = self
            .request(Method::POST, "/services/aigc/image-generation/generation")
            .header("X-DashScope-Async", "enable")
            .json(request)
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn get_image_task(
        &self,
        task_id: &str,
    ) -> AiApiResult<BailianImageGenerationResponse> {
        if task_id.trim().is_empty() {
            return Err(AiApiError::Validation(
                "Bailian task_id cannot be empty".to_string(),
            ));
        }
        let response = self
            .request(Method::GET, &format!("/tasks/{task_id}"))
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn wait_image_task(
        &self,
        task_id: &str,
        options: BailianWaitOptions,
    ) -> AiApiResult<BailianImageGenerationResponse> {
        let deadline = Instant::now() + options.timeout;
        loop {
            let response = self.get_image_task(task_id).await?;
            if response.is_finished() {
                return Ok(response);
            }
            if Instant::now() >= deadline {
                return Err(AiApiError::Timeout(format!(
                    "Bailian image task {task_id} did not finish within {:?}",
                    options.timeout
                )));
            }
            sleep(options.poll_interval).await;
        }
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, self.url(path))
            .bearer_auth(&self.config.api_key)
            .header("Content-Type", "application/json")
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BailianWaitOptions {
    pub poll_interval: Duration,
    pub timeout: Duration,
}

impl Default for BailianWaitOptions {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(2),
            timeout: Duration::from_secs(300),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianImageGenerationRequest {
    pub model: String,
    pub input: BailianImageInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<BailianImageParameters>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BailianObject,
}

impl BailianImageGenerationRequest {
    pub fn text_to_image(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: BailianImageInput {
                messages: vec![BailianImageMessage::user(vec![
                    BailianImageContentPart::text(prompt),
                ])],
            },
            parameters: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn validate(&self) -> AiApiResult<()> {
        if self.input.messages.len() != 1 {
            return Err(AiApiError::Validation(
                "Bailian image API supports exactly one user message".to_string(),
            ));
        }
        let message = &self.input.messages[0];
        if message.role != BailianMessageRole::User {
            return Err(AiApiError::Validation(
                "Bailian image API message role must be user".to_string(),
            ));
        }
        if message.content.is_empty() {
            return Err(AiApiError::Validation(
                "Bailian image API content cannot be empty".to_string(),
            ));
        }
        let image_count = message.image_count();
        if image_count > 9 {
            return Err(AiApiError::Validation(
                "Bailian image API accepts at most 9 input images".to_string(),
            ));
        }
        for part in &message.content {
            if !part.has_content() {
                return Err(AiApiError::Validation(
                    "Bailian content part must contain text, image, or extension fields"
                        .to_string(),
                ));
            }
        }
        if let Some(parameters) = &self.parameters {
            parameters.validate(image_count)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BailianImageModel {
    #[serde(rename = "wan2.7-image-pro")]
    Wan27ImagePro,
    #[serde(rename = "wan2.7-image")]
    Wan27Image,
}

impl BailianImageModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wan27ImagePro => "wan2.7-image-pro",
            Self::Wan27Image => "wan2.7-image",
        }
    }
}

impl From<BailianImageModel> for String {
    fn from(model: BailianImageModel) -> Self {
        model.as_str().to_string()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BailianImageInput {
    pub messages: Vec<BailianImageMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BailianImageMessage {
    pub role: BailianMessageRole,
    pub content: Vec<BailianImageContentPart>,
}

impl BailianImageMessage {
    pub fn user(content: Vec<BailianImageContentPart>) -> Self {
        Self {
            role: BailianMessageRole::User,
            content,
        }
    }

    fn image_count(&self) -> usize {
        self.content
            .iter()
            .filter(|part| part.image.is_some())
            .count()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BailianMessageRole {
    #[serde(rename = "user")]
    User,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BailianImageContentPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

impl BailianImageContentPart {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            image: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn image(image: impl Into<String>) -> Self {
        Self {
            text: None,
            image: Some(image.into()),
            extra: BTreeMap::new(),
        }
    }

    pub fn image_data(mime_type: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> Self {
        Self::image(bailian_image_data_url(mime_type, bytes))
    }

    fn has_content(&self) -> bool {
        self.text.is_some() || self.image.is_some() || !self.extra.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct BailianImageParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox_list: Option<Vec<Vec<[u32; 4]>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_sequential: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_palette: Option<Vec<BailianColorRatio>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BailianObject,
}

impl BailianImageParameters {
    pub fn validate(&self, image_count: usize) -> AiApiResult<()> {
        if let Some(n) = self.n {
            let sequential = self.enable_sequential.unwrap_or(false);
            let valid = if sequential { 1..=12 } else { 1..=4 };
            if !valid.contains(&n) {
                return Err(AiApiError::Validation(if sequential {
                    "Bailian n must be between 1 and 12 when enable_sequential is true".to_string()
                } else {
                    "Bailian n must be between 1 and 4 when enable_sequential is false".to_string()
                }));
            }
        }
        if let Some(seed) = self.seed
            && seed > MAX_BAILIAN_SEED
        {
            return Err(AiApiError::Validation(format!(
                "Bailian seed must be between 0 and {MAX_BAILIAN_SEED}"
            )));
        }
        if let Some(bbox_list) = &self.bbox_list {
            if bbox_list.len() != image_count {
                return Err(AiApiError::Validation(
                    "Bailian bbox_list length must match input image count".to_string(),
                ));
            }
            if bbox_list.iter().any(|boxes| boxes.len() > 2) {
                return Err(AiApiError::Validation(
                    "Bailian bbox_list supports at most 2 boxes per image".to_string(),
                ));
            }
        }
        if let Some(color_palette) = &self.color_palette {
            if !(3..=10).contains(&color_palette.len()) {
                return Err(AiApiError::Validation(
                    "Bailian color_palette must contain 3 to 10 colors".to_string(),
                ));
            }
            let total = color_palette
                .iter()
                .map(BailianColorRatio::ratio_basis_points)
                .try_fold(0_u32, |sum, ratio| ratio.map(|ratio| sum + ratio))?;
            if total != 10_000 {
                return Err(AiApiError::Validation(
                    "Bailian color_palette ratio values must sum to 100.00%".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianColorRatio {
    pub hex: String,
    pub ratio: String,
}

impl BailianColorRatio {
    fn ratio_basis_points(&self) -> AiApiResult<u32> {
        if !self.hex.starts_with('#') || self.hex.len() != 7 {
            return Err(AiApiError::Validation(
                "Bailian color_palette hex must use #RRGGBB format".to_string(),
            ));
        }
        let ratio = self.ratio.strip_suffix('%').ok_or_else(|| {
            AiApiError::Validation("Bailian color ratio must end with %".to_string())
        })?;
        let (whole, fraction) = ratio.split_once('.').ok_or_else(|| {
            AiApiError::Validation(
                "Bailian color ratio must contain exactly two decimal places".to_string(),
            )
        })?;
        if fraction.len() != 2 {
            return Err(AiApiError::Validation(
                "Bailian color ratio must contain exactly two decimal places".to_string(),
            ));
        }
        let whole: u32 = whole.parse().map_err(|_| {
            AiApiError::Validation("Bailian color ratio whole part is invalid".to_string())
        })?;
        let fraction: u32 = fraction.parse().map_err(|_| {
            AiApiError::Validation("Bailian color ratio decimal part is invalid".to_string())
        })?;
        Ok(whole * 100 + fraction)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianImageGenerationResponse {
    #[serde(default, alias = "requestId")]
    pub request_id: Option<String>,
    #[serde(default)]
    pub output: Option<BailianImageOutput>,
    #[serde(default)]
    pub usage: Option<BailianImageUsage>,
    #[serde(default)]
    pub status_code: Option<u16>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(flatten, default)]
    pub extra: BailianObject,
}

impl BailianImageGenerationResponse {
    pub fn is_finished(&self) -> bool {
        self.output
            .as_ref()
            .is_some_and(BailianImageOutput::is_finished)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianImageOutput {
    #[serde(default)]
    pub choices: Vec<BailianImageChoice>,
    #[serde(default)]
    pub finished: Option<bool>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub task_status: Option<String>,
    #[serde(default)]
    pub submit_time: Option<String>,
    #[serde(default)]
    pub scheduled_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(flatten, default)]
    pub extra: BailianObject,
}

impl BailianImageOutput {
    pub fn is_finished(&self) -> bool {
        if self.finished.unwrap_or(false) {
            return true;
        }
        self.task_status
            .as_deref()
            .is_some_and(|status| matches!(status, "SUCCEEDED" | "FAILED" | "CANCELED"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianImageChoice {
    #[serde(default)]
    pub finish_reason: Option<String>,
    pub message: BailianImageAssistantMessage,
    #[serde(flatten, default)]
    pub extra: BailianObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianImageAssistantMessage {
    pub role: String,
    #[serde(default)]
    pub content: Vec<BailianImageOutputContent>,
    #[serde(flatten, default)]
    pub extra: BailianObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BailianImageOutputContent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(flatten, default)]
    pub extra: BailianObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BailianImageUsage {
    #[serde(default)]
    pub image_count: Option<u32>,
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub size: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn text_to_image_request_serializes_documented_shape() {
        let mut request =
            BailianImageGenerationRequest::text_to_image(BailianImageModel::Wan27ImagePro, "花店");
        request.parameters = Some(BailianImageParameters {
            size: Some("2K".to_string()),
            n: Some(1),
            watermark: Some(false),
            thinking_mode: Some(true),
            ..Default::default()
        });

        let value = serde_json::to_value(&request).unwrap();

        assert_eq!(value["model"], "wan2.7-image-pro");
        assert_eq!(value["input"]["messages"][0]["role"], "user");
        assert_eq!(value["input"]["messages"][0]["content"][0]["text"], "花店");
        assert_eq!(value["parameters"]["thinking_mode"], true);
        request.validate().unwrap();
    }

    #[test]
    fn image_edit_request_supports_multiple_input_images() {
        let request = BailianImageGenerationRequest {
            model: BailianImageModel::Wan27ImagePro.into(),
            input: BailianImageInput {
                messages: vec![BailianImageMessage::user(vec![
                    BailianImageContentPart::image("https://example.test/car.webp"),
                    BailianImageContentPart::image_data("image/png", b"fake"),
                    BailianImageContentPart::text("把图2放到图1上"),
                ])],
            },
            parameters: Some(BailianImageParameters {
                size: Some("2K".to_string()),
                n: Some(1),
                watermark: Some(false),
                ..Default::default()
            }),
            extra: BTreeMap::new(),
        };

        let value = serde_json::to_value(&request).unwrap();

        assert_eq!(
            value["input"]["messages"][0]["content"][0]["image"],
            "https://example.test/car.webp"
        );
        assert_eq!(
            value["input"]["messages"][0]["content"][1]["image"],
            "data:image/png;base64,ZmFrZQ=="
        );
        request.validate().unwrap();
    }

    #[test]
    fn data_url_helper_encodes_local_images() {
        assert_eq!(
            bailian_image_data_url("image/webp", b"fake"),
            "data:image/webp;base64,ZmFrZQ=="
        );
    }

    #[test]
    fn interactive_bbox_request_validates_against_image_count() {
        let mut request = BailianImageGenerationRequest {
            model: BailianImageModel::Wan27ImagePro.into(),
            input: BailianImageInput {
                messages: vec![BailianImageMessage::user(vec![
                    BailianImageContentPart::image("https://example.test/a.png"),
                    BailianImageContentPart::image("https://example.test/b.png"),
                    BailianImageContentPart::text("替换框选区域"),
                ])],
            },
            parameters: Some(BailianImageParameters {
                bbox_list: Some(vec![vec![], vec![[989, 515, 1138, 681]]]),
                size: Some("2K".to_string()),
                n: Some(1),
                ..Default::default()
            }),
            extra: BTreeMap::new(),
        };

        request.validate().unwrap();
        request.parameters.as_mut().unwrap().bbox_list = Some(vec![vec![]]);
        assert!(matches!(
            request.validate(),
            Err(AiApiError::Validation(message))
                if message.contains("bbox_list length")
        ));
    }

    #[test]
    fn sequential_generation_allows_up_to_twelve_images() {
        let request = BailianImageGenerationRequest {
            model: BailianImageModel::Wan27ImagePro.into(),
            input: BailianImageInput {
                messages: vec![BailianImageMessage::user(vec![
                    BailianImageContentPart::text("四季组图"),
                ])],
            },
            parameters: Some(BailianImageParameters {
                enable_sequential: Some(true),
                n: Some(12),
                size: Some("2K".to_string()),
                ..Default::default()
            }),
            extra: BTreeMap::new(),
        };

        let value = serde_json::to_value(&request).unwrap();

        assert_eq!(value["parameters"]["enable_sequential"], true);
        assert_eq!(value["parameters"]["n"], 12);
        request.validate().unwrap();
    }

    #[test]
    fn validation_rejects_invalid_n_seed_bbox_and_color_ratios() {
        let mut request =
            BailianImageGenerationRequest::text_to_image(BailianImageModel::Wan27Image, "test");
        request.parameters = Some(BailianImageParameters {
            n: Some(5),
            ..Default::default()
        });
        assert!(matches!(
            request.validate(),
            Err(AiApiError::Validation(message)) if message.contains("1 and 4")
        ));

        request.parameters = Some(BailianImageParameters {
            seed: Some(MAX_BAILIAN_SEED + 1),
            ..Default::default()
        });
        assert!(matches!(
            request.validate(),
            Err(AiApiError::Validation(message)) if message.contains("seed")
        ));

        request.parameters = Some(BailianImageParameters {
            color_palette: Some(vec![
                BailianColorRatio {
                    hex: "#000000".to_string(),
                    ratio: "30.00%".to_string(),
                },
                BailianColorRatio {
                    hex: "#111111".to_string(),
                    ratio: "30.00%".to_string(),
                },
                BailianColorRatio {
                    hex: "#222222".to_string(),
                    ratio: "30.00%".to_string(),
                },
            ]),
            ..Default::default()
        });
        assert!(matches!(
            request.validate(),
            Err(AiApiError::Validation(message)) if message.contains("100.00%")
        ));
    }

    #[test]
    fn sync_success_response_deserializes_images_and_usage() {
        let response: BailianImageGenerationResponse = serde_json::from_value(json!({
            "request_id": "req_1",
            "output": {
                "choices": [{
                    "finish_reason": "stop",
                    "message": {
                        "role": "assistant",
                        "content": [{"type": "image", "image": "https://example.test/out.png"}]
                    }
                }],
                "finished": true
            },
            "usage": {
                "image_count": 1,
                "input_tokens": 10,
                "output_tokens": 2,
                "total_tokens": 12,
                "size": "2048*2048"
            }
        }))
        .unwrap();

        assert!(response.is_finished());
        assert_eq!(
            response.output.unwrap().choices[0].message.content[0].image,
            Some("https://example.test/out.png".to_string())
        );
        assert_eq!(response.usage.unwrap().image_count, Some(1));
    }

    #[test]
    fn error_response_deserializes_code_message() {
        let response: BailianImageGenerationResponse = serde_json::from_value(json!({
            "request_id": "req_2",
            "code": "InvalidParameter",
            "message": "num_images_per_prompt must be 1"
        }))
        .unwrap();

        assert_eq!(response.code, Some("InvalidParameter".to_string()));
        assert_eq!(
            response.message,
            Some("num_images_per_prompt must be 1".to_string())
        );
    }

    #[test]
    fn async_task_create_and_query_responses_deserialize_aliases() {
        let create: BailianImageGenerationResponse = serde_json::from_value(json!({
            "requestId": "req_3",
            "output": {
                "task_id": "task_1",
                "task_status": "PENDING"
            },
            "status_code": 200,
            "code": "",
            "message": ""
        }))
        .unwrap();
        assert_eq!(create.request_id, Some("req_3".to_string()));
        assert!(!create.is_finished());

        let query: BailianImageGenerationResponse = serde_json::from_value(json!({
            "requestId": "req_4",
            "output": {
                "task_id": "task_1",
                "task_status": "SUCCEEDED",
                "finished": true,
                "submit_time": "2026-03-31 19:57:58.840",
                "scheduled_time": "2026-03-31 19:57:58.877",
                "end_time": "2026-03-31 19:58:11.563",
                "choices": [{
                    "finish_reason": "stop",
                    "message": {
                        "role": "assistant",
                        "content": [{"type": "image", "image": "https://example.test/out.png"}]
                    }
                }]
            }
        }))
        .unwrap();

        assert!(query.is_finished());
        assert_eq!(
            query.output.unwrap().end_time,
            Some("2026-03-31 19:58:11.563".to_string())
        );
    }

    #[test]
    fn color_ratio_parser_requires_two_decimal_places() {
        let ratio = BailianColorRatio {
            hex: "#AABBCC".to_string(),
            ratio: "25.50%".to_string(),
        };
        assert_eq!(ratio.ratio_basis_points().unwrap(), 2550);

        let invalid = BailianColorRatio {
            hex: "#AABBCC".to_string(),
            ratio: "25%".to_string(),
        };
        assert!(invalid.ratio_basis_points().is_err());
    }

    #[test]
    fn response_extra_fields_are_preserved() {
        let response: BailianImageGenerationResponse = serde_json::from_value(json!({
            "request_id": "req_extra",
            "vendor_field": {"nested": true}
        }))
        .unwrap();

        assert_eq!(
            response.extra.get("vendor_field"),
            Some(&Value::Object(serde_json::Map::from_iter([(
                "nested".to_string(),
                Value::Bool(true)
            )])))
        );
    }
}
