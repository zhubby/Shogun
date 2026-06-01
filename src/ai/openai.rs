use super::{AiApiError, AiApiResult, ensure_success_response, parse_json_response};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_util::{Stream, StreamExt, stream};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::pin::Pin;
use std::time::Duration;

pub const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub type OpenAiObject = BTreeMap<String, Value>;
pub type OpenAiEventStream = Pin<Box<dyn Stream<Item = AiApiResult<OpenAiStreamEvent>> + Send>>;

pub fn openai_data_url(mime_type: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> String {
    format!(
        "data:{};base64,{}",
        mime_type.as_ref(),
        STANDARD.encode(bytes)
    )
}

#[derive(Clone, Debug)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub timeout: Option<Duration>,
}

impl OpenAiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: OPENAI_DEFAULT_BASE_URL.to_string(),
            organization: None,
            project: None,
            timeout: Some(Duration::from_secs(120)),
        }
    }

    pub fn from_env() -> AiApiResult<Self> {
        let api_key =
            env::var("OPENAI_API_KEY").map_err(|_| AiApiError::MissingEnvironmentVariable {
                name: "OPENAI_API_KEY",
            })?;
        let mut config = Self::new(api_key);
        if let Ok(base_url) = env::var("OPENAI_BASE_URL") {
            config.base_url = base_url;
        }
        config.organization = env::var("OPENAI_ORGANIZATION").ok();
        config.project = env::var("OPENAI_PROJECT").ok();
        Ok(config)
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    fn validate(&self) -> AiApiResult<()> {
        if self.api_key.trim().is_empty() {
            return Err(AiApiError::InvalidConfig(
                "OpenAI API key cannot be empty".to_string(),
            ));
        }
        if self.base_url.trim().is_empty() {
            return Err(AiApiError::InvalidConfig(
                "OpenAI base URL cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct OpenAiClient {
    http: reqwest::Client,
    config: OpenAiConfig,
}

impl OpenAiClient {
    pub fn new(config: OpenAiConfig) -> AiApiResult<Self> {
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
        Self::new(OpenAiConfig::from_env()?)
    }

    pub fn config(&self) -> &OpenAiConfig {
        &self.config
    }

    pub async fn create_response(
        &self,
        request: &OpenAiResponseRequest,
    ) -> AiApiResult<OpenAiResponse> {
        self.post_json("/responses", request).await
    }

    pub async fn stream_response(
        &self,
        request: &OpenAiResponseRequest,
    ) -> AiApiResult<OpenAiEventStream> {
        let mut request = request.clone();
        request.stream = Some(true);
        self.post_json_stream("/responses", &request, OpenAiStreamKind::Responses)
            .await
    }

    pub async fn create_chat_completion(
        &self,
        request: &OpenAiChatCompletionRequest,
    ) -> AiApiResult<OpenAiChatCompletionResponse> {
        self.post_json("/chat/completions", request).await
    }

    pub async fn stream_chat_completion(
        &self,
        request: &OpenAiChatCompletionRequest,
    ) -> AiApiResult<OpenAiEventStream> {
        let mut request = request.clone();
        request.stream = Some(true);
        self.post_json_stream(
            "/chat/completions",
            &request,
            OpenAiStreamKind::ChatCompletion,
        )
        .await
    }

    pub async fn create_completion(
        &self,
        request: &OpenAiCompletionRequest,
    ) -> AiApiResult<OpenAiCompletionResponse> {
        self.post_json("/completions", request).await
    }

    pub async fn stream_completion(
        &self,
        request: &OpenAiCompletionRequest,
    ) -> AiApiResult<OpenAiEventStream> {
        let mut request = request.clone();
        request.stream = Some(true);
        self.post_json_stream("/completions", &request, OpenAiStreamKind::Completion)
            .await
    }

    pub async fn upload_file(&self, request: OpenAiFileUploadRequest) -> AiApiResult<OpenAiFile> {
        let mut form = reqwest::multipart::Form::new();
        for (key, value) in request.multipart_text_fields()? {
            form = form.text(key, value);
        }
        let mut file_part =
            reqwest::multipart::Part::bytes(request.bytes).file_name(request.file_name.clone());
        if let Some(mime_type) = request.mime_type {
            file_part = file_part.mime_str(&mime_type)?;
        }
        form = form.part("file", file_part);

        let response = self
            .request(Method::POST, "/files")
            .multipart(form)
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn list_files(&self) -> AiApiResult<OpenAiList<OpenAiFile>> {
        let response = self.request(Method::GET, "/files").send().await?;
        parse_json_response(response).await
    }

    pub async fn list_files_with_query(
        &self,
        query: &OpenAiFileListQuery,
    ) -> AiApiResult<OpenAiList<OpenAiFile>> {
        let response = self
            .request(Method::GET, "/files")
            .query(query)
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn retrieve_file(&self, file_id: &str) -> AiApiResult<OpenAiFile> {
        let response = self
            .request(Method::GET, &format!("/files/{file_id}"))
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn delete_file(&self, file_id: &str) -> AiApiResult<OpenAiDeleteResponse> {
        let response = self
            .request(Method::DELETE, &format!("/files/{file_id}"))
            .send()
            .await?;
        parse_json_response(response).await
    }

    pub async fn download_file_content(&self, file_id: &str) -> AiApiResult<Vec<u8>> {
        let response = self
            .request(Method::GET, &format!("/files/{file_id}/content"))
            .send()
            .await?;
        let response = ensure_success_response(response).await?;
        Ok(response.bytes().await?.to_vec())
    }

    async fn post_json<B, T>(&self, path: &str, body: &B) -> AiApiResult<T>
    where
        B: Serialize + ?Sized,
        T: serde::de::DeserializeOwned,
    {
        let response = self.request(Method::POST, path).json(body).send().await?;
        parse_json_response(response).await
    }

    async fn post_json_stream<B>(
        &self,
        path: &str,
        body: &B,
        kind: OpenAiStreamKind,
    ) -> AiApiResult<OpenAiEventStream>
    where
        B: Serialize + ?Sized,
    {
        let response = self.request(Method::POST, path).json(body).send().await?;
        let response = ensure_success_response(response).await?;
        let mut decoder = SseDecoder::new(kind);
        let stream = response
            .bytes_stream()
            .map(move |chunk| match chunk {
                Ok(bytes) => stream::iter(decoder.push_bytes(&bytes)),
                Err(error) => stream::iter(vec![Err(AiApiError::Http(error))]),
            })
            .flatten();
        Ok(Box::pin(stream))
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let mut builder = self
            .http
            .request(method, self.url(path))
            .bearer_auth(&self.config.api_key);
        if let Some(organization) = &self.config.organization {
            builder = builder.header("OpenAI-Organization", organization);
        }
        if let Some(project) = &self.config.project {
            builder = builder.header("OpenAI-Project", project);
        }
        builder
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiResponseRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<OpenAiResponseInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_management: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiResponseRequest {
    pub fn new(model: impl Into<String>, input: OpenAiResponseInput) -> Self {
        Self {
            model: model.into(),
            input: Some(input),
            instructions: None,
            background: None,
            context_management: None,
            conversation: None,
            include: None,
            max_output_tokens: None,
            max_tool_calls: None,
            metadata: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            prompt: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            reasoning: None,
            safety_identifier: None,
            service_tier: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            text: None,
            tool_choice: None,
            tools: None,
            top_p: None,
            truncation: None,
            user: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAiResponseInput {
    Text(String),
    Items(Vec<OpenAiResponseInputItem>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAiResponseInputItem {
    Message(OpenAiResponseMessage),
    Other(Value),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiResponseMessage {
    pub role: String,
    pub content: OpenAiResponseContent,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiResponseMessage {
    pub fn user(content: Vec<OpenAiResponseContentPart>) -> Self {
        Self {
            role: "user".to_string(),
            content: OpenAiResponseContent::Parts(content),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAiResponseContent {
    Text(String),
    Parts(Vec<OpenAiResponseContentPart>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiResponseContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<OpenAiInputAudio>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiResponseContentPart {
    pub fn input_text(text: impl Into<String>) -> Self {
        Self::new("input_text").with_text(text)
    }

    pub fn input_image_url(url: impl Into<String>) -> Self {
        let mut part = Self::new("input_image");
        part.image_url = Some(url.into());
        part
    }

    pub fn input_image_data(mime_type: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> Self {
        Self::input_image_url(openai_data_url(mime_type, bytes))
    }

    pub fn input_image_file(file_id: impl Into<String>) -> Self {
        let mut part = Self::new("input_image");
        part.file_id = Some(file_id.into());
        part
    }

    pub fn input_file_id(file_id: impl Into<String>) -> Self {
        let mut part = Self::new("input_file");
        part.file_id = Some(file_id.into());
        part
    }

    pub fn input_file_data(filename: impl Into<String>, file_data: impl Into<String>) -> Self {
        let mut part = Self::new("input_file");
        part.filename = Some(filename.into());
        part.file_data = Some(file_data.into());
        part
    }

    pub fn input_file_bytes(
        filename: impl Into<String>,
        mime_type: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
    ) -> Self {
        Self::input_file_data(filename, openai_data_url(mime_type, bytes))
    }

    pub fn input_audio(data: impl Into<String>, format: impl Into<String>) -> Self {
        let mut part = Self::new("input_audio");
        part.input_audio = Some(OpenAiInputAudio {
            data: data.into(),
            format: format.into(),
        });
        part
    }

    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            text: None,
            image_url: None,
            file_id: None,
            file_url: None,
            file_data: None,
            filename: None,
            detail: None,
            input_audio: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiInputAudio {
    pub data: String,
    pub format: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAiChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<BTreeMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OpenAiObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<OpenAiStop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<OpenAiChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            audio: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            max_completion_tokens: None,
            max_tokens: None,
            metadata: None,
            modalities: None,
            n: None,
            parallel_tool_calls: None,
            prediction: None,
            presence_penalty: None,
            reasoning_effort: None,
            response_format: None,
            seed: None,
            service_tier: None,
            stop: None,
            store: None,
            stream: None,
            stream_options: None,
            temperature: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            user: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<OpenAiChatMessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Value>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiChatMessage {
    pub fn user(content: Vec<OpenAiChatContentPart>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(OpenAiChatMessageContent::Parts(content)),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            function_call: None,
            refusal: None,
            audio: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self::text("system", text)
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self::text("assistant", text)
    }

    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: Some(OpenAiChatMessageContent::Text(text.into())),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            function_call: None,
            refusal: None,
            audio: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAiChatMessageContent {
    Text(String),
    Parts(Vec<OpenAiChatContentPart>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiChatContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<OpenAiImageUrl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<OpenAiInputAudio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<OpenAiChatFile>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiChatContentPart {
    pub fn text(text: impl Into<String>) -> Self {
        let mut part = Self::new("text");
        part.text = Some(text.into());
        part
    }

    pub fn image_url(url: impl Into<String>) -> Self {
        let mut part = Self::new("image_url");
        part.image_url = Some(OpenAiImageUrl {
            url: url.into(),
            detail: None,
        });
        part
    }

    pub fn image_data(mime_type: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> Self {
        Self::image_url(openai_data_url(mime_type, bytes))
    }

    pub fn image_file(file_id: impl Into<String>) -> Self {
        let mut part = Self::new("image_url");
        part.image_url = Some(OpenAiImageUrl {
            url: format!("file:{}", file_id.into()),
            detail: None,
        });
        part
    }

    pub fn input_audio(data: impl Into<String>, format: impl Into<String>) -> Self {
        let mut part = Self::new("input_audio");
        part.input_audio = Some(OpenAiInputAudio {
            data: data.into(),
            format: format.into(),
        });
        part
    }

    pub fn file_id(file_id: impl Into<String>) -> Self {
        let mut part = Self::new("file");
        part.file = Some(OpenAiChatFile {
            file_id: Some(file_id.into()),
            file_data: None,
            filename: None,
        });
        part
    }

    pub fn file_data(filename: impl Into<String>, file_data: impl Into<String>) -> Self {
        let mut part = Self::new("file");
        part.file = Some(OpenAiChatFile {
            file_id: None,
            file_data: Some(file_data.into()),
            filename: Some(filename.into()),
        });
        part
    }

    pub fn file_bytes(
        filename: impl Into<String>,
        mime_type: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
    ) -> Self {
        Self::file_data(filename, openai_data_url(mime_type, bytes))
    }

    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            text: None,
            image_url: None,
            input_audio: None,
            file: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiChatFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OpenAiStop {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiCompletionRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<OpenAiCompletionPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<BTreeMap<String, i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<OpenAiStop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: OpenAiObject,
}

impl OpenAiCompletionRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: Some(OpenAiCompletionPrompt::Text(prompt.into())),
            best_of: None,
            echo: None,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            max_tokens: None,
            n: None,
            presence_penalty: None,
            seed: None,
            stop: None,
            stream: None,
            stream_options: None,
            suffix: None,
            temperature: None,
            top_p: None,
            user: None,
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAiCompletionPrompt {
    Text(String),
    Texts(Vec<String>),
    Tokens(Vec<i64>),
    TokenArrays(Vec<Vec<i64>>),
    Other(Value),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiResponse {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub output: Vec<Value>,
    #[serde(default)]
    pub output_text: Option<String>,
    #[serde(default)]
    pub usage: Option<Value>,
    #[serde(default)]
    pub error: Option<Value>,
    #[serde(default)]
    pub incomplete_details: Option<Value>,
    #[serde(default)]
    pub metadata: Option<OpenAiObject>,
    #[serde(flatten, default)]
    pub extra: OpenAiObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiChatCompletionResponse {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<Value>,
    #[serde(default)]
    pub usage: Option<Value>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(flatten, default)]
    pub extra: OpenAiObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiCompletionResponse {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Vec<Value>,
    #[serde(default)]
    pub usage: Option<Value>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(flatten, default)]
    pub extra: OpenAiObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiFileUploadRequest {
    pub file_name: String,
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub purpose: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_after: Option<OpenAiFileExpiresAfter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, String>,
}

impl OpenAiFileUploadRequest {
    pub fn new(
        file_name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            file_name: file_name.into(),
            bytes: bytes.into(),
            purpose: purpose.into(),
            expires_after: None,
            mime_type: None,
            fields: BTreeMap::new(),
        }
    }

    pub fn multipart_text_fields(&self) -> AiApiResult<BTreeMap<String, String>> {
        let mut fields = self.fields.clone();
        fields.insert("purpose".to_string(), self.purpose.clone());
        if let Some(expires_after) = &self.expires_after {
            fields.insert(
                "expires_after".to_string(),
                serde_json::to_string(expires_after)?,
            );
        }
        Ok(fields)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiFileExpiresAfter {
    pub anchor: String,
    pub seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiFile {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub bytes: Option<u64>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub status_details: Option<String>,
    #[serde(flatten, default)]
    pub extra: OpenAiObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OpenAiFileListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OpenAiList<T> {
    pub object: String,
    pub data: Vec<T>,
    #[serde(default)]
    pub has_more: Option<bool>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
    #[serde(flatten, default)]
    pub extra: OpenAiObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiDeleteResponse {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenAiStreamKind {
    Responses,
    ChatCompletion,
    Completion,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpenAiStreamEvent {
    Response { event: Option<String>, data: Value },
    ChatCompletion { data: Value },
    Completion { data: Value },
    Unknown { event: Option<String>, data: Value },
    Done,
}

#[derive(Clone, Debug)]
pub struct SseDecoder {
    buffer: String,
    kind: OpenAiStreamKind,
}

impl SseDecoder {
    pub fn new(kind: OpenAiStreamKind) -> Self {
        Self {
            buffer: String::new(),
            kind,
        }
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<AiApiResult<OpenAiStreamEvent>> {
        self.buffer.push_str(&String::from_utf8_lossy(bytes));
        self.buffer = self.buffer.replace("\r\n", "\n");
        let mut events = Vec::new();
        while let Some(boundary) = self.buffer.find("\n\n") {
            let record = self.buffer[..boundary].to_string();
            self.buffer.drain(..boundary + 2);
            if let Some(event) = self.parse_record(&record) {
                events.push(event);
            }
        }
        events
    }

    pub fn finish(&mut self) -> Vec<AiApiResult<OpenAiStreamEvent>> {
        if self.buffer.trim().is_empty() {
            self.buffer.clear();
            return Vec::new();
        }
        let record = std::mem::take(&mut self.buffer);
        self.parse_record(&record).into_iter().collect()
    }

    fn parse_record(&self, record: &str) -> Option<AiApiResult<OpenAiStreamEvent>> {
        let mut event_name = None;
        let mut data_lines = Vec::new();

        for line in record.lines() {
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            let (field, value) = line.split_once(':').unwrap_or((line, ""));
            let value = value.strip_prefix(' ').unwrap_or(value);
            match field {
                "event" => event_name = Some(value.to_string()),
                "data" => data_lines.push(value.to_string()),
                _ => {}
            }
        }

        if data_lines.is_empty() {
            return None;
        }
        let data = data_lines.join("\n");
        if data.trim() == "[DONE]" {
            return Some(Ok(OpenAiStreamEvent::Done));
        }
        let value = match serde_json::from_str::<Value>(&data) {
            Ok(value) => value,
            Err(error) => return Some(Err(AiApiError::Json(error))),
        };
        Some(Ok(match self.kind {
            OpenAiStreamKind::Responses => OpenAiStreamEvent::Response {
                event: event_name,
                data: value,
            },
            OpenAiStreamKind::ChatCompletion if event_name.is_none() => {
                OpenAiStreamEvent::ChatCompletion { data: value }
            }
            OpenAiStreamKind::Completion if event_name.is_none() => {
                OpenAiStreamEvent::Completion { data: value }
            }
            _ => OpenAiStreamEvent::Unknown {
                event: event_name,
                data: value,
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn unwrap_events(events: Vec<AiApiResult<OpenAiStreamEvent>>) -> Vec<OpenAiStreamEvent> {
        events.into_iter().map(Result::unwrap).collect()
    }

    #[test]
    fn responses_request_serializes_multimodal_input_and_extra_fields() {
        let mut request = OpenAiResponseRequest::new(
            "gpt-compatible",
            OpenAiResponseInput::Items(vec![OpenAiResponseInputItem::Message(
                OpenAiResponseMessage::user(vec![
                    OpenAiResponseContentPart::input_text("describe this"),
                    OpenAiResponseContentPart::input_image_data("image/png", b"fake"),
                    OpenAiResponseContentPart::input_file_id("file_123"),
                    OpenAiResponseContentPart::input_audio("AAAA", "wav"),
                ]),
            )]),
        );
        request.extra.insert("vendor_flag".into(), json!(true));

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(value["model"], "gpt-compatible");
        assert_eq!(value["vendor_flag"], true);
        let content = &value["input"][0]["content"];
        assert_eq!(content[0]["type"], "input_text");
        assert_eq!(content[1]["image_url"], "data:image/png;base64,ZmFrZQ==");
        assert_eq!(content[2]["file_id"], "file_123");
        assert_eq!(content[3]["input_audio"]["format"], "wav");
    }

    #[test]
    fn data_url_helper_encodes_local_bytes_for_multimodal_parts() {
        assert_eq!(
            openai_data_url("text/plain", b"notes"),
            "data:text/plain;base64,bm90ZXM="
        );

        let response_file =
            OpenAiResponseContentPart::input_file_bytes("notes.txt", "text/plain", b"notes");
        let chat_image = OpenAiChatContentPart::image_data("image/png", b"fake");

        let response_value = serde_json::to_value(response_file).unwrap();
        let chat_value = serde_json::to_value(chat_image).unwrap();

        assert_eq!(
            response_value["file_data"],
            "data:text/plain;base64,bm90ZXM="
        );
        assert_eq!(
            chat_value["image_url"]["url"],
            "data:image/png;base64,ZmFrZQ=="
        );
    }

    #[test]
    fn chat_request_serializes_image_audio_and_file_parts() {
        let request = OpenAiChatCompletionRequest::new(
            "chat-model",
            vec![OpenAiChatMessage::user(vec![
                OpenAiChatContentPart::text("inspect"),
                OpenAiChatContentPart::image_url("https://example.test/image.png"),
                OpenAiChatContentPart::input_audio("AAAA", "mp3"),
                OpenAiChatContentPart::file_data("notes.txt", "data:text/plain;base64,bm90ZXM="),
            ])],
        );

        let value = serde_json::to_value(request).unwrap();

        let content = &value["messages"][0]["content"];
        assert_eq!(content[0]["type"], "text");
        assert_eq!(
            content[1]["image_url"]["url"],
            "https://example.test/image.png"
        );
        assert_eq!(content[2]["input_audio"]["format"], "mp3");
        assert_eq!(content[3]["file"]["filename"], "notes.txt");
    }

    #[test]
    fn legacy_completion_request_serializes_prompt_and_controls() {
        let mut request = OpenAiCompletionRequest::new("legacy-model", "hello");
        request.suffix = Some(" done".to_string());
        request.echo = Some(true);
        request.best_of = Some(2);
        request.logprobs = Some(5);

        let value = serde_json::to_value(request).unwrap();

        assert_eq!(value["prompt"], "hello");
        assert_eq!(value["suffix"], " done");
        assert_eq!(value["echo"], true);
        assert_eq!(value["best_of"], 2);
        assert_eq!(value["logprobs"], 5);
    }

    #[test]
    fn file_upload_text_fields_include_purpose_and_expires_after() {
        let mut request = OpenAiFileUploadRequest::new("batch.jsonl", b"{}".to_vec(), "batch");
        request.expires_after = Some(OpenAiFileExpiresAfter {
            anchor: "created_at".to_string(),
            seconds: 3600,
        });
        request
            .fields
            .insert("custom".to_string(), "value".to_string());

        let fields = request.multipart_text_fields().unwrap();

        assert_eq!(fields["purpose"], "batch");
        assert_eq!(fields["custom"], "value");
        assert_eq!(
            serde_json::from_str::<Value>(&fields["expires_after"]).unwrap(),
            json!({"anchor": "created_at", "seconds": 3600})
        );
    }

    #[test]
    fn sse_decoder_parses_response_events_across_chunk_boundaries() {
        let mut decoder = SseDecoder::new(OpenAiStreamKind::Responses);

        assert!(
            decoder
                .push_bytes(b"event: response.output_text.delta\ndata: {\"delta\"")
                .is_empty()
        );
        let events = unwrap_events(decoder.push_bytes(b":\"hi\"}\n\n"));

        assert_eq!(
            events,
            vec![OpenAiStreamEvent::Response {
                event: Some("response.output_text.delta".to_string()),
                data: json!({"delta": "hi"})
            }]
        );
    }

    #[test]
    fn sse_decoder_parses_chat_completion_chunks_and_done() {
        let mut decoder = SseDecoder::new(OpenAiStreamKind::ChatCompletion);

        let events = unwrap_events(decoder.push_bytes(
            b"data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\ndata: [DONE]\n\n",
        ));

        assert_eq!(
            events,
            vec![
                OpenAiStreamEvent::ChatCompletion {
                    data: json!({"choices": [{"delta": {"content": "hi"}}]})
                },
                OpenAiStreamEvent::Done
            ]
        );
    }

    #[test]
    fn sse_decoder_parses_legacy_completion_chunks() {
        let mut decoder = SseDecoder::new(OpenAiStreamKind::Completion);

        let events =
            unwrap_events(decoder.push_bytes(b"data: {\"choices\":[{\"text\":\"hi\"}]}\n\n"));

        assert_eq!(
            events,
            vec![OpenAiStreamEvent::Completion {
                data: json!({"choices": [{"text": "hi"}]})
            }]
        );
    }

    #[test]
    fn sse_decoder_preserves_unexpected_named_events() {
        let mut decoder = SseDecoder::new(OpenAiStreamKind::ChatCompletion);

        let events =
            unwrap_events(decoder.push_bytes(b"event: vendor.event\ndata: {\"ok\":true}\n\n"));

        assert_eq!(
            events,
            vec![OpenAiStreamEvent::Unknown {
                event: Some("vendor.event".to_string()),
                data: json!({"ok": true})
            }]
        );
    }
}
