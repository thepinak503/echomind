use crate::error::{EchomindError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use futures::StreamExt;

#[derive(Debug, Clone)]
pub enum Provider {
    Chat,           // ch.at
    ChatAnywhere,   // chatanywhere.tech
    OpenAI,
    Gemini,
    Claude,
    Ollama,
    Custom(String),
}

impl Provider {
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "chat" | "ch.at" => Ok(Provider::Chat),
            "chatanywhere" | "chat-anywhere" => Ok(Provider::ChatAnywhere),
            "openai" => Ok(Provider::OpenAI),
            "gemini" | "google" => Ok(Provider::Gemini),
            "claude" | "anthropic" => Ok(Provider::Claude),
            "ollama" => Ok(Provider::Ollama),
            _ => {
                // If it looks like a URL, treat as custom endpoint
                if s.starts_with("http://") || s.starts_with("https://") {
                    Ok(Provider::Custom(s.to_string()))
                } else {
                    Err(EchomindError::InvalidProvider(s.to_string()))
                }
            }
        }
    }

    pub fn endpoint(&self) -> &str {
        match self {
            Provider::Chat => "https://ch.at/v1/chat/completions",
            Provider::ChatAnywhere => "https://api.chatanywhere.tech/v1/chat/completions",
            Provider::OpenAI => "https://api.openai.com/v1/chat/completions",
            // Gemini's endpoint depends on the model; use base here and construct per request
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Provider::Claude => "https://api.anthropic.com/v1/messages",
            Provider::Ollama => "http://localhost:11434/api/chat",
            Provider::Custom(url) => url,
        }
    }

    pub fn requires_api_key(&self) -> bool {
        matches!(self, Provider::OpenAI | Provider::Claude | Provider::ChatAnywhere | Provider::Gemini)
    }

    pub fn name(&self) -> &str {
        match self {
            Provider::Chat => "chat",
            Provider::ChatAnywhere => "chatanywhere",
            Provider::OpenAI => "openai",
            Provider::Gemini => "gemini",
            Provider::Claude => "claude",
            Provider::Ollama => "ollama",
            Provider::Custom(_) => "custom",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: Message,
    #[serde(default)]
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct StreamChunk {
    pub choices: Vec<StreamChoice>,
}

#[derive(Deserialize, Debug)]
pub struct StreamChoice {
    pub delta: Delta,
    #[serde(default)]
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    #[serde(default)]
    pub content: Option<String>,
}

pub struct ApiClient {
    client: Client,
    provider: Provider,
    api_key: Option<String>,
    #[allow(dead_code)]
    timeout: Duration,
}

impl ApiClient {
    pub fn new(provider: Provider, api_key: Option<String>, timeout: u64) -> Result<Self> {
        // Check if API key is required but not provided
        if provider.requires_api_key() && api_key.is_none() {
            // Try to get from environment
            let env_key = std::env::var("ECHOMIND_API_KEY").ok();
            if env_key.is_none() {
                return Err(EchomindError::MissingApiKey(provider.name().to_string()));
            }
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .map_err(|e| EchomindError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            provider,
            api_key: api_key.or_else(|| std::env::var("ECHOMIND_API_KEY").ok()),
            timeout: Duration::from_secs(timeout),
        })
    }

    // List available models for Gemini
    pub async fn list_models(&self) -> Result<Vec<GeminiModel>> {
        if let Provider::Gemini = self.provider {
            let base = self.provider.endpoint();
            let url = format!("{}/models", base);
            let api_key = self
                .api_key
                .clone()
                .ok_or_else(|| EchomindError::MissingApiKey("gemini".to_string()))?;

            let response = self
                .client
                .get(&url)
                .query(&[("key", api_key)])
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(EchomindError::ApiError { status, message: error_text });
            }

            let list: GeminiModelList = response.json().await?;
            Ok(list.models.unwrap_or_default())
        } else {
            Err(EchomindError::InvalidProvider(self.provider.name().to_string()))
        }
    }

    pub async fn send_message(&self, request: ChatRequest) -> Result<String> {
        match self.provider {
            Provider::Gemini => {
                // Build Gemini URL with model and API key as query param
                let model = request.model.unwrap_or_else(|| "gemini-1.5-pro-latest".to_string());
                let base = self.provider.endpoint();
                let url = format!("{}/models/{}:generateContent", base, model);

                let api_key = self
                    .api_key
                    .clone()
                    .ok_or_else(|| EchomindError::MissingApiKey("gemini".to_string()))?;

                let gemini_req = GeminiRequest::from_messages(&request.messages, request.temperature, request.max_tokens);

                let response = self
                    .client
                    .post(&url)
                    .query(&[("key", api_key)])
                    .json(&gemini_req)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(EchomindError::ApiError { status, message: error_text });
                }

                let resp: GeminiResponse = response.json().await?;
                resp.first_text().ok_or(EchomindError::EmptyResponse)
            }
            _ => {
                let endpoint = self.provider.endpoint();

                let mut req_builder = self.client.post(endpoint).json(&request);

                // Add authorization header if API key is available
                if let Some(ref key) = self.api_key {
                    req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
                }

                let response = req_builder.send().await?;

                // Check for API errors
                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(EchomindError::ApiError {
                        status,
                        message: error_text,
                    });
                }

                let chat_response: ChatResponse = response.json().await?;

                chat_response
                    .choices
                    .first()
                    .map(|choice| choice.message.content.clone())
                    .ok_or(EchomindError::EmptyResponse)
            }
        }
    }

    pub async fn send_message_stream<F>(&self, request: ChatRequest, mut callback: F) -> Result<String>
    where
        F: FnMut(&str),
    {
        // Gemini does not use OpenAI-style SSE here; emulate streaming by a single callback
        if let Provider::Gemini = self.provider {
            let text = self.send_message(request).await?;
            callback(&text);
            return Ok(text);
        }

        let endpoint = self.provider.endpoint();

        let mut req_builder = self.client.post(endpoint).json(&request);

        // Add authorization header if API key is available
        if let Some(ref key) = self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = req_builder.send().await?;

        // Check for API errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EchomindError::ApiError {
                status,
                message: error_text,
            });
        }

        let mut full_content = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| EchomindError::NetworkError(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);

            // Parse SSE format
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }

                    if let Ok(chunk_data) = serde_json::from_str::<StreamChunk>(data) {
                        if let Some(choice) = chunk_data.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                callback(content);
                                full_content.push_str(content);
                            }
                        }
                    }
                }
            }
        }

        Ok(full_content)
    }
}

// ===== Gemini types =====

#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generationConfig: Option<GeminiGenerationConfig>,
}

#[derive(Serialize, Debug)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    maxOutputTokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Debug)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Debug)]
struct GeminiPart {
    text: String,
}

impl GeminiRequest {
    fn from_messages(messages: &[Message], temperature: Option<f32>, max_tokens: Option<u32>) -> Self {
        let contents = messages
            .iter()
            .map(|m| GeminiContent {
                role: match m.role.as_str() {
                    "system" => "user".to_string(), // map system to user for simplicity
                    other => other.to_string(),
                },
                parts: vec![GeminiPart { text: m.content.clone() }],
            })
            .collect();

        let generationConfig = if temperature.is_some() || max_tokens.is_some() {
            Some(GeminiGenerationConfig {
                maxOutputTokens: max_tokens,
                temperature,
            })
        } else {
            None
        };

        GeminiRequest { contents, generationConfig }
    }
}

#[derive(Deserialize, Debug)]
struct GeminiResponseCandidate {
    content: Option<GeminiContentOut>,
}

#[derive(Deserialize, Debug)]
struct GeminiContentOut {
    parts: Option<Vec<GeminiPartOut>>,
}

#[derive(Deserialize, Debug)]
struct GeminiPartOut {
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiResponseCandidate>>,
}

impl GeminiResponse {
    fn first_text(self) -> Option<String> {
        self.candidates
            .and_then(|mut c| c.pop())
            .and_then(|cand| cand.content)
            .and_then(|c| c.parts)
            .and_then(|mut p| p.into_iter().find_map(|part| part.text))
    }
}

#[derive(Deserialize, Debug)]
pub struct GeminiModelList {
    pub models: Option<Vec<GeminiModel>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeminiModel {
    pub name: Option<String>,
    #[serde(default)]
    pub displayName: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub inputTokenLimit: Option<u32>,
    #[serde(default)]
    pub outputTokenLimit: Option<u32>,
}
