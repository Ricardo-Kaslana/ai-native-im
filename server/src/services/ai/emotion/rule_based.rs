//! Rule-Based Emotion Analyzer
//!
//! Fast emotion detection using keyword matching and emoji analysis.
//! No API calls required - suitable for real-time analysis.

use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, instrument};

use super::{EmotionAnalysis, EmotionAnalyzer, EmotionState};
use crate::services::ai::AIAgentError;

/// Chinese emotion keywords
#[derive(Debug, Clone)]
pub struct EmotionKeywords {
    pub happy: Vec<&'static str>,
    pub excited: Vec<&'static str>,
    pub sad: Vec<&'static str>,
    pub angry: Vec<&'static str>,
    pub curious: Vec<&'static str>,
    pub anxious: Vec<&'static str>,
    pub confused: Vec<&'static str>,
    pub playful: Vec<&'static str>,
}

impl Default for EmotionKeywords {
    fn default() -> Self {
        Self::chinese()
    }
}

impl EmotionKeywords {
    /// Create Chinese emotion keywords
    pub fn chinese() -> Self {
        Self {
            happy: vec![
                "开心", "高兴", "太棒了", "哈哈", "好耶", "太好了", "真棒", "厉害", "赞",
                "喜欢", "爱", "感谢", "谢谢", "棒", "好", "耶", "嘻嘻", "嘿嘿", "哇",
                "太开心", "超开心", "好开心", "真开心", "很开心", "非常开心",
            ],
            excited: vec![
                "兴奋", "激动", "太激动了", "好激动", "期待", "迫不及待", "终于",
                "太棒了", "太厉害了", "绝了", "牛", "666", "厉害了", "震惊",
            ],
            sad: vec![
                "难过", "伤心", "哭了", "失望", "郁闷", "唉", "呜呜", "悲伤",
                "心痛", "难受", "不开心", "沮丧", "低落", "委屈", "可怜",
            ],
            angry: vec![
                "生气", "愤怒", "烦死了", "讨厌", "气死", "恼火", "火大",
                "受不了", "忍不了", "太过分", "可恶", "混蛋", "滚", "烦",
            ],
            curious: vec![
                "好奇", "想知道", "为什么", "怎么", "什么", "是吗", "真的吗",
                "怎么回事", "发生了什么", "告诉我", "请问", "想了解",
            ],
            anxious: vec![
                "焦虑", "担心", "紧张", "害怕", "恐惧", "不安", "慌",
                "怎么办", "完了", "糟糕", "惨了", "急", "着急",
            ],
            confused: vec![
                "困惑", "不懂", "不明白", "迷茫", "搞不懂", "看不懂",
                "什么意思", "啥意思", "不理解", "疑惑", "奇怪",
            ],
            playful: vec![
                "哈哈哈", "笑死", "有趣", "好玩", "搞笑", "逗", "皮",
                "调皮", "开玩笑", "玩笑", "嘿嘿嘿", "嘻嘻嘻",
            ],
        }
    }

    /// Get keywords for a specific emotion
    pub fn get(&self, emotion: EmotionState) -> &[&'static str] {
        match emotion {
            EmotionState::Happy => &self.happy,
            EmotionState::Excited => &self.excited,
            EmotionState::Sad => &self.sad,
            EmotionState::Angry => &self.angry,
            EmotionState::Curious => &self.curious,
            EmotionState::Anxious => &self.anxious,
            EmotionState::Confused => &self.confused,
            EmotionState::Playful => &self.playful,
            _ => &[],
        }
    }
}

/// Emoji emotion mappings
#[derive(Debug, Clone)]
pub struct EmojiMappings {
    mappings: HashMap<&'static str, (EmotionState, f64)>,
}

impl Default for EmojiMappings {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiMappings {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Happy emojis
        for emoji in &["😄", "😊", "😃", "😁", "🙂", "😀", "☺️", "🥰", "😍", "🤗", "👍", "👏", "🎉", "✨"] {
            mappings.insert(*emoji, (EmotionState::Happy, 0.8));
        }

        // Excited emojis
        for emoji in &["🤩", "🥳", "🎊", "🔥", "💪", "⚡", "🚀"] {
            mappings.insert(*emoji, (EmotionState::Excited, 0.8));
        }

        // Sad emojis
        for emoji in &["😢", "😭", "😞", "😔", "🥺", "😿", "💔"] {
            mappings.insert(*emoji, (EmotionState::Sad, 0.8));
        }

        // Angry emojis
        for emoji in &["😠", "😡", "🤬", "💢", "😤"] {
            mappings.insert(*emoji, (EmotionState::Angry, 0.8));
        }

        // Curious emojis
        for emoji in &["🤔", "❓", "❔", "🧐"] {
            mappings.insert(*emoji, (EmotionState::Curious, 0.7));
        }

        // Anxious emojis
        for emoji in &["😰", "😨", "😱", "😥", "😓"] {
            mappings.insert(*emoji, (EmotionState::Anxious, 0.7));
        }

        // Confused emojis
        for emoji in &["😕", "😵", "🤷", "❓"] {
            mappings.insert(*emoji, (EmotionState::Confused, 0.6));
        }

        // Playful emojis
        for emoji in &["😜", "😝", "🤪", "😏", "😎", "🤣", "😂"] {
            mappings.insert(*emoji, (EmotionState::Playful, 0.7));
        }

        Self { mappings }
    }

    /// Analyze emojis in text
    pub fn analyze(&self, text: &str) -> Vec<(EmotionState, f64)> {
        let mut results = Vec::new();

        for (emoji, (emotion, score)) in &self.mappings {
            if text.contains(emoji) {
                results.push((*emotion, *score));
            }
        }

        results
    }
}

/// Rule-based emotion analyzer
pub struct RuleBasedEmotionAnalyzer {
    keywords: EmotionKeywords,
    emoji_mappings: EmojiMappings,
}

impl RuleBasedEmotionAnalyzer {
    /// Create a new analyzer with default Chinese keywords
    pub fn new() -> Self {
        Self {
            keywords: EmotionKeywords::chinese(),
            emoji_mappings: EmojiMappings::new(),
        }
    }

    /// Create with custom keywords
    pub fn with_keywords(keywords: EmotionKeywords) -> Self {
        Self {
            keywords,
            emoji_mappings: EmojiMappings::new(),
        }
    }

    /// Synchronous analysis (for internal use)
    #[instrument(skip(self), fields(text_len = text.len()))]
    pub fn analyze_sync(&self, text: &str) -> EmotionAnalysis {
        let text_lower = text.to_lowercase();
        let mut scores: HashMap<EmotionState, f64> = HashMap::new();
        let mut detected_keywords = Vec::new();

        // Check keywords for each emotion
        let emotions_to_check = [
            EmotionState::Happy,
            EmotionState::Excited,
            EmotionState::Sad,
            EmotionState::Angry,
            EmotionState::Curious,
            EmotionState::Anxious,
            EmotionState::Confused,
            EmotionState::Playful,
        ];

        for emotion in emotions_to_check {
            let keywords = self.keywords.get(emotion);
            for keyword in keywords {
                if text_lower.contains(keyword) || text.contains(keyword) {
                    *scores.entry(emotion).or_insert(0.0) += 0.3;
                    detected_keywords.push(keyword.to_string());
                }
            }
        }

        // Check emojis
        let emoji_results = self.emoji_mappings.analyze(text);
        for (emotion, score) in emoji_results {
            *scores.entry(emotion).or_insert(0.0) += score;
        }

        // Find primary emotion
        let (primary_emotion, primary_score) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(e, s)| (*e, *s))
            .unwrap_or((EmotionState::Neutral, 0.0));

        // Calculate confidence (capped at 1.0)
        let confidence = primary_score.min(1.0);

        // Get secondary emotions
        let mut secondary_emotions: Vec<(EmotionState, f64)> = scores
            .into_iter()
            .filter(|(e, _)| *e != primary_emotion)
            .map(|(e, s)| (e, s.min(1.0)))
            .collect();
        secondary_emotions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Calculate sentiment score
        let sentiment_score = self.calculate_sentiment(&primary_emotion, confidence);

        debug!(
            primary = ?primary_emotion,
            confidence = confidence,
            sentiment = sentiment_score,
            "Emotion analysis complete"
        );

        EmotionAnalysis {
            primary_emotion: if confidence > 0.2 {
                primary_emotion
            } else {
                EmotionState::Neutral
            },
            confidence,
            secondary_emotions,
            sentiment_score,
            detected_keywords,
        }
    }

    /// Calculate sentiment score based on emotion
    fn calculate_sentiment(&self, emotion: &EmotionState, confidence: f64) -> f64 {
        let base_sentiment = match emotion {
            EmotionState::Happy => 0.7,
            EmotionState::Excited => 0.8,
            EmotionState::Playful => 0.5,
            EmotionState::Curious => 0.2,
            EmotionState::Neutral => 0.0,
            EmotionState::Confused => -0.1,
            EmotionState::Anxious => -0.4,
            EmotionState::Sad => -0.6,
            EmotionState::Angry => -0.7,
            EmotionState::Empathetic => 0.3,
            EmotionState::Supportive => 0.4,
        };

        base_sentiment * confidence
    }
}

impl Default for RuleBasedEmotionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmotionAnalyzer for RuleBasedEmotionAnalyzer {
    async fn analyze(&self, text: &str) -> Result<EmotionAnalysis, AIAgentError> {
        Ok(self.analyze_sync(text))
    }

    async fn analyze_context(&self, messages: &[String]) -> Result<EmotionState, AIAgentError> {
        if messages.is_empty() {
            return Ok(EmotionState::Neutral);
        }

        // Analyze recent messages and find dominant emotion
        let mut emotion_counts: HashMap<EmotionState, usize> = HashMap::new();

        for msg in messages.iter().rev().take(5) {
            let analysis = self.analyze_sync(msg);
            *emotion_counts.entry(analysis.primary_emotion).or_insert(0) += 1;
        }

        let dominant = emotion_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(emotion, _)| emotion)
            .unwrap_or(EmotionState::Neutral);

        Ok(dominant)
    }
}

/// Mock emotion analyzer for testing
pub struct MockEmotionAnalyzer {
    emotion: EmotionState,
    confidence: f64,
}

impl MockEmotionAnalyzer {
    pub fn new(emotion: EmotionState) -> Self {
        Self {
            emotion,
            confidence: 0.8,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

#[async_trait]
impl EmotionAnalyzer for MockEmotionAnalyzer {
    async fn analyze(&self, _text: &str) -> Result<EmotionAnalysis, AIAgentError> {
        Ok(EmotionAnalysis {
            primary_emotion: self.emotion,
            confidence: self.confidence,
            secondary_emotions: vec![],
            sentiment_score: 0.5,
            detected_keywords: vec![],
        })
    }

    async fn analyze_context(&self, _messages: &[String]) -> Result<EmotionState, AIAgentError> {
        Ok(self.emotion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_keywords_chinese() {
        let keywords = EmotionKeywords::chinese();
        assert!(!keywords.happy.is_empty());
        assert!(!keywords.sad.is_empty());
        assert!(!keywords.angry.is_empty());
    }

    #[test]
    fn test_emoji_mappings() {
        let mappings = EmojiMappings::new();
        let results = mappings.analyze("我很开心 😄");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(e, _)| *e == EmotionState::Happy));
    }

    #[tokio::test]
    async fn test_rule_based_analyzer_happy() {
        let analyzer = RuleBasedEmotionAnalyzer::new();
        let result = analyzer.analyze("太棒了！我好开心！😄").await.unwrap();

        assert_eq!(result.primary_emotion, EmotionState::Happy);
        assert!(result.confidence > 0.5);
        assert!(result.is_positive());
    }

    #[tokio::test]
    async fn test_rule_based_analyzer_sad() {
        let analyzer = RuleBasedEmotionAnalyzer::new();
        let result = analyzer.analyze("我好难过 😢").await.unwrap();

        assert_eq!(result.primary_emotion, EmotionState::Sad);
        assert!(result.is_negative());
    }

    #[tokio::test]
    async fn test_rule_based_analyzer_angry() {
        let analyzer = RuleBasedEmotionAnalyzer::new();
        let result = analyzer.analyze("气死我了！太过分了！😠").await.unwrap();

        assert_eq!(result.primary_emotion, EmotionState::Angry);
        assert!(result.is_negative());
    }

    #[tokio::test]
    async fn test_rule_based_analyzer_neutral() {
        let analyzer = RuleBasedEmotionAnalyzer::new();
        let result = analyzer.analyze("今天天气不错").await.unwrap();

        // Should be neutral or low confidence
        assert!(result.confidence < 0.5 || result.primary_emotion == EmotionState::Neutral);
    }

    #[tokio::test]
    async fn test_analyze_context() {
        let analyzer = RuleBasedEmotionAnalyzer::new();
        let messages = vec![
            "太开心了！".to_string(),
            "真的很高兴！".to_string(),
            "好棒！".to_string(),
        ];

        let result = analyzer.analyze_context(&messages).await.unwrap();
        assert_eq!(result, EmotionState::Happy);
    }

    #[tokio::test]
    async fn test_mock_analyzer() {
        let analyzer = MockEmotionAnalyzer::new(EmotionState::Excited);
        let result = analyzer.analyze("any text").await.unwrap();

        assert_eq!(result.primary_emotion, EmotionState::Excited);
    }

    // Property test: Emotion analysis confidence bounds
    #[test]
    fn test_emotion_confidence_bounds_property() {
        use proptest::prelude::*;

        let analyzer = RuleBasedEmotionAnalyzer::new();

        proptest!(|(text in ".*")| {
            let result = analyzer.analyze_sync(&text);
            prop_assert!(result.confidence >= 0.0);
            prop_assert!(result.confidence <= 1.0);
            prop_assert!(result.sentiment_score >= -1.0);
            prop_assert!(result.sentiment_score <= 1.0);
        });
    }
}
