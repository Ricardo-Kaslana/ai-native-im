# AI Agent 模块详细文档

## 模块概述

`server/src/services/ai` 目录实现了一个完整的 AI Agent 系统，是整个 AI-Native IM 应用的核心智能引擎。该系统采用了现代化的分层架构设计，支持多模态交互、学习进化、情感分析等高级功能。

### 核心特性

- **多 LLM 提供商支持**: OpenAI、Anthropic、Google Gemini、SiliconFlow、智谱 AI、月之暗面、通义千问、文心一言
- **中文优化**: 针对中文场景进行了深度优化，支持中文情感分析和文化适配
- **多模态处理**: 文本、图像、语音的统一处理
- **认知架构**: 模拟人类认知过程的三层架构
- **学习进化**: 从用户交互中学习并持续改进
- **情感系统**: 11种情感状态，基于规则和 LLM 的混合分析
- **工具系统**: 可扩展的工具调用框架
- **流式处理**: 实时响应和打字模拟
- **主动消息**: 智能的主动对话发起

---

## 目录结构

```
ai/
├── mod.rs                    # 模块导出和公共接口
├── error.rs                  # 错误定义和处理
├── system.rs                 # 系统初始化和配置管理
│
├── core/                     # 核心认知架构
│   ├── mod.rs                # 核心模块导出
│   ├── agent_core.rs         # AgentCore trait 定义
│   ├── default_agent.rs      # 默认 Agent 实现
│   └── events.rs             # 事件和动作定义
│
├── cot/                      # 思维链 (Chain of Thought)
│   └── mod.rs                # CoT 处理器实现
│
├── emotion/                  # 情感分析系统
│   ├── mod.rs                # 情感模块导出
│   ├── rule_based.rs         # 基于规则的情感分析
│   └── llm_based.rs          # 基于 LLM 的情感分析
│
├── intent/                   # 意图识别
│   └── mod.rs                # 意图分类器
│
├── learning/                 # 学习系统
│   └── mod.rs                # 学习引擎和反馈处理
│
├── memory/                   # 记忆系统
│   ├── mod.rs                # 记忆模块导出
│   └── vector_store.rs       # 向量存储实现
│
├── persona/                  # 人设系统
│   └── mod.rs                # 人格配置和预设
│
├── proactive/                # 主动消息系统
│   └── mod.rs                # 主动消息触发器
│
├── providers/                # LLM 提供商
│   ├── mod.rs                # 提供商工厂和管理
│   └── provider_manager.rs   # 提供商管理器
│
├── quota/                    # 配额管理
│   └── mod.rs                # 使用量控制
│
├── search/                   # 搜索功能
│   └── mod.rs                # 语义搜索服务
│
├── speech/                   # 语音处理
│   ├── mod.rs                # 语音模块导出
│   ├── stt.rs                # 语音转文字
│   └── tts.rs                # 文字转语音
│
├── streaming/                # 流式处理
│   └── mod.rs                # 流式响应处理
│
├── summary/                  # 总结功能
│   ├── mod.rs                # 总结服务接口
│   └── service.rs            # 总结服务实现
│
├── tools/                    # 工具系统
│   ├── mod.rs                # 工具框架
│   └── registry.rs           # 工具注册表
│
├── user_profile/             # 用户画像
│   └── mod.rs                # 用户偏好和关系分析
│
├── agent_service.rs          # Agent 服务主实现
├── agent_message_handler.rs  # 消息处理器
├── agent_message_sender.rs   # 消息发送器
├── ai_message_router.rs      # AI 消息路由器
├── embedding.rs              # 嵌入服务
├── llm_client.rs             # LLM 客户端
├── multimodal.rs             # 多模态处理器
├── personality.rs            # 个性化管理器
├── trigger.rs                # 触发器系统
├── typing_simulator.rs       # 打字模拟器
├── vector_store.rs           # 向量存储接口
├── integration_example.rs    # 集成示例
├── integration_tests.rs      # 集成测试
└── property_tests.rs         # 属性测试
```

---

## 核心组件详解

### 1. 错误处理系统 (error.rs)

定义了完整的 AI Agent 错误类型层次结构，包含 33 种不同的错误类型。

#### 主要错误类型

```rust
#[derive(Debug, Error)]
pub enum AIAgentError {
    // 基础错误
    #[error("AI Agent not found: {0}")]
    AgentNotFound(Uuid),

    #[error("Group already has an AI Agent")]
    AgentAlreadyExists,

    // 配置错误
    #[error("Invalid personality parameter: {0}")]
    InvalidPersonality(String),

    // LLM 相关错误
    #[error("LLM API error: {0}")]
    LLMError(String),

    #[error("LLM API timeout after {0} seconds")]
    LLMTimeout(u64),

    #[error("All LLM providers failed")]
    AllProvidersFailed,

    // 功能错误
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionError(String),

    // 高级功能错误
    #[error("Streaming response interrupted")]
    StreamingInterrupted,
}
```

#### 特性
- 提供用户友好的中文错误消息
- 支持错误状态码映射
- 实现了可重试错误判断

### 2. 认知架构 (core/)

实现了三层认知架构，模拟人类的认知过程。

#### 2.1 概率类型 (Probability)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Probability(f64);

impl Probability {
    // 基本操作
    pub fn new(value: f64) -> Self;
    pub fn should_respond(&self, threshold: f64) -> bool;

    // 概率运算
    pub fn and(self, other: Probability) -> Probability;
    pub fn or(self, other: Probability) -> Probability;
    pub fn boost(self, factor: f64) -> Probability;
}
```

#### 2.2 人格配置 (Persona)

```rust
pub struct Persona {
    pub agent_id: Uuid,
    pub name: String,
    pub toxicity_level: i16,          // 毒舌程度 (0-10)
    pub enthusiasm_level: i16,        // 热情程度 (0-10)
    pub expertise_domains: Vec<String>, // 专业领域
    pub speech_suffix: Option<String>,   // 说话后缀
    pub forbidden_emotions: Vec<EmotionState>, // 禁止表达的情感
    pub proactive_enabled: bool,       // 主动消息开关
    pub learning_enabled: bool,        // 学习功能开关
}
```

#### 2.3 AgentCore Trait

定义了 AI Agent 的核心行为接口：

```rust
#[async_trait]
pub trait AgentCore: Send + Sync {
    async fn process(&self, event: AgentEvent) -> Result<Vec<AgentAction>, AIAgentError>;
    fn should_intervene(&self, event: &AgentEvent) -> Probability;
    async fn analyze_emotion(&self, context: &MessageContext) -> Result<EmotionState, AIAgentError>;
}
```

### 3. 系统管理 (system.rs)

提供统一的系统初始化和管理接口。

#### 3.1 系统配置

```rust
pub struct AIAgentSystemConfig {
    // 国际 LLM 提供商
    pub openai_api_key: Option<String>,
    pub openai_model: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: Option<String>,
    pub google_api_key: Option<String>,
    pub google_model: Option<String>,

    // 国内 LLM 提供商（优先级）
    pub siliconflow_api_key: Option<String>,
    pub siliconflow_model: Option<String>,
    pub zhipu_api_key: Option<String>,
    pub zhipu_model: Option<String>,
    pub moonshot_api_key: Option<String>,
    pub moonshot_model: Option<String>,
    pub tongyi_api_key: Option<String>,
    pub tongyi_model: Option<String>,
    pub wenxin_api_key: Option<String>,
    pub wenxin_secret_key: Option<String>,
    pub wenxin_model: Option<String>,

    // 自定义/本地 LLM
    pub custom_llm_api_base: Option<String>,
    pub custom_llm_model: Option<String>,

    // 其他配置
    pub embedding_model: Option<String>,
    pub vision_model: Option<String>,
    pub qdrant_url: Option<String>,
    pub enable_vision: bool,
    pub mock_mode: bool,
}
```

#### 3.2 系统特性

- **多提供商支持**: 支持 8+ 个 LLM 提供商
- **国内优先**: 优先使用国内服务商降低延迟
- **自动降级**: 组件失败时自动切换到 Mock 模式
- **配置灵活**: 通过环境变量或代码配置

### 4. 思维链系统 (cot/)

实现了隐藏式思维链功能，让 AI 在回复前进行"思考"。

#### 4.1 配置选项

```rust
pub struct CoTConfig {
    pub enabled: bool,
    pub max_thinking_tokens: usize,
    pub store_thinking: bool,
    pub style: ThinkingStyle,
}

pub enum ThinkingStyle {
    Analytical,    // 分析型
    Empathetic,    // 共情型
    Creative,      // 创意型
    Balanced,      // 平衡型
}
```

#### 4.2 功能特性

- 使用特殊标记分离思考过程和最终回复
- 支持意图识别和理解置信度评估
- 可配置的思考风格和深度

### 5. 多模态处理 (multimodal.rs)

支持文本、图像、语音的统一处理。

#### 5.1 多模态输入类型

```rust
pub struct MultimodalInput {
    pub text: Option<String>,
    pub images: Vec<ImageInput>,
    pub audio: Option<AudioInput>,
}

pub struct ImageInput {
    pub url: Option<String>,
    pub base64_data: Option<String>,
    pub description: Option<String>,
}
```

#### 5.2 处理流程

1. 解析多模态输入
2. 调用相应的模型处理
3. 融合不同模态的信息
4. 生成统一的文本表示

### 6. LLM 客户端系统 (llm_client.rs)

提供统一的 LLM 接口，支持多个提供商。

#### 6.1 支持的提供商

- **OpenAI**: GPT-3.5, GPT-4 系列
- **Anthropic**: Claude 系列
- **Google**: Gemini 系列
- **SiliconFlow**: 中文优化模型
- **智谱 AI**: GLM-4 系列
- **月之暗面**: Kimi 系列
- **通义千问**: Qwen 系列
- **文心一言**: ERNIE 系列

#### 6.2 统一接口

```rust
#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn generate(&self, request: &ChatMessage) -> Result<String, AIAgentError>;
    async fn generate_stream(&self, request: &ChatMessage) -> Result<BoxStream<'static, String>, AIAgentError>;
    async fn generate_with_tools(&self, request: &ChatMessage, tools: &[Tool]) -> Result<ToolCallResult, AIAgentError>;
}
```

#### 6.3 特性

- 流式响应支持
- 工具调用支持
- 重试机制和超时控制
- 请求追踪和日志

### 7. 触发器系统 (trigger.rs)

智能判断何时应该响应用户消息。

#### 7.1 触发类型

1. **@mention**: 100% 触发概率
2. **关键词触发**: 60-80% 概率
3. **上下文相似**: 40-60% 概率
4. **随机触发**: 1-5% 概率
5. **沉默打破**: 10-20% 概率（2小时无消息）

#### 7.2 概率计算

```rust
pub struct TriggerEvaluator {
    pub mention_threshold: f64,      // 0.9
    pub keyword_threshold: f64,      // 0.6
    pub context_threshold: f64,      // 0.4
    pub random_threshold: f64,       // 0.03
    pub silence_threshold: f64,      // 0.15
}
```

### 8. 情感系统 (emotion/)

实现了 11 种情感状态的识别和表达。

#### 8.1 支持的情感

- Neutral (中性)
- Happy (开心)
- Excited (兴奋)
- Sad (难过)
- Angry (生气)
- Empathetic (共情)
- Supportive (支持)
- Curious (好奇)
- Playful (俏皮)
- Anxious (焦虑)
- Confused (困惑)

#### 8.2 分析器类型

- **RuleBasedEmotionAnalyzer**: 基于规则和关键词
  - 速度快，无需 API 调用
  - 包含完整的中文情感词汇表

- **LLMBasedEmotionAnalyzer**: 基于 LLM 分析
  - 准确度高，能理解上下文
  - 消耗更多资源

#### 8.3 混合分析器

```rust
pub struct HybridEmotionAnalyzer {
    rule_based: RuleBasedEmotionAnalyzer,
    llm_based: LLMBasedEmotionAnalyzer,
    use_llm_threshold: f64,  // 复杂度超过阈值时使用 LLM
}
```

### 9. 学习系统 (learning/)

从用户交互中学习并持续改进。

#### 9.1 学习机制

```rust
pub struct FeedbackRecord {
    pub user_id: Uuid,
    pub agent_id: Uuid,
    pub context_hash: String,
    pub feedback_type: FeedbackType,
    pub timestamp: DateTime<Utc>,
}

pub enum FeedbackType {
    Positive,    // 👍 正面反馈
    Negative,    // 👎 负面反馈
    Correction,  // ✏️ 纠正
    Ignore,      // 🤷 忽略
}
```

#### 9.2 模式识别

```rust
pub struct LearningPattern {
    pub context_hash: String,
    pub successful_response: Option<String>,
    pub feedback_type: FeedbackType,
    pub frequency: i32,
    pub needs_review: bool,
    pub last_updated: DateTime<Utc>,
}
```

### 10. 工具系统 (tools/)

可扩展的工具调用框架。

#### 10.1 工具接口

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, parameters: Value) -> Result<Value, ToolError>;
}
```

#### 10.2 内置工具

- **CalculatorTool**: 数学计算
- **CurrentTimeTool**: 获取当前时间
- **WeatherTool**: 天气查询（Mock 实现）

#### 10.3 工具注册

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Box<dyn Tool>);
    pub fn get(&self, name: &str) -> Option<&dyn Tool>;
    pub fn list_tools(&self) -> Vec<&str>;
}
```

### 11. 流式处理 (streaming/)

实现实时响应传输和打字模拟。

#### 11.1 流式配置

```rust
pub struct StreamingConfig {
    pub enabled: bool,
    pub chunk_size: usize,
    pub delay_between_chunks: Duration,
    pub typing_simulation: bool,
}
```

#### 11.2 流式块

```rust
pub struct StreamingChunk {
    pub stream_id: Uuid,
    pub sequence: u32,
    pub content: String,
    pub is_final: bool,
    pub state: StreamState,
}

pub enum StreamState {
    Starting,
    Streaming,
    Paused,
    Finished,
    Error,
}
```

### 12. 打字模拟器 (typing_simulator.rs)

模拟真人打字延迟，增加真实感。

```rust
pub struct TypingSimulator {
    base_delay: Duration,
    variance: Duration,
    chars_per_second: f64,
}

impl TypingSimulator {
    pub fn calculate_delay(&self, message: &str) -> Duration {
        let base = (message.len() as f64 / self.chars_per_second) * 1000.0;
        let variance = rand::random::<f64>() * self.variance.as_millis() as f64;
        Duration::from_millis((base + variance) as u64)
    }
}
```

### 13. 语音处理 (speech/)

支持语音转文字和文字转语音。

#### 13.1 STT (Speech-to-Text)

```rust
#[async_trait]
pub trait STTProvider: Send + Sync {
    async fn transcribe(&self, audio_data: &[u8]) -> Result<String, STTError>;
    async fn transcribe_with_language(&self, audio_data: &[u8], language: &str) -> Result<String, STTError>;
}
```

#### 13.2 TTS (Text-to-Speech)

```rust
#[async_trait]
pub trait TTSProvider: Send + Sync {
    async fn synthesize(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError>;
    async fn synthesize_with_emotion(&self, text: &str, voice_id: &str, emotion: EmotionState) -> Result<Vec<u8>, TTSError>;
}
```

### 14. 总结服务 (summary/)

对长对话进行智能总结。

#### 14.1 总结风格

```rust
pub enum SummaryStyle {
    Brief,        // 简洁要点
    Detailed,     // 详细叙述
    Topics,       // 主要话题
    ActionItems,  // 行动项
}
```

#### 14.2 总结请求

```rust
pub struct SummaryRequest {
    pub agent_id: Uuid,
    pub group_id: Uuid,
    pub time_range: TimeRange,
    pub max_messages: usize,
    pub style: SummaryStyle,
}
```

### 15. 向量存储 (vector_store.rs)

基于 Qdrant 的向量数据库，支持语义搜索。

#### 15.1 向量存储接口

```rust
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn store(&self, id: &str, vector: Vec<f32>, metadata: Value) -> Result<(), VectorStoreError>;
    async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, VectorStoreError>;
    async fn delete(&self, id: &str) -> Result<(), VectorStoreError>;
}
```

#### 15.2 搜索结果

```rust
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Value,
    pub time_decay: f32,  // 时间衰减权重
}
```

### 16. 嵌入服务 (embedding.rs)

将文本转换为向量表示。

#### 16.1 支持的嵌入模型

- **OpenAI Embedding**: text-embedding-ada-002
- **SiliconFlow Embedding**: 中文优化嵌入

#### 16.2 批量处理

```rust
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    async fn generate_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;
    fn dimension(&self) -> usize;
}
```

### 17. 配额管理 (quota/)

控制 API 调用频率和使用量。

#### 17.1 配额类型

- **日配额**: 每日调用次数限制
- **月配额**: 每月调用次数限制
- **服务配额**: 按服务类型限制
- **用户配额**: 按用户级别限制

#### 17.2 配额检查

```rust
pub async fn check_and_record_usage(
    quota_manager: &dyn QuotaManager,
    user_id: Uuid,
    service_type: ServiceType,
) -> Result<QuotaStatus, QuotaError> {
    // 检查配额
    // 记录使用
    // 返回状态
}
```

### 18. 主动消息系统 (proactive/)

智能发起对话，而不是被动响应。

#### 18.1 触发器

```rust
pub enum ProactiveTrigger {
    SilenceBreaker {    // 沉默打破
        hours_of_silence: u32,
        probability: f64,
    },
    Greeting {          // 问候
        time_of_day: TimeOfDay,
        greeting_type: GreetingType,
    },
    CareMessage {       // 关心消息
        reason: CareReason,
        probability: f64,
    },
    ContextAware {      // 上下文感知
        trigger_keywords: Vec<String>,
        probability: f64,
    },
}
```

#### 18.2 活动模式

```rust
pub struct ActivityPattern {
    pub user_id: Uuid,
    pub active_hours: Vec<TimeOfDay>,
    pub preferred_topics: Vec<String>,
    pub response_patterns: HashMap<String, f64>,
}
```

### 19. 用户画像 (user_profile/)

分析用户行为和偏好。

#### 19.1 用户画像数据

```rust
pub struct UserProfile {
    pub user_id: Uuid,
    pub basic_info: BasicInfo,
    pub interaction_patterns: InteractionPatterns,
    pub preferences: UserPreferences,
    pub relationship_level: RelationshipLevel,
    pub last_updated: DateTime<Utc>,
}
```

#### 19.2 关系等级

```rust
pub enum RelationshipLevel {
    Stranger,      // 陌生人
    Acquaintance,  // 熟人
    Friend,        // 朋友
    CloseFriend,   // 好朋友
    BestFriend,    // 最好的朋友
}
```

### 20. 搜索功能 (search/)

提供语义搜索和关键词搜索。

#### 20.1 搜索类型

```rust
pub enum SearchType {
    Semantic,      // 语义搜索
    Keyword,       // 关键词搜索
    Hybrid,        // 混合搜索
}
```

#### 20.2 搜索请求

```rust
pub struct SearchRequest {
    pub query: String,
    pub search_type: SearchType,
    pub time_range: Option<TimeRange>,
    pub limit: usize,
    pub include_metadata: bool,
}
```

### 21. 人设系统 (persona/)

管理 AI 的人格设定。

#### 21.1 人格配置

```rust
pub struct Persona {
    pub agent_id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub personality: PersonalityTraits,
    pub response_style: ResponseStyle,
    pub voice_settings: VoiceSettings,
    pub emotion_boundaries: EmotionBoundaries,
    pub sticker_preferences: StickerPreferences,
}
```

#### 21.2 预设人设

```rust
pub struct PersonaPresets;
impl PersonaPresets {
    pub fn friendly_helper() -> Persona;
    pub fn tsundere() -> Persona;
    pub fn professional_expert() -> Persona;
    pub fn playful_companion() -> Persona;
    pub fn wise_mentor() -> Persona;
}
```

### 22. LLM 提供商管理 (providers/)

管理多个 LLM 提供商的切换和负载均衡。

#### 22.1 提供商工厂

```rust
pub struct LLMClientFactory;

impl LLMClientFactory {
    pub fn create_openai_client(config: &OpenAIConfig) -> Arc<dyn LLMClient>;
    pub fn create_anthropic_client(config: &AnthropicConfig) -> Arc<dyn LLMClient>;
    pub fn create_siliconflow_client(config: &SiliconFlowConfig) -> Arc<dyn LLMClient>;
    // ... 其他提供商
}
```

#### 22.2 提供商管理器

```rust
pub struct ProviderManager {
    providers: Vec<Box<dyn LLMClient>>,
    current_index: usize,
    health_status: HashMap<String, bool>,
}

impl ProviderManager {
    pub async fn get_next_provider(&mut self) -> Result<Box<dyn LLMClient>, AIAgentError>;
    pub async fn check_health(&self) -> HashMap<String, bool>;
    pub fn set_priority_order(&mut self, order: &[String]);
}
```

---

## 集成示例

### 基础集成

```rust
use ai_im_server::services::ai::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建配置
    let config = AIAgentSystemConfig::default();

    // 2. 初始化系统
    let pool = create_database_pool().await?;
    let mut ai_system = AIAgentSystem::new(pool, config).await?;

    // 3. 启动响应工作器
    let message_router = create_message_router().await?;
    let worker_handle = ai_system.start_response_worker(message_router).await?;

    // 4. 创建代理
    let agent_id = ai_system.create_agent(
        group_id,
        "AI Assistant",
        PersonaPresets::friendly_helper()
    ).await?;

    // 5. 发送消息
    let response = ai_system.handle_message(
        agent_id,
        user_id,
        "你好！"
    ).await?;

    println!("AI 回复: {}", response.content);

    Ok(())
}
```

### 自定义工具

```rust
use ai_im_server::services::ai::tools::*;

struct CustomWeatherTool;

#[async_trait]
impl Tool for CustomWeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Get current weather for a city"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city name"
                }
            },
            "required": ["city"]
        })
    }

    async fn execute(&self, parameters: Value) -> Result<Value, ToolError> {
        let city = parameters.get("city")
            .and_then(|v| v.as_str())
            .ok_or(ToolError::InvalidParameters("city is required".to_string()))?;

        // 调用天气 API
        let weather = fetch_weather(city).await?;

        Ok(json!({
            "city": city,
            "temperature": weather.temperature,
            "description": weather.description
        }))
    }
}

// 注册工具
let mut registry = ToolRegistry::new();
registry.register(Box::new(CustomWeatherTool));
```

### 自定义情感分析器

```rust
use ai_im_server::services::ai::emotion::*;

struct CustomEmotionAnalyzer;

#[async_trait]
impl EmotionAnalyzer for CustomEmotionAnalyzer {
    async fn analyze(
        &self,
        text: &str,
        context: Option<&MessageContext>
    ) -> Result<EmotionAnalysis, AIAgentError> {
        // 自定义分析逻辑
        let emotion = if text.contains("开心") || text.contains("高兴") {
            EmotionState::Happy
        } else if text.contains("难过") || text.contains("伤心") {
            EmotionState::Sad
        } else {
            EmotionState::Neutral
        };

        Ok(EmotionAnalysis {
            primary_emotion: emotion,
            confidence: 0.8,
            emotional_keywords: vec![],
            sentiment: Sentiment::Neutral,
        })
    }
}
```

---

## 测试框架

### 1. 单元测试

每个模块都包含完整的单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_probability_operations() {
        let p1 = Probability::new(0.6);
        let p2 = Probability::new(0.4);

        assert_eq!(p1.and(p2).value(), 0.24);
        assert_eq!(p1.or(p2).value(), 0.6);
    }
}
```

### 2. 集成测试

测试完整的处理管道：

```rust
#[tokio::test]
async fn test_full_message_pipeline() {
    let mut system = create_test_system().await;

    // 创建代理
    let agent_id = system
        .create_agent(
            test_group_id(),
            "Test Agent",
            Persona::default()
        )
        .await
        .unwrap();

    // 发送消息
    let response = system
        .handle_message(
            agent_id,
            test_user_id(),
            "Hello, AI!"
        )
        .await
        .unwrap();

    assert!(!response.content.is_empty());
}
```

### 3. 属性测试

使用 proptest 进行属性测试：

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_probability_constraints(
        value in any::<f64>()
    ) {
        let p = Probability::new(value);
        assert!(p.value() >= 0.0 && p.value() <= 1.0);
    }
}
```

---

## 性能优化

### 1. 缓存策略

- **消息缓存**: 最近的对话消息缓存在内存中
- **嵌入缓存**: 常用文本的向量嵌入缓存
- **用户画像缓存**: 用户偏好和关系数据缓存

### 2. 批处理

- **批量嵌入**: 一次处理多个文本
- **批量向量操作**: 批量存储和搜索
- **批量数据库操作**: 减少数据库往返

### 3. 连接池

- **数据库连接池**: PostgreSQL 连接复用
- **Redis 连接池**: 缓存和队列操作
- **HTTP 连接池**: LLM API 调用优化

### 4. 异步处理

- **流式响应**: 边生成边发送
- **并行处理**: 多个任务并行执行
- **非阻塞 I/O**: 全面使用 async/await

---

## 安全考虑

### 1. 输入验证

- 所有用户输入都经过验证
- 防止注入攻击
- 文件上传安全检查

### 2. 权限控制

- 基于角色的访问控制
- API 调用频率限制
- 敏感操作审计

### 3. 数据保护

- 敏感数据加密存储
- 传输过程 TLS 加密
- 定期数据清理

### 4. LLM 安全

- 提示词注入防护
- 输出内容过滤
- 滥用检测和阻止

---

## 监控和日志

### 1. 指标收集

```rust
use prometheus::{IntCounter, Histogram};

lazy_static! {
    static ref AI_REQUESTS_TOTAL: IntCounter = IntCounter::new(
        "ai_requests_total", "Total AI requests"
    ).unwrap();

    static ref AI_RESPONSE_TIME: Histogram = Histogram::new(
        "ai_response_time_seconds", "AI response time"
    ).unwrap();
}
```

### 2. 结构化日志

```rust
use tracing::{info, warn, error};

info!(
    agent_id = %agent_id,
    user_id = %user_id,
    message_length = message.len(),
    "Processing AI message"
);

error!(
    error = %e,
    agent_id = %agent_id,
    "Failed to generate AI response"
);
```

### 3. 分布式追踪

使用 OpenTelemetry 进行请求追踪：

```rust
use opentelemetry::trace::Tracer;

let span = tracer.start("ai_process_message");
span.set_attribute("agent.id", agent_id.to_string());
span.set_attribute("user.id", user_id.to_string());
```

---

## 部署建议

### 1. 环境配置

```bash
# LLM 提供商配置
export OPENAI_API_KEY="your-openai-key"
export SILICONFLOW_API_KEY="your-siliconflow-key"
export ZHIPU_API_KEY="your-zhipu-key"

# 向量数据库
export QDRANT_URL="http://localhost:6333"

# 缓存
export REDIS_URL="redis://localhost:6379"

# 功能开关
export ENABLE_VISION=true
export ENABLE_SPEECH=true
export MOCK_MODE=false
```

### 2. 资源要求

- **CPU**: 4 核心以上（推荐 8 核心）
- **内存**: 8GB 以上（推荐 16GB）
- **存储**: SSD，至少 100GB
- **网络**: 低延迟连接到 LLM 服务商

### 3. 扩展策略

- **水平扩展**: 支持多实例部署
- **负载均衡**: 使用 Redis 共享状态
- **故障转移**: 自动切换备用提供商

---

## 最佳实践

### 1. 提示词设计

- 使用清晰、具体的提示词
- 包含示例和格式要求
- 考虑上下文长度限制

### 2. 错误处理

- 优雅处理 LLM 服务不可用
- 提供有意义的错误消息
- 实现自动重试机制

### 3. 性能优化

- 使用流式响应减少延迟
- 合理设置缓存时间
- 监控和优化热点代码

### 4. 用户体验

- 提供打字指示器
- 支持中断长响应
- 个性化 AI 回复风格

---

## 总结

AI Agent 模块是一个功能强大、设计精良的系统，具有以下特点：

1. **架构清晰**: 分层设计，职责明确
2. **功能完备**: 涵盖 AI 交互的各个方面
3. **扩展性强**: 支持自定义工具、分析器等
4. **性能优化**: 异步处理、缓存、批处理
5. **中文友好**: 针对中文场景优化
6. **生产就绪**: 完整的错误处理、监控、测试

该系统为 AI-Native IM 应用提供了智能的核心引擎，能够提供自然、有趣、有帮助的对话体验。通过持续的学习和优化，AI Agent 会变得越来越智能和个性化。