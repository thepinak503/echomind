use crate::error::{EchomindError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api: ApiConfig,

    #[serde(default)]
    pub defaults: Defaults,

    #[serde(default)]
    pub presets: std::collections::HashMap<String, Preset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub system_prompt: Option<String>,
    pub messages: Option<Vec<crate::api::Message>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_provider")]
    pub provider: String,

    pub api_key: Option<String>,

    #[serde(default)]
    pub endpoint: Option<String>,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_timeout")]
    pub timeout: u64,

    #[serde(default)]
    pub fallback_providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    #[serde(default)]
    pub max_tokens: Option<u32>,

    #[serde(default)]
    pub top_p: Option<f32>,

    #[serde(default)]
    pub top_k: Option<u32>,

    #[serde(default)]
    pub coder_mode: bool,

    #[serde(default = "default_stream")]
    pub stream: bool,
}

fn default_provider() -> String {
    "chat".to_string()
}

fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_temperature() -> f32 {
    0.7
}

fn default_stream() -> bool {
    false
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key: None,
            endpoint: None,
            model: default_model(),
            timeout: default_timeout(),
            fallback_providers: Vec::new(),
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: None,
            top_p: None,
            top_k: None,
            coder_mode: false,
            stream: default_stream(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            defaults: Defaults::default(),
            presets: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path).map_err(|e| {
                EchomindError::ConfigError(format!("Failed to read config file: {}", e))
            })?;

            toml::from_str(&contents).map_err(|e| {
                EchomindError::ConfigError(format!("Failed to parse config file: {}", e))
            })
        } else {
            // Return default config if file doesn't exist
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                EchomindError::ConfigError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let contents = toml::to_string_pretty(self).map_err(|e| {
            EchomindError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&config_path, contents).map_err(|e| {
            EchomindError::ConfigError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            EchomindError::ConfigError("Could not determine config directory".to_string())
        })?;

        Ok(config_dir.join("echomind").join("config.toml"))
    }

    pub fn init_default_config() -> Result<()> {
        let config = Config::default();
        config.save()?;
        println!(
            "Created default configuration at: {}",
            Self::config_path()?.display()
        );
        Ok(())
    }
}
