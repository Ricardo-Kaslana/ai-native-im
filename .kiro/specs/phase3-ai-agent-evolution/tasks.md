# Implementation Plan - Phase 3: AI Agent Evolution

## 1. Core Agent Architecture Refactoring

- [x] 1.1 Create AgentEvent and AgentAction enums
  - Create `server/src/services/ai/core/events.rs`
  - Define `AgentEvent` enum with all variants (UserMessage, UserImage, UserVoice, SystemSignal, ToolOutput, UserFeedback)
  - Define `AgentAction` enum with all variants (ReplyText, ReplyAudio, ReplyImage, ReplySticker, CallTool, Summarize, Ignore, Think)
  - Define `EmotionState` enum
  - _Requirements: 1.2, 2.2_

- [x] 1.2 Write property test for AgentEvent validity
  - **Property 1: Perception Layer Input Transformation**
  - **Validates: Requirements 1.2**

- [x] 1.3 Create AgentCore trait
  - Create `server/src/services/ai/core/agent_core.rs`
  - Define `AgentCore` trait with `process()`, `should_intervene()`, `analyze_emotion()` methods
  - Define `Probability` type with validation
  - _Requirements: 2.1, 2.2, 2.5_

- [x] 1.4 Write property test for AgentCore process
  - **Property 2: AgentCore Process Returns Valid Actions**
  - **Validates: Requirements 2.2**

- [x] 1.5 Write property test for graceful error handling
  - **Property 3: Graceful Error Handling**
  - **Validates: Requirements 2.5**

- [x] 1.6 Implement DefaultAgentCore
  - Create `server/src/services/ai/core/default_agent.rs`
  - Implement `AgentCore` trait using existing LLM client
  - Integrate with existing `AIAgentService`
  - _Requirements: 2.1, 2.3_

## 2. Provider Manager with Domestic Priority

- [x] 2.1 Create ProviderConfig and ProviderManager
  - Create `server/src/services/ai/providers/provider_manager.rs`
  - Define `ProviderConfig` struct with priority field
  - Implement `ProviderManager` with `execute_with_fallback()` method
  - _Requirements: 16.2, 16.3_

- [x] 2.2 Write property test for provider fallback
  - **Property 21: Provider Fallback Chain**
  - **Validates: Requirements 16.3**

- [x] 2.3 Write property test for domestic priority
  - **Property 22: Domestic Provider Priority**
  - **Validates: Requirements 16.2**

- [x] 2.4 Add domestic LLM provider implementations
  - Add `ZhipuClient` for 智谱 AI
  - Add `MoonshotClient` for 月之暗面
  - Add `TongyiClient` for 通义千问
  - Add `WenxinClient` for 文心一言
  - Note: SiliconFlow already supports these models via unified API
  - _Requirements: 16.1_

- [x] 2.5 Update AIAgentSystemConfig for domestic providers
  - Add configuration fields for all domestic providers
  - Update `create_llm_client()` to respect priority
  - Domestic providers now have priority over international providers
  - _Requirements: 16.1, 16.2_

- [x] 2.6 Checkpoint - Ensure provider tests pass
  - All 34 LLM client tests pass
  - All 21 system config tests pass

## 3. Speech-to-Text (STT) Integration

- [x] 3.1 Create STT service trait and types
  - Create `server/src/services/ai/speech/stt.rs`
  - Define `STTService` trait
  - Define `Transcription`, `TranscriptionSegment` types
  - _Requirements: 3.1_

- [x] 3.2 Implement WhisperSTT service
  - Implemented in `server/src/services/ai/speech/stt.rs`
  - Implement OpenAI Whisper API integration
  - Add audio chunking for long files (>5 minutes)
  - _Requirements: 3.1, 3.4_

- [x] 3.3 Write property test for audio chunking
  - **Property 4: Audio Chunking for Long Messages**
  - **Validates: Requirements 3.4**

- [x] 3.4 Add domestic STT alternatives
  - Add `AliSTT` for Alibaba speech recognition (阿里云语音识别)
  - Add `STTProviderManager` with fallback logic
  - Domestic providers prioritized over international
  - _Requirements: 16.5_

## 4. Text-to-Speech (TTS) Integration

- [x] 4.1 Create TTS service trait and types
  - Create `server/src/services/ai/speech/tts.rs`
  - Define `TTSService` trait
  - Define `VoiceConfig`, `VoiceInfo`, `AudioData` types
  - _Requirements: 4.1_

- [x] 4.2 Implement OpenAI TTS service
  - Implemented in `server/src/services/ai/speech/tts.rs`
  - Implement OpenAI TTS API integration
  - Support voice selection and speed/pitch configuration
  - _Requirements: 4.1, 4.5_

- [x] 4.3 Write property test for voice settings
  - **Property 5: TTS Voice Settings Application**
  - **Validates: Requirements 4.5**

- [x] 4.4 Implement Edge TTS service
  - Implemented `EdgeTTS` in `server/src/services/ai/speech/tts.rs`
  - Microsoft Edge TTS (free alternative) with Chinese and English voices
  - Voice mapping for Chinese names (晓晓, 云希, etc.)
  - SSML building with rate and pitch support
  - _Requirements: 4.1_

- [x] 4.5 Add domestic TTS alternatives
  - Added `AliTTS` for Alibaba Cloud TTS (阿里云语音合成)
  - Added `TTSProviderManager` with automatic fallback
  - Domestic priority: Edge TTS (free) → Alibaba Cloud → OpenAI
  - _Requirements: 16.5_

- [x] 4.6 Checkpoint - Ensure speech services tests pass
  - All 40 speech tests pass (13 STT + 27 TTS)

## 5. Summarization Service

- [x] 5.1 Create SummarizeService trait and types
  - Create `server/src/services/ai/summary/mod.rs`
  - Define `SummarizeService` trait
  - Define `SummaryRequest`, `Summary`, `SummaryStyle` types
  - _Requirements: 5.1, 5.2_

- [x] 5.2 Implement DefaultSummarizeService
  - Create `server/src/services/ai/summary/service.rs`
  - Implement message fetching and LLM summarization
  - Implement caching with expiration
  - _Requirements: 5.1, 5.3_

- [x] 5.3 Write property test for message count
  - **Property 6: Summary Message Count**
  - **Validates: Requirements 5.1**

- [x] 5.4 Write property test for caching
  - **Property 7: Summary Caching**
  - **Validates: Requirements 5.3**

- [x] 5.5 Add summary API endpoint
  - Added `POST /api/ai/groups/:group_id/summary` endpoint in `server/src/api/ai_agents.rs`
  - Supports `SummaryApiRequest` with message_count, style, and use_cache options
  - Returns cached or generates new summary
  - Added `ai_agent_routes_with_llm()` for custom LLM client configuration
  - _Requirements: 5.1_

## 6. Semantic Search Enhancement

- [x] 6.1 Enhance message embedding pipeline
  - Created `server/src/services/ai/search/mod.rs`
  - `SemanticSearchService` trait with `search()`, `queue_embedding()`, `process_pending_embeddings()`
  - `DefaultSemanticSearchService` with embedding service and vector store integration
  - `EmbeddingStatus` enum (Pending, Processing, Completed, Failed)
  - `MessageEmbeddingInfo` for tracking embedding status
  - Async embedding queue with batch processing
  - _Requirements: 6.1_

- [x] 6.2 Write property test for embedding generation
  - **Property 8: Embedding Generation on Message Storage**
  - Test embedding status tracking (Pending → Completed)
  - All 8 search tests pass
  - **Validates: Requirements 6.1**

- [x] 6.3 Implement semantic search endpoint
  - `SearchRequest` with query, group_id, semantic flag, limit, min_score, time_range, sender_id
  - `SearchResponse` with results, total_count, search_type, query_time_ms
  - `SearchResult` with similarity_score, time_decay_weight, final_score
  - Automatic fallback to text search on vector DB failure
  - `SearchConfig` for customizing weights and behavior
  - _Requirements: 6.2, 6.4_

- [x] 6.4 Write property test for search result ordering
  - **Property 9: Semantic Search Result Ordering**
  - Results sorted by final_score (similarity * 0.7 + time_decay * 0.3)
  - **Validates: Requirements 6.3**

- [x] 6.5 Checkpoint - Ensure search tests pass
  - All 8 search tests pass

## 7. Tool Calling System

- [x] 7.1 Create Tool trait and ToolRegistry
  - Create `server/src/services/ai/tools/mod.rs`
  - Define `Tool` trait with `name()`, `description()`, `parameters()`, `execute()` methods
  - Implement `ToolRegistry` for tool management
  - _Requirements: 7.1, 7.4_

- [x] 7.2 Write property test for tool call format
  - **Property 10: Tool Call Action Format**
  - **Validates: Requirements 7.1**

- [x] 7.3 Write property test for tool output feedback
  - **Property 11: Tool Output Feedback Loop**
  - **Validates: Requirements 7.2**

- [x] 7.4 Write property test for tool error handling
  - **Property 12: Tool Error Graceful Handling**
  - **Validates: Requirements 7.3**

- [x] 7.5 Implement example tools
  - Implement `WeatherTool` using weather API (mock)
  - Implement `CalculatorTool` for math operations
  - Implement `CurrentTimeTool` for time queries
  - _Requirements: 7.1_

- [x] 7.6 Integrate tool calling with AgentCore
  - Modified `DefaultAgentCore` to support tool calling loop
  - Added `parse_tool_calls()`, `execute_tool_calls()`, `format_tool_results()` methods
  - Added `clean_response()` to remove tool markup from final response
  - Added `build_tool_instructions()` for system prompt
  - Supports max_tool_iterations to prevent infinite loops
  - _Requirements: 7.2, 7.5_

## 8. Emotion Analysis and Expression

- [x] 8.1 Create EmotionAnalyzer trait and types
  - Create `server/src/services/ai/emotion/mod.rs`
  - Define `EmotionAnalyzer` trait
  - Define `EmotionAnalysis`, `EmotionState` types
  - _Requirements: 17.1_

- [x] 8.2 Implement RuleBasedEmotionAnalyzer
  - Create `server/src/services/ai/emotion/rule_based.rs`
  - Implement keyword-based emotion detection
  - Support Chinese emotion keywords and emojis
  - _Requirements: 17.1_

- [x] 8.3 Implement LLMEmotionAnalyzer
  - Created `server/src/services/ai/emotion/llm_based.rs`
  - LLM-based emotion analysis with JSON response parsing
  - HybridEmotionAnalyzer combining rule-based and LLM approaches
  - Fallback to rule-based on LLM failure
  - _Requirements: 17.1_

- [x] 8.4 Write property test for emotion-driven expression
  - **Property 23: Emotion-Driven Expression**
  - **Validates: Requirements 17.2, 17.3, 17.4**

- [x] 8.5 Write property test for emotion boundary
  - **Property 24: Emotion Boundary Respect**
  - **Validates: Requirements 17.6**

- [x] 8.6 Integrate emotion into response generation
  - Added `emotion_analyzer` field to `DefaultAgentCore`
  - Added `with_emotion_analyzer()` builder method
  - Added `convert_emotion_state()` to map between emotion module and core types
  - Emotion analysis used in response generation with fallback to simple analysis
  - _Requirements: 17.2, 17.3, 17.4, 17.5_

- [x] 8.7 Checkpoint - Ensure emotion tests pass
  - All 29 emotion tests pass
  - All 14 default_agent tests pass (including emotion analyzer integration)

## 9. Trigger Evaluation Enhancement

- [x] 9.1 Enhance TriggerEvaluator
  - Added `TriggerConfig` for configurable parameters
  - Added `is_in_cooldown()` with response time tracking
  - Added `check_expertise_domains()` for domain-based triggering
  - Added direct reply detection (`is_reply_to_agent` in GroupContext)
  - Added `record_response()` for cooldown management
  - Keyword boost and expertise boost multipliers
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 9.2 Write property test for trigger probability
  - **Property 14: Trigger Probability Calculation**
  - All 18 trigger tests pass
  - **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5**

## 10. Memory System Enhancement

- [x] 10.1 Create MemoryStore trait and types
  - MemoryStore trait already defined in `server/src/services/ai/core/agent_core.rs`
  - Created `server/src/services/ai/memory/mod.rs` module
  - Memory, MemoryMetadata, ContextMessage types defined
  - _Requirements: 10.1_

- [x] 10.2 Implement VectorMemoryStore
  - Created `server/src/services/ai/memory/vector_store.rs`
  - Implemented `InMemoryVectorStore` with cosine similarity
  - Implemented `EmbeddingMemoryStore` with embedding service integration
  - Conversation history management with limits
  - Memory importance scoring and pruning
  - _Requirements: 10.1, 10.2_

- [x] 10.3 Write property test for memory retrieval
  - 7 memory tests pass (store, retrieve, context, limits, similarity)
  - **Property 15: Memory Retrieval and Incorporation**
  - **Validates: Requirements 10.1, 10.2**

- [x] 10.4 Integrate memory into response generation
  - DefaultAgentCore already uses MemoryStore in `build_prompt()`
  - Retrieves relevant memories and conversation context
  - Includes memories in LLM context
  - _Requirements: 10.2_

## 11. Proactive Messaging

- [x] 11.1 Create ProactiveMessenger service
  - Created `server/src/services/ai/proactive/mod.rs`
  - Implemented silence detection with configurable threshold
  - Implemented activity pattern analysis
  - `ProactiveMessenger` trait with `check_silence`, `record_activity`, `should_send_proactive`
  - `DefaultProactiveMessenger` in-memory implementation
  - `ProactiveTrigger` enum (SilenceBreaker, Scheduled, ActivityBased, SpecialEvent)
  - _Requirements: 11.1, 11.2_

- [x] 11.2 Write property test for proactive control
  - **Property 16: Proactive Messaging Control**
  - All 8 proactive tests pass
  - **Validates: Requirements 11.3**

- [x] 11.3 Add proactive messaging configuration
  - Added `ProactiveConfig` with `silence_threshold_minutes`, `cooldown_minutes`, `daily_limit`
  - Added proactive message recording with `record_proactive_sent`
  - `MockProactiveMessenger` for testing
  - _Requirements: 11.3, 11.4_

## 12. Rate Limiting and Quota Management

- [x] 12.1 Create QuotaManager service
  - Created `server/src/services/ai/quota/mod.rs`
  - Implemented per-user and per-group quota tracking
  - Implemented daily/monthly limits with `InMemoryQuotaManager`
  - `QuotaManager` trait with `check_quota`, `record_usage`, `get_usage_stats`
  - `ServiceType` enum (LLM, Embedding, STT, TTS, ImageGen, Vision)
  - _Requirements: 12.1, 12.4_

- [x] 12.2 Write property test for quota management
  - **Property 17: Quota Management**
  - All 10 quota tests pass
  - **Validates: Requirements 12.1, 12.2, 12.3**

- [x] 12.3 Add usage tracking for all services
  - Track LLM token usage
  - Track STT/TTS audio seconds
  - Calculate cost estimates with `estimate_cost()`
  - `UsageRecord` and `QuotaStatus` types
  - _Requirements: 12.3, 12.5_

- [x] 12.4 Integrate quota check into agent processing
  - `check_quota()` returns `QuotaStatus` with remaining quota
  - `QuotaError::QuotaExceeded` when limits reached
  - `MockQuotaManager` for testing
  - _Requirements: 12.1, 12.2_

- [x] 12.5 Checkpoint - Ensure quota tests pass
  - All 10 quota tests pass

## 13. Response Streaming

- [x] 13.1 Enhance streaming support
  - Created `server/src/services/ai/streaming/mod.rs`
  - `StreamingService` trait with `start_stream()` method
  - `DefaultStreamingService` with LLM client integration
  - `StreamHandle` for controlling active streams with interruption support
  - `StreamingChunk` with sequence numbers, state, and timestamps
  - `StreamMessageQueue` for queuing messages during streaming
  - `StreamState` enum (Active, Completed, Interrupted, Failed)
  - `StreamingConfig` with timeout and buffer settings
  - _Requirements: 13.1, 13.3, 13.4_

- [x] 13.2 Write property test for streaming interruption
  - **Property 18: Streaming Interruption Handling**
  - All 8 streaming tests pass
  - Tests for interruption, failure, chunk sequence, message queue
  - **Validates: Requirements 13.3**

## 14. Multi-Modal Input Processing

- [x] 14.1 Enhance image processing
  - Enhanced `MultimodalProcessor` with `process_multimodal()` method
  - Added `MultiModalInput` for combining text and multiple images
  - Added `ImageInput` struct for flexible image handling
  - Added `MultiModalResult` with vision_success, degraded, degradation_reason
  - Added `FailingVisionModel` for testing graceful degradation
  - Graceful degradation uses caption or default description on failure
  - _Requirements: 14.1, 14.3_

- [x] 14.2 Write property test for multi-modal processing
  - **Property 19: Multi-Modal Input Processing**
  - Tests text-only, image-only, and combined inputs
  - All 18 multimodal tests pass
  - **Validates: Requirements 14.1**

- [x] 14.3 Write property test for image degradation
  - **Property 20: Image Processing Graceful Degradation**
  - Tests fallback to caption when vision fails
  - Tests fallback to default description when no caption
  - **Validates: Requirements 14.3**

## 15. Persona System Enhancement

- [x] 15.1 Enhance Persona configuration
  - Add voice settings to Persona
  - Add emotion boundaries to Persona
  - Add sticker preferences to Persona
  - Created `server/src/services/ai/persona/mod.rs` with full implementation
  - `VoiceSettings` for TTS configuration (voice_id, speed, pitch, provider)
  - `EmotionBoundaries` for allowed/forbidden emotions with intensity limits
  - `StickerPreferences` with emotion-to-sticker mappings
  - `ResponseStyle` with length, emoji, formality settings
  - `PersonaPresets` for common persona types
  - _Requirements: 8.1, 8.4, 15.4, 17.6_

- [x] 15.2 Write property test for persona incorporation
  - **Property 13: Persona Incorporation in Response**
  - Property tests for toxicity/enthusiasm clamping
  - Property tests for emotion boundary enforcement
  - All 11 persona tests pass
  - **Validates: Requirements 8.2, 8.4**

## 16. Learning Engine

- [x] 16.1 Create LearningEngine trait and types
  - Created `server/src/services/ai/learning/mod.rs`
  - Defined `LearningEngine` trait with `record_feedback`, `get_patterns`, `check_for_review`, `mark_reviewed`
  - Defined `LearningPattern`, `FeedbackType`, `FeedbackRecord`, `FeedbackStats` types
  - `LearningConfig` with `review_threshold`, `anonymize_data`, `max_patterns_per_group`
  - _Requirements: 18.1, 18.2_

- [x] 16.2 Implement DefaultLearningEngine
  - Implemented `InMemoryLearningEngine` in `server/src/services/ai/learning/mod.rs`
  - Implemented feedback recording with pattern creation/update
  - Implemented pattern retrieval by group
  - Implemented review flagging when frequency >= threshold
  - `MockLearningEngine` for testing
  - _Requirements: 18.1, 18.2, 18.4_

- [x] 16.3 Write property test for feedback recording
  - **Property 25: Feedback Recording**
  - All feedback types have valid string representation
  - **Validates: Requirements 18.1, 18.2**

- [x] 16.4 Write property test for mistake flagging
  - **Property 26: Repeated Mistake Flagging**
  - Pattern needs review when frequency >= threshold
  - **Validates: Requirements 18.4**

- [x] 16.5 Write property test for anonymization
  - **Property 27: Learning Data Anonymization**
  - Email and phone number anonymization with `[EMAIL]` and `[PHONE]` placeholders
  - **Validates: Requirements 18.5**

- [x] 16.6 Write property test for disabled learning
  - **Property 28: Disabled Learning Functionality**
  - Returns `LearningError::LearningDisabled` when learning is disabled
  - **Validates: Requirements 18.6**

- [x] 16.7 Add feedback API endpoints
  - Added `POST /api/ai/messages/:message_id/feedback` endpoint
  - Added `GET /api/ai/groups/:group_id/feedback/stats` endpoint
  - Record feedback and update patterns
  - _Requirements: 18.1, 18.2_

## 17. Database Migrations

- [x] 17.1 Create database migrations
  - Created `20251209000001_create_agent_emotion_history.sql` - emotion tracking
  - Created `20251209000002_create_learning_patterns.sql` - learning patterns with vector embedding
  - Created `20251209000003_create_user_feedback.sql` - user feedback on responses
  - Created `20251209000004_create_summary_cache.sql` - summary caching
  - Created `20251209000005_create_provider_usage.sql` - usage tracking and quota
  - Created `20251209000006_add_ai_agent_persona_fields.sql` - enhanced persona fields
  - All tables have proper indexes and foreign key constraints
  - _Requirements: All_

## 18. Final Integration and Testing

- [x] 18.1 Integration testing
  - Created `server/src/services/ai/integration_tests.rs`
  - Test tool calling integration (calculator, time tools)
  - Test multimodal processing integration
  - Test persona integration with presets
  - Test emotion analysis integration
  - Test memory store integration
  - Test search service integration
  - Test streaming, quota, learning, proactive types
  - Test speech service types
  - Test summary service types
  - Test embedding service
  - Test vector store
  - All 14 integration tests pass
  - _Requirements: All_

- [x] 18.2 Final Checkpoint - Ensure all tests pass
  - All AI module tests pass (329+ tests)
  - All integration tests pass (14 tests)

