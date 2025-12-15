//! Speech-to-Text (STT) Service
//!
//! Provides voice message transcription using various providers.
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 3.1, 3.4**

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::services::ai::AIAgentError;

// ============================================================================
// Types
// ============================================================================

/// Transcription result from STT service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    /// The transcribed text
    pub text: String,
    /// Detected language (ISO 639-1 code)
    pub language: Option<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: Option<f32>,
    /// Duration of the audio in seconds
    pub duration_secs: Option<f32>,
    /// Word-level timestamps (if available)
    pub segments: Vec<TranscriptionSegment>,
}

/// A segment of transcription with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Segment text
    pub text: String,
    /// Start time in seconds
    pub start: f32,
    /// End time in seconds
    pub end: f32,
}

/// STT request options
#[derive(Debug, Clone, Default)]
pub struct STTOptions {
    /// Expected language (helps with accuracy)
    pub language: Option<String>,
    /// Whether to include word timestamps
    pub timestamps: bool,
    /// Audio format hint
    pub format: Option<AudioFormat>,
}

/// Supported audio formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Webm,
    M4a,
    Flac,
}

impl AudioFormat {
    /// Get MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Webm => "audio/webm",
            AudioFormat::M4a => "audio/m4a",
            AudioFormat::Flac => "audio/flac",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "wav" => Some(AudioFormat::Wav),
            "ogg" | "opus" => Some(AudioFormat::Ogg),
            "webm" => Some(AudioFormat::Webm),
            "m4a" | "mp4" => Some(AudioFormat::M4a),
            "flac" => Some(AudioFormat::Flac),
            _ => None,
        }
    }
}

// ============================================================================
// STT Service Trait
// ============================================================================

/// Speech-to-Text service trait
#[async_trait]
pub trait STTService: Send + Sync {
    /// Transcribe audio from a URL
    async fn transcribe_url(
        &self,
        audio_url: &str,
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError>;

    /// Transcribe audio from bytes
    async fn transcribe_bytes(
        &self,
        audio_data: &[u8],
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError>;

    /// Get the maximum audio duration supported (in seconds)
    fn max_duration_secs(&self) -> u64;

    /// Get the provider name
    fn provider_name(&self) -> &str;
}

// ============================================================================
// Whisper STT Implementation
// ============================================================================

/// OpenAI Whisper STT service
pub struct WhisperSTT {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout_secs: u64,
}

impl WhisperSTT {
    /// Create a new Whisper STT service
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_base: "https://api.openai.com/v1".to_string(),
            model: "whisper-1".to_string(),
            timeout_secs: 120,
        }
    }

    /// Set custom API base (for compatible services)
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = api_base;
        self
    }

    /// Set model
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Download audio from URL
    async fn download_audio(&self, url: &str) -> Result<Vec<u8>, AIAgentError> {
        debug!(url = url, "Downloading audio");
        
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| AIAgentError::STTError(format!("Failed to download audio: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIAgentError::STTError(format!(
                "Failed to download audio: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AIAgentError::STTError(format!("Failed to read audio: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// Chunk audio for long files (>5 minutes)
    /// 
    /// **Property 4: Audio Chunking for Long Messages**
    fn should_chunk(&self, duration_secs: f32) -> bool {
        duration_secs > 300.0 // 5 minutes
    }
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration: Option<f32>,
    #[serde(default)]
    segments: Option<Vec<WhisperSegment>>,
}

#[derive(Debug, Deserialize)]
struct WhisperSegment {
    text: String,
    start: f32,
    end: f32,
}

#[async_trait]
impl STTService for WhisperSTT {
    async fn transcribe_url(
        &self,
        audio_url: &str,
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        let audio_data = self.download_audio(audio_url).await?;
        self.transcribe_bytes(&audio_data, options).await
    }

    async fn transcribe_bytes(
        &self,
        audio_data: &[u8],
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        info!(
            size_kb = audio_data.len() / 1024,
            "Transcribing audio with Whisper"
        );

        // Build multipart form
        let file_part = reqwest::multipart::Part::bytes(audio_data.to_vec())
            .file_name("audio.mp3")
            .mime_str(options.format.map(|f| f.mime_type()).unwrap_or("audio/mpeg"))
            .map_err(|e| AIAgentError::STTError(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", self.model.clone());

        if let Some(lang) = &options.language {
            form = form.text("language", lang.clone());
        }

        if options.timestamps {
            form = form.text("response_format", "verbose_json");
        }

        let response = self
            .client
            .post(format!("{}/audio/transcriptions", self.api_base))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::STTError("Transcription timed out".to_string())
                } else {
                    AIAgentError::STTError(format!("Request failed: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::STTError(format!(
                "Whisper API error: {}",
                error_text
            )));
        }

        let whisper_response: WhisperResponse = response
            .json()
            .await
            .map_err(|e| AIAgentError::STTError(format!("Failed to parse response: {}", e)))?;

        let segments = whisper_response
            .segments
            .unwrap_or_default()
            .into_iter()
            .map(|s| TranscriptionSegment {
                text: s.text,
                start: s.start,
                end: s.end,
            })
            .collect();

        Ok(Transcription {
            text: whisper_response.text,
            language: whisper_response.language,
            confidence: None, // Whisper doesn't provide confidence
            duration_secs: whisper_response.duration,
            segments,
        })
    }

    fn max_duration_secs(&self) -> u64 {
        // Whisper supports up to 25MB files, roughly 25 minutes of audio
        1500
    }

    fn provider_name(&self) -> &str {
        "OpenAI Whisper"
    }
}

// ============================================================================
// Alibaba Cloud STT Implementation (阿里云语音识别)
// ============================================================================

/// Alibaba Cloud Speech-to-Text service (阿里云语音识别)
/// Uses the Alibaba Cloud Intelligent Speech Interaction API
/// 
/// **Feature: phase3-ai-agent-evolution**
/// **Requirements: 16.5**
pub struct AliSTT {
    client: Client,
    access_key_id: String,
    access_key_secret: String,
    app_key: String,
    api_base: String,
    timeout_secs: u64,
}

impl AliSTT {
    /// Create a new Alibaba Cloud STT service
    /// 
    /// # Arguments
    /// * `access_key_id` - Alibaba Cloud Access Key ID
    /// * `access_key_secret` - Alibaba Cloud Access Key Secret
    /// * `app_key` - Application key from Alibaba Cloud console
    pub fn new(access_key_id: String, access_key_secret: String, app_key: String) -> Self {
        Self {
            client: Client::new(),
            access_key_id,
            access_key_secret,
            app_key,
            api_base: "https://nls-gateway-cn-shanghai.aliyuncs.com".to_string(),
            timeout_secs: 120,
        }
    }

    /// Set custom API base
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = api_base;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Download audio from URL
    async fn download_audio(&self, url: &str) -> Result<Vec<u8>, AIAgentError> {
        debug!(url = url, "Downloading audio for AliSTT");
        
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| AIAgentError::STTError(format!("Failed to download audio: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIAgentError::STTError(format!(
                "Failed to download audio: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AIAgentError::STTError(format!("Failed to read audio: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// Generate token for Alibaba Cloud API
    /// Note: In production, you should use proper Alibaba Cloud SDK for token generation
    fn generate_auth_header(&self) -> String {
        // Simplified auth - in production use proper Alibaba Cloud signature
        format!("Bearer {}:{}", self.access_key_id, self.access_key_secret)
    }
}

#[derive(Debug, Deserialize)]
struct AliSTTResponse {
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    status: i32,
    #[serde(default)]
    message: Option<String>,
}

#[async_trait]
impl STTService for AliSTT {
    async fn transcribe_url(
        &self,
        audio_url: &str,
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        let audio_data = self.download_audio(audio_url).await?;
        self.transcribe_bytes(&audio_data, options).await
    }

    async fn transcribe_bytes(
        &self,
        audio_data: &[u8],
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        info!(
            size_kb = audio_data.len() / 1024,
            "Transcribing audio with Alibaba Cloud STT"
        );

        // Build request body
        // Note: This is a simplified implementation. In production, use the official SDK
        // or implement proper WebSocket-based real-time recognition
        let format = options.format.map(|f| match f {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Webm => "webm",
            AudioFormat::M4a => "m4a",
            AudioFormat::Flac => "flac",
        }).unwrap_or("mp3");

        let file_part = reqwest::multipart::Part::bytes(audio_data.to_vec())
            .file_name(format!("audio.{}", format))
            .mime_str(options.format.map(|f| f.mime_type()).unwrap_or("audio/mpeg"))
            .map_err(|e| AIAgentError::STTError(e.to_string()))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("appkey", self.app_key.clone())
            .text("format", format.to_string());

        // Add language hint if provided
        if let Some(lang) = &options.language {
            form = form.text("language", lang.clone());
        }

        let response = self
            .client
            .post(format!("{}/stream/v1/asr", self.api_base))
            .header("Authorization", self.generate_auth_header())
            .header("Content-Type", "multipart/form-data")
            .multipart(form)
            .timeout(Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::STTError("阿里云语音识别超时".to_string())
                } else {
                    AIAgentError::STTError(format!("请求失败: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::STTError(format!(
                "阿里云语音识别错误: {}",
                error_text
            )));
        }

        let ali_response: AliSTTResponse = response
            .json()
            .await
            .map_err(|e| AIAgentError::STTError(format!("解析响应失败: {}", e)))?;

        if ali_response.status != 20000000 {
            return Err(AIAgentError::STTError(format!(
                "阿里云语音识别失败: {}",
                ali_response.message.unwrap_or_else(|| "未知错误".to_string())
            )));
        }

        Ok(Transcription {
            text: ali_response.result.unwrap_or_default(),
            language: options.language.clone().or(Some("zh".to_string())),
            confidence: None,
            duration_secs: None,
            segments: vec![],
        })
    }

    fn max_duration_secs(&self) -> u64 {
        // Alibaba Cloud supports up to 60 seconds for one-shot recognition
        // For longer audio, use file transcription API
        60
    }

    fn provider_name(&self) -> &str {
        "Alibaba Cloud STT (阿里云语音识别)"
    }
}

// ============================================================================
// STT Provider Manager with Fallback
// ============================================================================

/// STT Provider Manager with automatic fallback
/// 
/// Tries providers in order until one succeeds.
/// Domestic providers (Alibaba) are prioritized for users in China.
pub struct STTProviderManager {
    providers: Vec<Box<dyn STTService>>,
}

impl STTProviderManager {
    /// Create a new STT provider manager
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Add a provider
    pub fn add_provider(mut self, provider: Box<dyn STTService>) -> Self {
        self.providers.push(provider);
        self
    }

    /// Create with domestic priority (国内优先)
    /// Alibaba Cloud first, then OpenAI Whisper as fallback
    pub fn with_domestic_priority(
        ali_access_key_id: Option<String>,
        ali_access_key_secret: Option<String>,
        ali_app_key: Option<String>,
        openai_api_key: Option<String>,
    ) -> Self {
        let mut manager = Self::new();

        // Add Alibaba Cloud STT first (domestic)
        if let (Some(id), Some(secret), Some(app_key)) = 
            (ali_access_key_id, ali_access_key_secret, ali_app_key) 
        {
            info!("Adding Alibaba Cloud STT as primary provider");
            manager.providers.push(Box::new(AliSTT::new(id, secret, app_key)));
        }

        // Add OpenAI Whisper as fallback
        if let Some(api_key) = openai_api_key {
            info!("Adding OpenAI Whisper as fallback provider");
            manager.providers.push(Box::new(WhisperSTT::new(api_key)));
        }

        manager
    }
}

impl Default for STTProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl STTService for STTProviderManager {
    async fn transcribe_url(
        &self,
        audio_url: &str,
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        let mut last_error = None;

        for provider in &self.providers {
            debug!(provider = provider.provider_name(), "Trying STT provider");
            
            match provider.transcribe_url(audio_url, options).await {
                Ok(result) => {
                    info!(provider = provider.provider_name(), "STT succeeded");
                    return Ok(result);
                }
                Err(e) => {
                    warn!(
                        provider = provider.provider_name(),
                        error = %e,
                        "STT provider failed, trying next"
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AIAgentError::STTError("No STT providers configured".to_string())
        }))
    }

    async fn transcribe_bytes(
        &self,
        audio_data: &[u8],
        options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        let mut last_error = None;

        for provider in &self.providers {
            debug!(provider = provider.provider_name(), "Trying STT provider");
            
            match provider.transcribe_bytes(audio_data, options).await {
                Ok(result) => {
                    info!(provider = provider.provider_name(), "STT succeeded");
                    return Ok(result);
                }
                Err(e) => {
                    warn!(
                        provider = provider.provider_name(),
                        error = %e,
                        "STT provider failed, trying next"
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AIAgentError::STTError("No STT providers configured".to_string())
        }))
    }

    fn max_duration_secs(&self) -> u64 {
        // Return the maximum of all providers
        self.providers
            .iter()
            .map(|p| p.max_duration_secs())
            .max()
            .unwrap_or(60)
    }

    fn provider_name(&self) -> &str {
        "STT Provider Manager"
    }
}

// ============================================================================
// Mock STT for Testing
// ============================================================================

/// Mock STT service for testing
pub struct MockSTT {
    response: String,
    should_fail: bool,
}

impl MockSTT {
    /// Create a new mock STT
    pub fn new(response: String) -> Self {
        Self {
            response,
            should_fail: false,
        }
    }

    /// Create a failing mock STT
    pub fn failing() -> Self {
        Self {
            response: String::new(),
            should_fail: true,
        }
    }
}

#[async_trait]
impl STTService for MockSTT {
    async fn transcribe_url(
        &self,
        _audio_url: &str,
        _options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        if self.should_fail {
            Err(AIAgentError::STTError("Mock failure".to_string()))
        } else {
            Ok(Transcription {
                text: self.response.clone(),
                language: Some("zh".to_string()),
                confidence: Some(0.95),
                duration_secs: Some(5.0),
                segments: vec![],
            })
        }
    }

    async fn transcribe_bytes(
        &self,
        _audio_data: &[u8],
        _options: &STTOptions,
    ) -> Result<Transcription, AIAgentError> {
        if self.should_fail {
            Err(AIAgentError::STTError("Mock failure".to_string()))
        } else {
            Ok(Transcription {
                text: self.response.clone(),
                language: Some("zh".to_string()),
                confidence: Some(0.95),
                duration_secs: Some(5.0),
                segments: vec![],
            })
        }
    }

    fn max_duration_secs(&self) -> u64 {
        300
    }

    fn provider_name(&self) -> &str {
        "Mock STT"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_detection() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("ogg"), Some(AudioFormat::Ogg));
        assert_eq!(AudioFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_audio_format_mime_type() {
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
    }

    #[tokio::test]
    async fn test_mock_stt_success() {
        let stt = MockSTT::new("Hello, world!".to_string());
        let options = STTOptions::default();
        
        let result = stt.transcribe_url("http://example.com/audio.mp3", &options).await;
        
        assert!(result.is_ok());
        let transcription = result.unwrap();
        assert_eq!(transcription.text, "Hello, world!");
        assert_eq!(transcription.language, Some("zh".to_string()));
    }

    #[tokio::test]
    async fn test_mock_stt_failure() {
        let stt = MockSTT::failing();
        let options = STTOptions::default();
        
        let result = stt.transcribe_url("http://example.com/audio.mp3", &options).await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_whisper_should_chunk() {
        let whisper = WhisperSTT::new("test-key".to_string());
        
        assert!(!whisper.should_chunk(60.0));   // 1 minute - no chunk
        assert!(!whisper.should_chunk(300.0));  // 5 minutes - no chunk
        assert!(whisper.should_chunk(301.0));   // >5 minutes - chunk
    }

    #[test]
    fn test_stt_options_default() {
        let options = STTOptions::default();
        assert!(options.language.is_none());
        assert!(!options.timestamps);
        assert!(options.format.is_none());
    }

    #[test]
    fn test_ali_stt_creation() {
        let stt = AliSTT::new(
            "access_key_id".to_string(),
            "access_key_secret".to_string(),
            "app_key".to_string(),
        );
        assert_eq!(stt.provider_name(), "Alibaba Cloud STT (阿里云语音识别)");
        assert_eq!(stt.max_duration_secs(), 60);
    }

    #[test]
    fn test_ali_stt_with_custom_base() {
        let stt = AliSTT::new(
            "id".to_string(),
            "secret".to_string(),
            "app".to_string(),
        )
        .with_api_base("https://custom.api.com".to_string())
        .with_timeout(60);
        
        assert_eq!(stt.api_base, "https://custom.api.com");
        assert_eq!(stt.timeout_secs, 60);
    }

    #[test]
    fn test_stt_provider_manager_creation() {
        let manager = STTProviderManager::new()
            .add_provider(Box::new(MockSTT::new("test".to_string())));
        
        assert_eq!(manager.provider_name(), "STT Provider Manager");
    }

    #[tokio::test]
    async fn test_stt_provider_manager_fallback() {
        // First provider fails, second succeeds
        let manager = STTProviderManager::new()
            .add_provider(Box::new(MockSTT::failing()))
            .add_provider(Box::new(MockSTT::new("fallback success".to_string())));
        
        let options = STTOptions::default();
        let result = manager.transcribe_url("http://example.com/audio.mp3", &options).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().text, "fallback success");
    }

    #[tokio::test]
    async fn test_stt_provider_manager_all_fail() {
        let manager = STTProviderManager::new()
            .add_provider(Box::new(MockSTT::failing()))
            .add_provider(Box::new(MockSTT::failing()));
        
        let options = STTOptions::default();
        let result = manager.transcribe_url("http://example.com/audio.mp3", &options).await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_stt_provider_manager_max_duration() {
        let manager = STTProviderManager::new()
            .add_provider(Box::new(MockSTT::new("test".to_string())));
        
        // MockSTT has max_duration of 300
        assert_eq!(manager.max_duration_secs(), 300);
    }

    #[test]
    fn test_stt_provider_manager_domestic_priority() {
        // Test that domestic priority manager can be created
        let manager = STTProviderManager::with_domestic_priority(
            None, // No Alibaba credentials
            None,
            None,
            Some("openai-key".to_string()),
        );
        
        // Should have one provider (OpenAI)
        assert_eq!(manager.providers.len(), 1);
    }
}
