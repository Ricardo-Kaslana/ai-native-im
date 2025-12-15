# Phase 3 实现模式参考

## 核心设计模式

### 1. 三层架构模式 (Perception → Cognition → Action)

```rust
// ============== Perception Layer (感知层) ==============
// 负责将原始输入转换为结构化的 AgentEvent

pub struct PerceptionLayer {
    stt_service: Arc<dyn STTService>,
    vision_service: Arc<dyn VisionService>,
    emotion_analyzer: Arc<dyn EmotionAnalyzer>,
}

impl PerceptionLayer {
    /// 处理原始输入，转换为 AgentEvent
    pub async fn process_input(&self, input: RawInput) -> Result<AgentEvent, PerceptionError> {
        match input {
            RawInput::Text { content, context } => {
                Ok(AgentEvent::UserMessage(MessageContext {
                    content,
                    sender_id: context.sender_id,
                    group_id: context.group_id,
                    timestamp: context.timestamp,
                }))
            }
            RawInput::Audio { url, context } => {
                let transcription = self.stt_service.transcribe(&url).await?;
                Ok(AgentEvent::UserVoice {
                    transcription: transcription.text,
                    audio_url: url,
                    context,
                })
            }
            RawInput::Image { url, caption, context } => {
                Ok(AgentEvent::UserImage {
                    image_url: url,
                    caption,
                    context,
                })
            }
        }
    }
}

// ============== Cognition Layer (认知层) ==============
// 负责决策和响应生成

pub struct CognitionLayer {
    agent_core: Arc<dyn AgentCore>,
    memory_store: Arc<dyn MemoryStore>,
    learning_engine: Arc<dyn LearningEngine>,
}

impl CognitionLayer {
    /// 处理事件，生成动作
    pub async fn process_event(
        &self,
        event: AgentEvent,
        persona: &Persona,
    ) -> Result<Vec<AgentAction>, CognitionError> {
        // 1. 检索相关记忆
        let memories = self.memory_store.retrieve(&event.content(), 5).await?;
        
        // 2. 获取学习模式
        let patterns = self.learning_engine.get_patterns(&event.content(), 3).await?;
        
        // 3. 调用 AgentCore 生成响应
        let actions = self.agent_core.process(event, &*self.memory_store, persona).await?;
        
        Ok(actions)
    }
}

// ============== Action Layer (执行层) ==============
// 负责执行具体动作

pub struct ActionLayer {
    tts_service: Arc<dyn TTSService>,
    tool_registry: Arc<ToolRegistry>,
    message_sender: Arc<dyn MessageSender>,
}

impl ActionLayer {
    /// 执行动作
    pub async fn execute(&self, action: AgentAction, persona: &Persona) -> Result<(), ActionError> {
        match action {
            AgentAction::ReplyText { content, emotion } => {
                self.message_sender.send_text(&content).await?;
            }
            AgentAction::ReplyAudio { text, voice_id } => {
                let voice_config = VoiceConfig {
                    voice_id: voice_id.unwrap_or_else(|| persona.voice_id.clone()),
                    speed: persona.voice_speed,
                    pitch: persona.voice_pitch,
                    emotion: None,
                };
                let audio = self.tts_service.synthesize(&text, &voice_config).await?;
                self.message_sender.send_audio(audio).await?;
            }
            AgentAction::CallTool { name, args, request_id } => {
                let tool = self.tool_registry.get(&name)
                    .ok_or(ActionError::ToolNotFound(name.clone()))?;
                let result = tool.execute(args).await?;
                // 将结果反馈给认知层
            }
            AgentAction::Ignore => {
                // 不做任何事
            }
            // ... 其他动作
        }
        Ok(())
    }
}
```

### 2. Provider Manager 模式 (带 Fallback)

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::future::BoxFuture;

pub struct ProviderManager {
    providers: Vec<(ProviderConfig, Arc<dyn LLMClient>)>,
    current_index: AtomicUsize,
}

impl ProviderManager {
    /// 带自动 fallback 的执行
    pub async fn execute_with_fallback<F, T, E>(&self, operation: F) -> Result<T, AgentError>
    where
        F: Fn(Arc<dyn LLMClient>) -> BoxFuture<'static, Result<T, E>>,
        E: Into<AgentError>,
    {
        let mut last_error = None;
        
        for (config, client) in &self.providers {
            if !config.enabled {
                continue;
            }
            
            tracing::debug!(provider = %config.provider, "Trying provider");
            
            match tokio::time::timeout(
                Duration::from_secs(config.timeout_secs.unwrap_or(30)),
                operation(client.clone())
            ).await {
                Ok(Ok(result)) => {
                    tracing::info!(provider = %config.provider, "Provider succeeded");
                    return Ok(result);
                }
                Ok(Err(e)) => {
                    let error: AgentError = e.into();
                    if error.is_transient() {
                        tracing::warn!(
                            provider = %config.provider,
                            error = %error,
                            "Provider failed with transient error, trying next"
                        );
                        last_error = Some(error);
                        continue;
                    } else {
                        return Err(error);
                    }
                }
                Err(_) => {
                    tracing::warn!(
                        provider = %config.provider,
                        "Provider timed out, trying next"
                    );
                    last_error = Some(AgentError::LLMTimeout(config.timeout_secs.unwrap_or(30)));
                    continue;
                }
            }
        }
        
        Err(last_error.unwrap_or(AgentError::AllProvidersFailed))
    }
}
```

### 3. Tool 系统模式

```rust
use serde_json::Value;

/// Tool trait 定义
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> Result<String, ToolError>;
}

/// Tool Registry
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }
    
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }
    
    /// 生成 LLM function calling 格式
    pub fn to_function_definitions(&self) -> Vec<FunctionDefinition> {
        self.tools.values().map(|tool| {
            FunctionDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters(),
            }
        }).collect()
    }
}

/// 示例：天气工具
pub struct WeatherTool {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str { "get_weather" }
    
    fn description(&self) -> &str {
        "获取指定城市的天气信息"
    }
    
    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "城市名称，如：北京、上海"
                }
            },
            "required": ["city"]
        })
    }
    
    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let city = args["city"].as_str()
            .ok_or(ToolError::InvalidArguments("missing city".to_string()))?;
        
        // 调用天气 API
        let weather = self.fetch_weather(city).await?;
        
        Ok(format!("{}的天气：{}，温度：{}°C", city, weather.condition, weather.temperature))
    }
}
```

### 4. 情绪分析模式

```rust
/// 情绪状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionState {
    Neutral,
    Happy,
    Excited,
    Empathetic,
    Supportive,
    Curious,
    Playful,
}

/// 情绪分析结果
pub struct EmotionAnalysis {
    pub primary_emotion: EmotionState,
    pub confidence: f64,
    pub secondary_emotions: Vec<(EmotionState, f64)>,
    pub sentiment_score: f64,  // -1.0 到 1.0
}

/// 规则分析器实现
pub struct RuleBasedEmotionAnalyzer {
    keywords: EmotionKeywords,
}

impl RuleBasedEmotionAnalyzer {
    pub fn new() -> Self {
        Self {
            keywords: EmotionKeywords::default_chinese(),
        }
    }
    
    fn analyze_sync(&self, text: &str) -> EmotionAnalysis {
        let text_lower = text.to_lowercase();
        let mut scores: HashMap<EmotionState, f64> = HashMap::new();
        
        // 检查关键词
        for (emotion, keywords) in self.keywords.iter() {
            let count = keywords.iter()
                .filter(|kw| text_lower.contains(*kw))
                .count();
            if count > 0 {
                *scores.entry(*emotion).or_insert(0.0) += count as f64 * 0.3;
            }
        }
        
        // 检查表情符号
        let emoji_emotions = self.analyze_emojis(text);
        for (emotion, score) in emoji_emotions {
            *scores.entry(emotion).or_insert(0.0) += score;
        }
        
        // 找出主要情绪
        let (primary_emotion, confidence) = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(e, s)| (*e, s.min(&1.0).clone()))
            .unwrap_or((EmotionState::Neutral, 0.5));
        
        EmotionAnalysis {
            primary_emotion,
            confidence,
            secondary_emotions: scores.into_iter()
                .filter(|(e, _)| *e != primary_emotion)
                .collect(),
            sentiment_score: self.calculate_sentiment(text),
        }
    }
}

#[async_trait]
impl EmotionAnalyzer for RuleBasedEmotionAnalyzer {
    async fn analyze(&self, text: &str) -> EmotionAnalysis {
        self.analyze_sync(text)
    }
    
    async fn analyze_context(&self, messages: &[ChatMessage]) -> EmotionState {
        if messages.is_empty() {
            return EmotionState::Neutral;
        }
        
        // 分析最近几条消息的情绪趋势
        let recent = messages.iter().rev().take(5);
        let mut emotion_counts: HashMap<EmotionState, usize> = HashMap::new();
        
        for msg in recent {
            let analysis = self.analyze_sync(&msg.content);
            *emotion_counts.entry(analysis.primary_emotion).or_insert(0) += 1;
        }
        
        emotion_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(emotion, _)| emotion)
            .unwrap_or(EmotionState::Neutral)
    }
}
```

### 5. 配额管理模式

```rust
use chrono::{DateTime, Utc, Duration};

pub struct QuotaManager {
    pool: PgPool,
    redis: RedisPool,
}

impl QuotaManager {
    /// 检查用户配额
    pub async fn check_quota(&self, user_id: Uuid, service_type: &str) -> Result<QuotaStatus, QuotaError> {
        let key = format!("quota:{}:{}:{}", user_id, service_type, Utc::now().format("%Y-%m-%d"));
        
        // 从 Redis 获取当前使用量
        let current: i64 = self.redis.get(&key).await?.unwrap_or(0);
        
        // 获取用户限额
        let limit = self.get_user_limit(user_id, service_type).await?;
        
        Ok(QuotaStatus {
            used: current,
            limit,
            remaining: (limit - current).max(0),
            reset_at: self.next_reset_time(),
        })
    }
    
    /// 记录使用量
    pub async fn record_usage(
        &self,
        user_id: Uuid,
        group_id: Option<Uuid>,
        service_type: &str,
        tokens: i64,
        audio_seconds: f64,
    ) -> Result<(), QuotaError> {
        let key = format!("quota:{}:{}:{}", user_id, service_type, Utc::now().format("%Y-%m-%d"));
        
        // 增加 Redis 计数
        self.redis.incr(&key, tokens).await?;
        self.redis.expire(&key, 86400 * 2).await?;  // 2天过期
        
        // 记录到数据库（用于统计）
        sqlx::query!(
            r#"
            INSERT INTO provider_usage (user_id, group_id, provider, service_type, tokens_used, audio_seconds, cost_estimate)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user_id,
            group_id,
            "default",
            service_type,
            tokens as i32,
            audio_seconds,
            self.estimate_cost(service_type, tokens, audio_seconds)
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 估算成本
    fn estimate_cost(&self, service_type: &str, tokens: i64, audio_seconds: f64) -> f64 {
        match service_type {
            "llm" => tokens as f64 * 0.000002,  // $0.002 per 1K tokens
            "embedding" => tokens as f64 * 0.0000001,
            "tts" => audio_seconds * 0.000015,  // $0.015 per 1K characters
            "stt" => audio_seconds * 0.0001,    // $0.006 per minute
            _ => 0.0,
        }
    }
}
```

### 6. 学习引擎模式

```rust
pub struct DefaultLearningEngine {
    pool: PgPool,
    embedding_service: Arc<dyn EmbeddingService>,
}

#[async_trait]
impl LearningEngine for DefaultLearningEngine {
    async fn record_feedback(
        &self,
        message_id: Uuid,
        response_id: Uuid,
        feedback: FeedbackType,
    ) -> Result<(), LearningError> {
        // 1. 获取消息内容
        let message = self.get_message(message_id).await?;
        let response = self.get_message(response_id).await?;
        
        // 2. 生成上下文嵌入
        let context_embedding = self.embedding_service
            .embed(&message.content)
            .await
            .map_err(|e| LearningError::EmbeddingFailed(e.to_string()))?;
        
        // 3. 计算上下文哈希（用于去重）
        let context_hash = self.hash_context(&message.content);
        
        // 4. 存储或更新模式
        sqlx::query!(
            r#"
            INSERT INTO learning_patterns (group_id, context_hash, context_embedding, successful_response, feedback_type)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (group_id, context_hash) DO UPDATE SET
                frequency = learning_patterns.frequency + 1,
                feedback_type = CASE 
                    WHEN $5 = 'negative' THEN 'negative'
                    ELSE learning_patterns.feedback_type
                END,
                successful_response = CASE
                    WHEN $5 = 'positive' THEN $4
                    ELSE learning_patterns.successful_response
                END,
                last_used = NOW()
            "#,
            message.group_id,
            context_hash,
            &context_embedding as &[f32],
            if matches!(feedback, FeedbackType::Positive) { Some(&response.content) } else { None },
            feedback.to_string()
        )
        .execute(&self.pool)
        .await?;
        
        // 5. 检查是否需要标记审查
        self.check_and_flag_for_review(message.group_id, &context_hash).await?;
        
        Ok(())
    }
    
    async fn get_patterns(&self, context: &str, limit: usize) -> Result<Vec<LearningPattern>, LearningError> {
        let query_embedding = self.embedding_service
            .embed(context)
            .await
            .map_err(|e| LearningError::EmbeddingFailed(e.to_string()))?;
        
        // 使用向量相似度搜索
        let patterns = sqlx::query_as!(
            LearningPattern,
            r#"
            SELECT id, context_embedding, successful_response, feedback_type, frequency, last_used
            FROM learning_patterns
            WHERE feedback_type = 'positive'
            ORDER BY context_embedding <=> $1
            LIMIT $2
            "#,
            &query_embedding as &[f32],
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(patterns)
    }
    
    async fn check_for_review(&self, pattern_id: Uuid) -> bool {
        // 检查负面反馈次数是否超过阈值
        let result = sqlx::query_scalar!(
            r#"
            SELECT frequency FROM learning_patterns
            WHERE id = $1 AND feedback_type = 'negative'
            "#,
            pattern_id
        )
        .fetch_optional(&self.pool)
        .await;
        
        matches!(result, Ok(Some(freq)) if freq >= 3)
    }
}
```

## 测试模式

### Mock 实现模板

```rust
/// Mock LLM Client
pub struct MockLLMClient {
    response: String,
    should_fail: bool,
    delay_ms: u64,
}

impl MockLLMClient {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            should_fail: false,
            delay_ms: 0,
        }
    }
    
    pub fn failing() -> Self {
        Self {
            response: String::new(),
            should_fail: true,
            delay_ms: 0,
        }
    }
    
    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn generate_response(&self, _system: &str, _user: &str) -> Result<String, AIAgentError> {
        if self.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        }
        
        if self.should_fail {
            Err(AIAgentError::LLMError("Mock failure".to_string()))
        } else {
            Ok(self.response.clone())
        }
    }
    
    // ... 其他方法
}

/// Mock Memory Store
pub struct MockMemoryStore {
    memories: Vec<Memory>,
}

impl MockMemoryStore {
    pub fn new() -> Self {
        Self { memories: vec![] }
    }
    
    pub fn with_memories(memories: Vec<Memory>) -> Self {
        Self { memories }
    }
}

#[async_trait]
impl MemoryStore for MockMemoryStore {
    async fn store(&self, _memory: Memory) -> Result<(), MemoryError> {
        Ok(())
    }
    
    async fn retrieve(&self, _query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError> {
        Ok(self.memories.iter().take(limit).cloned().collect())
    }
    
    async fn get_context(&self, _group_id: Uuid, limit: usize) -> Result<Vec<ChatMessage>, MemoryError> {
        Ok(vec![])
    }
}
```

### 属性测试模板

```rust
use proptest::prelude::*;

// 自定义生成器
fn arb_emotion_state() -> impl Strategy<Value = EmotionState> {
    prop_oneof![
        Just(EmotionState::Neutral),
        Just(EmotionState::Happy),
        Just(EmotionState::Excited),
        Just(EmotionState::Empathetic),
        Just(EmotionState::Supportive),
        Just(EmotionState::Curious),
        Just(EmotionState::Playful),
    ]
}

fn arb_agent_event() -> impl Strategy<Value = AgentEvent> {
    prop_oneof![
        any::<String>().prop_map(|content| AgentEvent::UserMessage(MessageContext {
            content,
            sender_id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            timestamp: Utc::now(),
        })),
        (any::<String>(), any::<String>()).prop_map(|(url, caption)| AgentEvent::UserImage {
            image_url: url,
            caption: Some(caption),
            context: MessageContext::default(),
        }),
    ]
}

proptest! {
    /// **Feature: phase3-ai-agent-evolution, Property 2: AgentCore Process Returns Valid Actions**
    #[test]
    fn test_agent_core_returns_valid_actions(event in arb_agent_event()) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let agent_core = DefaultAgentCore::new(
            Arc::new(MockLLMClient::new("test response")),
            Arc::new(MockMemoryStore::new()),
        );
        let persona = Persona::default();
        let memory = MockMemoryStore::new();
        
        let result = rt.block_on(agent_core.process(event, &memory, &persona));
        
        prop_assert!(result.is_ok());
        let actions = result.unwrap();
        prop_assert!(!actions.is_empty(), "Actions should not be empty");
    }
}
```

## 配置模板

### 环境变量

```env
# LLM Providers (国内优先)
SILICONFLOW_API_KEY=sk-xxx
SILICONFLOW_MODEL=Qwen/Qwen2.5-7B-Instruct

ZHIPU_API_KEY=xxx
ZHIPU_MODEL=glm-4

MOONSHOT_API_KEY=sk-xxx
MOONSHOT_MODEL=moonshot-v1-8k

# 国际备选
OPENAI_API_KEY=sk-xxx
OPENAI_MODEL=gpt-3.5-turbo

# Speech Services
WHISPER_API_KEY=sk-xxx
TTS_PROVIDER=edge  # edge, openai, ali

# Vector Database
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=xxx

# Quota Settings
DEFAULT_DAILY_TOKEN_LIMIT=100000
DEFAULT_MONTHLY_TOKEN_LIMIT=3000000
```

### Provider 配置结构

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AIAgentSystemConfig {
    // 国内服务商
    pub siliconflow: Option<ProviderConfig>,
    pub zhipu: Option<ProviderConfig>,
    pub moonshot: Option<ProviderConfig>,
    pub tongyi: Option<ProviderConfig>,
    pub wenxin: Option<ProviderConfig>,
    
    // 国际服务商
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    
    // 语音服务
    pub stt: STTConfig,
    pub tts: TTSConfig,
    
    // 配额设置
    pub quota: QuotaConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub api_base: Option<String>,
    pub model: String,
    pub priority: u8,
    pub enabled: bool,
    pub timeout_secs: Option<u64>,
}
```
