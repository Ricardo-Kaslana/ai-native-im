//! LLM-based Emotion Analyzer
//!
//! Uses LLM for accurate emotion analysis when rule-based detection
//! is insufficient or when higher accuracy is needed.
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 17.1**

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

use super::{EmotionAnalysis, EmotionAnalyzer, EmotionState};
use crate::services::ai::{AIAgentError, LLMClient};

/// LLM-based emotion analyzer
///
/// Uses an LLM to analyze emotions in text with higher accuracy
/// than rule-based approaches, especially for nuanced or
/// context-dependent emotions.
pub struct LLMEmotionAnalyzer {
    /// LLM client for analysis
    llm_client: Arc<dyn LLMClient>,
    /// Whether to include detailed analysis
    detailed: bool,
    /// Fallback to rule-based on LLM failure
    fallback_enabled: bool,
    /// Rule-based analyzer for fallback
    rule_based: super::RuleBasedEmotionAnalyzer,
}

impl LLMEmotionAnalyzer {
    /// Create a new LLM emotion analyzer
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self {
            llm_client,
            detailed: false,
            fallback_enabled: true,
            rule_based: super::RuleBasedEmotionAnalyzer::new(),
        }
    }

    /// Enable detailed analysis
    pub fn with_detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }

    /// Enable/disable fallback to rule-based
    pub fn with_fallback(mut self, enabled: bool) -> Self {
        self.fallback_enabled = enabled;
        self
    }

    /// Build the system prompt for emotion analysis
    fn build_system_prompt(&self) -> String {
        r#"你是一个情绪分析专家。分析用户消息中的情绪，并以JSON格式返回结果。

可用的情绪类型：
- neutral: 平静/中性
- happy: 开心/高兴
- excited: 兴奋/激动
- sad: 难过/悲伤
- angry: 生气/愤怒
- empathetic: 共情/理解
- supportive: 支持/鼓励
- curious: 好奇/疑问
- playful: 俏皮/幽默
- anxious: 焦虑/担忧
- confused: 困惑/迷茫

返回格式（仅返回JSON，不要其他内容）：
{
  "primary_emotion": "情绪类型",
  "confidence": 0.0-1.0之间的数字,
  "sentiment_score": -1.0到1.0之间的数字（负面到正面）,
  "keywords": ["检测到的情绪关键词"]
}"#.to_string()
    }

    /// Build the user prompt for analysis
    fn build_user_prompt(&self, text: &str) -> String {
        format!("分析以下消息的情绪：\n\n{}", text)
    }

    /// Parse LLM response into EmotionAnalysis
    fn parse_response(&self, response: &str) -> Option<EmotionAnalysis> {
        // Try to extract JSON from response
        let json_str = if response.contains('{') {
            let start = response.find('{')?;
            let end = response.rfind('}')?;
            &response[start..=end]
        } else {
            response
        };

        let parsed: LLMEmotionResponse = serde_json::from_str(json_str).ok()?;

        let primary_emotion = match parsed.primary_emotion.to_lowercase().as_str() {
            "neutral" => EmotionState::Neutral,
            "happy" => EmotionState::Happy,
            "excited" => EmotionState::Excited,
            "sad" => EmotionState::Sad,
            "angry" => EmotionState::Angry,
            "empathetic" => EmotionState::Empathetic,
            "supportive" => EmotionState::Supportive,
            "curious" => EmotionState::Curious,
            "playful" => EmotionState::Playful,
            "anxious" => EmotionState::Anxious,
            "confused" => EmotionState::Confused,
            _ => EmotionState::Neutral,
        };

        Some(EmotionAnalysis {
            primary_emotion,
            confidence: parsed.confidence.clamp(0.0, 1.0),
            secondary_emotions: vec![],
            sentiment_score: parsed.sentiment_score.clamp(-1.0, 1.0),
            detected_keywords: parsed.keywords.unwrap_or_default(),
        })
    }

    /// Build context analysis prompt
    fn build_context_prompt(&self, messages: &[String]) -> String {
        let context = messages
            .iter()
            .enumerate()
            .map(|(i, m)| format!("{}. {}", i + 1, m))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "分析以下对话的整体情绪趋势，返回最主要的情绪类型（仅返回情绪类型名称）：\n\n{}",
            context
        )
    }

    /// Parse context analysis response
    fn parse_context_response(&self, response: &str) -> EmotionState {
        let response_lower = response.to_lowercase().trim().to_string();

        match response_lower.as_str() {
            s if s.contains("happy") || s.contains("开心") => EmotionState::Happy,
            s if s.contains("excited") || s.contains("兴奋") => EmotionState::Excited,
            s if s.contains("sad") || s.contains("难过") => EmotionState::Sad,
            s if s.contains("angry") || s.contains("生气") => EmotionState::Angry,
            s if s.contains("empathetic") || s.contains("共情") => EmotionState::Empathetic,
            s if s.contains("supportive") || s.contains("支持") => EmotionState::Supportive,
            s if s.contains("curious") || s.contains("好奇") => EmotionState::Curious,
            s if s.contains("playful") || s.contains("俏皮") => EmotionState::Playful,
            s if s.contains("anxious") || s.contains("焦虑") => EmotionState::Anxious,
            s if s.contains("confused") || s.contains("困惑") => EmotionState::Confused,
            _ => EmotionState::Neutral,
        }
    }
}

/// LLM response structure
#[derive(Debug, Deserialize, Serialize)]
struct LLMEmotionResponse {
    primary_emotion: String,
    confidence: f64,
    sentiment_score: f64,
    keywords: Option<Vec<String>>,
}

#[async_trait]
impl EmotionAnalyzer for LLMEmotionAnalyzer {
    async fn analyze(&self, text: &str) -> Result<EmotionAnalysis, AIAgentError> {
        if text.trim().is_empty() {
            return Ok(EmotionAnalysis::neutral());
        }

        let system_prompt = self.build_system_prompt();
        let user_prompt = self.build_user_prompt(text);

        match self.llm_client.generate_response(&system_prompt, &user_prompt).await {
            Ok(response) => {
                debug!("LLM emotion response: {}", response);

                if let Some(analysis) = self.parse_response(&response) {
                    Ok(analysis)
                } else {
                    warn!("Failed to parse LLM emotion response, using fallback");
                    if self.fallback_enabled {
                        self.rule_based.analyze(text).await
                    } else {
                        Ok(EmotionAnalysis::neutral())
                    }
                }
            }
            Err(e) => {
                warn!("LLM emotion analysis failed: {}, using fallback", e);
                if self.fallback_enabled {
                    self.rule_based.analyze(text).await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn analyze_context(&self, messages: &[String]) -> Result<EmotionState, AIAgentError> {
        if messages.is_empty() {
            return Ok(EmotionState::Neutral);
        }

        // For single message, use regular analysis
        if messages.len() == 1 {
            let analysis = self.analyze(&messages[0]).await?;
            return Ok(analysis.primary_emotion);
        }

        let system_prompt = "你是一个情绪分析专家。分析对话的整体情绪趋势。".to_string();
        let user_prompt = self.build_context_prompt(messages);

        match self.llm_client.generate_response(&system_prompt, &user_prompt).await {
            Ok(response) => {
                Ok(self.parse_context_response(&response))
            }
            Err(e) => {
                warn!("LLM context analysis failed: {}, using fallback", e);
                if self.fallback_enabled {
                    self.rule_based.analyze_context(messages).await
                } else {
                    Err(e)
                }
            }
        }
    }
}

/// Hybrid emotion analyzer that combines rule-based and LLM analysis
///
/// Uses rule-based for quick analysis and LLM for uncertain cases.
pub struct HybridEmotionAnalyzer {
    rule_based: super::RuleBasedEmotionAnalyzer,
    llm_based: LLMEmotionAnalyzer,
    /// Confidence threshold below which to use LLM
    confidence_threshold: f64,
}

impl HybridEmotionAnalyzer {
    /// Create a new hybrid analyzer
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self {
            rule_based: super::RuleBasedEmotionAnalyzer::new(),
            llm_based: LLMEmotionAnalyzer::new(llm_client),
            confidence_threshold: 0.6,
        }
    }

    /// Set confidence threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }
}

#[async_trait]
impl EmotionAnalyzer for HybridEmotionAnalyzer {
    async fn analyze(&self, text: &str) -> Result<EmotionAnalysis, AIAgentError> {
        // First try rule-based
        let rule_result = self.rule_based.analyze(text).await?;

        // If confidence is high enough, use rule-based result
        if rule_result.confidence >= self.confidence_threshold {
            debug!(
                "Using rule-based result (confidence: {})",
                rule_result.confidence
            );
            return Ok(rule_result);
        }

        // Otherwise, use LLM for more accurate analysis
        debug!(
            "Rule-based confidence too low ({}), using LLM",
            rule_result.confidence
        );
        self.llm_based.analyze(text).await
    }

    async fn analyze_context(&self, messages: &[String]) -> Result<EmotionState, AIAgentError> {
        // For context analysis, prefer LLM as it handles context better
        self.llm_based.analyze_context(messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::MockLLMClient;

    fn create_mock_llm(response: &str) -> Arc<dyn LLMClient> {
        Arc::new(MockLLMClient::new(response.to_string()))
    }

    #[tokio::test]
    async fn test_llm_analyzer_happy() {
        let response = r#"{"primary_emotion": "happy", "confidence": 0.9, "sentiment_score": 0.8, "keywords": ["开心", "太棒了"]}"#;
        let analyzer = LLMEmotionAnalyzer::new(create_mock_llm(response));

        let result = analyzer.analyze("太棒了！我很开心！").await.unwrap();
        assert_eq!(result.primary_emotion, EmotionState::Happy);
        assert!(result.confidence > 0.8);
        assert!(result.sentiment_score > 0.5);
    }

    #[tokio::test]
    async fn test_llm_analyzer_sad() {
        let response = r#"{"primary_emotion": "sad", "confidence": 0.85, "sentiment_score": -0.6, "keywords": ["难过"]}"#;
        let analyzer = LLMEmotionAnalyzer::new(create_mock_llm(response));

        let result = analyzer.analyze("我很难过...").await.unwrap();
        assert_eq!(result.primary_emotion, EmotionState::Sad);
        assert!(result.sentiment_score < 0.0);
    }

    #[tokio::test]
    async fn test_llm_analyzer_empty_text() {
        let analyzer = LLMEmotionAnalyzer::new(create_mock_llm(""));

        let result = analyzer.analyze("").await.unwrap();
        assert_eq!(result.primary_emotion, EmotionState::Neutral);
    }

    #[tokio::test]
    async fn test_llm_analyzer_fallback() {
        let analyzer = LLMEmotionAnalyzer::new(Arc::new(MockLLMClient::failing()))
            .with_fallback(true);

        // Should fallback to rule-based
        let result = analyzer.analyze("太棒了！").await.unwrap();
        // Rule-based should detect happy
        assert!(matches!(
            result.primary_emotion,
            EmotionState::Happy | EmotionState::Neutral
        ));
    }

    #[tokio::test]
    async fn test_llm_analyzer_no_fallback() {
        let analyzer = LLMEmotionAnalyzer::new(Arc::new(MockLLMClient::failing()))
            .with_fallback(false);

        let result = analyzer.analyze("太棒了！").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_llm_analyzer_context() {
        let response = "happy";
        let analyzer = LLMEmotionAnalyzer::new(create_mock_llm(response));

        let messages = vec![
            "今天天气真好！".to_string(),
            "是啊，心情也很好".to_string(),
        ];

        let result = analyzer.analyze_context(&messages).await.unwrap();
        assert_eq!(result, EmotionState::Happy);
    }

    #[tokio::test]
    async fn test_hybrid_analyzer_high_confidence() {
        // LLM won't be called if rule-based confidence is high
        let analyzer = HybridEmotionAnalyzer::new(create_mock_llm(""))
            .with_threshold(0.3);

        let result = analyzer.analyze("太棒了！哈哈哈！😄").await.unwrap();
        // Should use rule-based result
        assert!(matches!(
            result.primary_emotion,
            EmotionState::Happy | EmotionState::Playful
        ));
    }

    #[tokio::test]
    async fn test_parse_response_with_extra_text() {
        let response = r#"根据分析，结果如下：
{"primary_emotion": "curious", "confidence": 0.75, "sentiment_score": 0.1, "keywords": ["为什么"]}
这是一个好奇的问题。"#;

        let analyzer = LLMEmotionAnalyzer::new(create_mock_llm(response));
        let result = analyzer.parse_response(response);

        assert!(result.is_some());
        let analysis = result.unwrap();
        assert_eq!(analysis.primary_emotion, EmotionState::Curious);
    }

    #[test]
    fn test_parse_context_response() {
        let analyzer = LLMEmotionAnalyzer::new(create_mock_llm(""));

        assert_eq!(
            analyzer.parse_context_response("happy"),
            EmotionState::Happy
        );
        assert_eq!(
            analyzer.parse_context_response("开心"),
            EmotionState::Happy
        );
        assert_eq!(
            analyzer.parse_context_response("The emotion is excited"),
            EmotionState::Excited
        );
        assert_eq!(
            analyzer.parse_context_response("unknown"),
            EmotionState::Neutral
        );
    }
}
