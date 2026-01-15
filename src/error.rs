use thiserror::Error;

#[derive(Error, Debug)]
pub enum EchomindError {
    #[error("Failed to read input from stdin: {0}")]
    InputError(#[from] std::io::Error),

    #[error("Network error: {0}. Please check your internet connection.")]
    NetworkError(String),

    #[error("API request failed with status {status}: {message}. {suggestion}")]
    ApiError {
        status: u16,
        message: String,
        suggestion: String,
    },

    #[error("Request timed out after {0} seconds. The API might be slow or unavailable. Try increasing timeout with --timeout option.")]
    TimeoutError(u64),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid API provider: '{0}'. Supported providers: chat, chatanywhere, openai, claude, gemini, ollama, grok, mistral, cohere, or a custom URL.\nCheck your config or use --provider option.")]
    InvalidProvider(String),

    #[error("API key required for provider '{0}'.\nSet it in config or use ECHOMIND_API_KEY environment variable.")]
    MissingApiKey(String),

    #[error("No response received from API. The API might be unavailable or returned an empty response.")]
    EmptyResponse,

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Platform-specific error: {0}")]
    PlatformError(String),

    #[error("{0}")]
    Other(String),
}

// Add platform-specific hints
fn get_config_hint() -> String {
    #[cfg(target_os = "windows")]
    {
        format!(
            "{}\\AppData\\Local\\echomind\\config.toml",
            std::env::var("USERPROFILE").unwrap_or_else(|_| "~".to_string())
        )
    }
    #[cfg(target_os = "macos")]
    {
        "~/.config/echomind/config.toml".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "~/.config/echomind/config.toml or $XDG_CONFIG_HOME/echomind/config.toml".to_string()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "~/.config/echomind/config.toml (or platform-specific config directory)".to_string()
    }
}

impl EchomindError {
    pub fn with_platform_context(self) -> Self {
        match self {
            EchomindError::ConfigError(msg) => EchomindError::ConfigError(msg),
            EchomindError::MissingApiKey(provider) => EchomindError::MissingApiKey(provider),
            other => other,
        }
    }
}

impl From<reqwest::Error> for EchomindError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            EchomindError::TimeoutError(30)
        } else if err.is_connect() || err.is_request() {
            let error_msg = if cfg!(target_os = "windows") {
                format!(
                    "{}. On Windows, ensure firewall isn't blocking the connection.",
                    err
                )
            } else if cfg!(target_os = "macos") {
                format!(
                    "{}. On macOS, check System Preferences > Security & Privacy.",
                    err
                )
            } else {
                format!(
                    "{}. On Linux, check network connectivity and firewall rules.",
                    err
                )
            };
            EchomindError::NetworkError(error_msg)
        } else if let Some(status) = err.status() {
            let suggestion = match status.as_u16() {
                401 => "Unauthorized: Check your API key is correct and not expired.".to_string(),
                403 => "Forbidden: Your API key may not have permission for this operation."
                    .to_string(),
                429 => {
                    "Rate limited: You're making requests too quickly. Try again later.".to_string()
                }
                500..=599 => "Server error: The API is having issues. Try again later.".to_string(),
                _ => "Check the API documentation for this status code.".to_string(),
            };
            EchomindError::ApiError {
                status: status.as_u16(),
                message: err.to_string(),
                suggestion,
            }
        } else {
            EchomindError::NetworkError(err.to_string())
        }
    }
}

impl From<std::string::FromUtf8Error> for EchomindError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        EchomindError::ParseError(err.to_string())
    }
}

impl From<serde_json::Error> for EchomindError {
    fn from(err: serde_json::Error) -> Self {
        EchomindError::ParseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, EchomindError>;
