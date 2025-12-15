//! Hidden Chain of Thought (CoT) Module
//!
//! This module provides functionality for AI agents to "think" before responding.
//! The thinking process is hidden from users but used to improve response quality.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Chain of Thought configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTConfig {
    /// Whether CoT is enabled
    pub enabled: bool,
    /// Maximum thinking tokens
    pub max_thinking_tokens: usize,
    /// Whether to store thinking process for analysis
    pub store_thinking: bool,
    /// Thinking style (analytical, empathetic, creative)
    pub style: ThinkingStyle,
}

impl Default for CoTConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_thinking_tokens: 500,
            store_thinking: false,
            style: ThinkingStyle::Balanced,
        }
    }
}

/// Thinking style for CoT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingStyle {
    /// Logical and step-by-step reasoning
    Analytical,
    /// Focus on emotional understanding
    Empathetic,
    /// Creative and divergent thinking
    Creative,
    /// Balanced approach (default)
    Balanced,
}

impl ThinkingStyle {
    /// Get the thinking instruction for this style
    pub fn instruction(&self) -> &'static str {
        match self {
            ThinkingStyle::Analytical => 
                "请用逻辑分析的方式思考：1) 用户意图 2) 关键信息 3) 最佳回复策略",
            ThinkingStyle::Empathetic => 
                "请从情感角度思考：1) 用户的情绪状态 2) 他们可能需要什么 3) 如何让回复更有温度",
            ThinkingStyle::Creative => 
                "请发散性思考：1) 有什么有趣的角度 2) 如何让回复更生动 3) 可以加入什么创意元素",
            ThinkingStyle::Balanced => 
                "请综合思考：1) 用户想要什么 2) 我应该如何回应 3) 符合我的人设",
        }
    }
}

/// Result of CoT processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoTResult {
    /// The hidden thinking process
    pub thinking: String,
    /// The final response to show user
    pub response: String,
    /// Detected user intent
    pub detected_intent: Option<String>,
    /// Confidence in understanding (0.0-1.0)
    pub understanding_confidence: f32,
}

/// Markers for CoT in LLM response
pub struct CoTMarkers;

impl CoTMarkers {
    /// Opening marker for thinking section
    pub const THINK_OPEN: &'static str = "<think>";
    /// Closing marker for thinking section
    pub const THINK_CLOSE: &'static str = "</think>";
    /// Opening marker for response section  
    pub const RESPONSE_OPEN: &'static str = "<response>";
    /// Closing marker for response section
    pub const RESPONSE_CLOSE: &'static str = "</response>";
}

/// Chain of Thought processor
pub struct CoTProcessor {
    config: CoTConfig,
}

impl CoTProcessor {
    /// Create a new CoT processor with default config
    pub fn new() -> Self {
        Self {
            config: CoTConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: CoTConfig) -> Self {
        Self { config }
    }

    /// Check if CoT is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Generate the CoT instruction to add to system prompt
    pub fn generate_instruction(&self) -> String {
        if !self.config.enabled {
            return String::new();
        }

        format!(
            r#"

## 思考过程
在回复之前，请先进行内部思考。{}

请按照以下格式回复：
{}
（你的思考过程，用户看不到这部分）
{}

{}
（你对用户的最终回复）
{}

重要：思考部分是你的内心独白，用于理清思路。用户只会看到回复部分的内容。"#,
            self.config.style.instruction(),
            CoTMarkers::THINK_OPEN,
            CoTMarkers::THINK_CLOSE,
            CoTMarkers::RESPONSE_OPEN,
            CoTMarkers::RESPONSE_CLOSE,
        )
    }

    /// Parse LLM response to extract thinking and final response
    pub fn parse_response(&self, raw_response: &str) -> CoTResult {
        if !self.config.enabled {
            return CoTResult {
                thinking: String::new(),
                response: raw_response.to_string(),
                detected_intent: None,
                understanding_confidence: 1.0,
            };
        }

        let (thinking, response) = self.extract_sections(raw_response);
        
        // Try to extract intent from thinking
        let detected_intent = self.extract_intent(&thinking);
        
        // Estimate understanding confidence based on thinking quality
        let understanding_confidence = self.estimate_confidence(&thinking);

        if self.config.store_thinking {
            info!("CoT thinking: {}", thinking);
        } else {
            debug!("CoT thinking length: {} chars", thinking.len());
        }

        CoTResult {
            thinking,
            response,
            detected_intent,
            understanding_confidence,
        }
    }

    /// Extract thinking and response sections from raw LLM output
    fn extract_sections(&self, raw: &str) -> (String, String) {
        let mut thinking = String::new();
        let mut response = String::new();

        // Try to find think markers
        if let Some(think_start) = raw.find(CoTMarkers::THINK_OPEN) {
            if let Some(think_end) = raw.find(CoTMarkers::THINK_CLOSE) {
                let start = think_start + CoTMarkers::THINK_OPEN.len();
                if start < think_end {
                    thinking = raw[start..think_end].trim().to_string();
                }
            }
        }

        // Try to find response markers
        if let Some(resp_start) = raw.find(CoTMarkers::RESPONSE_OPEN) {
            if let Some(resp_end) = raw.find(CoTMarkers::RESPONSE_CLOSE) {
                let start = resp_start + CoTMarkers::RESPONSE_OPEN.len();
                if start < resp_end {
                    response = raw[start..resp_end].trim().to_string();
                }
            }
        }

        // Fallback: if no markers found, treat entire response as final
        if response.is_empty() {
            // Remove any think tags if present but response tags missing
            response = raw
                .replace(CoTMarkers::THINK_OPEN, "")
                .replace(CoTMarkers::THINK_CLOSE, "")
                .trim()
                .to_string();
            
            // If still has markers, something went wrong - just use raw
            if response.contains('<') && response.contains('>') {
                // Try to get content after </think>
                if let Some(pos) = raw.find(CoTMarkers::THINK_CLOSE) {
                    response = raw[pos + CoTMarkers::THINK_CLOSE.len()..].trim().to_string();
                }
            }
        }

        (thinking, response)
    }

    /// Try to extract user intent from thinking process
    fn extract_intent(&self, thinking: &str) -> Option<String> {
        if thinking.is_empty() {
            return None;
        }

        // Look for common intent keywords in Chinese
        let intent_keywords = [
            ("询问", "question"),
            ("请求帮助", "help_request"),
            ("闲聊", "casual_chat"),
            ("吐槽", "venting"),
            ("寻求建议", "advice_seeking"),
            ("表达情绪", "emotional_expression"),
            ("技术问题", "technical_question"),
            ("打招呼", "greeting"),
        ];

        for (cn, en) in &intent_keywords {
            if thinking.contains(*cn) {
                return Some(en.to_string());
            }
        }

        None
    }

    /// Estimate understanding confidence based on thinking quality
    fn estimate_confidence(&self, thinking: &str) -> f32 {
        if thinking.is_empty() {
            return 0.5; // Default confidence when no thinking
        }

        let mut score = 0.5f32;

        // Longer thinking generally means better understanding
        let len = thinking.chars().count();
        if len > 50 {
            score += 0.1;
        }
        if len > 100 {
            score += 0.1;
        }
        if len > 200 {
            score += 0.1;
        }

        // Check for structured thinking indicators
        if thinking.contains("1)") || thinking.contains("1.") || thinking.contains("首先") {
            score += 0.1;
        }
        if thinking.contains("2)") || thinking.contains("2.") || thinking.contains("其次") {
            score += 0.05;
        }

        score.min(1.0)
    }
}

impl Default for CoTProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_with_markers() {
        let processor = CoTProcessor::new();
        
        let raw = r#"<think>
用户在问技术问题，需要专业回答。
1) 用户想了解 Rust 所有权
2) 我应该用简单的例子解释
3) 保持友好但专业的语气
</think>

<response>
Rust 的所有权系统是它最独特的特性之一！简单来说就是：
- 每个值都有一个所有者
- 同一时间只能有一个所有者
- 所有者离开作用域时，值会被丢弃
</response>"#;

        let result = processor.parse_response(raw);
        
        assert!(!result.thinking.is_empty());
        assert!(result.thinking.contains("技术问题"));
        assert!(result.response.contains("Rust"));
        assert!(!result.response.contains("<think>"));
        assert!(!result.response.contains("<response>"));
    }

    #[test]
    fn test_parse_response_without_markers() {
        let processor = CoTProcessor::new();
        
        let raw = "这是一个普通的回复，没有使用思考标记。";
        let result = processor.parse_response(raw);
        
        assert!(result.thinking.is_empty());
        assert_eq!(result.response, raw);
    }

    #[test]
    fn test_generate_instruction() {
        let processor = CoTProcessor::new();
        let instruction = processor.generate_instruction();
        
        assert!(instruction.contains("<think>"));
        assert!(instruction.contains("</think>"));
        assert!(instruction.contains("<response>"));
        assert!(instruction.contains("</response>"));
    }

    #[test]
    fn test_thinking_styles() {
        assert!(ThinkingStyle::Analytical.instruction().contains("逻辑"));
        assert!(ThinkingStyle::Empathetic.instruction().contains("情感"));
        assert!(ThinkingStyle::Creative.instruction().contains("创意"));
    }

    #[test]
    fn test_disabled_cot() {
        let config = CoTConfig {
            enabled: false,
            ..Default::default()
        };
        let processor = CoTProcessor::with_config(config);
        
        assert!(processor.generate_instruction().is_empty());
        
        let result = processor.parse_response("test response");
        assert!(result.thinking.is_empty());
        assert_eq!(result.response, "test response");
    }

    #[test]
    fn test_confidence_estimation() {
        let processor = CoTProcessor::new();
        
        // Short thinking
        let short_confidence = processor.estimate_confidence("用户在问问题");
        
        // Longer structured thinking
        let long_confidence = processor.estimate_confidence(
            "1) 首先分析用户意图，他们想要了解技术细节\n\
             2) 其次考虑如何用简单的方式解释\n\
             3) 最后保持我的人设风格"
        );
        
        assert!(long_confidence > short_confidence);
    }
}
