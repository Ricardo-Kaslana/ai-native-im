# Design Document - Phase 3: AI Agent Evolution

## Overview

This design document describes the architecture and implementation details for evolving the AI Agent system into a true "Cognitive Architecture" with human-like capabilities. The system follows a three-layer architecture (Perception → Cognition → Action) and emphasizes high extensibility, robustness, and independence.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        AI Agent System                               │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Perception Layer (感知层)                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │   │
│  │  │   Text   │  │  Image   │  │  Audio   │  │  Emotion │    │   │
│  │  │ Processor│  │ Processor│  │ (STT)    │  │ Analyzer │    │   │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │   │
│  │       └──────────────┴──────────────┴──────────────┘        │   │
│  │                           ↓                                  │   │
│  │                    AgentEvent                                │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ↓                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Cognition Layer (认知层)                   │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │   │
│  │  │ Trigger  │  │ Memory   │  │ Persona  │  │ Learning │    │   │
│  │  │Evaluator │  │ System   │  │ Manager  │  │ Engine   │    │   │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │   │
│  │       └──────────────┴──────────────┴──────────────┘        │   │
│  │                           ↓                                  │   │
│  │                    AgentCore                                 │   │
│  │                           ↓                                  │   │
│  │                    AgentAction                               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ↓                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Action Layer (执行层)                      │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │   │
│  │  │  Text    │  │  Audio   │  │  Tool    │  │ Sticker  │    │   │
│  │  │ Sender   │  │  (TTS)   │  │ Executor │  │ Selector │    │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Provider Priority Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Provider Manager                                  │
├─────────────────────────────────────────────────────────────────────┤
│  Priority Order (Configurable):                                      │
│  1. SiliconFlow (硅基流动) - Default for China                       │
│  2. Zhipu AI (智谱)                                                  │
│  3. Moonshot (月之暗面)                                              │
│  4. Alibaba Tongyi (通义千问)                                        │
│  5. Baidu Wenxin (文心一言)                                          │
│  6. OpenAI (International fallback)                                  │
│  7. Anthropic (International fallback)                               │
├─────────────────────────────────────────────────────────────────────┤
│  Fallback Strategy:                                                  │
│  - On timeout (>30s): Try next provider                              │
│  - On rate limit: Try next provider                                  │
│  - On error: Log and try next provider                               │
│  - All failed: Return graceful error message                         │
└─────────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Core Traits

```rust
// server/src/services/ai/core/agent_core.rs

/// Multi-modal input event
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// User sent a text message
    UserMessage(MessageContext),
    /// User sent an image (with optional text)
    UserImage { image_url: String, caption: Option<String>, context: MessageContext },
    /// User sent a voice message (transcribed)
    UserVoice { transcription: String, audio_url: String, context: MessageContext },
    /// System signal (silence detected, scheduled event)
    SystemSignal(SignalType),
    /// Tool execution completed
    ToolOutput { tool_name: String, result: String, request_id: String },
    /// User feedback received
    UserFeedback { message_id: Uuid, feedback_type: FeedbackType },
}

/// Agent output action
#[derive(Debug, Clone)]
pub enum AgentAction {
    /// Reply with text
    ReplyText { content: String, emotion: EmotionState },
    /// Reply with audio (TTS)
    ReplyAudio { text: String, voice_id: Option<String> },
    /// Reply with image
    ReplyImage { url: String, caption: Option<String> },
    /// Reply with sticker
    ReplySticker { sticker_id: String },
    /// Call an external tool
    CallTool { name: String, args: serde_json::Value, request_id: String },
    /// Generate summary
    Summarize { message_count: usize },
    /// Do nothing (decided not to respond)
    Ignore,
    /// Continue thinking (chain of thought)
    Think { thought: String },
}

/// Emotion state for expression variation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmotionState {
    Neutral,
    Happy,
    Excited,
    Empathetic,
    Supportive,
    Curious,
    Playful,
}

/// Core agent trait - the "Brain"
#[async_trait]
pub trait AgentCore: Send + Sync {
    /// Process an event and return actions
    async fn process(
        &self,
        event: AgentEvent,
        memory: &dyn MemoryStore,
        persona: &Persona,
    ) -> Result<Vec<AgentAction>, AgentError>;
    
    /// Determine if the agent should respond
    async fn should_intervene(
        &self,
        context: &MessageContext,
        persona: &Persona,
    ) -> Probability;
    
    /// Analyze emotion from context
    async fn analyze_emotion(&self, context: &MessageContext) -> EmotionState;
}

/// Probability type (0.0 to 1.0)
pub struct Probability(f64);

impl Probability {
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
    
    pub fn should_respond(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}
```

### Provider Manager

```rust
// server/src/services/ai/providers/provider_manager.rs

/// Provider configuration with priority
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: LLMProvider,
    pub api_key: String,
    pub api_base: Option<String>,
    pub model: String,
    pub priority: u8,  // Lower = higher priority
    pub enabled: bool,
}

/// Provider manager with automatic fallback
pub struct ProviderManager {
    providers: Vec<(ProviderConfig, Arc<dyn LLMClient>)>,
    current_index: AtomicUsize,
}

impl ProviderManager {
    /// Create with domestic priority
    pub fn with_domestic_priority(configs: Vec<ProviderConfig>) -> Self {
        let mut sorted = configs;
        sorted.sort_by_key(|c| c.priority);
        // ... initialize clients
    }
    
    /// Get the best available client
    pub async fn get_client(&self) -> Option<Arc<dyn LLMClient>> {
        // Return first enabled and healthy client
    }
    
    /// Execute with automatic fallback
    pub async fn execute_with_fallback<F, T>(&self, operation: F) -> Result<T, AgentError>
    where
        F: Fn(Arc<dyn LLMClient>) -> BoxFuture<'static, Result<T, AgentError>>,
    {
        for (config, client) in &self.providers {
            if !config.enabled {
                continue;
            }
            match operation(client.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) if e.is_transient() => {
                    warn!("Provider {} failed, trying next: {}", config.provider, e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Err(AgentError::AllProvidersFailed)
    }
}
```

### Speech Services

```rust
// server/src/services/ai/speech/mod.rs

/// Speech-to-Text service trait
#[async_trait]
pub trait STTService: Send + Sync {
    /// Transcribe audio to text
    async fn transcribe(&self, audio_url: &str) -> Result<Transcription, SpeechError>;
    
    /// Get supported audio formats
    fn supported_formats(&self) -> Vec<AudioFormat>;
}

/// Text-to-Speech service trait
#[async_trait]
pub trait TTSService: Send + Sync {
    /// Generate audio from text
    async fn synthesize(&self, text: &str, voice: &VoiceConfig) -> Result<AudioData, SpeechError>;
    
    /// List available voices
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, SpeechError>;
}

/// Transcription result
pub struct Transcription {
    pub text: String,
    pub language: String,
    pub confidence: f64,
    pub segments: Vec<TranscriptionSegment>,
}

/// Voice configuration
pub struct VoiceConfig {
    pub voice_id: String,
    pub speed: f32,      // 0.5 - 2.0
    pub pitch: f32,      // 0.5 - 2.0
    pub emotion: Option<EmotionState>,
}

// Implementations
pub struct WhisperSTT { /* OpenAI Whisper or whisper.cpp */ }
pub struct OpenAITTS { /* OpenAI TTS */ }
pub struct EdgeTTS { /* Microsoft Edge TTS */ }
pub struct AliTTS { /* Alibaba TTS for China */ }
```

### Memory System

```rust
// server/src/services/ai/memory/mod.rs

/// Memory store trait
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a memory
    async fn store(&self, memory: Memory) -> Result<(), MemoryError>;
    
    /// Retrieve relevant memories
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError>;
    
    /// Get recent conversation context
    async fn get_context(&self, group_id: Uuid, limit: usize) -> Result<Vec<ChatMessage>, MemoryError>;
}

/// Memory entry
pub struct Memory {
    pub id: Uuid,
    pub group_id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
    pub metadata: MemoryMetadata,
}

/// Memory metadata
pub struct MemoryMetadata {
    pub source_message_id: Option<Uuid>,
    pub topic: Option<String>,
    pub entities: Vec<String>,
    pub emotion: Option<EmotionState>,
}
```

### Emotion Analyzer

```rust
// server/src/services/ai/emotion/analyzer.rs

/// Emotion analyzer trait
#[async_trait]
pub trait EmotionAnalyzer: Send + Sync {
    /// Analyze emotion from text
    async fn analyze(&self, text: &str) -> EmotionAnalysis;
    
    /// Analyze emotion from conversation context
    async fn analyze_context(&self, messages: &[ChatMessage]) -> EmotionState;
}

/// Emotion analysis result
pub struct EmotionAnalysis {
    pub primary_emotion: EmotionState,
    pub confidence: f64,
    pub secondary_emotions: Vec<(EmotionState, f64)>,
    pub sentiment_score: f64,  // -1.0 to 1.0
}

/// Rule-based emotion analyzer (fast, no API call)
pub struct RuleBasedEmotionAnalyzer {
    happy_keywords: HashSet<String>,
    sad_keywords: HashSet<String>,
    angry_keywords: HashSet<String>,
    // ...
}

/// LLM-based emotion analyzer (accurate, requires API call)
pub struct LLMEmotionAnalyzer {
    llm_client: Arc<dyn LLMClient>,
}
```

### Learning Engine

```rust
// server/src/services/ai/learning/engine.rs

/// Feedback type
#[derive(Debug, Clone, Copy)]
pub enum FeedbackType {
    Positive,   // Thumbs up, "thanks", etc.
    Negative,   // Thumbs down, "wrong", etc.
    Correction, // User provided correct answer
}

/// Learning pattern
pub struct LearningPattern {
    pub id: Uuid,
    pub context_embedding: Vec<f32>,
    pub successful_response: Option<String>,
    pub feedback_type: FeedbackType,
    pub frequency: u32,
    pub last_used: DateTime<Utc>,
}

/// Learning engine trait
#[async_trait]
pub trait LearningEngine: Send + Sync {
    /// Record feedback
    async fn record_feedback(
        &self,
        message_id: Uuid,
        response_id: Uuid,
        feedback: FeedbackType,
    ) -> Result<(), LearningError>;
    
    /// Get relevant patterns for context
    async fn get_patterns(&self, context: &str, limit: usize) -> Result<Vec<LearningPattern>, LearningError>;
    
    /// Check if a pattern should be flagged for review
    async fn check_for_review(&self, pattern_id: Uuid) -> bool;
}
```

### Summarization Service

```rust
// server/src/services/ai/summary/service.rs

/// Summary request
pub struct SummaryRequest {
    pub group_id: Uuid,
    pub message_count: usize,
    pub since: Option<DateTime<Utc>>,
    pub focus_topics: Vec<String>,
}

/// Summary result
pub struct Summary {
    pub id: Uuid,
    pub group_id: Uuid,
    pub content: String,
    pub topics: Vec<String>,
    pub key_decisions: Vec<String>,
    pub notable_events: Vec<String>,
    pub message_range: (DateTime<Utc>, DateTime<Utc>),
    pub generated_at: DateTime<Utc>,
}

/// Summarization service
#[async_trait]
pub trait SummarizeService: Send + Sync {
    /// Generate summary
    async fn summarize(&self, request: SummaryRequest) -> Result<Summary, SummaryError>;
    
    /// Get cached summary if available
    async fn get_cached(&self, group_id: Uuid) -> Option<Summary>;
}
```

### Tool System

```rust
// server/src/services/ai/tools/mod.rs

/// Tool trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;
    
    /// Tool description for LLM
    fn description(&self) -> &str;
    
    /// Parameter schema (JSON Schema)
    fn parameters(&self) -> serde_json::Value;
    
    /// Execute the tool
    async fn execute(&self, args: serde_json::Value) -> Result<String, ToolError>;
}

/// Tool registry
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
    
    pub fn list_for_llm(&self) -> Vec<ToolDefinition> {
        // Format tools for LLM function calling
    }
}

// Example tools
pub struct WeatherTool { /* ... */ }
pub struct SearchTool { /* ... */ }
pub struct CalculatorTool { /* ... */ }
```

## Data Models

### Database Schema Extensions

```sql
-- Emotion tracking
CREATE TABLE agent_emotion_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id),
    message_id UUID REFERENCES messages(id),
    emotion_state VARCHAR(20) NOT NULL,
    confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Learning patterns
CREATE TABLE learning_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id),
    context_hash VARCHAR(64) NOT NULL,
    context_embedding VECTOR(1024),
    successful_response TEXT,
    feedback_type VARCHAR(20) NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    flagged_for_review BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User feedback
CREATE TABLE user_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    message_id UUID NOT NULL REFERENCES messages(id),
    response_id UUID NOT NULL REFERENCES messages(id),
    feedback_type VARCHAR(20) NOT NULL,
    correction_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Summary cache
CREATE TABLE summary_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id),
    content TEXT NOT NULL,
    topics JSONB NOT NULL DEFAULT '[]',
    key_decisions JSONB NOT NULL DEFAULT '[]',
    notable_events JSONB NOT NULL DEFAULT '[]',
    message_range_start TIMESTAMPTZ NOT NULL,
    message_range_end TIMESTAMPTZ NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- Provider usage tracking
CREATE TABLE provider_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    group_id UUID REFERENCES groups(id),
    provider VARCHAR(50) NOT NULL,
    service_type VARCHAR(20) NOT NULL, -- 'llm', 'embedding', 'tts', 'stt'
    tokens_used INTEGER NOT NULL DEFAULT 0,
    audio_seconds FLOAT NOT NULL DEFAULT 0,
    cost_estimate DECIMAL(10, 6) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_emotion_history_group ON agent_emotion_history(group_id, created_at DESC);
CREATE INDEX idx_learning_patterns_context ON learning_patterns USING ivfflat (context_embedding vector_cosine_ops);
CREATE INDEX idx_user_feedback_message ON user_feedback(message_id);
CREATE INDEX idx_summary_cache_group ON summary_cache(group_id, expires_at);
CREATE INDEX idx_provider_usage_user ON provider_usage(user_id, created_at DESC);
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Perception Layer Input Transformation

*For any* raw input (text, image URL, audio URL), the Perception Layer SHALL produce a valid AgentEvent variant with all required fields populated.

**Validates: Requirements 1.2**

### Property 2: AgentCore Process Returns Valid Actions

*For any* AgentEvent input, the AgentCore.process() method SHALL return a non-empty list of valid AgentAction variants.

**Validates: Requirements 2.2**

### Property 3: Graceful Error Handling

*For any* error condition during agent processing, the AgentCore SHALL return a fallback AgentAction (e.g., ReplyText with error message) instead of propagating the error.

**Validates: Requirements 2.5**

### Property 4: Audio Chunking for Long Messages

*For any* audio input longer than 5 minutes, the STT Service SHALL split it into chunks of at most 5 minutes each, and the total duration of chunks SHALL equal the original duration.

**Validates: Requirements 3.4**

### Property 5: TTS Voice Settings Application

*For any* Persona with configured voice settings, the TTS Service SHALL use those exact settings (voice_id, speed, pitch) when generating audio.

**Validates: Requirements 4.5**

### Property 6: Summary Message Count

*For any* SummaryRequest with message_count N, the SummarizeService SHALL read exactly min(N, available_messages) messages from the group.

**Validates: Requirements 5.1**

### Property 7: Summary Caching

*For any* generated Summary, it SHALL be stored in the cache with a valid expiration time, and subsequent requests within the cache window SHALL return the cached summary.

**Validates: Requirements 5.3**

### Property 8: Embedding Generation on Message Storage

*For any* message stored in the database, an embedding SHALL be generated and stored in the vector database within a configurable timeout.

**Validates: Requirements 6.1**

### Property 9: Semantic Search Result Ordering

*For any* semantic search query, the returned results SHALL be sorted in descending order by similarity score.

**Validates: Requirements 6.3**

### Property 10: Tool Call Action Format

*For any* tool call decision, the AgentAction::CallTool SHALL contain a valid tool name (registered in ToolRegistry) and valid JSON arguments matching the tool's parameter schema.

**Validates: Requirements 7.1**

### Property 11: Tool Output Feedback Loop

*For any* completed tool execution, the result SHALL be fed back to the agent as AgentEvent::ToolOutput with matching request_id.

**Validates: Requirements 7.2**

### Property 12: Tool Error Graceful Handling

*For any* tool execution failure, the agent SHALL return a user-friendly error message instead of crashing or returning raw error details.

**Validates: Requirements 7.3**

### Property 13: Persona Incorporation in Response

*For any* Persona configuration, the agent's response style SHALL reflect the persona's toxicity level, enthusiasm level, and expertise areas.

**Validates: Requirements 8.2, 8.4**

### Property 14: Trigger Probability Calculation

*For any* message context, the TriggerEvaluator SHALL return a Probability in range [0.0, 1.0], where:
- Mentions of agent name → probability > 0.9
- Configured keywords present → probability increases
- Recent agent response → probability decreases
- Direct reply to agent → probability > 0.9

**Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5**

### Property 15: Memory Retrieval and Incorporation

*For any* message processing, relevant memories SHALL be retrieved from the vector store, and the response context SHALL include these memories.

**Validates: Requirements 10.1, 10.2**

### Property 16: Proactive Messaging Control

*For any* group with proactive messaging disabled, the agent SHALL never send unsolicited messages regardless of silence duration.

**Validates: Requirements 11.3**

### Property 17: Quota Management

*For any* user message to the agent, the system SHALL:
1. Check the user's remaining quota before processing
2. Return a friendly limit message if quota exceeded
3. Record token usage after successful processing

**Validates: Requirements 12.1, 12.2, 12.3**

### Property 18: Streaming Interruption Handling

*For any* interrupted streaming response, the system SHALL preserve the partial response and mark it as incomplete.

**Validates: Requirements 13.3**

### Property 19: Multi-Modal Input Processing

*For any* input containing both image and text, the agent SHALL process both components and combine insights in the response.

**Validates: Requirements 14.1**

### Property 20: Image Processing Graceful Degradation

*For any* image processing failure, the agent SHALL fall back to text-only processing and inform the user about the limitation.

**Validates: Requirements 14.3**

### Property 21: Provider Fallback Chain

*For any* primary provider failure (timeout, rate limit, error), the system SHALL automatically try the next provider in the priority list until success or all providers exhausted.

**Validates: Requirements 16.3**

### Property 22: Domestic Provider Priority

*For any* provider selection without explicit override, domestic providers (SiliconFlow, Zhipu, etc.) SHALL be tried before international providers (OpenAI, Anthropic).

**Validates: Requirements 16.2**

### Property 23: Emotion-Driven Expression

*For any* detected emotion state, the agent's response SHALL reflect appropriate expression:
- Happy/Excited → enthusiastic language, positive emojis
- Sad/Serious → empathetic language, supportive tone
- Frustrated → acknowledgment first, then help

**Validates: Requirements 17.2, 17.3, 17.4**

### Property 24: Emotion Boundary Respect

*For any* Persona with defined emotion boundaries (e.g., "always calm"), the agent SHALL never express emotions outside those boundaries.

**Validates: Requirements 17.6**

### Property 25: Feedback Recording

*For any* user feedback (positive or negative), the system SHALL record the pattern with context embedding, feedback type, and timestamp.

**Validates: Requirements 18.1, 18.2**

### Property 26: Repeated Mistake Flagging

*For any* pattern with negative feedback count exceeding threshold (e.g., 3), the system SHALL flag it for review.

**Validates: Requirements 18.4**

### Property 27: Learning Data Anonymization

*For any* stored learning pattern, user-identifiable information SHALL be removed or hashed before storage.

**Validates: Requirements 18.5**

### Property 28: Disabled Learning Functionality

*For any* agent with learning disabled, the agent SHALL function normally without accessing or storing learning patterns.

**Validates: Requirements 18.6**

## Error Handling

### Error Categories

```rust
/// Agent-specific errors
#[derive(Debug, Error)]
pub enum AgentError {
    // Perception errors
    #[error("Failed to process input: {0}")]
    PerceptionError(String),
    
    #[error("Unsupported input format: {0}")]
    UnsupportedFormat(String),
    
    // Cognition errors
    #[error("LLM provider error: {0}")]
    LLMError(String),
    
    #[error("All providers failed")]
    AllProvidersFailed,
    
    #[error("Memory retrieval failed: {0}")]
    MemoryError(String),
    
    // Action errors
    #[error("Tool execution failed: {tool_name} - {reason}")]
    ToolError { tool_name: String, reason: String },
    
    #[error("TTS generation failed: {0}")]
    TTSError(String),
    
    #[error("STT transcription failed: {0}")]
    STTError(String),
    
    // Resource errors
    #[error("Quota exceeded for user {user_id}")]
    QuotaExceeded { user_id: Uuid },
    
    #[error("Rate limited by provider: {provider}")]
    RateLimited { provider: String },
    
    // Configuration errors
    #[error("Invalid persona configuration: {0}")]
    InvalidPersona(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

impl AgentError {
    /// Check if error is transient and can be retried
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            AgentError::LLMError(_)
                | AgentError::RateLimited { .. }
                | AgentError::MemoryError(_)
                | AgentError::TTSError(_)
                | AgentError::STTError(_)
        )
    }
    
    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            AgentError::QuotaExceeded { .. } => {
                "您今日的 AI 对话次数已用完，明天再来吧！".to_string()
            }
            AgentError::AllProvidersFailed => {
                "AI 服务暂时不可用，请稍后再试。".to_string()
            }
            AgentError::STTError(_) => {
                "语音识别失败，请重新发送或使用文字。".to_string()
            }
            AgentError::TTSError(_) => {
                "语音合成失败，已为您显示文字回复。".to_string()
            }
            _ => "处理您的请求时出现问题，请稍后再试。".to_string()
        }
    }
}
```

### Graceful Degradation Strategy

| Failure | Fallback |
|---------|----------|
| Primary LLM provider fails | Try next provider in priority list |
| All LLM providers fail | Return cached response or error message |
| STT fails | Show audio with "transcription unavailable" |
| TTS fails | Show text-only response |
| Vision model fails | Process text only, inform user |
| Memory retrieval fails | Continue with short-term context |
| Tool execution fails | Inform user, suggest alternative |
| Embedding generation fails | Log error, continue without embedding |

## Testing Strategy

### Property-Based Testing

The system uses `proptest` for property-based testing in Rust. Each correctness property is implemented as a property test.

**Testing Framework**: `proptest` (Rust)

**Minimum Iterations**: 100 per property test

**Test File Location**: `server/src/services/ai/property_tests.rs`

### Test Categories

1. **Unit Tests**: Test individual components in isolation
   - Emotion analyzer accuracy
   - Trigger probability calculation
   - Tool parameter validation
   - Quota calculation

2. **Property Tests**: Verify universal properties
   - Input transformation validity
   - Error handling graceful degradation
   - Provider fallback chain
   - Memory retrieval consistency

3. **Integration Tests**: Test component interactions
   - Full agent processing pipeline
   - Provider manager with real/mock providers
   - Memory store with vector database

### Mock Implementations

```rust
// Mock LLM client for testing
pub struct MockLLMClient {
    response: String,
    should_fail: bool,
    delay_ms: u64,
}

// Mock STT service for testing
pub struct MockSTTService {
    transcription: String,
    should_fail: bool,
}

// Mock TTS service for testing
pub struct MockTTSService {
    audio_data: Vec<u8>,
    should_fail: bool,
}

// Mock memory store for testing
pub struct MockMemoryStore {
    memories: Vec<Memory>,
}
```

### Test Annotations

Each property test must be annotated with the property it validates:

```rust
/// **Feature: phase3-ai-agent-evolution, Property 14: Trigger Probability Calculation**
#[test]
fn test_trigger_probability_mention() {
    // Test that mentioning agent name returns probability > 0.9
}
```

