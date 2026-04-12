use crate::error::{EchomindError, Result};
use futures::StreamExt;
use lru::LruCache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CACHE_TTL_SECS: u64 = 300;
const CACHE_CAPACITY: usize = 100;
const DEFAULT_CLAUDE_MODEL: &str = "claude-3-sonnet-20240229";
const DEFAULT_GEMINI_MODEL: &str = "gemini-pro";
const DEFAULT_COHERE_MODEL: &str = "command";
const USER_AGENT: &str = concat!("echomind/", env!("CARGO_PKG_VERSION"));

// ── Provider ──────────────────────────────────────────────────────────

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
            "chat" => Ok(Self::Chat),
            "chatanywhere" => Ok(Self::ChatAnywhere),
            "openai" => Ok(Self::OpenAI),
            "claude" => Ok(Self::Claude),
            "ollama" => Ok(Self::Ollama),
            "grok" => Ok(Self::Grok),
            "mistral" => Ok(Self::Mistral),
            "cohere" => Ok(Self::Cohere),
            "gemini" => Ok(Self::Gemini),
            s if s.starts_with("http") => Ok(Self::Custom(s.to_string())),
            _ => Err(EchomindError::InvalidProvider(s.to_string())),
        }
    }

    pub fn endpoint(&self) -> &str {
        match self {
            Self::Chat => "https://ch.at/v1/chat/completions",
            Self::ChatAnywhere => "https://api.chatanywhere.tech/v1/chat/completions",
            Self::OpenAI => "https://api.openai.com/v1/chat/completions",
            Self::Claude => "https://api.anthropic.com/v1/messages",
            Self::Ollama => "http://localhost:11434/api/chat",
            Self::Grok => "https://api.x.ai/v1/chat/completions",
            Self::Mistral => "https://api.mistral.ai/v1/chat/completions",
            Self::Cohere => "https://api.cohere.ai/v1/chat",
            Self::Gemini => "https://generativelanguage.googleapis.com",
            Self::Custom(url) => url,
        }
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, Self::Chat | Self::Ollama)
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Chat => "chat",
            Self::ChatAnywhere => "chatanywhere",
            Self::OpenAI => "openai",
            Self::Claude => "claude",
            Self::Ollama => "ollama",
            Self::Grok => "grok",
            Self::Mistral => "mistral",
            Self::Cohere => "cohere",
            Self::Gemini => "gemini",
            Self::Custom(_) => "custom",
        }
    }
}

// ── Message types (shared) ────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
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
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

// ── OpenAI-compatible response types ──────────────────────────────────

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize, Debug)]
struct OpenAIChoice {
    message: Message,
}

#[derive(Deserialize, Debug)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize, Debug)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize, Debug)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

// ── Claude types ──────────────────────────────────────────────────────

#[derive(Serialize, Debug)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Debug)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

#[derive(Deserialize, Debug)]
struct ClaudeContentBlock {
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<ClaudeDelta>,
}

#[derive(Deserialize, Debug)]
struct ClaudeDelta {
    text: Option<String>,
}

impl ClaudeRequest {
    fn from_chat(req: &ChatRequest) -> Self {
        let mut system = None;
        let mut messages = Vec::with_capacity(req.messages.len());

        for msg in &req.messages {
            if msg.role == "system" {
                system = Some(msg.content.clone());
            } else {
                messages.push(ClaudeMessage {
                    role: msg.role.clone(),
                    content: msg.content.clone(),
                });
            }
        }

        Self {
            model: req.model.clone().unwrap_or_else(|| DEFAULT_CLAUDE_MODEL.into()),
            messages,
            max_tokens: req.max_tokens.unwrap_or(4096),
            system,
            temperature: req.temperature,
            stream: req.stream,
        }
    }
}

// ── Cohere types ──────────────────────────────────────────────────────

#[derive(Serialize, Debug)]
struct CohereRequest {
    message: String,
    model: String,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct CohereResponse {
    text: String,
}

impl CohereRequest {
    fn from_chat(req: &ChatRequest) -> Self {
        Self {
            message: req.messages.last().map(|m| m.content.clone()).unwrap_or_default(),
            model: req.model.clone().unwrap_or_else(|| DEFAULT_COHERE_MODEL.into()),
            temperature: req.temperature.unwrap_or(0.7),
            max_tokens: req.max_tokens,
        }
    }
}

// ── Gemini types ──────────────────────────────────────────────────────

#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}

impl GeminiResponse {
    fn first_text(&self) -> Result<String> {
        self.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or(EchomindError::EmptyResponse)
    }
}

impl GeminiRequest {
    fn from_chat(req: &ChatRequest) -> Self {
        Self {
            contents: vec![GeminiContent {
                parts: req.messages.iter().map(|m| GeminiPart { text: m.content.clone() }).collect(),
            }],
        }
    }
}

// ── Cache ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct CacheEntry {
    response: String,
    timestamp: Instant,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > Duration::from_secs(CACHE_TTL_SECS)
    }
}

// ── ApiClient ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ApiClient {
    client: Arc<Client>,
    provider: Provider,
    api_key: Option<String>,
    cache: Arc<Mutex<LruCache<u64, CacheEntry>>>,
}

impl std::fmt::Debug for ApiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiClient")
            .field("provider", &self.provider)
            .field("has_api_key", &self.api_key.is_some())
            .finish()
    }
}

impl ApiClient {
    pub fn new(provider: Provider, api_key: Option<String>, timeout: u64) -> Result<Self> {
        let api_key = api_key.or_else(|| std::env::var("ECHOMIND_API_KEY").ok());

        if provider.requires_api_key() && api_key.is_none() {
            return Err(EchomindError::MissingApiKey(provider.name().to_string()));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| EchomindError::NetworkError(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
            provider,
            api_key,
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_CAPACITY).unwrap(),
            ))),
        })
    }

    // ── Cache helpers ─────────────────────────────────────────────────

    fn cache_key(&self, request: &ChatRequest) -> u64 {
        let mut h = DefaultHasher::new();
        self.provider.name().hash(&mut h);
        request.model.hash(&mut h);
        if let Some(t) = request.temperature {
            t.to_bits().hash(&mut h);
        }
        request.max_tokens.hash(&mut h);
        for m in &request.messages {
            m.role.hash(&mut h);
            m.content.hash(&mut h);
        }
        h.finish()
    }

    fn get_cached(&self, key: u64) -> Option<String> {
        let mut cache = self.cache.lock().ok()?;
        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                return Some(entry.response.clone());
            }
            cache.pop(&key);
        }
        None
    }

    fn put_cache(&self, key: u64, response: String) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key, CacheEntry {
                response,
                timestamp: Instant::now(),
            });
        }
    }

    // ── Error helper ──────────────────────────────────────────────────

    fn api_error_suggestion(status: u16) -> &'static str {
        match status {
            400 => "Check your request format and parameters.",
            401 => "Check your API key is correct and has the right permissions.",
            403 => "Your API key may not have access to this resource or may be expired.",
            429 => "Rate limit exceeded. Try again later or reduce request frequency.",
            500..=599 => "Server error. The API service may be down, try again later.",
            _ => "Check the API documentation for this status code.",
        }
    }

    async fn check_response(response: reqwest::Response) -> Result<reqwest::Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status().as_u16();
        let message = response.text().await.unwrap_or_else(|_| "Unknown error".into());
        Err(EchomindError::ApiError {
            status,
            message,
            suggestion: Self::api_error_suggestion(status).into(),
        })
    }

    // ── send_message (non-streaming) ──────────────────────────────────

    pub async fn send_message(&self, request: ChatRequest) -> Result<String> {
        let is_cacheable = !request.stream.unwrap_or(false);
        let key = if is_cacheable { self.cache_key(&request) } else { 0 };

        if is_cacheable {
            if let Some(cached) = self.get_cached(key) {
                return Ok(cached);
            }
        }

        let result = match self.provider {
            Provider::Claude => self.send_claude(&request).await?,
            Provider::Gemini => self.send_gemini(&request).await?,
            Provider::Cohere => self.send_cohere(&request).await?,
            _ => self.send_openai_compat(&request).await?,
        };

        if is_cacheable {
            self.put_cache(key, result.clone());
        }
        Ok(result)
    }

    // ── Provider-specific non-streaming ───────────────────────────────

    async fn send_openai_compat(&self, request: &ChatRequest) -> Result<String> {
        let mut rb = self.client.post(self.provider.endpoint()).json(request);
        if let Some(ref key) = self.api_key {
            rb = rb.header("Authorization", format!("Bearer {}", key));
        }
        let resp = Self::check_response(rb.send().await?).await?;
        let data: OpenAIResponse = resp.json().await?;
        data.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(EchomindError::EmptyResponse)
    }

    async fn send_claude(&self, request: &ChatRequest) -> Result<String> {
        let claude_req = ClaudeRequest::from_chat(request);
        let mut rb = self.client.post(self.provider.endpoint()).json(&claude_req);
        if let Some(ref key) = self.api_key {
            rb = rb.header("x-api-key", key).header("anthropic-version", "2023-06-01");
        }
        let resp = Self::check_response(rb.send().await?).await?;
        let data: ClaudeResponse = resp.json().await?;
        data.content
            .into_iter()
            .next()
            .and_then(|c| c.text)
            .ok_or(EchomindError::EmptyResponse)
    }

    async fn send_gemini(&self, request: &ChatRequest) -> Result<String> {
        let model = request.model.as_deref().unwrap_or(DEFAULT_GEMINI_MODEL);
        let url = format!("{}/v1beta/models/{}:generateContent", self.provider.endpoint(), model);
        let gemini_req = GeminiRequest::from_chat(request);
        let mut rb = self.client.post(&url).json(&gemini_req);
        if let Some(ref key) = self.api_key {
            rb = rb.query(&[("key", key)]);
        }
        let resp = Self::check_response(rb.send().await?).await?;
        let data: GeminiResponse = resp.json().await?;
        data.first_text()
    }

    async fn send_cohere(&self, request: &ChatRequest) -> Result<String> {
        let cohere_req = CohereRequest::from_chat(request);
        let mut rb = self.client.post(self.provider.endpoint()).json(&cohere_req);
        if let Some(ref key) = self.api_key {
            rb = rb.header("Authorization", format!("Bearer {}", key));
        }
        let resp = Self::check_response(rb.send().await?).await?;
        let data: CohereResponse = resp.json().await?;
        Ok(data.text)
    }

    // ── send_message_stream ───────────────────────────────────────────

    pub async fn send_message_stream<F>(&self, request: ChatRequest, callback: F) -> Result<String>
    where
        F: FnMut(&str),
    {
        match self.provider {
            Provider::Cohere => {
                let text = self.send_message(request).await?;
                let mut cb = callback;
                cb(&text);
                Ok(text)
            }
            Provider::Claude => self.stream_claude(&request, callback).await,
            Provider::Gemini => self.stream_gemini(&request, callback).await,
            _ => self.stream_openai_compat(&request, callback).await,
        }
    }

    // ── SSE helpers ───────────────────────────────────────────────────

    async fn stream_openai_compat<F>(&self, request: &ChatRequest, mut cb: F) -> Result<String>
    where
        F: FnMut(&str),
    {
        let mut rb = self.client.post(self.provider.endpoint()).json(request);
        if let Some(ref key) = self.api_key {
            rb = rb.header("Authorization", format!("Bearer {}", key));
        }
        let resp = Self::check_response(rb.send().await?).await?;

        let mut full = String::with_capacity(4096);
        let mut buf = String::with_capacity(1024);
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| EchomindError::NetworkError(e.to_string()))?;
            buf.push_str(&String::from_utf8_lossy(&chunk));
            self.process_sse_lines(&mut buf, &mut full, &mut |text| cb(text), |data| {
                if let Ok(c) = serde_json::from_str::<StreamChunk>(data) {
                    return c.choices.into_iter().next().and_then(|ch| ch.delta.content);
                }
                None
            });
        }
        // drain remainder
        self.process_sse_lines(&mut buf, &mut full, &mut |text| cb(text), |data| {
            serde_json::from_str::<StreamChunk>(data)
                .ok()
                .and_then(|c| c.choices.into_iter().next().and_then(|ch| ch.delta.content))
        });

        Ok(full)
    }

    async fn stream_claude<F>(&self, request: &ChatRequest, mut cb: F) -> Result<String>
    where
        F: FnMut(&str),
    {
        let mut claude_req = ClaudeRequest::from_chat(request);
        claude_req.stream = Some(true);
        let mut rb = self.client.post(self.provider.endpoint()).json(&claude_req);
        if let Some(ref key) = self.api_key {
            rb = rb.header("x-api-key", key).header("anthropic-version", "2023-06-01");
        }
        let resp = Self::check_response(rb.send().await?).await?;

        let mut full = String::with_capacity(4096);
        let mut buf = String::new();
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| EchomindError::NetworkError(e.to_string()))?;
            buf.push_str(&String::from_utf8_lossy(&chunk));
            self.process_sse_lines(&mut buf, &mut full, &mut |text| cb(text), |data| {
                if let Ok(ev) = serde_json::from_str::<ClaudeStreamEvent>(data) {
                    if ev.event_type == "content_block_delta" {
                        return ev.delta.and_then(|d| d.text);
                    }
                }
                None
            });
        }

        Ok(full)
    }

    async fn stream_gemini<F>(&self, request: &ChatRequest, mut cb: F) -> Result<String>
    where
        F: FnMut(&str),
    {
        let model = request.model.as_deref().unwrap_or(DEFAULT_GEMINI_MODEL);
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
            self.provider.endpoint(),
            model
        );
        let gemini_req = GeminiRequest::from_chat(request);
        let mut rb = self.client.post(&url).json(&gemini_req);
        if let Some(ref key) = self.api_key {
            rb = rb.query(&[("key", key)]);
        }
        let resp = Self::check_response(rb.send().await?).await?;

        let mut full = String::with_capacity(4096);
        let mut buf = String::new();
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| EchomindError::NetworkError(e.to_string()))?;
            buf.push_str(&String::from_utf8_lossy(&chunk));
            self.process_sse_lines(&mut buf, &mut full, &mut |text| cb(text), |data| {
                serde_json::from_str::<GeminiResponse>(data)
                    .ok()
                    .and_then(|r| r.first_text().ok())
            });
        }

        Ok(full)
    }

    /// Generic SSE line processor. Extracts `data:` lines, skips `[DONE]`,
    /// calls `extractor` to pull text from JSON, then invokes `cb`.
    fn process_sse_lines<F, E>(
        &self,
        buf: &mut String,
        full: &mut String,
        cb: &mut F,
        extractor: E,
    ) where
        F: FnMut(&str),
        E: Fn(&str) -> Option<String>,
    {
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            buf.drain(..=pos);

            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    continue;
                }
                if let Some(text) = extractor(data) {
                    cb(&text);
                    full.push_str(&text);
                }
            }
        }
    }
}
