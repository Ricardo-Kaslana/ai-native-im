//! LLM Client Module
//!
//! Provides unified interface for multiple LLM providers:
//! - OpenAI (GPT-3.5, GPT-4, etc.)
//! - Anthropic (Claude)
//! - Google (Gemini)
//! - Local/Custom (Ollama, vLLM, etc.)
//!
//! Features:
//! - Unified API across providers
//! - Conversation history support
//! - Streaming response support (via callbacks)
//! - Automatic retry with exponential backoff
//! - Rate limiting handling

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use super::AIAgentError;

/// LLM Provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Google,
    SiliconFlow,
    Custom,
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProvider::OpenAI => write!(f, "OpenAI"),
            LLMProvider::Anthropic => write!(f, "Anthropic"),
            LLMProvider::Google => write!(f, "Google"),
            LLMProvider::SiliconFlow => write!(f, "SiliconFlow"),
            LLMProvider::Custom => write!(f, "Custom"),
        }
    }
}

/// LLM client configuration
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: String,
    pub api_base: Option<String>,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl LLMConfig {
    /// Create OpenAI configuration
    pub fn openai(api_key: String, model: Option<String>) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key,
            api_base: None,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            max_tokens: 500,
            temperature: 0.8,
            timeout_secs: 30,
            max_retries: 3,
        }
    }

    /// Create Anthropic Claude configuration
    pub fn anthropic(api_key: String, model: Option<String>) -> Self {
        Self {
            provider: LLMProvider::Anthropic,
            api_key,
            api_base: None,
            model: model.unwrap_or_else(|| "claude-3-haiku-20240307".to_string()),
            max_tokens: 500,
            temperature: 0.8,
            timeout_secs: 30,
            max_retries: 3,
        }
    }

    /// Create Google Gemini configuration
    pub fn google(api_key: String, model: Option<String>) -> Self {
        Self {
            provider: LLMProvider::Google,
            api_key,
            api_base: None,
            model: model.unwrap_or_else(|| "gemini-pro".to_string()),
            max_tokens: 500,
            temperature: 0.8,
            timeout_secs: 30,
            max_retries: 3,
        }
    }

    /// Create custom/local LLM configuration (OpenAI-compatible API)
    pub fn custom(api_base: String, model: String, api_key: Option<String>) -> Self {
        Self {
            provider: LLMProvider::Custom,
            api_key: api_key.unwrap_or_default(),
            api_base: Some(api_base),
            model,
            max_tokens: 500,
            temperature: 0.8,
            timeout_secs: 60,
            max_retries: 3,
        }
    }

    /// Create SiliconFlow configuration
    /// SiliconFlow (硅基流动) is a Chinese AI platform providing various open-source models
    /// API is OpenAI-compatible
    pub fn siliconflow(api_key: String, model: Option<String>) -> Self {
        Self {
            provider: LLMProvider::SiliconFlow,
            api_key,
            api_base: Some("https://api.siliconflow.cn/v1".to_string()),
            // Default to Qwen2.5-7B-Instruct, a popular Chinese model
            model: model.unwrap_or_else(|| "Qwen/Qwen2.5-7B-Instruct".to_string()),
            max_tokens: 500,
            temperature: 0.8,
            timeout_secs: 60,
            max_retries: 3,
        }
    }

    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = Some(api_base);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// Chat message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// A single chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// Streaming chunk from LLM
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub is_final: bool,
}

/// LLM client trait
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Generate a response given system prompt and user message
    async fn generate_response(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, AIAgentError>;

    /// Generate a response with full conversation history
    async fn generate_with_history(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String, AIAgentError>;

    /// Generate a streaming response (returns a channel receiver)
    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError>;

    /// Get the provider name
    fn provider(&self) -> LLMProvider;

    /// Get the model name
    fn model(&self) -> &str;

    /// Get configuration
    fn config(&self) -> &LLMConfig;
}

// ============== OpenAI Client ==============

/// OpenAI-compatible LLM client (also works with Azure OpenAI, local models with OpenAI API)
pub struct OpenAIClient {
    client: Client,
    config: LLMConfig,
}

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIChatMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
}

impl OpenAIClient {
    pub fn new(api_key: String, api_base: Option<String>, model: Option<String>) -> Self {
        let mut config = LLMConfig::openai(api_key, model);
        if let Some(base) = api_base {
            config = config.with_api_base(base);
        }
        Self::from_config(config)
    }

    pub fn from_config(config: LLMConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.timeout_secs = timeout_secs;
        self
    }

    fn api_base(&self) -> &str {
        self.config
            .api_base
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
    }

    fn convert_messages(&self, messages: &[ChatMessage]) -> Vec<OpenAIChatMessage> {
        messages
            .iter()
            .map(|m| OpenAIChatMessage {
                role: match m.role {
                    ChatRole::System => "system".to_string(),
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }

    async fn call_api(&self, system_prompt: &str, user_message: &str) -> Result<String, AIAgentError> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ];
        self.call_api_with_messages(&messages).await
    }

    async fn call_api_with_messages(&self, messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        let request = OpenAIChatRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_base()))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        handle_response::<OpenAIChatResponse>(response, |r| {
            r.choices.first().map(|c| c.message.content.clone())
        })
        .await
    }

    async fn call_api_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        let request = OpenAIChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAIChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenAIChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: Some(true),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_base()))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::LLMError(format!(
                "API returned error: {}",
                error_text
            )));
        }

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));

                        // Process SSE lines
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer = buffer[line_end + 1..].to_string();

                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    let _ = tx.send(StreamChunk {
                                        content: String::new(),
                                        is_final: true,
                                    }).await;
                                    return;
                                }

                                if let Ok(parsed) = serde_json::from_str::<OpenAIStreamResponse>(data) {
                                    if let Some(choice) = parsed.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            let _ = tx.send(StreamChunk {
                                                content: content.clone(),
                                                is_final: false,
                                            }).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn generate_response(&self, system_prompt: &str, user_message: &str) -> Result<String, AIAgentError> {
        retry_with_backoff(self.config.max_retries, || {
            self.call_api(system_prompt, user_message)
        })
        .await
    }

    async fn generate_with_history(&self, messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        let messages = messages.to_vec();
        retry_with_backoff(self.config.max_retries, || {
            self.call_api_with_messages(&messages)
        })
        .await
    }

    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        self.call_api_stream(system_prompt, user_message).await
    }

    fn provider(&self) -> LLMProvider {
        self.config.provider
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============== Anthropic Claude Client ==============

/// Anthropic Claude LLM client
pub struct AnthropicClient {
    client: Client,
    config: LLMConfig,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicStreamDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    text: Option<String>,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self::from_config(LLMConfig::anthropic(api_key, model))
    }

    pub fn from_config(config: LLMConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    fn api_base(&self) -> &str {
        self.config
            .api_base
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1")
    }

    fn convert_messages(&self, messages: &[ChatMessage]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_prompt = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            match msg.role {
                ChatRole::System => {
                    system_prompt = Some(msg.content.clone());
                }
                ChatRole::User => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                ChatRole::Assistant => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        (system_prompt, anthropic_messages)
    }

    async fn call_api(&self, system_prompt: &str, user_message: &str) -> Result<String, AIAgentError> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ];
        self.call_api_with_messages(&messages).await
    }

    async fn call_api_with_messages(&self, messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        let (system, anthropic_messages) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages: anthropic_messages,
            system,
            stream: None,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.api_base()))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        handle_response::<AnthropicResponse>(response, |r| {
            r.content.first().map(|c| c.text.clone())
        })
        .await
    }

    async fn call_api_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        let request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
            system: Some(system_prompt.to_string()),
            stream: Some(true),
        };

        let response = self
            .client
            .post(format!("{}/messages", self.api_base()))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::LLMError(format!(
                "API returned error: {}",
                error_text
            )));
        }

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));

                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer = buffer[line_end + 1..].to_string();

                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                                    match event.event_type.as_str() {
                                        "content_block_delta" => {
                                            if let Some(delta) = event.delta {
                                                if let Some(text) = delta.text {
                                                    let _ = tx.send(StreamChunk {
                                                        content: text,
                                                        is_final: false,
                                                    }).await;
                                                }
                                            }
                                        }
                                        "message_stop" => {
                                            let _ = tx.send(StreamChunk {
                                                content: String::new(),
                                                is_final: true,
                                            }).await;
                                            return;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[async_trait]
impl LLMClient for AnthropicClient {
    async fn generate_response(&self, system_prompt: &str, user_message: &str) -> Result<String, AIAgentError> {
        retry_with_backoff(self.config.max_retries, || {
            self.call_api(system_prompt, user_message)
        })
        .await
    }

    async fn generate_with_history(&self, messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        let messages = messages.to_vec();
        retry_with_backoff(self.config.max_retries, || {
            self.call_api_with_messages(&messages)
        })
        .await
    }

    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        self.call_api_stream(system_prompt, user_message).await
    }

    fn provider(&self) -> LLMProvider {
        LLMProvider::Anthropic
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============== Google Gemini Client ==============

/// Google Gemini LLM client
pub struct GoogleGeminiClient {
    client: Client,
    config: LLMConfig,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiGenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: String,
}

impl GoogleGeminiClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self::from_config(LLMConfig::google(api_key, model))
    }

    pub fn from_config(config: LLMConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    fn api_base(&self) -> &str {
        self.config
            .api_base
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com/v1beta")
    }

    fn api_url(&self, stream: bool) -> String {
        let action = if stream { "streamGenerateContent" } else { "generateContent" };
        format!(
            "{}/models/{}:{}?key={}",
            self.api_base(), self.config.model, action, self.config.api_key
        )
    }

    fn convert_messages(&self, messages: &[ChatMessage]) -> Vec<GeminiContent> {
        let mut contents = Vec::new();
        let mut system_instruction = String::new();

        for msg in messages {
            match msg.role {
                ChatRole::System => {
                    system_instruction = msg.content.clone();
                }
                ChatRole::User => {
                    // Prepend system instruction to first user message if present
                    let text = if !system_instruction.is_empty() && contents.is_empty() {
                        let combined = format!("{}\n\n{}", system_instruction, msg.content);
                        system_instruction.clear();
                        combined
                    } else {
                        msg.content.clone()
                    };
                    contents.push(GeminiContent {
                        parts: vec![GeminiPart { text }],
                        role: Some("user".to_string()),
                    });
                }
                ChatRole::Assistant => {
                    contents.push(GeminiContent {
                        parts: vec![GeminiPart {
                            text: msg.content.clone(),
                        }],
                        role: Some("model".to_string()),
                    });
                }
            }
        }

        contents
    }

    async fn call_api(&self, system_prompt: &str, user_message: &str) -> Result<String, AIAgentError> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ];
        self.call_api_with_messages(&messages).await
    }

    async fn call_api_with_messages(&self, messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        let request = GeminiRequest {
            contents: self.convert_messages(messages),
            generation_config: GeminiGenerationConfig {
                max_output_tokens: self.config.max_tokens,
                temperature: self.config.temperature,
            },
        };

        let response = self
            .client
            .post(self.api_url(false))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        handle_response::<GeminiResponse>(response, |r| {
            r.candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .map(|p| p.text.clone())
        })
        .await
    }

    async fn call_api_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        let combined_message = format!("{}\n\nUser: {}", system_prompt, user_message);

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: combined_message,
                }],
                role: Some("user".to_string()),
            }],
            generation_config: GeminiGenerationConfig {
                max_output_tokens: self.config.max_tokens,
                temperature: self.config.temperature,
            },
        };

        let response = self
            .client
            .post(self.api_url(true))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::LLMError(format!(
                "API returned error: {}",
                error_text
            )));
        }

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));

                        // Gemini streams JSON array elements
                        // Try to parse complete JSON objects
                        if let Ok(responses) = serde_json::from_str::<Vec<GeminiResponse>>(&buffer) {
                            for resp in responses {
                                if let Some(candidate) = resp.candidates.first() {
                                    if let Some(part) = candidate.content.parts.first() {
                                        let _ = tx.send(StreamChunk {
                                            content: part.text.clone(),
                                            is_final: false,
                                        }).await;
                                    }
                                }
                            }
                            buffer.clear();
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }

            let _ = tx.send(StreamChunk {
                content: String::new(),
                is_final: true,
            }).await;
        });

        Ok(rx)
    }
}

#[async_trait]
impl LLMClient for GoogleGeminiClient {
    async fn generate_response(&self, system_prompt: &str, user_message: &str) -> Result<String, AIAgentError> {
        retry_with_backoff(self.config.max_retries, || {
            self.call_api(system_prompt, user_message)
        })
        .await
    }

    async fn generate_with_history(&self, messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        let messages = messages.to_vec();
        retry_with_backoff(self.config.max_retries, || {
            self.call_api_with_messages(&messages)
        })
        .await
    }

    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        self.call_api_stream(system_prompt, user_message).await
    }

    fn provider(&self) -> LLMProvider {
        LLMProvider::Google
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============== SiliconFlow Client ==============

/// SiliconFlow (硅基流动) LLM client
/// Uses OpenAI-compatible API format
/// Supports models like Qwen, DeepSeek, GLM, etc.
pub struct SiliconFlowClient {
    client: Client,
    config: LLMConfig,
}

impl SiliconFlowClient {
    /// Create a new SiliconFlow client
    /// Default model: Qwen/Qwen2.5-7B-Instruct
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self::from_config(LLMConfig::siliconflow(api_key, model))
    }

    pub fn from_config(config: LLMConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    fn api_base(&self) -> &str {
        self.config
            .api_base
            .as_deref()
            .unwrap_or("https://api.siliconflow.cn/v1")
    }

    fn convert_messages(&self, messages: &[ChatMessage]) -> Vec<OpenAIChatMessage> {
        messages
            .iter()
            .map(|m| OpenAIChatMessage {
                role: match m.role {
                    ChatRole::System => "system".to_string(),
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }

    async fn call_api(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, AIAgentError> {
        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ];
        self.call_api_with_messages(&messages).await
    }

    async fn call_api_with_messages(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String, AIAgentError> {
        let request = OpenAIChatRequest {
            model: self.config.model.clone(),
            messages: self.convert_messages(messages),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_base()))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        handle_response::<OpenAIChatResponse>(response, |r| {
            r.choices.first().map(|c| c.message.content.clone())
        })
        .await
    }

    async fn call_api_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        let request = OpenAIChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAIChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenAIChatMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: Some(true),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_base()))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.config.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::LLMError(format!(
                "API returned error: {}",
                error_text
            )));
        }

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));

                        // Process SSE lines
                        while let Some(line_end) = buffer.find('\n') {
                            let line = buffer[..line_end].trim().to_string();
                            buffer = buffer[line_end + 1..].to_string();

                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    let _ = tx
                                        .send(StreamChunk {
                                            content: String::new(),
                                            is_final: true,
                                        })
                                        .await;
                                    return;
                                }

                                if let Ok(parsed) =
                                    serde_json::from_str::<OpenAIStreamResponse>(data)
                                {
                                    if let Some(choice) = parsed.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            let _ = tx
                                                .send(StreamChunk {
                                                    content: content.clone(),
                                                    is_final: false,
                                                })
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[async_trait]
impl LLMClient for SiliconFlowClient {
    async fn generate_response(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, AIAgentError> {
        retry_with_backoff(self.config.max_retries, || {
            self.call_api(system_prompt, user_message)
        })
        .await
    }

    async fn generate_with_history(
        &self,
        messages: &[ChatMessage],
    ) -> Result<String, AIAgentError> {
        let messages = messages.to_vec();
        retry_with_backoff(self.config.max_retries, || {
            self.call_api_with_messages(&messages)
        })
        .await
    }

    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        self.call_api_stream(system_prompt, user_message).await
    }

    fn provider(&self) -> LLMProvider {
        LLMProvider::SiliconFlow
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============== Helper Functions ==============

/// Handle HTTP response and extract content
async fn handle_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    extract: impl FnOnce(T) -> Option<String>,
) -> Result<String, AIAgentError> {
    let status = response.status();

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        return Err(AIAgentError::LLMRateLimited(retry_after));
    }

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AIAgentError::LLMError(format!(
            "API returned status {}: {}",
            status, error_text
        )));
    }

    let parsed: T = response
        .json()
        .await
        .map_err(|e| AIAgentError::InvalidLLMResponse(e.to_string()))?;

    let content = extract(parsed)
        .ok_or_else(|| AIAgentError::InvalidLLMResponse("No content in response".to_string()))?;

    if content.is_empty() {
        return Err(AIAgentError::InvalidLLMResponse("Empty response".to_string()));
    }

    Ok(content)
}

/// Retry with exponential backoff
async fn retry_with_backoff<F, Fut>(max_retries: u32, mut f: F) -> Result<String, AIAgentError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<String, AIAgentError>>,
{
    let backoff_ms = [100, 500, 2000, 5000];
    let mut last_error = None;

    for attempt in 0..max_retries {
        match f().await {
            Ok(response) => return Ok(response),
            Err(e) => {
                if !e.is_transient() {
                    return Err(e);
                }

                warn!(
                    "LLM API call failed (attempt {}/{}): {}",
                    attempt + 1,
                    max_retries,
                    e
                );

                if attempt < max_retries - 1 {
                    let delay = backoff_ms.get(attempt as usize).copied().unwrap_or(5000);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }

                last_error = Some(e);
            }
        }
    }

    error!("LLM API call failed after {} attempts", max_retries);
    Err(last_error.unwrap_or_else(|| AIAgentError::LLMError("Unknown error".to_string())))
}

// ============== Mock Client ==============

/// Mock LLM client for testing
pub struct MockLLMClient {
    response: String,
    should_fail: bool,
    config: LLMConfig,
}

impl MockLLMClient {
    pub fn new(response: String) -> Self {
        Self {
            response,
            should_fail: false,
            config: LLMConfig::custom("mock://localhost".to_string(), "mock".to_string(), None),
        }
    }

    pub fn failing() -> Self {
        Self {
            response: String::new(),
            should_fail: true,
            config: LLMConfig::custom("mock://localhost".to_string(), "mock".to_string(), None),
        }
    }

    pub fn with_response(mut self, response: String) -> Self {
        self.response = response;
        self
    }
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn generate_response(
        &self,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<String, AIAgentError> {
        if self.should_fail {
            Err(AIAgentError::LLMError("Mock failure".to_string()))
        } else {
            Ok(self.response.clone())
        }
    }

    async fn generate_with_history(&self, _messages: &[ChatMessage]) -> Result<String, AIAgentError> {
        if self.should_fail {
            Err(AIAgentError::LLMError("Mock failure".to_string()))
        } else {
            Ok(self.response.clone())
        }
    }

    async fn generate_stream(
        &self,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<mpsc::Receiver<StreamChunk>, AIAgentError> {
        let (tx, rx) = mpsc::channel(2);
        let response = self.response.clone();
        let should_fail = self.should_fail;

        tokio::spawn(async move {
            if should_fail {
                return;
            }
            let _ = tx.send(StreamChunk {
                content: response,
                is_final: false,
            }).await;
            let _ = tx.send(StreamChunk {
                content: String::new(),
                is_final: true,
            }).await;
        });

        Ok(rx)
    }

    fn provider(&self) -> LLMProvider {
        LLMProvider::Custom
    }

    fn model(&self) -> &str {
        "mock"
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

// ============== Factory ==============

/// LLM client factory for creating clients from configuration
pub struct LLMClientFactory;

impl LLMClientFactory {
    /// Create LLM client from environment variables
    /// Priority: OpenAI > Anthropic > Google > SiliconFlow > Custom
    pub fn from_env() -> Option<Arc<dyn LLMClient>> {
        // Try providers in order of preference
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let api_base = std::env::var("OPENAI_API_BASE").ok();
            let model = std::env::var("OPENAI_MODEL").ok();
            info!(
                "Using OpenAI LLM provider with model: {}",
                model.as_deref().unwrap_or("gpt-3.5-turbo")
            );
            return Some(Arc::new(OpenAIClient::new(api_key, api_base, model)));
        }

        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let model = std::env::var("ANTHROPIC_MODEL").ok();
            info!(
                "Using Anthropic LLM provider with model: {}",
                model.as_deref().unwrap_or("claude-3-haiku-20240307")
            );
            return Some(Arc::new(AnthropicClient::new(api_key, model)));
        }

        if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
            let model = std::env::var("GOOGLE_MODEL").ok();
            info!(
                "Using Google Gemini LLM provider with model: {}",
                model.as_deref().unwrap_or("gemini-pro")
            );
            return Some(Arc::new(GoogleGeminiClient::new(api_key, model)));
        }

        // Try SiliconFlow (硅基流动)
        if let Ok(api_key) = std::env::var("SILICONFLOW_API_KEY") {
            let model = std::env::var("SILICONFLOW_MODEL").ok();
            info!(
                "Using SiliconFlow LLM provider with model: {}",
                model.as_deref().unwrap_or("Qwen/Qwen2.5-7B-Instruct")
            );
            return Some(Arc::new(SiliconFlowClient::new(api_key, model)));
        }

        // Try custom/local LLM
        if let Ok(api_base) = std::env::var("LLM_API_BASE") {
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "default".to_string());
            let api_key = std::env::var("LLM_API_KEY").ok();
            info!(
                "Using custom LLM provider at {} with model: {}",
                api_base, model
            );
            return Some(Arc::new(OpenAIClient::from_config(LLMConfig::custom(
                api_base, model, api_key,
            ))));
        }

        warn!("No LLM provider configured. Set OPENAI_API_KEY, ANTHROPIC_API_KEY, GOOGLE_API_KEY, SILICONFLOW_API_KEY, or LLM_API_BASE");
        None
    }

    /// Create OpenAI client
    pub fn openai(api_key: String, model: Option<String>) -> Arc<dyn LLMClient> {
        Arc::new(OpenAIClient::new(api_key, None, model))
    }

    /// Create OpenAI client with custom base URL (for Azure OpenAI or compatible APIs)
    pub fn openai_compatible(
        api_key: String,
        api_base: String,
        model: String,
    ) -> Arc<dyn LLMClient> {
        Arc::new(OpenAIClient::new(api_key, Some(api_base), Some(model)))
    }

    /// Create Anthropic Claude client
    pub fn anthropic(api_key: String, model: Option<String>) -> Arc<dyn LLMClient> {
        Arc::new(AnthropicClient::new(api_key, model))
    }

    /// Create Google Gemini client
    pub fn google(api_key: String, model: Option<String>) -> Arc<dyn LLMClient> {
        Arc::new(GoogleGeminiClient::new(api_key, model))
    }

    /// Create SiliconFlow (硅基流动) client
    /// Popular models: Qwen/Qwen2.5-7B-Instruct, deepseek-ai/DeepSeek-V2.5,
    /// THUDM/glm-4-9b-chat, meta-llama/Meta-Llama-3.1-8B-Instruct
    pub fn siliconflow(api_key: String, model: Option<String>) -> Arc<dyn LLMClient> {
        Arc::new(SiliconFlowClient::new(api_key, model))
    }

    /// Create client from config
    pub fn from_config(config: LLMConfig) -> Arc<dyn LLMClient> {
        match config.provider {
            LLMProvider::OpenAI | LLMProvider::Custom => {
                Arc::new(OpenAIClient::from_config(config))
            }
            LLMProvider::Anthropic => Arc::new(AnthropicClient::from_config(config)),
            LLMProvider::Google => Arc::new(GoogleGeminiClient::from_config(config)),
            LLMProvider::SiliconFlow => Arc::new(SiliconFlowClient::from_config(config)),
        }
    }

    /// Create mock client for testing
    pub fn mock(response: String) -> Arc<dyn LLMClient> {
        Arc::new(MockLLMClient::new(response))
    }
}

/// Create LLM client from environment variables (legacy function)
#[deprecated(since = "0.1.0", note = "Use LLMClientFactory::from_env() instead")]
pub fn create_llm_client_from_env() -> Option<Box<dyn LLMClient>> {
    LLMClientFactory::from_env().map(|arc| {
        // Convert Arc to Box - this is a bit awkward but maintains backward compatibility
        let client: Box<dyn LLMClient> = match arc.provider() {
            LLMProvider::OpenAI => Box::new(OpenAIClient::from_config(arc.config().clone())),
            LLMProvider::Anthropic => Box::new(AnthropicClient::from_config(arc.config().clone())),
            LLMProvider::Google => Box::new(GoogleGeminiClient::from_config(arc.config().clone())),
            LLMProvider::SiliconFlow => {
                Box::new(SiliconFlowClient::from_config(arc.config().clone()))
            }
            LLMProvider::Custom => Box::new(OpenAIClient::from_config(arc.config().clone())),
        };
        client
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_client() {
        let client = MockLLMClient::new("Hello, I'm a bot!".to_string());
        let result = client.generate_response("system", "user").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, I'm a bot!");
    }

    #[tokio::test]
    async fn test_mock_llm_client_failure() {
        let client = MockLLMClient::failing();
        let result = client.generate_response("system", "user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_llm_client_with_history() {
        let client = MockLLMClient::new("Response with history".to_string());
        let messages = vec![
            ChatMessage::system("You are helpful"),
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
            ChatMessage::user("How are you?"),
        ];
        let result = client.generate_with_history(&messages).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Response with history");
    }

    #[tokio::test]
    async fn test_mock_llm_client_stream() {
        let client = MockLLMClient::new("Streamed response".to_string());
        let mut rx = client.generate_stream("system", "user").await.unwrap();

        let mut content = String::new();
        while let Some(chunk) = rx.recv().await {
            if chunk.is_final {
                break;
            }
            content.push_str(&chunk.content);
        }
        assert_eq!(content, "Streamed response");
    }

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new(
            "test-key".to_string(),
            Some("https://api.example.com".to_string()),
            Some("gpt-4".to_string()),
        );
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.api_base(), "https://api.example.com");
        assert_eq!(client.config.model, "gpt-4");
        assert_eq!(client.provider(), LLMProvider::OpenAI);
    }

    #[test]
    fn test_anthropic_client_creation() {
        let client = AnthropicClient::new("test-key".to_string(), Some("claude-3-opus".to_string()));
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.config.model, "claude-3-opus");
        assert_eq!(client.provider(), LLMProvider::Anthropic);
    }

    #[test]
    fn test_google_client_creation() {
        let client = GoogleGeminiClient::new("test-key".to_string(), Some("gemini-pro".to_string()));
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.config.model, "gemini-pro");
        assert_eq!(client.provider(), LLMProvider::Google);
    }

    #[test]
    fn test_custom_config() {
        let config = LLMConfig::custom(
            "http://localhost:11434/v1".to_string(),
            "llama2".to_string(),
            None,
        );
        assert_eq!(config.provider, LLMProvider::Custom);
        assert_eq!(config.api_base, Some("http://localhost:11434/v1".to_string()));
        assert_eq!(config.model, "llama2");
    }

    #[test]
    fn test_config_builder() {
        let config = LLMConfig::openai("key".to_string(), None)
            .with_api_base("https://custom.api".to_string())
            .with_max_tokens(1000)
            .with_temperature(0.5)
            .with_timeout(60);

        assert_eq!(config.api_base, Some("https://custom.api".to_string()));
        assert_eq!(config.max_tokens, 1000);
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_error_is_transient() {
        assert!(AIAgentError::LLMTimeout(30).is_transient());
        assert!(AIAgentError::LLMRateLimited(60).is_transient());
        assert!(!AIAgentError::InvalidLLMResponse("test".to_string()).is_transient());
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(format!("{}", LLMProvider::OpenAI), "OpenAI");
        assert_eq!(format!("{}", LLMProvider::Anthropic), "Anthropic");
        assert_eq!(format!("{}", LLMProvider::Google), "Google");
        assert_eq!(format!("{}", LLMProvider::SiliconFlow), "SiliconFlow");
        assert_eq!(format!("{}", LLMProvider::Custom), "Custom");
    }

    #[test]
    fn test_siliconflow_client_creation() {
        let client =
            SiliconFlowClient::new("test-key".to_string(), Some("Qwen/Qwen2.5-7B-Instruct".to_string()));
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.config.model, "Qwen/Qwen2.5-7B-Instruct");
        assert_eq!(client.provider(), LLMProvider::SiliconFlow);
        assert_eq!(client.api_base(), "https://api.siliconflow.cn/v1");
    }

    #[test]
    fn test_siliconflow_default_model() {
        let client = SiliconFlowClient::new("test-key".to_string(), None);
        assert_eq!(client.config.model, "Qwen/Qwen2.5-7B-Instruct");
    }

    #[test]
    fn test_siliconflow_config() {
        let config = LLMConfig::siliconflow("test-key".to_string(), Some("deepseek-ai/DeepSeek-V2.5".to_string()));
        assert_eq!(config.provider, LLMProvider::SiliconFlow);
        assert_eq!(
            config.api_base,
            Some("https://api.siliconflow.cn/v1".to_string())
        );
        assert_eq!(config.model, "deepseek-ai/DeepSeek-V2.5");
    }

    #[test]
    fn test_siliconflow_convert_messages() {
        let client = SiliconFlowClient::new("key".to_string(), None);
        let messages = vec![
            ChatMessage::system("System prompt"),
            ChatMessage::user("User message"),
            ChatMessage::assistant("Assistant response"),
        ];

        let converted = client.convert_messages(&messages);
        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[2].role, "assistant");
    }

    #[test]
    fn test_chat_message_constructors() {
        let system = ChatMessage::system("You are helpful");
        assert_eq!(system.role, ChatRole::System);
        assert_eq!(system.content, "You are helpful");

        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, ChatRole::User);
        assert_eq!(user.content, "Hello");

        let assistant = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant.role, ChatRole::Assistant);
        assert_eq!(assistant.content, "Hi there!");
    }

    #[test]
    fn test_factory_methods() {
        let openai = LLMClientFactory::openai("key".to_string(), Some("gpt-4".to_string()));
        assert_eq!(openai.provider(), LLMProvider::OpenAI);
        assert_eq!(openai.model(), "gpt-4");

        let anthropic =
            LLMClientFactory::anthropic("key".to_string(), Some("claude-3-opus".to_string()));
        assert_eq!(anthropic.provider(), LLMProvider::Anthropic);
        assert_eq!(anthropic.model(), "claude-3-opus");

        let google = LLMClientFactory::google("key".to_string(), Some("gemini-pro".to_string()));
        assert_eq!(google.provider(), LLMProvider::Google);
        assert_eq!(google.model(), "gemini-pro");

        let siliconflow = LLMClientFactory::siliconflow(
            "key".to_string(),
            Some("Qwen/Qwen2.5-7B-Instruct".to_string()),
        );
        assert_eq!(siliconflow.provider(), LLMProvider::SiliconFlow);
        assert_eq!(siliconflow.model(), "Qwen/Qwen2.5-7B-Instruct");

        let mock = LLMClientFactory::mock("test".to_string());
        assert_eq!(mock.provider(), LLMProvider::Custom);
        assert_eq!(mock.model(), "mock");
    }

    #[test]
    fn test_openai_convert_messages() {
        let client = OpenAIClient::new("key".to_string(), None, None);
        let messages = vec![
            ChatMessage::system("System prompt"),
            ChatMessage::user("User message"),
            ChatMessage::assistant("Assistant response"),
        ];

        let converted = client.convert_messages(&messages);
        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[2].role, "assistant");
    }

    #[test]
    fn test_anthropic_convert_messages() {
        let client = AnthropicClient::new("key".to_string(), None);
        let messages = vec![
            ChatMessage::system("System prompt"),
            ChatMessage::user("User message"),
            ChatMessage::assistant("Assistant response"),
        ];

        let (system, converted) = client.convert_messages(&messages);
        assert_eq!(system, Some("System prompt".to_string()));
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].role, "user");
        assert_eq!(converted[1].role, "assistant");
    }

    #[test]
    fn test_gemini_convert_messages() {
        let client = GoogleGeminiClient::new("key".to_string(), None);
        let messages = vec![
            ChatMessage::system("System prompt"),
            ChatMessage::user("User message"),
        ];

        let converted = client.convert_messages(&messages);
        assert_eq!(converted.len(), 1);
        assert!(converted[0].parts[0].text.contains("System prompt"));
        assert!(converted[0].parts[0].text.contains("User message"));
    }
}
