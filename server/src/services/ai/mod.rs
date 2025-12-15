mod agent_message_handler;
mod agent_message_sender;
mod agent_service;
mod ai_message_router;
pub mod core;
pub mod cot;
mod embedding;
pub mod emotion;
mod error;
pub mod intent;
mod integration_example;
#[cfg(test)]
mod integration_tests;
pub mod learning;
mod llm_client;
pub mod memory;
mod multimodal;
pub mod persona;
mod personality;
pub mod proactive;
pub mod user_profile;
pub mod providers;
#[cfg(test)]
mod property_tests;
pub mod quota;
pub mod search;
pub mod speech;
pub mod streaming;
pub mod summary;
mod system;
pub mod tools;
mod trigger;
mod typing_simulator;
mod vector_store;

pub use agent_message_handler::*;
pub use agent_message_sender::*;
pub use agent_service::*;
pub use ai_message_router::*;
pub use core::*;
pub use embedding::*;
// Note: emotion::EmotionState is more comprehensive than core::EmotionState
// Use emotion module directly for full emotion analysis capabilities
pub use emotion::{
    EmotionAnalysis, EmotionAnalyzer, EmotionKeywords, EmojiMappings,
    MockEmotionAnalyzer, RuleBasedEmotionAnalyzer,
};
pub use error::*;
pub use memory::{EmbeddingMemoryStore, InMemoryVectorStore};
pub use integration_example::*;
pub use proactive::{
    ActivityPattern, DefaultProactiveMessenger, MockProactiveMessenger,
    ProactiveConfig, ProactiveDecision, ProactiveError, ProactiveMessenger,
    ProactiveTrigger, RelationshipTriggerReason, GreetingType, CareReason,
    RelationshipContext,
};
pub use quota::{
    check_and_record_usage, InMemoryQuotaManager, MockQuotaManager,
    QuotaConfig, QuotaError, QuotaManager, QuotaStatus, ServiceType,
    UsageRecord,
};
pub use learning::{
    FeedbackRecord, FeedbackStats, FeedbackType, InMemoryLearningEngine,
    LearningConfig, LearningEngine, LearningError, LearningPattern,
    MockLearningEngine,
};
pub use streaming::{
    DefaultStreamingService, MockStreamingService, QueuedMessage,
    StreamHandle, StreamMessageQueue, StreamState, StreamingChunk,
    StreamingConfig, StreamingError, StreamingService,
};
pub use persona::{
    EmotionBoundaries, EmotionStickerMapping, Persona, PersonaPresets,
    ResponseLength, ResponseStyle, StickerPreferences, VoiceSettings, PresetInfo,
};
pub use cot::{
    CoTConfig, CoTProcessor, CoTResult, CoTMarkers, ThinkingStyle,
};
pub use user_profile::{
    UserProfile, UserProfileService, RelationshipLevel, InteractionEvent,
    InteractionType, Sentiment, UpdateUserProfile,
};
pub use intent::{
    Intent, IntentClassifier, IntentResult, ResponseStrategy, ResponsePriority,
    generate_intent_prompt,
};
pub use search::{
    DefaultSemanticSearchService, EmbeddingStatus, MessageEmbeddingInfo,
    MockSemanticSearchService, SearchConfig, SearchError, SearchRequest,
    SearchResponse, SearchResult, SearchType, SemanticSearchService,
    TimeRange,
};
pub use llm_client::*;
pub use multimodal::*;
pub use personality::*;
pub use providers::*;
pub use speech::*;
pub use summary::*;
pub use system::*;
pub use tools::*;
pub use trigger::*;
pub use typing_simulator::*;
pub use vector_store::*;
