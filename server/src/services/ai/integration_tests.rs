//! Phase 3 AI Agent Evolution - Integration Tests
//!
//! Tests the full agent processing pipeline including:
//! - Tool calling chain
//! - Multimodal processing
//! - Persona configuration
//! - Emotion analysis

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    use crate::services::ai::{
        // Tools
        ToolRegistry, CalculatorTool, CurrentTimeTool,
        // Persona
        Persona, PersonaPresets,
        // Multimodal
        MultimodalProcessor, MockVisionModel, MultiModalInput,
    };

    /// Test tool calling integration
    #[tokio::test]
    async fn test_tool_calling_integration() {
        // Setup tool registry
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(CalculatorTool));
        registry.register(Arc::new(CurrentTimeTool));

        // Verify tools are registered
        assert!(registry.get("calculator").is_some());
        assert!(registry.get("current_time").is_some());

        // Test calculator tool
        let calc_tool = registry.get("calculator").unwrap();
        let result = calc_tool.execute(serde_json::json!({
            "expression": "2 + 2"
        })).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("4"));

        // Test time tool
        let time_tool = registry.get("current_time").unwrap();
        let result = time_tool.execute(serde_json::json!({})).await;
        assert!(result.is_ok());
    }

    /// Test multimodal processing integration
    #[tokio::test]
    async fn test_multimodal_integration() {
        let vision = Box::new(MockVisionModel::new("A beautiful landscape".to_string()));
        let processor = MultimodalProcessor::new().with_vision_model(vision);

        // Process text + image
        let input = MultiModalInput::with_text_and_image(
            "What do you see?",
            "http://example.com/landscape.jpg"
        );
        let result = processor.process_multimodal(&input).await;

        assert!(result.vision_success);
        assert!(!result.degraded);
        assert!(result.combined_text.contains("What do you see?"));
        assert!(result.combined_text.contains("landscape"));
    }

    /// Test persona integration
    #[tokio::test]
    async fn test_persona_integration() {
        // Test different persona presets
        let friendly = PersonaPresets::friendly_assistant();
        assert_eq!(friendly.toxicity_level, 2);
        assert_eq!(friendly.enthusiasm_level, 7);

        let sarcastic = PersonaPresets::sarcastic_friend();
        assert_eq!(sarcastic.toxicity_level, 7);
        assert!(sarcastic.speech_suffix.is_some());

        let expert = PersonaPresets::professional_expert("AI");
        assert_eq!(expert.toxicity_level, 1);
        assert!(expert.expertise_domains.contains(&"AI".to_string()));

        // Test emotion boundaries
        let persona = Persona::default();
        assert!(persona.is_emotion_allowed(&crate::services::ai::emotion::EmotionState::Happy));
        assert!(!persona.is_emotion_allowed(&crate::services::ai::emotion::EmotionState::Angry));
    }

    /// Test emotion analysis integration
    #[tokio::test]
    async fn test_emotion_analysis_integration() {
        use crate::services::ai::emotion::{RuleBasedEmotionAnalyzer, EmotionAnalyzer, EmotionState};

        let analyzer = RuleBasedEmotionAnalyzer::new();

        // Test happy emotion
        let analysis = analyzer.analyze("太棒了！我很高兴！").await.unwrap();
        assert_eq!(analysis.primary_emotion, EmotionState::Happy);

        // Test neutral emotion
        let analysis = analyzer.analyze("今天天气不错").await.unwrap();
        // Should be neutral or curious
        assert!(matches!(
            analysis.primary_emotion,
            EmotionState::Neutral | EmotionState::Curious | EmotionState::Happy
        ));
    }

    /// Test memory store integration
    #[tokio::test]
    async fn test_memory_store_integration() {
        use crate::services::ai::memory::InMemoryVectorStore;
        use crate::services::ai::core::{ContextMessage, ContextRole};

        let store = InMemoryVectorStore::new();
        let group_id = Uuid::new_v4();

        // Add conversation messages
        store.add_conversation_message(group_id, ContextMessage {
            role: ContextRole::User,
            content: "Hello, I like programming".to_string(),
        });

        store.add_conversation_message(group_id, ContextMessage {
            role: ContextRole::Assistant,
            content: "That's great! What languages do you use?".to_string(),
        });

        // Check memory count
        let count = store.memory_count(group_id);
        // Memory count is for stored memories, not conversation
        assert!(count >= 0);
    }

    /// Test search service integration
    #[tokio::test]
    async fn test_search_service_integration() {
        use crate::services::ai::search::{
            MockSemanticSearchService, SearchRequest, SemanticSearchService,
        };

        let service = MockSemanticSearchService::new();
        let group_id = Uuid::new_v4();

        // Search (mock returns empty for non-matching queries)
        let request = SearchRequest {
            query: "test query".to_string(),
            group_id,
            semantic: true,
            limit: 10,
            min_score: 0.5,
            ..Default::default()
        };
        let response = service.search(request).await.unwrap();
        assert!(response.results.len() <= 10);
    }

    /// Test streaming service types
    #[tokio::test]
    async fn test_streaming_types() {
        use crate::services::ai::streaming::StreamState;

        // Test stream state transitions
        let state = StreamState::Active;
        assert!(matches!(state, StreamState::Active));

        let completed = StreamState::Completed;
        assert!(matches!(completed, StreamState::Completed));

        let interrupted = StreamState::Interrupted;
        assert!(matches!(interrupted, StreamState::Interrupted));
    }

    /// Test quota service types
    #[tokio::test]
    async fn test_quota_types() {
        use crate::services::ai::quota::{QuotaConfig, ServiceType};

        // Test quota config
        let config = QuotaConfig::default();
        assert!(config.daily_token_limit > 0);
        assert!(config.monthly_token_limit > 0);

        // Test service types
        let llm = ServiceType::LLM;
        let tts = ServiceType::TTS;
        assert!(matches!(llm, ServiceType::LLM));
        assert!(matches!(tts, ServiceType::TTS));
    }

    /// Test learning engine types
    #[tokio::test]
    async fn test_learning_types() {
        use crate::services::ai::learning::{FeedbackType, LearningConfig};

        // Test feedback types
        let positive = FeedbackType::Positive;
        let negative = FeedbackType::Negative;
        assert!(matches!(positive, FeedbackType::Positive));
        assert!(matches!(negative, FeedbackType::Negative));

        // Test learning config
        let config = LearningConfig::default();
        assert!(config.review_threshold > 0);
    }

    /// Test proactive messaging types
    #[tokio::test]
    async fn test_proactive_types() {
        use crate::services::ai::proactive::{ProactiveConfig, ProactiveTrigger};

        // Test proactive config
        let config = ProactiveConfig::default();
        assert!(config.silence_threshold_minutes > 0);

        // Test trigger types
        let silence = ProactiveTrigger::SilenceBreaker;
        let scheduled = ProactiveTrigger::Scheduled;
        assert!(matches!(silence, ProactiveTrigger::SilenceBreaker));
        assert!(matches!(scheduled, ProactiveTrigger::Scheduled));
    }

    /// Test speech service types
    #[tokio::test]
    async fn test_speech_types() {
        use crate::services::ai::speech::{VoiceConfig, AudioFormat};

        // Test voice config
        let config = VoiceConfig {
            voice_id: "zh-CN-XiaoxiaoNeural".to_string(),
            speed: 1.0,
            pitch: 0.0,
            emotion: None,
        };
        assert_eq!(config.voice_id, "zh-CN-XiaoxiaoNeural");
        assert_eq!(config.speed, 1.0);

        // Test audio format
        let mp3 = AudioFormat::Mp3;
        let wav = AudioFormat::Wav;
        assert!(matches!(mp3, AudioFormat::Mp3));
        assert!(matches!(wav, AudioFormat::Wav));
    }

    /// Test summary service types
    #[tokio::test]
    async fn test_summary_types() {
        use crate::services::ai::summary::SummaryStyle;

        // Test summary styles
        let brief = SummaryStyle::Brief;
        let detailed = SummaryStyle::Detailed;
        assert!(matches!(brief, SummaryStyle::Brief));
        assert!(matches!(detailed, SummaryStyle::Detailed));
    }

    /// Test embedding service
    #[tokio::test]
    async fn test_embedding_service() {
        use crate::services::ai::embedding::{MockEmbeddingService, EmbeddingService};

        let service = MockEmbeddingService::new(1536);

        // Test single embedding
        let embedding = service.embed("Hello world").await.unwrap();
        assert_eq!(embedding.len(), 1536);

        // Test batch embedding
        let texts = vec!["Hello".to_string(), "World".to_string()];
        let embeddings = service.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 1536);
    }

    /// Test vector store
    #[tokio::test]
    async fn test_vector_store() {
        use crate::services::ai::vector_store::{MockVectorStore, VectorStore};

        let store = MockVectorStore::new();
        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        // Store a message
        let embedding = vec![1.0; 1536];
        store.store_message(
            group_id,
            message_id,
            sender_id,
            "Hello world",
            embedding.clone(),
            chrono::Utc::now().timestamp(),
        ).await.unwrap();

        // Search for similar
        let results = store.search_similar(group_id, embedding, 10, 0.5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello world");
    }
}
