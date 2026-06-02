pub mod bailian;
pub mod openai;

pub use bailian::*;
pub use openai::*;

use reqwest::StatusCode;
use serde_json::Value;

pub type AiApiResult<T> = Result<T, AiApiError>;

#[derive(Debug, thiserror::Error)]
pub enum AiApiError {
    #[error("missing required environment variable {name}")]
    MissingEnvironmentVariable { name: &'static str },
    #[error("invalid AI API config: {0}")]
    InvalidConfig(String),
    #[error("invalid AI API request: {0}")]
    Validation(String),
    #[error("AI API HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("AI API JSON handling failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("AI API file handling failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("AI API returned {status}{}: {message}", api_error_code_suffix(code))]
    Api {
        status: StatusCode,
        code: Option<String>,
        message: String,
        body: Value,
    },
    #[error("AI API stream failed: {0}")]
    Stream(String),
    #[error("AI API operation timed out: {0}")]
    Timeout(String),
}

fn api_error_code_suffix(code: &Option<String>) -> String {
    code.as_ref()
        .map(|code| format!(" ({code})"))
        .unwrap_or_default()
}

pub(crate) async fn parse_json_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> AiApiResult<T> {
    let status = response.status();
    let text = response.text().await?;
    if status.is_success() {
        return serde_json::from_str(&text).map_err(AiApiError::from);
    }
    Err(api_error_from_body(status, &text))
}

pub(crate) async fn ensure_success_response(
    response: reqwest::Response,
) -> AiApiResult<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let text = response.text().await?;
    Err(api_error_from_body(status, &text))
}

pub(crate) fn api_error_from_body(status: StatusCode, text: &str) -> AiApiError {
    let body = serde_json::from_str::<Value>(text).unwrap_or_else(|_| Value::String(text.into()));
    let (code, message) = extract_api_error(&body)
        .unwrap_or_else(|| (None, text.trim().to_string()))
        .pipe(|(code, message)| {
            let message = if message.is_empty() {
                status
                    .canonical_reason()
                    .unwrap_or("API request failed")
                    .to_string()
            } else {
                message
            };
            (code, message)
        });
    AiApiError::Api {
        status,
        code,
        message,
        body,
    }
}

fn extract_api_error(body: &Value) -> Option<(Option<String>, String)> {
    if let Some(error) = body.get("error") {
        return Some((
            error
                .get("code")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| {
                    error
                        .get("type")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                }),
            error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("API request failed")
                .to_string(),
        ));
    }
    let code = body
        .get("code")
        .and_then(Value::as_str)
        .filter(|code| !code.is_empty())
        .map(ToOwned::to_owned);
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .filter(|message| !message.is_empty())?;
    Some((code, message.to_string()))
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}
