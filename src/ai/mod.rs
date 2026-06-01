pub mod bailian;
pub mod openai;

pub use bailian::*;
pub use openai::*;

use reqwest::StatusCode;
use serde_json::Value;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub type AiApiResult<T> = Result<T, AiApiError>;

#[derive(Debug)]
pub enum AiApiError {
    MissingEnvironmentVariable {
        name: &'static str,
    },
    InvalidConfig(String),
    Validation(String),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    Api {
        status: StatusCode,
        code: Option<String>,
        message: String,
        body: Value,
    },
    Stream(String),
    Timeout(String),
}

impl Display for AiApiError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnvironmentVariable { name } => {
                write!(formatter, "missing required environment variable {name}")
            }
            Self::InvalidConfig(message) => write!(formatter, "invalid AI API config: {message}"),
            Self::Validation(message) => write!(formatter, "invalid AI API request: {message}"),
            Self::Http(error) => write!(formatter, "AI API HTTP request failed: {error}"),
            Self::Json(error) => write!(formatter, "AI API JSON handling failed: {error}"),
            Self::Io(error) => write!(formatter, "AI API file handling failed: {error}"),
            Self::Api {
                status,
                code,
                message,
                ..
            } => {
                if let Some(code) = code {
                    write!(formatter, "AI API returned {status} ({code}): {message}")
                } else {
                    write!(formatter, "AI API returned {status}: {message}")
                }
            }
            Self::Stream(message) => write!(formatter, "AI API stream failed: {message}"),
            Self::Timeout(message) => write!(formatter, "AI API operation timed out: {message}"),
        }
    }
}

impl Error for AiApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for AiApiError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<serde_json::Error> for AiApiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<std::io::Error> for AiApiError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
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
