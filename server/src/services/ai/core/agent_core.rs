//! AgentCore Trait Definition
//!
//! This module defines the core trait for AI Agent behavior.
//! The AgentCore trait represents the "Brain" of the cognitive architecture,
//! responsible for processing events and producing actions.
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 2.1, 2.2, 2.5**

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AgentAction, AgentEvent, EmotionState, MessageContext};
use crate::services::ai::AIAgentError;

// ============================================================================
// Probability Type
// ============================================================================

/// Probability value constrained to [0.0, 1.0]
///
/// Used for trigger evaluation and decision making.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Probability(f64);

impl Probability {
    /// Create a new probability value, clamping to [0.0, 1.0]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Create a probability of 0.0 (never)
    pub fn never() -> Self {
        Self(0.0)
    }

    /// Create a probability of 1.0 (always)
    pub fn always() -> Self {
        Self(1.0)
    }

    /// Get the raw probability value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if should respond given a threshold
    pub fn should_respond(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Check if should respond using random sampling
    pub fn should_respond_random(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.0
    }

    /// Combine with another probability (multiply)
    pub fn and(self, other: Probability) -> Probability {
        Probability::new(self.0 * other.0)
    }

    /// Combine with another probability (max)
    pub fn or(self, other: Probability) -> Probability {
        Probability::new(self.0.max(other.0))
    }

    /// Boost probability by a factor
    pub fn boost(self, factor: f64) -> Probability {
        Probability::new(self.0 * factor)
    }

    /// Reduce probability by a factor
    pub fn reduce(self, factor: f64) -> Probability {
        Probability::new(self.0 / factor.max(0.001))
    }
}

impl Default for Probability {
    fn default() -> Self {
        Self(0.5)
    }
}

impl From<f64> for Probability {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for Probability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}%", self.0 * 100.0)
    }
}

// ============================================================================
// Persona Configuration
// ============================================================================

/// Persona configuration for the agent
///
/// Defines the agent's personality, voice, and behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Agent ID
    pub agent_id: Uuid,
    /// Agent name
    pub name: String,
    /// Toxicity level (0-10, higher = more sarcastic/edgy)
    pub toxicity_level: i16,
    /// Enthusiasm level (0-10, higher = more energetic)
    pub enthusiasm_level: i16,
    /// Areas of expertise
    pub expertise_domains: Vec<String>,
    /// Speech suffix (e.g., "喵~", "desu")
    pub speech_suffix: Option<String>,
    /// Keywords that trigger the agent
    pub trigger_keywords: Vec<String>,
    /// Voice ID for TTS
    pub voice_id: String,
    /// Voice speed (0.5-2.0)
    pub voice_speed: f32,
    /// Voice pitch (0.5-2.0)
    pub voice_pitch: f32,
    /// Emotion boundaries (emotions the agent should never express)
    pub forbidden_emotions: Vec<EmotionState>,
    /// Whether proactive messaging is enabled
    pub proactive_enabled: bool,
    /// Silence threshold for proactive messaging (in minutes)
    pub silence_threshold_minutes: u32,
    /// Whether learning is enabled
    pub learning_enabled: bool,
}

impl Default for Persona {
    fn default() -> Self {
        Self {
            agent_id: Uuid::nil(),
            name: "AI Assistant".to_string(),
            toxicity_level: 3,
            enthusiasm_level: 5,
            expertise_domains: vec![],
            speech_suffix: None,
            trigger_keywords: vec![],
            voice_id: "default".to_string(),
            voice_speed: 1.0,
            voice_pitch: 1.0,
            forbidden_emotions: vec![],
            proactive_enabled: false,
            silence_threshold_minutes: 120,
            learning_enabled: true,
        }
    }
}

impl Persona {
    /// Create a new persona with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the agent ID
    pub fn with_agent_id(mut self, id: Uuid) -> Self {
        self.agent_id = id;
        self
    }

    /// Set toxicity level
    pub fn with_toxicity(mut self, level: i16) -> Self {
        self.toxicity_level = level.clamp(0, 10);
        self
    }

    /// Set enthusiasm level
    pub fn with_enthusiasm(mut self, level: i16) -> Self {
        self.enthusiasm_level = level.clamp(0, 10);
        self
    }

    /// Add expertise domains
    pub fn with_expertise(mut self, domains: Vec<String>) -> Self {
        self.expertise_domains = domains;
        self
    }

    /// Set speech suffix
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.speech_suffix = Some(suffix.into());
        self
    }

    /// Set trigger keywords
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.trigger_keywords = keywords;
        self
    }

    /// Check if an emotion is allowed for this persona
    pub fn is_emotion_allowed(&self, emotion: EmotionState) -> bool {
        !self.forbidden_emotions.contains(&emotion)
    }

    /// Get the appropriate emotion, respecting boundaries
    pub fn constrain_emotion(&self, emotion: EmotionState) -> EmotionState {
        if self.is_emotion_allowed(emotion) {
            emotion
        } else {
            EmotionState::Neutral
        }
    }

    /// Generate system prompt based on persona
    pub fn generate_system_prompt(&self) -> String {
        let mut prompt = format!(
            "你是{}，一个群聊中的AI助手。\n\n",
            self.name
        );

        // Personality traits
        prompt.push_str("性格特点：\n");
        
        let toxicity_desc = match self.toxicity_level {
            0..=2 => "非常友善温和",
            3..=4 => "友善但偶尔会开玩笑",
            5..=6 => "有点毒舌但不过分",
            7..=8 => "经常吐槽和调侃",
            _ => "毒舌王，喜欢犀利点评",
        };
        prompt.push_str(&format!("- 毒舌程度：{}\n", toxicity_desc));

        let enthusiasm_desc = match self.enthusiasm_level {
            0..=2 => "冷静沉稳",
            3..=4 => "温和平静",
            5..=6 => "积极热情",
            7..=8 => "非常活跃",
            _ => "超级热情，充满能量",
        };
        prompt.push_str(&format!("- 热情程度：{}\n", enthusiasm_desc));

        // Expertise
        if !self.expertise_domains.is_empty() {
            prompt.push_str(&format!(
                "- 专业领域：{}\n",
                self.expertise_domains.join("、")
            ));
        }

        // Speech suffix
        if let Some(suffix) = &self.speech_suffix {
            prompt.push_str(&format!(
                "- 说话习惯：句末喜欢加上「{}」\n",
                suffix
            ));
        }

        prompt.push_str("\n请根据以上性格特点回复用户消息。保持自然、有趣的对话风格。");

        prompt
    }
}

// ============================================================================
// Memory Store Trait
// ============================================================================

/// Memory store trait for agent memory system
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a memory
    async fn store(&self, memory: Memory) -> Result<(), MemoryError>;

    /// Retrieve relevant memories based on query
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError>;

    /// Get recent conversation context
    async fn get_context(&self, group_id: Uuid, limit: usize) -> Result<Vec<ContextMessage>, MemoryError>;
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Memory ID
    pub id: Uuid,
    /// Group ID this memory belongs to
    pub group_id: Uuid,
    /// Memory content
    pub content: String,
    /// Embedding vector
    pub embedding: Vec<f32>,
    /// Importance score (0.0-1.0)
    pub importance: f32,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Metadata
    pub metadata: MemoryMetadata,
}

/// Memory metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Source message ID
    pub source_message_id: Option<Uuid>,
    /// Topic/category
    pub topic: Option<String>,
    /// Extracted entities
    pub entities: Vec<String>,
    /// Emotion at the time
    pub emotion: Option<EmotionState>,
}

/// Context chat message for agent processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    /// Message role
    pub role: ContextRole,
    /// Message content
    pub content: String,
}

/// Context message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextRole {
    System,
    User,
    Assistant,
}

/// Memory error type
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Failed to store memory: {0}")]
    StoreFailed(String),
    #[error("Failed to retrieve memories: {0}")]
    RetrieveFailed(String),
    #[error("Memory not found: {0}")]
    NotFound(Uuid),
}

// ============================================================================
// AgentCore Trait
// ============================================================================

/// Core agent trait - the "Brain" of the cognitive architecture
///
/// This trait defines the main interface for AI agent behavior.
/// Implementations should handle event processing, decision making,
/// and action generation.
///
/// **Feature: phase3-ai-agent-evolution, Property 2: AgentCore Process Returns Valid Actions**
/// **Feature: phase3-ai-agent-evolution, Property 3: Graceful Error Handling**
#[async_trait]
pub trait AgentCore: Send + Sync {
    /// Process an event and return a list of actions
    ///
    /// This is the main entry point for agent processing.
    /// The agent should analyze the event, retrieve relevant context,
    /// and generate appropriate actions.
    ///
    /// # Arguments
    /// * `event` - The input event to process
    /// * `memory` - Memory store for context retrieval
    /// * `persona` - Agent's personality configuration
    ///
    /// # Returns
    /// A list of actions to execute. Should never be empty - at minimum
    /// return `AgentAction::Ignore` if no response is needed.
    async fn process(
        &self,
        event: AgentEvent,
        memory: &dyn MemoryStore,
        persona: &Persona,
    ) -> Result<Vec<AgentAction>, AIAgentError>;

    /// Determine if the agent should respond to a message
    ///
    /// Returns a probability indicating how likely the agent should respond.
    /// The caller can use this to make the final decision.
    ///
    /// # Arguments
    /// * `context` - The message context
    /// * `persona` - Agent's personality configuration
    ///
    /// # Returns
    /// A probability value between 0.0 and 1.0
    async fn should_intervene(
        &self,
        context: &MessageContext,
        persona: &Persona,
    ) -> Probability;

    /// Analyze the emotional context of a message
    ///
    /// # Arguments
    /// * `context` - The message context
    ///
    /// # Returns
    /// The detected emotion state
    async fn analyze_emotion(&self, context: &MessageContext) -> EmotionState;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probability_creation() {
        let p = Probability::new(0.5);
        assert_eq!(p.value(), 0.5);

        // Test clamping
        let p = Probability::new(1.5);
        assert_eq!(p.value(), 1.0);

        let p = Probability::new(-0.5);
        assert_eq!(p.value(), 0.0);
    }

    #[test]
    fn test_probability_should_respond() {
        let p = Probability::new(0.8);
        assert!(p.should_respond(0.5));
        assert!(!p.should_respond(0.9));
    }

    #[test]
    fn test_probability_operations() {
        let p1 = Probability::new(0.5);
        let p2 = Probability::new(0.6);

        // AND (multiply)
        let combined = p1.and(p2);
        assert!((combined.value() - 0.3).abs() < 0.001);

        // OR (max)
        let combined = p1.or(p2);
        assert_eq!(combined.value(), 0.6);

        // Boost
        let boosted = p1.boost(1.5);
        assert!((boosted.value() - 0.75).abs() < 0.001);

        // Reduce
        let reduced = p1.reduce(2.0);
        assert!((reduced.value() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_probability_display() {
        let p = Probability::new(0.75);
        assert_eq!(p.to_string(), "75.00%");
    }

    #[test]
    fn test_persona_default() {
        let persona = Persona::default();
        assert_eq!(persona.name, "AI Assistant");
        assert_eq!(persona.toxicity_level, 3);
        assert_eq!(persona.enthusiasm_level, 5);
    }

    #[test]
    fn test_persona_builder() {
        let persona = Persona::new("TestBot")
            .with_toxicity(7)
            .with_enthusiasm(8)
            .with_suffix("喵~")
            .with_expertise(vec!["编程".to_string(), "AI".to_string()]);

        assert_eq!(persona.name, "TestBot");
        assert_eq!(persona.toxicity_level, 7);
        assert_eq!(persona.enthusiasm_level, 8);
        assert_eq!(persona.speech_suffix, Some("喵~".to_string()));
        assert_eq!(persona.expertise_domains.len(), 2);
    }

    #[test]
    fn test_persona_emotion_constraints() {
        let mut persona = Persona::default();
        persona.forbidden_emotions = vec![EmotionState::Playful];

        assert!(persona.is_emotion_allowed(EmotionState::Happy));
        assert!(!persona.is_emotion_allowed(EmotionState::Playful));

        assert_eq!(persona.constrain_emotion(EmotionState::Happy), EmotionState::Happy);
        assert_eq!(persona.constrain_emotion(EmotionState::Playful), EmotionState::Neutral);
    }

    #[test]
    fn test_persona_system_prompt() {
        let persona = Persona::new("小助手")
            .with_toxicity(5)
            .with_enthusiasm(7)
            .with_expertise(vec!["编程".to_string()])
            .with_suffix("~");

        let prompt = persona.generate_system_prompt();
        
        assert!(prompt.contains("小助手"));
        assert!(prompt.contains("编程"));
        assert!(prompt.contains("~"));
    }

    #[test]
    fn test_context_message() {
        let msg = ContextMessage {
            role: ContextRole::User,
            content: "Hello".to_string(),
        };
        assert_eq!(msg.role, ContextRole::User);
        assert_eq!(msg.content, "Hello");
    }
}
