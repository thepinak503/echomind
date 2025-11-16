use crate::error::{EchomindError, Result};
use futures::StreamExt;
use lru::LruCache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Provider {
    Chat,
    ChatAnywhere,
    OpenAI,
    Claude,
    Ollama,
    Grok,
    Mistral,
    Cohere,
    Gemini,
    Custom(String),
}

impl Provider {
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "chat" => Ok(Provider::Chat),
            "chatanywhere" => Ok(Provider::ChatAnywhere),
            "openai" => Ok(Provider::OpenAI),
            "claude" => Ok(Provider::Claude),
            "ollama" => Ok(Provider::Ollama),
            "grok" => Ok(Provider::Grok),
            "mistral" => Ok(Provider::Mistral),
            "cohere" => Ok(Provider::Cohere),
            "gemini" => Ok(Provider::Gemini),
            s if s.starts_with("http") => Ok(Provider::Custom(s.to_string())),
            _ => Err(EchomindError::Other(format!("Unknown provider: {}", s))),
        }
    }

    pub fn endpoint(&self) -> &str {
        match self {
            Provider::Chat => "https://ch.at/v1/chat/completions",
            Provider::ChatAnywhere => "https://api.chatanywhere.tech/v1/chat/completions",
            Provider::OpenAI => "https://api.openai.com/v1/chat/completions",
            Provider::Claude => "https://api.anthropic.com/v1/messages",
            Provider::Ollama => "http://localhost:11434/api/chat",
            Provider::Grok => "https://api.x.ai/v1/chat/completions",
            Provider::Mistral => "https://api.mistral.ai/v1/chat/completions",
            Provider::Cohere => "https://api.cohere.ai/v1/chat",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta/models",
            Provider::Custom(url) => url,
        }
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, Provider::Chat | Provider::Ollama)
    }

    pub fn name(&self) -> &str {
        match self {
            Provider::Chat => "chat",
            Provider::ChatAnywhere => "chatanywhere",
            Provider::OpenAI => "openai",
            Provider::Claude => "claude",
            Provider::Ollama => "ollama",
            Provider::Grok => "grok",
            Provider::Mistral => "mistral",
            Provider::Cohere => "cohere",
            Provider::Gemini => "gemini",
            Provider::Custom(_) => "custom",
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GeminiModel {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
}

#[derive(Deserialize, Debug)]
pub struct GeminiModelList {
    pub models: Vec<GeminiModel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CohereRequest {
    pub message: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct CohereResponse {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

impl GeminiResponse {
    pub fn first_text(&self) -> Result<String> {
        self.candidates.first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or(EchomindError::EmptyResponse)
    }
}

#[derive(Deserialize, Debug)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
}

impl CohereRequest {
    pub fn from_chat_request(request: &ChatRequest) -> Self {
        Self {
            message: request.messages.last().map(|m| m.content.to_string()).unwrap_or_default(),
            model: request.model.clone().unwrap_or_else(|| "command".to_string()),
            temperature: request.temperature.unwrap_or(0.7),
            max_tokens: request.max_tokens,
        }
    }
}

impl GeminiRequest {
    pub fn from_chat_request(request: &ChatRequest) -> Result<Self> {
        let parts = request.messages.iter().map(|m| GeminiPart {
            text: m.content.to_string(),
        }).collect();

        Ok(Self {
            contents: vec![GeminiContent { parts }],
        })
    }
}

#[derive(Clone, Debug)]
struct CacheEntry {
    response: String,
    timestamp: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > self.ttl
    }
}

impl Clone for ApiClient {
    fn clone(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            provider: self.provider.clone(),
            api_key: self.api_key.clone(),
            timeout: self.timeout,
            cache: Arc::clone(&self.cache),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    // MultiModal(Vec<ContentPart>),
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageContent::Text(text) => write!(f, "{}", text),
            // MessageContent::MultiModal(parts) => {
            //     for part in parts {
            //         write!(f, "{}", part)?;
            //     }
            //     Ok(())
            // }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

impl Message {
    pub fn text(role: String, content: String) -> Self {
        Self {
            role,
            content: MessageContent::Text(content),
        }
    }

    // pub fn multimodal(role: String, parts: Vec<ContentPart>) -> Self {
    //     Self {
    //         role,
    //         content: MessageContent::MultiModal(parts),
    //     }
    // }

    pub fn get_text(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(text) => Some(text),
            // MessageContent::MultiModal(_) => None,
        }
    }
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "type")]
// pub enum ContentPart {
//     #[serde(rename = "text")]
//     Text { text: String },
//     #[serde(rename = "image_url")]
//     ImageUrl { image_url: ImageUrl },
// }

// impl std::fmt::Display for ContentPart {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ContentPart::Text { text } => write!(f, "{}", text),
//             ContentPart::ImageUrl { image_url } => write!(f, "[Image: {}]", image_url.url),
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct ImageUrl {
//     pub url: String,
// }

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
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
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

#[derive(Debug)]
pub struct ApiClient {
    client: Arc<Client>,
    provider: Provider,
    api_key: Option<String>,
    #[allow(dead_code)]
    timeout: Duration,
    cache: Arc<Mutex<LruCache<String, CacheEntry>>>,
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

        let client = Arc::new(Client::builder()
            .timeout(Duration::from_secs(timeout))
            .pool_max_idle_per_host(20) // Increased connection pooling
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
            .tcp_keepalive(Duration::from_secs(60)) // TCP keepalive
            .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
            .user_agent("echomind/0.3.0") // Set user agent
            .build()
            .map_err(|e| EchomindError::NetworkError(e.to_string()))?);

        Ok(Self {
            client,
            provider,
            api_key: api_key.or_else(|| std::env::var("ECHOMIND_API_KEY").ok()),
            timeout: Duration::from_secs(timeout),
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))), // Cache up to 100 entries
        })
    }

    // List available models for Gemini
    #[allow(dead_code)]
    pub async fn list_models(&self) -> Result<Vec<GeminiModel>> {
        if let Provider::Gemini = self.provider {
            let base = self.provider.endpoint();
            let url = format!("{}/v1beta/models", base);
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
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let suggestion = match status {
                    401 => "Check your Gemini API key is correct and has the right permissions.",
                    403 => "Your API key may not have access to this resource or may be expired.",
                    429 => "Rate limit exceeded. Try again later or reduce request frequency.",
                    500..=599 => "Server error. The Gemini API service may be down, try again later.",
                    _ => "Check the Gemini API documentation for this status code.",
                };
                return Err(EchomindError::ApiError {
                    status,
                    message: error_text,
                    suggestion: suggestion.to_string(),
                });
            }

            let list: GeminiModelList = response.json().await?;
            Ok(list.models)
        } else {
            Err(EchomindError::InvalidProvider(
                self.provider.name().to_string(),
            ))
        }
    }

    fn generate_cache_key(&self, request: &ChatRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.provider.name().hash(&mut hasher);
        request.model.hash(&mut hasher);
        if let Some(temp) = request.temperature {
            temp.to_bits().hash(&mut hasher);
        }
        if let Some(tokens) = request.max_tokens {
            tokens.hash(&mut hasher);
        }

        // Hash message contents
        for message in &request.messages {
            message.role.hash(&mut hasher);
            if let Some(text) = message.get_text() {
                text.hash(&mut hasher);
            }
        }

        format!("{:x}", hasher.finish())
    }

    pub async fn send_message(&self, request: ChatRequest) -> Result<String> {
        // Check cache first (only for non-streaming requests)
        if request.stream.is_none() || !request.stream.unwrap_or(false) {
            let cache_key = self.generate_cache_key(&request);
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(entry) = cache.get(&cache_key) {
                    if !entry.is_expired() {
                        return Ok(entry.response.clone());
                    } else {
                        cache.pop(&cache_key); // Remove expired entry
                    }
                }
            }
        }
        match self.provider {
            Provider::Cohere => {
                let endpoint = self.provider.endpoint();

                let cohere_req = CohereRequest::from_chat_request(&request);

                let mut req_builder = self.client.post(endpoint).json(&cohere_req);

                // Add authorization header if API key is available
                if let Some(ref key) = self.api_key {
                    req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
                }

                let response = req_builder.send().await?;

        // Check for API errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let suggestion = match status {
                401 => "Check your API key is correct and has the right permissions.",
                403 => "Your API key may not have access to this resource or may be expired.",
                429 => "Rate limit exceeded. Try again later or reduce request frequency.",
                500..=599 => "Server error. The API service may be down, try again later.",
                _ => "Check the API documentation for this status code.",
            };
            return Err(EchomindError::ApiError {
                status,
                message: error_text,
                suggestion: suggestion.to_string(),
            });
        }

        let cohere_response: CohereResponse = response.json().await?;

        let result = cohere_response.text;

        // Cache the result
        if request.stream.is_none() || !request.stream.unwrap_or(false) {
            let cache_key = self.generate_cache_key(&request);
            if let Ok(mut cache) = self.cache.lock() {
                let entry = CacheEntry {
                    response: result.clone(),
                    timestamp: Instant::now(),
                    ttl: Duration::from_secs(300), // 5 minute TTL
                };
                cache.put(cache_key, entry);
            }
        }

        return Ok(result);
            },
            Provider::Gemini => {
                let base_endpoint = self.provider.endpoint();
                let model = request.model.as_deref().unwrap_or("gemini-pro");
                let endpoint = format!("{}/v1beta/models/{}:generateContent", base_endpoint, model);

                let gemini_request = GeminiRequest::from_chat_request(&request)?;

                let mut req_builder = self.client.post(&endpoint).json(&gemini_request);

                // Add API key as query parameter
                if let Some(ref key) = self.api_key {
                    req_builder = req_builder.query(&[("key", key)]);
                }

                let response = req_builder.send().await?;

                // Check for API errors
                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let suggestion = match status {
                        400 => "Check your request format and model name.",
                        401 => "Check your Gemini API key is correct and has the right permissions.",
                        403 => "Your API key may not have access to this resource or may be expired.",
                        429 => "Rate limit exceeded. Try again later or reduce request frequency.",
                        500..=599 => "Server error. The Gemini API service may be down, try again later.",
                        _ => "Check the Gemini API documentation for this status code.",
                    };
                    return Err(EchomindError::ApiError {
                        status,
                        message: error_text,
                        suggestion: suggestion.to_string(),
                    });
                }

                let resp: GeminiResponse = response.json().await?;
                let result = resp.first_text()?;

                // Cache the result
                if request.stream.is_none() || !request.stream.unwrap_or(false) {
                    let cache_key = self.generate_cache_key(&request);
                    if let Ok(mut cache) = self.cache.lock() {
                        let entry = CacheEntry {
                            response: result.clone(),
                            timestamp: Instant::now(),
                            ttl: Duration::from_secs(300), // 5 minute TTL
                        };
                        cache.put(cache_key, entry);
                    }
                }

                return Ok(result);
            },
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
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let suggestion = match status {
                    401 => "Check your API key is correct and has the right permissions.",
                    403 => "Your API key may not have access to this resource or may be expired.",
                    429 => "Rate limit exceeded. Try again later or reduce request frequency.",
                    500..=599 => "Server error. The API service may be down, try again later.",
                    _ => "Check the API documentation for this status code.",
                };
                return Err(EchomindError::ApiError {
                    status,
                    message: error_text,
                    suggestion: suggestion.to_string(),
                });
            }

            let chat_response: ChatResponse = response.json().await?;

            let result = chat_response
                .choices
                .first()
                .and_then(|choice| choice.message.get_text())
                .map(|s| s.to_string())
                .ok_or(EchomindError::EmptyResponse)?;

            // Cache the result
            if request.stream.is_none() || !request.stream.unwrap_or(false) {
                let cache_key = self.generate_cache_key(&request);
                if let Ok(mut cache) = self.cache.lock() {
                    let entry = CacheEntry {
                        response: result.clone(),
                        timestamp: Instant::now(),
                        ttl: Duration::from_secs(300), // 5 minute TTL
                    };
                    cache.put(cache_key, entry);
                }
            }

            return Ok(result);
            },
        }
    }

    pub async fn send_message_stream<F>(
        &self,
        request: ChatRequest,
        mut callback: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        // Cohere and Gemini do not use OpenAI-style SSE here; emulate streaming by a single callback
        if matches!(self.provider, Provider::Gemini | Provider::Cohere) {
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let suggestion = match status {
                401 => "Check your API key is correct and has the right permissions.",
                403 => "Your API key may not have access to this resource or may be expired.",
                429 => "Rate limit exceeded. Try again later or reduce request frequency.",
                500..=599 => "Server error. The API service may be down, try again later.",
                _ => "Check the API documentation for this status code.",
            };
            return Err(EchomindError::ApiError {
                status,
                message: error_text,
                suggestion: suggestion.to_string(),
            });
        }

        let mut full_content = String::with_capacity(4096); // Pre-allocate reasonable capacity
        let mut stream = response.bytes_stream();
        let mut buffer = String::with_capacity(1024); // Buffer for accumulating partial lines

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| EchomindError::NetworkError(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);

            // Accumulate text in buffer to handle partial lines
            buffer.push_str(&text);

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim_end().to_string();
                let remaining = buffer[newline_pos + 1..].to_string();
                buffer = remaining;

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

        // Process any remaining content in buffer
        for line in buffer.lines() {
            let line = line.trim_end();
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data != "[DONE]" {
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
