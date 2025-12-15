use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Re-export persona types for convenience
pub use crate::services::ai::persona::{
    VoiceSettings, EmotionBoundaries, StickerPreferences, ResponseStyle, PersonaPresets,
};

/// AI Agent model - Core database entity
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AIAgent {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub toxicity_level: i16,
    pub enthusiasm_level: i16,
    pub expertise_domains: Vec<String>,
    pub speech_suffix: Option<String>,
    pub trigger_keywords: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Phase 3: Enhanced persona fields (JSONB)
    #[sqlx(default)]
    pub voice_settings: Option<sqlx::types::Json<VoiceSettings>>,
    #[sqlx(default)]
    pub emotion_boundaries: Option<sqlx::types::Json<EmotionBoundaries>>,
    #[sqlx(default)]
    pub sticker_preferences: Option<sqlx::types::Json<StickerPreferences>>,
    #[sqlx(default)]
    pub response_style: Option<sqlx::types::Json<ResponseStyle>>,
    #[sqlx(default)]
    pub proactive_config: Option<sqlx::types::Json<ProactiveConfig>>,
    // Phase 4: User customization fields
    #[sqlx(default)]
    pub custom_prompt: Option<String>,
    #[sqlx(default)]
    pub backstory: Option<String>,
    #[sqlx(default)]
    pub avatar_url: Option<String>,
    #[sqlx(default)]
    pub greeting_message: Option<String>,
    #[sqlx(default)]
    pub farewell_message: Option<String>,
    #[sqlx(default)]
    pub persona_preset: Option<String>,
}

/// Proactive messaging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProactiveConfig {
    pub enabled: bool,
    pub silence_threshold_minutes: i32,
    pub cooldown_minutes: i32,
    pub daily_limit: i32,
}

/// AI Agent creation request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAIAgent {
    pub name: String,
    pub toxicity_level: Option<i16>,
    pub enthusiasm_level: Option<i16>,
    pub expertise_domains: Option<Vec<String>>,
    pub speech_suffix: Option<String>,
    pub trigger_keywords: Option<Vec<String>>,
    /// Apply a preset template
    pub preset: Option<String>,
}

/// AI Agent personality update request
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdatePersonality {
    // Basic fields
    pub name: Option<String>,
    pub toxicity_level: Option<i16>,
    pub enthusiasm_level: Option<i16>,
    pub expertise_domains: Option<Vec<String>>,
    pub speech_suffix: Option<String>,
    pub trigger_keywords: Option<Vec<String>>,
    // Phase 3: Enhanced persona settings
    pub voice_settings: Option<VoiceSettings>,
    pub emotion_boundaries: Option<EmotionBoundaries>,
    pub sticker_preferences: Option<StickerPreferences>,
    pub response_style: Option<ResponseStyle>,
    pub proactive_config: Option<ProactiveConfig>,
    // Phase 4: User customization
    pub custom_prompt: Option<String>,
    pub backstory: Option<String>,
    pub avatar_url: Option<String>,
    pub greeting_message: Option<String>,
    pub farewell_message: Option<String>,
    /// Apply a preset template (overwrites other fields)
    pub preset: Option<String>,
}

/// Preset information for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaPresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub toxicity_level: i16,
    pub enthusiasm_level: i16,
    pub speech_suffix: Option<String>,
    pub preview_greeting: String,
}

/// Test persona request
#[derive(Debug, Clone, Deserialize)]
pub struct TestPersonaRequest {
    pub message: String,
    #[serde(default)]
    pub sender_name: Option<String>,
}

/// Test persona response
#[derive(Debug, Clone, Serialize)]
pub struct TestPersonaResponse {
    pub response: String,
    pub emotion: String,
    pub processing_time_ms: u64,
}

/// User affinity for an agent
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentUserAffinity {
    pub agent_id: Uuid,
    pub user_id: Uuid,
    pub affinity_score: i16,
}

/// Trigger types for AI agent responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    Mention,           // @Bot - 100%
    Keyword,           // Regex match - 60-80%
    ContextSimilarity, // Vector similarity > 0.8 - 40-60%
    Random,            // Random interjection - 1-5%
    SilenceBreaker,    // 2hr+ silence - 10-20%
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::Mention => write!(f, "mention"),
            TriggerType::Keyword => write!(f, "keyword"),
            TriggerType::ContextSimilarity => write!(f, "context_similarity"),
            TriggerType::Random => write!(f, "random"),
            TriggerType::SilenceBreaker => write!(f, "silence_breaker"),
        }
    }
}

impl TriggerType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mention" => Some(TriggerType::Mention),
            "keyword" => Some(TriggerType::Keyword),
            "context_similarity" => Some(TriggerType::ContextSimilarity),
            "random" => Some(TriggerType::Random),
            "silence_breaker" => Some(TriggerType::SilenceBreaker),
            _ => None,
        }
    }
}


/// Trigger decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDecision {
    pub should_respond: bool,
    pub probability: f32,
    pub trigger_type: TriggerType,
}

/// Trigger log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentTriggerLog {
    pub id: i64,
    pub agent_id: Uuid,
    pub group_id: Uuid,
    pub message_id: Uuid,
    pub trigger_type: String,
    pub did_respond: bool,
    pub probability: f32,
    pub created_at: DateTime<Utc>,
}

/// Message statistics for an agent
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentMessageStats {
    pub agent_id: Uuid,
    pub total_messages: i64,
    pub last_message_at: Option<DateTime<Utc>>,
    pub avg_response_time_ms: Option<i32>,
}

/// Statistics response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatistics {
    pub agent_id: Uuid,
    pub total_messages: i64,
    pub last_message_at: Option<DateTime<Utc>>,
    pub avg_response_time_ms: Option<i32>,
    pub trigger_counts: TriggerCounts,
}

/// Trigger counts by type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriggerCounts {
    pub mention: i64,
    pub keyword: i64,
    pub context_similarity: i64,
    pub random: i64,
    pub silence_breaker: i64,
    pub total_evaluations: i64,
    pub total_responses: i64,
}

/// Historical message from vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMessage {
    pub message_id: String,
    pub content: String,
    pub sender_id: String,
    pub timestamp: i64,
    pub similarity_score: f32,
    pub time_decay_weight: f32,
}

/// Personality configuration for prompt generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    pub agent_id: Uuid,
    pub name: String,
    pub toxicity_level: i16,
    pub enthusiasm_level: i16,
    pub expertise_domains: Vec<String>,
    pub speech_suffix: Option<String>,
    pub user_affinities: Vec<UserAffinityInfo>,
}

/// User affinity info for prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAffinityInfo {
    pub user_id: Uuid,
    pub username: String,
    pub affinity_score: i16,
}

/// Group context for trigger evaluation
#[derive(Debug, Clone)]
pub struct GroupContext {
    pub group_id: Uuid,
    pub last_message_time: Option<DateTime<Utc>>,
    pub agent: AIAgent,
    /// Whether the current message is a reply to the agent's previous message
    pub is_reply_to_agent: bool,
}

/// LLM message role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// LLM message for API calls
#[derive(Debug, Clone)]
pub struct LLMMessage {
    pub role: MessageRole,
    pub content: String,
}
