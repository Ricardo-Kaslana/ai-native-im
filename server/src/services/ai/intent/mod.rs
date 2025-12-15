//! Intent Classification Module
//!
//! Classifies user messages into different intent categories to help
//! AI agents adjust their response strategies accordingly.

use serde::{Deserialize, Serialize};

/// Primary intent categories for user messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    /// User is asking a question that expects a factual answer
    Question,
    /// User needs help with a problem or task
    HelpRequest,
    /// Casual conversation, chitchat, social interaction
    Chitchat,
    /// User is expressing emotions or venting
    Emotional,
    /// User is giving a command or making a request
    Command,
    /// User is providing feedback or opinion
    Feedback,
    /// User is greeting or saying goodbye
    Greeting,
    /// User is sharing information or news
    Sharing,
    /// User is joking or being playful
    Humor,
    /// User's intent is unclear or mixed
    Unclear,
}

impl Intent {
    /// Get all intent variants
    pub fn all() -> &'static [Intent] {
        &[
            Intent::Question,
            Intent::HelpRequest,
            Intent::Chitchat,
            Intent::Emotional,
            Intent::Command,
            Intent::Feedback,
            Intent::Greeting,
            Intent::Sharing,
            Intent::Humor,
            Intent::Unclear,
        ]
    }

    /// Get the display name in Chinese
    pub fn display_name(&self) -> &'static str {
        match self {
            Intent::Question => "提问",
            Intent::HelpRequest => "求助",
            Intent::Chitchat => "闲聊",
            Intent::Emotional => "情绪表达",
            Intent::Command => "指令",
            Intent::Feedback => "反馈",
            Intent::Greeting => "问候",
            Intent::Sharing => "分享",
            Intent::Humor => "玩笑",
            Intent::Unclear => "不明确",
        }
    }

    /// Get the response strategy hint for this intent
    pub fn response_strategy(&self) -> ResponseStrategy {
        match self {
            Intent::Question => ResponseStrategy {
                should_be_informative: true,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: false,
                priority: ResponsePriority::High,
            },
            Intent::HelpRequest => ResponseStrategy {
                should_be_informative: true,
                should_be_concise: false,
                emotional_support: true,
                can_be_playful: false,
                priority: ResponsePriority::High,
            },
            Intent::Chitchat => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: true,
                priority: ResponsePriority::Normal,
            },
            Intent::Emotional => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: false,
                emotional_support: true,
                can_be_playful: false,
                priority: ResponsePriority::High,
            },
            Intent::Command => ResponseStrategy {
                should_be_informative: true,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: false,
                priority: ResponsePriority::High,
            },
            Intent::Feedback => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: true,
                emotional_support: true,
                can_be_playful: false,
                priority: ResponsePriority::Normal,
            },
            Intent::Greeting => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: true,
                priority: ResponsePriority::Low,
            },
            Intent::Sharing => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: true,
                priority: ResponsePriority::Normal,
            },
            Intent::Humor => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: true,
                priority: ResponsePriority::Low,
            },
            Intent::Unclear => ResponseStrategy {
                should_be_informative: false,
                should_be_concise: true,
                emotional_support: false,
                can_be_playful: false,
                priority: ResponsePriority::Normal,
            },
        }
    }
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Response priority based on intent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponsePriority {
    /// Should respond quickly and thoughtfully
    High,
    /// Normal response timing
    Normal,
    /// Can delay or skip if busy
    Low,
}

/// Strategy hints for generating responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStrategy {
    /// Should the response contain factual/helpful information
    pub should_be_informative: bool,
    /// Should the response be brief
    pub should_be_concise: bool,
    /// Should the response provide emotional support
    pub emotional_support: bool,
    /// Can the response be playful/humorous
    pub can_be_playful: bool,
    /// Priority level for this response
    pub priority: ResponsePriority,
}

/// Result of intent classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// Primary detected intent
    pub primary_intent: Intent,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Secondary intents if message has multiple purposes
    pub secondary_intents: Vec<(Intent, f32)>,
    /// Detected entities or key phrases
    pub entities: Vec<String>,
    /// Response strategy based on intent
    pub strategy: ResponseStrategy,
}

impl IntentResult {
    /// Create a new intent result with a single intent
    pub fn new(intent: Intent, confidence: f32) -> Self {
        Self {
            strategy: intent.response_strategy(),
            primary_intent: intent,
            confidence,
            secondary_intents: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Add a secondary intent
    pub fn with_secondary(mut self, intent: Intent, confidence: f32) -> Self {
        self.secondary_intents.push((intent, confidence));
        self
    }

    /// Add detected entities
    pub fn with_entities(mut self, entities: Vec<String>) -> Self {
        self.entities = entities;
        self
    }
}

/// Intent classifier using rule-based approach
/// Can be enhanced with ML models in the future
pub struct IntentClassifier {
    /// Keywords for each intent category
    intent_keywords: IntentKeywords,
}

/// Keywords associated with each intent
#[derive(Debug, Clone)]
struct IntentKeywords {
    question: Vec<&'static str>,
    help_request: Vec<&'static str>,
    emotional: Vec<&'static str>,
    command: Vec<&'static str>,
    greeting: Vec<&'static str>,
    humor: Vec<&'static str>,
}

impl Default for IntentKeywords {
    fn default() -> Self {
        Self {
            question: vec![
                "什么", "怎么", "如何", "为什么", "哪里", "哪个", "谁", "几",
                "多少", "是不是", "能不能", "可以吗", "吗？", "呢？", "?",
                "what", "how", "why", "where", "which", "who", "when",
            ],
            help_request: vec![
                "帮我", "帮忙", "求助", "救命", "怎么办", "不会", "不懂",
                "教我", "指导", "解决", "处理", "help", "please", "能帮",
                "麻烦", "搞不定", "卡住", "出错", "报错", "失败",
            ],
            emotional: vec![
                "开心", "高兴", "难过", "伤心", "生气", "愤怒", "害怕",
                "担心", "焦虑", "烦", "累", "郁闷", "无聊", "兴奋",
                "感动", "心情", "情绪", "😭", "😢", "😡", "😊", "🥺",
                "呜呜", "哭", "泪", "崩溃", "emo", "破防",
            ],
            command: vec![
                "请", "帮", "给我", "告诉我", "发送", "查询", "搜索",
                "打开", "关闭", "设置", "更改", "执行", "运行", "do",
                "send", "get", "show", "list", "create", "delete",
            ],
            greeting: vec![
                "你好", "早", "晚安", "hello", "hi", "hey", "嗨", "哈喽",
                "再见", "拜拜", "bye", "晚上好", "下午好", "早上好",
                "在吗", "在不在", "有人吗",
            ],
            humor: vec![
                "哈哈", "笑死", "太好笑", "搞笑", "逗", "梗", "笑",
                "lol", "haha", "xd", "233", "666", "绝了", "笑不活",
                "整活", "抽象", "离谱",
            ],
        }
    }
}

impl IntentClassifier {
    /// Create a new intent classifier
    pub fn new() -> Self {
        Self {
            intent_keywords: IntentKeywords::default(),
        }
    }

    /// Classify a message's intent
    pub fn classify(&self, message: &str) -> IntentResult {
        let message_lower = message.to_lowercase();
        let mut scores: Vec<(Intent, f32)> = Vec::new();

        // Check each intent category
        scores.push((Intent::Question, self.score_question(&message_lower)));
        scores.push((Intent::HelpRequest, self.score_help_request(&message_lower)));
        scores.push((Intent::Emotional, self.score_emotional(&message_lower)));
        scores.push((Intent::Command, self.score_command(&message_lower)));
        scores.push((Intent::Greeting, self.score_greeting(&message_lower)));
        scores.push((Intent::Humor, self.score_humor(&message_lower)));

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Get primary intent
        let (primary_intent, primary_confidence) = if scores[0].1 > 0.3 {
            scores[0]
        } else {
            // Default to chitchat if no strong signal
            (Intent::Chitchat, 0.5)
        };

        // Get secondary intents (those with significant scores)
        let secondary_intents: Vec<(Intent, f32)> = scores
            .into_iter()
            .skip(1)
            .filter(|(_, score)| *score > 0.2)
            .take(2)
            .collect();

        // Extract entities (simple keyword extraction)
        let entities = self.extract_entities(&message_lower);

        IntentResult::new(primary_intent, primary_confidence)
            .with_secondary_intents(secondary_intents)
            .with_entities(entities)
    }

    /// Score how likely the message is a question
    fn score_question(&self, message: &str) -> f32 {
        let mut score: f32 = 0.0;
        
        // Check for question marks
        if message.contains('?') || message.contains('？') {
            score += 0.4;
        }
        
        // Check for question keywords
        for keyword in &self.intent_keywords.question {
            if message.contains(keyword) {
                score += 0.2;
            }
        }

        score.min(1.0_f32)
    }

    /// Score how likely the message is a help request
    fn score_help_request(&self, message: &str) -> f32 {
        let mut score: f32 = 0.0;
        
        for keyword in &self.intent_keywords.help_request {
            if message.contains(keyword) {
                score += 0.25;
            }
        }

        score.min(1.0_f32)
    }

    /// Score how likely the message is emotional
    fn score_emotional(&self, message: &str) -> f32 {
        let mut score: f32 = 0.0;
        
        for keyword in &self.intent_keywords.emotional {
            if message.contains(keyword) {
                score += 0.3;
            }
        }

        // Exclamation marks can indicate emotion
        let exclamation_count = message.matches(['!', '！']).count();
        if exclamation_count > 2 {
            score += 0.2;
        }

        score.min(1.0_f32)
    }

    /// Score how likely the message is a command
    fn score_command(&self, message: &str) -> f32 {
        let mut score: f32 = 0.0;
        
        for keyword in &self.intent_keywords.command {
            if message.starts_with(keyword) || message.contains(&format!(" {}", keyword)) {
                score += 0.25;
            }
        }

        // Commands are often short and imperative
        if message.len() < 50 && !message.contains('?') {
            score += 0.1;
        }

        score.min(1.0_f32)
    }

    /// Score how likely the message is a greeting
    fn score_greeting(&self, message: &str) -> f32 {
        let mut score: f32 = 0.0;
        
        for keyword in &self.intent_keywords.greeting {
            if message.contains(keyword) {
                score += 0.4;
            }
        }

        // Greetings are usually short
        if message.len() < 20 {
            score += 0.1;
        }

        score.min(1.0_f32)
    }

    /// Score how likely the message is humorous
    fn score_humor(&self, message: &str) -> f32 {
        let mut score: f32 = 0.0;
        
        for keyword in &self.intent_keywords.humor {
            if message.contains(keyword) {
                score += 0.3;
            }
        }

        score.min(1.0_f32)
    }

    /// Extract key entities from message
    fn extract_entities(&self, message: &str) -> Vec<String> {
        let mut entities = Vec::new();
        
        // Simple entity extraction - can be enhanced with NER
        // Look for quoted strings
        let quote_chars: &[char] = &['"', '\u{201c}', '\u{201d}', '\u{2018}', '\u{2019}'];
        for cap in message.split(quote_chars).enumerate() {
            if cap.0 % 2 == 1 && !cap.1.is_empty() {
                entities.push(cap.1.to_string());
            }
        }

        // Look for @mentions
        for word in message.split_whitespace() {
            if word.starts_with('@') && word.len() > 1 {
                entities.push(word.to_string());
            }
        }

        entities
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentResult {
    fn with_secondary_intents(mut self, intents: Vec<(Intent, f32)>) -> Self {
        self.secondary_intents = intents;
        self
    }
}

/// Generate prompt modification based on intent
pub fn generate_intent_prompt(result: &IntentResult) -> String {
    let mut prompt = format!(
        "\n用户意图分析：\n- 主要意图：{} (置信度: {:.0}%)\n",
        result.primary_intent.display_name(),
        result.confidence * 100.0
    );

    if !result.secondary_intents.is_empty() {
        prompt.push_str("- 次要意图：");
        for (intent, conf) in &result.secondary_intents {
            prompt.push_str(&format!("{}({:.0}%) ", intent.display_name(), conf * 100.0));
        }
        prompt.push('\n');
    }

    // Add strategy hints
    let strategy = &result.strategy;
    prompt.push_str("\n回复策略建议：\n");
    
    if strategy.should_be_informative {
        prompt.push_str("- 提供有用的信息或答案\n");
    }
    if strategy.should_be_concise {
        prompt.push_str("- 保持回复简洁\n");
    }
    if strategy.emotional_support {
        prompt.push_str("- 注意情感支持和共情\n");
    }
    if strategy.can_be_playful {
        prompt.push_str("- 可以适当轻松幽默\n");
    }

    match strategy.priority {
        ResponsePriority::High => prompt.push_str("- 优先级：高，认真对待\n"),
        ResponsePriority::Normal => {},
        ResponsePriority::Low => prompt.push_str("- 优先级：低，可以简短回复\n"),
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_classification() {
        let classifier = IntentClassifier::new();
        
        let result = classifier.classify("这个功能怎么用？");
        assert_eq!(result.primary_intent, Intent::Question);
        
        let result = classifier.classify("什么是 Rust？");
        assert_eq!(result.primary_intent, Intent::Question);
    }

    #[test]
    fn test_help_request_classification() {
        let classifier = IntentClassifier::new();
        
        let result = classifier.classify("帮我看看这段代码哪里出错了");
        assert_eq!(result.primary_intent, Intent::HelpRequest);
        
        let result = classifier.classify("求助，程序报错了怎么办");
        assert_eq!(result.primary_intent, Intent::HelpRequest);
    }

    #[test]
    fn test_greeting_classification() {
        let classifier = IntentClassifier::new();
        
        let result = classifier.classify("早上好！");
        assert_eq!(result.primary_intent, Intent::Greeting);
        
        let result = classifier.classify("hello");
        assert_eq!(result.primary_intent, Intent::Greeting);
    }

    #[test]
    fn test_emotional_classification() {
        let classifier = IntentClassifier::new();
        
        let result = classifier.classify("太开心了！！！");
        assert_eq!(result.primary_intent, Intent::Emotional);
        
        let result = classifier.classify("心情好郁闷😢");
        assert_eq!(result.primary_intent, Intent::Emotional);
    }

    #[test]
    fn test_humor_classification() {
        let classifier = IntentClassifier::new();
        
        let result = classifier.classify("哈哈哈笑死我了");
        assert_eq!(result.primary_intent, Intent::Humor);
    }

    #[test]
    fn test_chitchat_default() {
        let classifier = IntentClassifier::new();
        
        let result = classifier.classify("今天天气不错");
        assert_eq!(result.primary_intent, Intent::Chitchat);
    }

    #[test]
    fn test_response_strategy() {
        let strategy = Intent::HelpRequest.response_strategy();
        assert!(strategy.should_be_informative);
        assert!(strategy.emotional_support);
        assert_eq!(strategy.priority, ResponsePriority::High);
        
        let strategy = Intent::Humor.response_strategy();
        assert!(strategy.can_be_playful);
        assert_eq!(strategy.priority, ResponsePriority::Low);
    }
}
