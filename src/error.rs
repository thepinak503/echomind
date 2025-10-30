use thiserror::Error;

#[derive(Error, Debug)]
pub enum EchomindError {
    #[error("Failed to read input from stdin: {0}")]
    InputError(#[from] std::io::Error),

    #[error("Network error: {0}. Please check your internet connection.")]
    NetworkError(String),

    #[error("API request failed with status {status}: {message}. {suggestion}")]
    ApiError { status: u16, message: String, suggestion: String },

    #[error("Request timed out after {0} seconds. The API might be slow or unavailable.")]
    TimeoutError(u64),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid API provider: '{0}'. Supported providers: chat, chatanywhere, openai, claude, gemini, ollama, grok, mistral, cohere, or a custom URL. Check your config or use --provider option.")]
    InvalidProvider(String),

    #[error("API key required for provider '{0}'. Set it in config or use ECHOMIND_API_KEY environment variable.")]
    MissingApiKey(String),

    #[error("No response received from API")]
    EmptyResponse,

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for EchomindError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            EchomindError::TimeoutError(30)
        } else if err.is_connect() || err.is_request() {
            EchomindError::NetworkError(err.to_string())
        } else if let Some(status) = err.status() {
            EchomindError::ApiError {
                status: status.as_u16(),
                message: err.to_string(),
                suggestion: "Check the API documentation for this status code.".to_string(),
            }
        } else {
            EchomindError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for EchomindError {
    fn from(err: serde_json::Error) -> Self {
        EchomindError::ParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EchomindError>;
