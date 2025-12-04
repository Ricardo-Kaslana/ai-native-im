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
#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    Image { url: String, caption: Option<String> },
    ImageBase64 { data: String, mime_type: String, caption: Option<String> },
}

impl MessageContent {
    /// Check if this is an image content
    pub fn is_image(&self) -> bool {
        matches!(self, MessageContent::Image { .. } | MessageContent::ImageBase64 { .. })
    }

    /// Get text content or image caption
    pub fn get_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(text) => Some(text),
            MessageContent::Image { caption, .. } => caption.as_deref(),
            MessageContent::ImageBase64 { caption, .. } => caption.as_deref(),
        }
    }
}

/// Multimodal message processor
pub struct MultimodalProcessor {
    vision_model: Option<Box<dyn VisionModel>>,
}

impl MultimodalProcessor {
    pub fn new() -> Self {
        Self { vision_model: None }
    }

    pub fn with_vision_model(mut self, model: Box<dyn VisionModel>) -> Self {
        self.vision_model = Some(model);
        self
    }

    /// Process a message and extract/analyze any images
    pub async fn process_content(&self, content: &MessageContent) -> Result<String, AIAgentError> {
        match content {
            MessageContent::Text(text) => Ok(text.clone()),
            MessageContent::Image { url, caption } => {
                if let Some(vision) = &self.vision_model {
                    let description = vision.analyze_image(url, None).await?;
                    if let Some(cap) = caption {
                        Ok(format!("[Image: {}] {}", description, cap))
                    } else {
                        Ok(format!("[Image: {}]", description))
                    }
                } else {
                    warn!("Vision model not configured, using caption only");
                    Ok(caption.clone().unwrap_or_else(|| "[Image]".to_string()))
                }
            }
            MessageContent::ImageBase64 { data, mime_type, caption } => {
                if let Some(vision) = &self.vision_model {
                    let description = vision.analyze_image_base64(data, mime_type, None).await?;
                    if let Some(cap) = caption {
                        Ok(format!("[Image: {}] {}", description, cap))
                    } else {
                        Ok(format!("[Image: {}]", description))
                    }
                } else {
                    warn!("Vision model not configured, using caption only");
                    Ok(caption.clone().unwrap_or_else(|| "[Image]".to_string()))
                }
            }
        }
    }

    /// Check if vision capabilities are available
    pub fn has_vision(&self) -> bool {
        self.vision_model.is_some()
    }
}

impl Default for MultimodalProcessor {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(result.unwrap(), "My cat");
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
}
