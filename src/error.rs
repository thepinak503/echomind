use thiserror::Error;

#[derive(Error, Debug)]
pub enum EchomindError {
    #[error("Failed to read input: {0}")]
    InputError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error (HTTP {status}): {message}. {suggestion}")]
    ApiError {
        status: u16,
        message: String,
        suggestion: String,
    },

    #[error("Request timed out after {0}s. Try --timeout to increase.")]
    TimeoutError(u64),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unknown provider: '{0}'. Use --list-providers to see available options.")]
    InvalidProvider(String),

    #[error("API key required for '{0}'. Set ECHOMIND_API_KEY or use --api-key.")]
    MissingApiKey(String),

    #[error("Empty response from API.")]
    EmptyResponse,

    #[error("File error: {0}")]
    FileError(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for EchomindError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            return Self::TimeoutError(30);
        }
        if err.is_connect() || err.is_request() {
            return Self::NetworkError(err.to_string());
        }
        if let Some(status) = err.status() {
            let s = status.as_u16();
            return Self::ApiError {
                status: s,
                message: err.to_string(),
                suggestion: match s {
                    401 => "Check your API key.",
                    403 => "API key lacks permissions.",
                    429 => "Rate limited. Try again later.",
                    500..=599 => "Server error. Try again later.",
                    _ => "Check API docs.",
                }
                .into(),
            };
        }
        Self::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for EchomindError {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EchomindError>;
