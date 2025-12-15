//! Emotion Analysis Module
//!
//! Provides emotion detection and analysis for chat messages.
//!
//! Features:
//! - Rule-based emotion detection (fast, no API calls)
//! - LLM-based emotion analysis (accurate, requires API)
//! - Hybrid analyzer (combines both approaches)
//! - Chinese emotion keyword support
//! - Emoji and emoticon detection

mod llm_based;
mod rule_based;

pub use llm_based::*;
pub use rule_based::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::AIAgentError;

/// Emotion state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionState {
    /// Neutral/calm state
    #[default]
    Neutral,
    /// Happy/joyful
    Happy,
    /// Excited/enthusiastic
    Excited,
    /// Sad/melancholic
    Sad,
    /// Angry/frustrated
    Angry,
    /// Empathetic/understanding
    Empathetic,
    /// Supportive/encouraging
    Supportive,
    /// Curious/interested
    Curious,
    /// Playful/humorous
    Playful,
    /// Anxious/worried
    Anxious,
    /// Confused/uncertain
    Confused,
}

impl EmotionState {
    /// Get all emotion states
    pub fn all() -> &'static [EmotionState] {
        &[
            EmotionState::Neutral,
            EmotionState::Happy,
            EmotionState::Excited,
            EmotionState::Sad,
            EmotionState::Angry,
            EmotionState::Empathetic,
            EmotionState::Supportive,
            EmotionState::Curious,
            EmotionState::Playful,
            EmotionState::Anxious,
            EmotionState::Confused,
        ]
    }

    /// Get Chinese name for the emotion
    pub fn chinese_name(&self) -> &'static str {
        match self {
            EmotionState::Neutral => "平静",
            EmotionState::Happy => "开心",
            EmotionState::Excited => "兴奋",
            EmotionState::Sad => "难过",
            EmotionState::Angry => "生气",
            EmotionState::Empathetic => "共情",
            EmotionState::Supportive => "支持",
            EmotionState::Curious => "好奇",
            EmotionState::Playful => "俏皮",
            EmotionState::Anxious => "焦虑",
            EmotionState::Confused => "困惑",
        }
    }

    /// Get suggested response tone for this emotion
    pub fn response_tone(&self) -> &'static str {
        match self {
            EmotionState::Neutral => "保持友好和专业",
            EmotionState::Happy => "分享喜悦，积极回应",
            EmotionState::Excited => "热情回应，表达兴奋",
            EmotionState::Sad => "表达理解和安慰",
            EmotionState::Angry => "保持冷静，表示理解",
            EmotionState::Empathetic => "深入理解，提供支持",
            EmotionState::Supportive => "给予鼓励和肯定",
            EmotionState::Curious => "提供信息，满足好奇心",
            EmotionState::Playful => "轻松幽默地回应",
            EmotionState::Anxious => "安抚情绪，提供帮助",
            EmotionState::Confused => "耐心解释，提供清晰信息",
        }
    }
}

impl std::fmt::Display for EmotionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chinese_name())
    }
}

/// Emotion analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionAnalysis {
    /// Primary detected emotion
    pub primary_emotion: EmotionState,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Secondary emotions with their scores
    pub secondary_emotions: Vec<(EmotionState, f64)>,
    /// Overall sentiment score (-1.0 negative to 1.0 positive)
    pub sentiment_score: f64,
    /// Detected emotional keywords
    pub detected_keywords: Vec<String>,
}

impl EmotionAnalysis {
    /// Create a neutral analysis
    pub fn neutral() -> Self {
        Self {
            primary_emotion: EmotionState::Neutral,
            confidence: 0.5,
            secondary_emotions: vec![],
            sentiment_score: 0.0,
            detected_keywords: vec![],
        }
    }

    /// Check if the emotion is positive
    pub fn is_positive(&self) -> bool {
        self.sentiment_score > 0.2
    }

    /// Check if the emotion is negative
    pub fn is_negative(&self) -> bool {
        self.sentiment_score < -0.2
    }

    /// Get the suggested response approach
    pub fn response_suggestion(&self) -> String {
        let tone = self.primary_emotion.response_tone();
        if self.confidence > 0.7 {
            format!("建议：{}", tone)
        } else {
            format!("建议（低置信度）：{}", tone)
        }
    }
}

impl Default for EmotionAnalysis {
    fn default() -> Self {
        Self::neutral()
    }
}

/// Emotion analyzer trait
#[async_trait]
pub trait EmotionAnalyzer: Send + Sync {
    /// Analyze emotion in a single text
    async fn analyze(&self, text: &str) -> Result<EmotionAnalysis, AIAgentError>;

    /// Analyze emotion across multiple messages (context-aware)
    async fn analyze_context(&self, messages: &[String]) -> Result<EmotionState, AIAgentError>;
}

/// Chat message for context analysis
#[derive(Debug, Clone)]
pub struct ChatMessageForEmotion {
    pub content: String,
    pub is_user: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_state_all() {
        let all = EmotionState::all();
        assert!(all.len() >= 10);
        assert!(all.contains(&EmotionState::Neutral));
        assert!(all.contains(&EmotionState::Happy));
    }

    #[test]
    fn test_emotion_state_chinese_name() {
        assert_eq!(EmotionState::Happy.chinese_name(), "开心");
        assert_eq!(EmotionState::Sad.chinese_name(), "难过");
    }

    #[test]
    fn test_emotion_analysis_neutral() {
        let analysis = EmotionAnalysis::neutral();
        assert_eq!(analysis.primary_emotion, EmotionState::Neutral);
        assert!(!analysis.is_positive());
        assert!(!analysis.is_negative());
    }

    #[test]
    fn test_emotion_analysis_sentiment() {
        let mut analysis = EmotionAnalysis::neutral();

        analysis.sentiment_score = 0.5;
        assert!(analysis.is_positive());
        assert!(!analysis.is_negative());

        analysis.sentiment_score = -0.5;
        assert!(!analysis.is_positive());
        assert!(analysis.is_negative());
    }

    #[test]
    fn test_emotion_state_display() {
        assert_eq!(format!("{}", EmotionState::Happy), "开心");
    }
}
