//! Multimodal AI capabilities
//!
//! Provides image understanding and processing for AI Agents.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

use super::AIAgentError;

/// Vision model capabilities
#[async_trait]
pub trait VisionModel: Send + Sync {
    /// Analyze an image and return a description
    async fn analyze_image(&self, image_url: &str, prompt: Option<&str>) -> Result<String, AIAgentError>;
    
    /// Analyze an image with base64 data
    async fn analyze_image_base64(&self, base64_data: &str, mime_type: &str, prompt: Option<&str>) -> Result<String, AIAgentError>;
}

/// OpenAI Vision model (GPT-4 Vision)
pub struct OpenAIVision {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout_secs: u64,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct VisionRequest {
    model: String,
    messages: Vec<VisionMessage>,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct VisionMessage {
    role: String,
    content: Vec<VisionContent>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum VisionContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlContent },
}

#[derive(Debug, Serialize)]
struct ImageUrlContent {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VisionResponse {
    choices: Vec<VisionChoice>,
}

#[derive(Debug, Deserialize)]
struct VisionChoice {
    message: VisionResponseMessage,
}

#[derive(Debug, Deserialize)]
struct VisionResponseMessage {
    content: String,
}

impl OpenAIVision {
    pub fn new(api_key: String, api_base: Option<String>, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_base: api_base.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.unwrap_or_else(|| "gpt-4-vision-preview".to_string()),
            timeout_secs: 60,
            max_tokens: 500,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    fn build_request(&self, image_url: String, prompt: Option<&str>) -> VisionRequest {
        let text_prompt = prompt.unwrap_or("What's in this image? Describe it briefly.");
        
        VisionRequest {
            model: self.model.clone(),
            messages: vec![VisionMessage {
                role: "user".to_string(),
                content: vec![
                    VisionContent::Text {
                        text: text_prompt.to_string(),
                    },
                    VisionContent::ImageUrl {
                        image_url: ImageUrlContent {
                            url: image_url,
                            detail: Some("low".to_string()), // Use low detail for faster processing
                        },
                    },
                ],
            }],
            max_tokens: self.max_tokens,
        }
    }
}

#[async_trait]
impl VisionModel for OpenAIVision {
    async fn analyze_image(&self, image_url: &str, prompt: Option<&str>) -> Result<String, AIAgentError> {
        let request = self.build_request(image_url.to_string(), prompt);

        debug!("Analyzing image: {}", image_url);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_base))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::LLMError(format!(
                "Vision API returned status {}: {}",
                status, error_text
            )));
        }

        let vision_response: VisionResponse = response
            .json()
            .await
            .map_err(|e| AIAgentError::InvalidLLMResponse(e.to_string()))?;

        vision_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AIAgentError::InvalidLLMResponse("No response from vision model".to_string()))
    }

    async fn analyze_image_base64(
        &self,
        base64_data: &str,
        mime_type: &str,
        prompt: Option<&str>,
    ) -> Result<String, AIAgentError> {
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);
        self.analyze_image(&data_url, prompt).await
    }
}

/// SiliconFlow Vision model (硅基流动)
/// Uses OpenAI-compatible API format
/// Supports models like Qwen/Qwen2-VL-7B-Instruct, OpenGVLab/InternVL2-8B, etc.
pub struct SiliconFlowVision {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout_secs: u64,
    max_tokens: u32,
}

impl SiliconFlowVision {
    /// Create a new SiliconFlow vision model
    /// Default model: Qwen/Qwen2-VL-7B-Instruct
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_base: "https://api.siliconflow.cn/v1".to_string(),
            model: model.unwrap_or_else(|| "Qwen/Qwen2-VL-7B-Instruct".to_string()),
            timeout_secs: 60,
            max_tokens: 500,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    fn build_request(&self, image_url: String, prompt: Option<&str>) -> VisionRequest {
        let text_prompt = prompt.unwrap_or("请描述这张图片的内容。");

        VisionRequest {
            model: self.model.clone(),
            messages: vec![VisionMessage {
                role: "user".to_string(),
                content: vec![
                    VisionContent::Text {
                        text: text_prompt.to_string(),
                    },
                    VisionContent::ImageUrl {
                        image_url: ImageUrlContent {
                            url: image_url,
                            detail: Some("low".to_string()),
                        },
                    },
                ],
            }],
            max_tokens: self.max_tokens,
        }
    }
}

#[async_trait]
impl VisionModel for SiliconFlowVision {
    async fn analyze_image(
        &self,
        image_url: &str,
        prompt: Option<&str>,
    ) -> Result<String, AIAgentError> {
        let request = self.build_request(image_url.to_string(), prompt);

        debug!("SiliconFlow analyzing image: {}", image_url);

        let response = self
            .client
            .post(format!("{}/chat/completions", self.api_base))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::LLMTimeout(self.timeout_secs)
                } else {
                    AIAgentError::LLMError(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::LLMError(format!(
                "Vision API returned status {}: {}",
                status, error_text
            )));
        }

        let vision_response: VisionResponse = response
            .json()
            .await
            .map_err(|e| AIAgentError::InvalidLLMResponse(e.to_string()))?;

        vision_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| {
                AIAgentError::InvalidLLMResponse("No response from vision model".to_string())
            })
    }

    async fn analyze_image_base64(
        &self,
        base64_data: &str,
        mime_type: &str,
        prompt: Option<&str>,
    ) -> Result<String, AIAgentError> {
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);
        self.analyze_image(&data_url, prompt).await
    }
}

/// Mock vision model for testing
pub struct MockVisionModel {
    response: String,
}

impl MockVisionModel {
    pub fn new(response: String) -> Self {
        Self { response }
    }

    pub fn default_response() -> Self {
        Self {
            response: "This image shows a scenic landscape with mountains and a lake.".to_string(),
        }
    }
}

#[async_trait]
impl VisionModel for MockVisionModel {
    async fn analyze_image(&self, _image_url: &str, _prompt: Option<&str>) -> Result<String, AIAgentError> {
        Ok(self.response.clone())
    }

    async fn analyze_image_base64(
        &self,
        _base64_data: &str,
        _mime_type: &str,
        _prompt: Option<&str>,
    ) -> Result<String, AIAgentError> {
        Ok(self.response.clone())
    }
}

/// Multimodal message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image { url: String, caption: Option<String> },
    ImageBase64 { data: String, mime_type: String, caption: Option<String> },
    /// Audio content (voice messages, audio files)
    Audio { url: String, duration_secs: Option<f32>, transcript: Option<String> },
    /// Video content
    Video { url: String, duration_secs: Option<f32>, thumbnail_url: Option<String>, description: Option<String> },
    /// File/document content
    File { url: String, filename: String, mime_type: String, size_bytes: Option<u64> },
    /// Sticker/emoji content
    Sticker { url: String, emoji: Option<String>, pack_name: Option<String> },
}

impl MessageContent {
    /// Check if this is an image content
    pub fn is_image(&self) -> bool {
        matches!(self, MessageContent::Image { .. } | MessageContent::ImageBase64 { .. })
    }

    /// Check if this is media content (non-text)
    pub fn is_media(&self) -> bool {
        !matches!(self, MessageContent::Text(_))
    }

    /// Check if this is audio content
    pub fn is_audio(&self) -> bool {
        matches!(self, MessageContent::Audio { .. })
    }

    /// Check if this is video content
    pub fn is_video(&self) -> bool {
        matches!(self, MessageContent::Video { .. })
    }

    /// Get the media type as string
    pub fn media_type(&self) -> &'static str {
        match self {
            MessageContent::Text(_) => "text",
            MessageContent::Image { .. } | MessageContent::ImageBase64 { .. } => "image",
            MessageContent::Audio { .. } => "audio",
            MessageContent::Video { .. } => "video",
            MessageContent::File { .. } => "file",
            MessageContent::Sticker { .. } => "sticker",
        }
    }

    /// Get text content or media description
    pub fn get_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(text) => Some(text),
            MessageContent::Image { caption, .. } => caption.as_deref(),
            MessageContent::ImageBase64 { caption, .. } => caption.as_deref(),
            MessageContent::Audio { transcript, .. } => transcript.as_deref(),
            MessageContent::Video { description, .. } => description.as_deref(),
            MessageContent::File { filename, .. } => Some(filename),
            MessageContent::Sticker { emoji, .. } => emoji.as_deref(),
        }
    }

    /// Get URL if applicable
    pub fn get_url(&self) -> Option<&str> {
        match self {
            MessageContent::Text(_) => None,
            MessageContent::Image { url, .. } => Some(url),
            MessageContent::ImageBase64 { .. } => None,
            MessageContent::Audio { url, .. } => Some(url),
            MessageContent::Video { url, .. } => Some(url),
            MessageContent::File { url, .. } => Some(url),
            MessageContent::Sticker { url, .. } => Some(url),
        }
    }
}

/// Multi-modal input combining text and images
#[derive(Debug, Clone)]
pub struct MultiModalInput {
    /// Text content
    pub text: Option<String>,
    /// Image contents
    pub images: Vec<ImageInput>,
}

/// Image input for multi-modal processing
#[derive(Debug, Clone)]
pub struct ImageInput {
    /// Image URL or base64 data URL
    pub url: String,
    /// Optional caption
    pub caption: Option<String>,
    /// Whether this is base64 data
    pub is_base64: bool,
    /// MIME type (for base64)
    pub mime_type: Option<String>,
}

impl MultiModalInput {
    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            images: Vec::new(),
        }
    }

    pub fn image_only(url: impl Into<String>) -> Self {
        Self {
            text: None,
            images: vec![ImageInput {
                url: url.into(),
                caption: None,
                is_base64: false,
                mime_type: None,
            }],
        }
    }

    pub fn with_text_and_image(text: impl Into<String>, image_url: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            images: vec![ImageInput {
                url: image_url.into(),
                caption: None,
                is_base64: false,
                mime_type: None,
            }],
        }
    }

    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    pub fn has_text(&self) -> bool {
        self.text.is_some()
    }
}

/// Result of multi-modal processing
#[derive(Debug, Clone)]
pub struct MultiModalResult {
    /// Combined text representation
    pub combined_text: String,
    /// Image descriptions (if vision was used)
    pub image_descriptions: Vec<String>,
    /// Whether vision processing was successful
    pub vision_success: bool,
    /// Whether graceful degradation was used
    pub degraded: bool,
    /// Error message if degradation occurred
    pub degradation_reason: Option<String>,
}

/// Multimodal message processor with graceful degradation
pub struct MultimodalProcessor {
    vision_model: Option<Box<dyn VisionModel>>,
    /// Whether to use graceful degradation on vision failure
    graceful_degradation: bool,
    /// Default description when vision fails
    default_image_description: String,
}

impl MultimodalProcessor {
    pub fn new() -> Self {
        Self {
            vision_model: None,
            graceful_degradation: true,
            default_image_description: "[Image content]".to_string(),
        }
    }

    pub fn with_vision_model(mut self, model: Box<dyn VisionModel>) -> Self {
        self.vision_model = Some(model);
        self
    }

    pub fn with_graceful_degradation(mut self, enabled: bool) -> Self {
        self.graceful_degradation = enabled;
        self
    }

    pub fn with_default_description(mut self, description: impl Into<String>) -> Self {
        self.default_image_description = description.into();
        self
    }

    /// Process multi-modal input with graceful degradation
    pub async fn process_multimodal(&self, input: &MultiModalInput) -> MultiModalResult {
        let mut image_descriptions = Vec::new();
        let mut vision_success = true;
        let mut degraded = false;
        let mut degradation_reason = None;

        // Process images
        for image in &input.images {
            match self.process_single_image(image).await {
                Ok(description) => {
                    image_descriptions.push(description);
                }
                Err(e) => {
                    vision_success = false;
                    if self.graceful_degradation {
                        degraded = true;
                        degradation_reason = Some(e.to_string());
                        let fallback = image.caption.clone()
                            .unwrap_or_else(|| self.default_image_description.clone());
                        image_descriptions.push(fallback);
                        warn!("Vision processing failed, using fallback: {}", e);
                    } else {
                        image_descriptions.push(format!("[Image processing failed: {}]", e));
                    }
                }
            }
        }

        let combined_text = self.combine_content(input.text.as_deref(), &image_descriptions);

        MultiModalResult {
            combined_text,
            image_descriptions,
            vision_success,
            degraded,
            degradation_reason,
        }
    }

    async fn process_single_image(&self, image: &ImageInput) -> Result<String, AIAgentError> {
        if let Some(vision) = &self.vision_model {
            if image.is_base64 {
                let mime_type = image.mime_type.as_deref().unwrap_or("image/jpeg");
                vision.analyze_image_base64(&image.url, mime_type, image.caption.as_deref()).await
            } else {
                vision.analyze_image(&image.url, image.caption.as_deref()).await
            }
        } else {
            Ok(image.caption.clone().unwrap_or_else(|| self.default_image_description.clone()))
        }
    }

    fn combine_content(&self, text: Option<&str>, image_descriptions: &[String]) -> String {
        let mut parts = Vec::new();
        if let Some(t) = text {
            parts.push(t.to_string());
        }
        for (i, desc) in image_descriptions.iter().enumerate() {
            if image_descriptions.len() == 1 {
                parts.push(format!("[Image: {}]", desc));
            } else {
                parts.push(format!("[Image {}: {}]", i + 1, desc));
            }
        }
        parts.join(" ")
    }

    pub async fn process_content(&self, content: &MessageContent) -> Result<String, AIAgentError> {
        match content {
            MessageContent::Text(text) => Ok(text.clone()),
            MessageContent::Image { url, caption } => {
                let input = MultiModalInput {
                    text: None,
                    images: vec![ImageInput {
                        url: url.clone(),
                        caption: caption.clone(),
                        is_base64: false,
                        mime_type: None,
                    }],
                };
                let result = self.process_multimodal(&input).await;
                Ok(result.combined_text)
            }
            MessageContent::ImageBase64 { data, mime_type, caption } => {
                let input = MultiModalInput {
                    text: None,
                    images: vec![ImageInput {
                        url: data.clone(),
                        caption: caption.clone(),
                        is_base64: true,
                        mime_type: Some(mime_type.clone()),
                    }],
                };
                let result = self.process_multimodal(&input).await;
                Ok(result.combined_text)
            }
            MessageContent::Audio { url: _, duration_secs, transcript } => {
                let duration_str = duration_secs
                    .map(|d| format!(" ({:.1}s)", d))
                    .unwrap_or_default();
                let content = transcript
                    .clone()
                    .unwrap_or_else(|| "[Voice message]".to_string());
                Ok(format!("[Audio{}: {}]", duration_str, content))
            }
            MessageContent::Video { url: _, duration_secs, thumbnail_url: _, description } => {
                let duration_str = duration_secs
                    .map(|d| format!(" ({:.1}s)", d))
                    .unwrap_or_default();
                let desc = description
                    .clone()
                    .unwrap_or_else(|| "Video content".to_string());
                Ok(format!("[Video{}: {}]", duration_str, desc))
            }
            MessageContent::File { url: _, filename, mime_type, size_bytes } => {
                let size_str = size_bytes
                    .map(|s| format!(" ({} bytes)", s))
                    .unwrap_or_default();
                Ok(format!("[File: {} ({}){}]", filename, mime_type, size_str))
            }
            MessageContent::Sticker { url: _, emoji, pack_name } => {
                let emoji_str = emoji.clone().unwrap_or_else(|| "😀".to_string());
                let pack_str = pack_name
                    .as_ref()
                    .map(|p| format!(" from {}", p))
                    .unwrap_or_default();
                Ok(format!("[Sticker: {}{}]", emoji_str, pack_str))
            }
        }
    }

    pub fn has_vision(&self) -> bool {
        self.vision_model.is_some()
    }
}

impl Default for MultimodalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified multimodal context for AI Agent processing
/// Aggregates all media understanding results for prompt generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalContext {
    /// Original text content from user message
    pub text_content: Option<String>,
    /// Processed media descriptions
    pub media_descriptions: Vec<MediaDescription>,
    /// Combined text for LLM prompt
    pub combined_prompt_text: String,
    /// Whether any vision processing was performed
    pub has_vision_content: bool,
    /// Whether any audio processing was performed
    pub has_audio_content: bool,
    /// Total media items processed
    pub total_media_count: usize,
}

/// Description of a single media item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaDescription {
    /// Type of media
    pub media_type: String,
    /// AI-generated or provided description
    pub description: String,
    /// Confidence level if AI-generated (0.0-1.0)
    pub confidence: Option<f32>,
    /// URL of the original media
    pub source_url: Option<String>,
}

impl MultimodalContext {
    /// Create a text-only context
    pub fn text_only(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            text_content: Some(text.clone()),
            media_descriptions: Vec::new(),
            combined_prompt_text: text,
            has_vision_content: false,
            has_audio_content: false,
            total_media_count: 0,
        }
    }

    /// Create context from multimodal result
    pub fn from_result(text: Option<String>, result: &MultiModalResult) -> Self {
        let media_descriptions: Vec<MediaDescription> = result
            .image_descriptions
            .iter()
            .map(|desc| MediaDescription {
                media_type: "image".to_string(),
                description: desc.clone(),
                confidence: if result.vision_success { Some(0.9) } else { None },
                source_url: None,
            })
            .collect();

        Self {
            text_content: text,
            media_descriptions,
            combined_prompt_text: result.combined_text.clone(),
            has_vision_content: !result.image_descriptions.is_empty(),
            has_audio_content: false,
            total_media_count: result.image_descriptions.len(),
        }
    }

    /// Add a media description
    pub fn add_media(&mut self, media_type: &str, description: impl Into<String>) {
        let desc = description.into();
        self.media_descriptions.push(MediaDescription {
            media_type: media_type.to_string(),
            description: desc.clone(),
            confidence: None,
            source_url: None,
        });
        self.total_media_count += 1;
        
        // Update flags
        match media_type {
            "image" => self.has_vision_content = true,
            "audio" => self.has_audio_content = true,
            _ => {}
        }
        
        // Update combined text
        self.combined_prompt_text.push_str(&format!(" [{}]", desc));
    }

    /// Generate prompt section for multimodal content
    pub fn generate_prompt_section(&self) -> String {
        if self.media_descriptions.is_empty() {
            return String::new();
        }

        let mut section = String::from("\n\u{591a}媒体内容理解:\n");
        
        for (i, media) in self.media_descriptions.iter().enumerate() {
            let idx = if self.media_descriptions.len() > 1 {
                format!("{}. ", i + 1)
            } else {
                String::new()
            };
            
            let confidence_str = media.confidence
                .map(|c| format!(" (置信度: {:.0}%)", c * 100.0))
                .unwrap_or_default();
            
            section.push_str(&format!(
                "- {}[类型: {}]{}: {}\n",
                idx,
                media.media_type,
                confidence_str,
                media.description
            ));
        }

        section
    }

    /// Check if context has any media
    pub fn has_media(&self) -> bool {
        self.total_media_count > 0
    }
}

/// Failing vision model for testing graceful degradation
pub struct FailingVisionModel {
    error_message: String,
}

impl FailingVisionModel {
    pub fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
        }
    }
}

#[async_trait]
impl VisionModel for FailingVisionModel {
    async fn analyze_image(&self, _image_url: &str, _prompt: Option<&str>) -> Result<String, AIAgentError> {
        Err(AIAgentError::LLMError(self.error_message.clone()))
    }

    async fn analyze_image_base64(
        &self,
        _base64_data: &str,
        _mime_type: &str,
        _prompt: Option<&str>,
    ) -> Result<String, AIAgentError> {
        Err(AIAgentError::LLMError(self.error_message.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_vision_model() {
        let model = MockVisionModel::new("A beautiful sunset".to_string());
        let result = model.analyze_image("http://example.com/image.jpg", None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "A beautiful sunset");
    }

    #[tokio::test]
    async fn test_mock_vision_model_base64() {
        let model = MockVisionModel::default_response();
        let result = model.analyze_image_base64("base64data", "image/jpeg", None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_content_is_image() {
        let text = MessageContent::Text("Hello".to_string());
        assert!(!text.is_image());

        let image = MessageContent::Image {
            url: "http://example.com/img.jpg".to_string(),
            caption: None,
        };
        assert!(image.is_image());
    }

    #[test]
    fn test_message_content_get_text() {
        let text = MessageContent::Text("Hello".to_string());
        assert_eq!(text.get_text(), Some("Hello"));

        let image = MessageContent::Image {
            url: "http://example.com/img.jpg".to_string(),
            caption: Some("My photo".to_string()),
        };
        assert_eq!(image.get_text(), Some("My photo"));

        let image_no_caption = MessageContent::Image {
            url: "http://example.com/img.jpg".to_string(),
            caption: None,
        };
        assert_eq!(image_no_caption.get_text(), None);
    }

    #[tokio::test]
    async fn test_multimodal_processor_text() {
        let processor = MultimodalProcessor::new();
        let content = MessageContent::Text("Hello world".to_string());
        let result = processor.process_content(&content).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world");
    }

    #[tokio::test]
    async fn test_multimodal_processor_image_with_vision() {
        let vision = Box::new(MockVisionModel::new("A cat".to_string()));
        let processor = MultimodalProcessor::new().with_vision_model(vision);
        
        let content = MessageContent::Image {
            url: "http://example.com/cat.jpg".to_string(),
            caption: None,
        };
        
        let result = processor.process_content(&content).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("A cat"));
    }

    #[tokio::test]
    async fn test_multimodal_processor_image_without_vision() {
        let processor = MultimodalProcessor::new();
        
        let content = MessageContent::Image {
            url: "http://example.com/cat.jpg".to_string(),
            caption: Some("My cat".to_string()),
        };
        
        let result = processor.process_content(&content).await;
        assert!(result.is_ok());
        // Now returns formatted output with [Image: caption]
        assert!(result.unwrap().contains("My cat"));
    }

    #[test]
    fn test_openai_vision_creation() {
        let vision = OpenAIVision::new(
            "test-key".to_string(),
            Some("https://api.example.com".to_string()),
            Some("gpt-4-vision".to_string()),
        );
        assert_eq!(vision.api_key, "test-key");
        assert_eq!(vision.api_base, "https://api.example.com");
        assert_eq!(vision.model, "gpt-4-vision");
    }

    #[test]
    fn test_siliconflow_vision_creation() {
        let vision = SiliconFlowVision::new("test-key".to_string(), None);
        assert_eq!(vision.api_key, "test-key");
        assert_eq!(vision.api_base, "https://api.siliconflow.cn/v1");
        assert_eq!(vision.model, "Qwen/Qwen2-VL-7B-Instruct");
    }

    #[test]
    fn test_siliconflow_vision_custom_model() {
        let vision = SiliconFlowVision::new(
            "test-key".to_string(),
            Some("OpenGVLab/InternVL2-8B".to_string()),
        );
        assert_eq!(vision.model, "OpenGVLab/InternVL2-8B");
    }

    #[test]
    fn test_siliconflow_vision_with_options() {
        let vision = SiliconFlowVision::new("test-key".to_string(), None)
            .with_timeout(120)
            .with_max_tokens(1000);
        assert_eq!(vision.timeout_secs, 120);
        assert_eq!(vision.max_tokens, 1000);
    }

    #[test]
    fn test_processor_has_vision() {
        let processor = MultimodalProcessor::new();
        assert!(!processor.has_vision());

        let vision = Box::new(MockVisionModel::default_response());
        let processor = MultimodalProcessor::new().with_vision_model(vision);
        assert!(processor.has_vision());
    }

    #[test]
    fn test_multimodal_input_constructors() {
        let text_only = MultiModalInput::text_only("Hello");
        assert!(text_only.has_text());
        assert!(!text_only.has_images());

        let image_only = MultiModalInput::image_only("http://example.com/img.jpg");
        assert!(!image_only.has_text());
        assert!(image_only.has_images());

        let both = MultiModalInput::with_text_and_image("Hello", "http://example.com/img.jpg");
        assert!(both.has_text());
        assert!(both.has_images());
    }

    #[tokio::test]
    async fn test_multimodal_text_and_image() {
        let vision = Box::new(MockVisionModel::new("A cat".to_string()));
        let processor = MultimodalProcessor::new().with_vision_model(vision);

        let input = MultiModalInput::with_text_and_image("Look at this!", "http://example.com/cat.jpg");
        let result = processor.process_multimodal(&input).await;

        assert!(result.vision_success);
        assert!(!result.degraded);
        assert!(result.combined_text.contains("Look at this!"));
        assert!(result.combined_text.contains("A cat"));
    }

    /// **Feature: phase3-ai-agent-evolution, Property 19: Multi-Modal Input Processing**
    /// **Validates: Requirements 14.1**
    #[tokio::test]
    async fn test_multimodal_processing_property() {
        let vision = Box::new(MockVisionModel::new("Image description".to_string()));
        let processor = MultimodalProcessor::new().with_vision_model(vision);

        // Property: Text-only input should return text unchanged
        let text_input = MultiModalInput::text_only("Hello world");
        let result = processor.process_multimodal(&text_input).await;
        assert!(result.combined_text.contains("Hello world"));
        assert!(result.image_descriptions.is_empty());

        // Property: Image input should include image description
        let image_input = MultiModalInput::image_only("http://example.com/img.jpg");
        let result = processor.process_multimodal(&image_input).await;
        assert!(result.combined_text.contains("[Image:"));
        assert_eq!(result.image_descriptions.len(), 1);

        // Property: Combined input should include both
        let combined = MultiModalInput::with_text_and_image("Caption", "http://example.com/img.jpg");
        let result = processor.process_multimodal(&combined).await;
        assert!(result.combined_text.contains("Caption"));
        assert!(result.combined_text.contains("[Image:"));
    }

    /// **Feature: phase3-ai-agent-evolution, Property 20: Image Processing Graceful Degradation**
    /// **Validates: Requirements 14.3**
    #[tokio::test]
    async fn test_graceful_degradation_property() {
        let failing_vision = Box::new(FailingVisionModel::new("Vision API error"));
        let processor = MultimodalProcessor::new()
            .with_vision_model(failing_vision)
            .with_graceful_degradation(true)
            .with_default_description("Fallback description");

        let input = MultiModalInput {
            text: Some("Check this image".to_string()),
            images: vec![ImageInput {
                url: "http://example.com/img.jpg".to_string(),
                caption: Some("My photo".to_string()),
                is_base64: false,
                mime_type: None,
            }],
        };

        let result = processor.process_multimodal(&input).await;

        // Property: When vision fails with graceful degradation enabled,
        // the result should still be valid and use fallback
        assert!(!result.vision_success);
        assert!(result.degraded);
        assert!(result.degradation_reason.is_some());
        // Should use caption as fallback
        assert!(result.combined_text.contains("My photo"));
        assert!(result.combined_text.contains("Check this image"));
    }

    #[tokio::test]
    async fn test_graceful_degradation_uses_default() {
        let failing_vision = Box::new(FailingVisionModel::new("Vision API error"));
        let processor = MultimodalProcessor::new()
            .with_vision_model(failing_vision)
            .with_graceful_degradation(true)
            .with_default_description("Default fallback");

        // Image without caption
        let input = MultiModalInput::image_only("http://example.com/img.jpg");
        let result = processor.process_multimodal(&input).await;

        assert!(result.degraded);
        // Should use default description when no caption
        assert!(result.combined_text.contains("Default fallback"));
    }

    #[tokio::test]
    async fn test_multiple_images() {
        let vision = Box::new(MockVisionModel::new("Image content".to_string()));
        let processor = MultimodalProcessor::new().with_vision_model(vision);

        let input = MultiModalInput {
            text: Some("Multiple images".to_string()),
            images: vec![
                ImageInput {
                    url: "http://example.com/img1.jpg".to_string(),
                    caption: None,
                    is_base64: false,
                    mime_type: None,
                },
                ImageInput {
                    url: "http://example.com/img2.jpg".to_string(),
                    caption: None,
                    is_base64: false,
                    mime_type: None,
                },
            ],
        };

        let result = processor.process_multimodal(&input).await;

        assert_eq!(result.image_descriptions.len(), 2);
        assert!(result.combined_text.contains("[Image 1:"));
        assert!(result.combined_text.contains("[Image 2:"));
    }
}
