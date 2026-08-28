use thiserror::Error;

#[derive(Debug, Error)]
pub enum LarkAlertError {
    #[error("invalid webhook URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Feishu returned HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("Feishu business error (code {code}): {msg}")]
    Business { code: i64, msg: String },

    #[error("invalid response from Feishu: {0}")]
    InvalidResponse(String),

    #[error("retry exhausted after {retries} retries: {source}")]
    RetryExhausted {
        retries: u32,
        source: Box<LarkAlertError>,
    },

    #[error("message validation error: {0}")]
    Validation(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
