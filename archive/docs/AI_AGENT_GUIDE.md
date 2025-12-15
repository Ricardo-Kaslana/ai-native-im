# 🤖 AI Agent 系统开发指南

本文档详细介绍了 FeChat AI Agent 系统的架构、功能和使用方法。

## 📋 目录

- [系统概述](#系统概述)
- [架构设计](#架构设计)
- [核心模块](#核心模块)
- [快速开始](#快速开始)
- [配置指南](#配置指南)
- [API 参考](#api-参考)
- [扩展开发](#扩展开发)
- [最佳实践](#最佳实践)

---

## 系统概述

FeChat AI Agent 是一个功能完整的群聊 AI 助手系统，具备以下核心能力：

### 核心特性

| 特性 | 描述 |
|------|------|
| 🧠 三层认知架构 | 感知层 → 认知层 → 执行层 |
| 🇨🇳 国内服务商优先 | 硅基流动、智谱、月之暗面、通义、文心 |
| 🎤 语音交互 | STT (Whisper/阿里云) + TTS (Edge/阿里云/OpenAI) |
| 💬 智能总结 | 多种风格的消息总结 |
| 🔍 语义搜索 | 向量嵌入 + 语义检索 |
| 😊 情绪分析 | 规则 + LLM 混合分析 |
| 🔧 工具调用 | 可扩展的工具系统 |
| 📚 自我学习 | 反馈记录与模式学习 |
| 👤 人设系统 | 毒舌度、热情度、专业领域、口癖 |

### Phase 4 新增特性 (2025-12)

| 特性 | 描述 |
|------|------|
| 🎭 用户自定义人设 | 完整的 PersonaConfig 配置，包含情绪边界、贴纸偏好、语音设置 |
| 💡 思维链 (Hidden CoT) | 隐藏式思考过程，提升回复质量，支持多种思考风格 |
| 👥 用户画像系统 | 6级关系识别 (Stranger→CloseFriend)，亲密度/尊重度/信任度追踪 |
| 🎯 意图分类系统 | 10种意图类型，智能 ResponseStrategy 调整 |
| 🖼️ 多模态增强 | MultimodalContext 统一处理图片/音频/视频/文件/贴纸 |
| 👋 主动对话系统 | 基于用户关系的主动问候、关心消息、日常问候 |

### 技术栈

- **语言**: Rust
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **序列化**: Serde
- **向量存储**: 内置 + Qdrant 支持
- **测试框架**: proptest (属性测试)

---

## 架构设计

### 三层认知架构

```
┌─────────────────────────────────────────────────────────────┐
│                     Perception Layer (感知层)                │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │   STT   │  │  Vision │  │ Emotion │  │ Input Transform │ │
│  │ Service │  │ Service │  │Analyzer │  │   (AgentEvent)  │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
└───────┼────────────┼────────────┼────────────────┼──────────┘
        │            │            │                │
        └────────────┴────────────┴────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Cognition Layer (认知层)                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │ Memory  │  │ Trigger │  │ Learning│  │   AgentCore     │ │
│  │  Store  │  │Evaluator│  │ Engine  │  │   (决策中心)     │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
└───────┼────────────┼────────────┼────────────────┼──────────┘
        │            │            │                │
        └────────────┴────────────┴────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Action Layer (执行层)                   │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │   TTS   │  │  Tool   │  │ Message │  │ Output Execute  │ │
│  │ Service │  │ Registry│  │ Sender  │  │  (AgentAction)  │ │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 数据流

```
用户消息 → AgentEvent → AgentCore.process() → Vec<AgentAction> → 执行
```

---

## 核心模块

### 1. 事件与动作 (`core/events.rs`)

#### AgentEvent - 输入事件

```rust
pub enum AgentEvent {
    /// 用户文本消息
    UserMessage(MessageContext),
    
    /// 用户图片消息
    UserImage {
        image_url: String,
        caption: Option<String>,
        context: MessageContext,
    },
    
    /// 用户语音消息 (已转录)
    UserVoice {
        transcription: String,
        audio_url: String,
        context: MessageContext,
    },
    
    /// 系统信号 (沉默检测、定时事件等)
    SystemSignal {
        signal_type: SignalType,
        group_id: Uuid,
    },
    
    /// 工具执行结果
    ToolOutput {
        tool_name: String,
        result: String,
        request_id: String,
        success: bool,
    },
    
    /// 用户反馈
    UserFeedback {
        message_id: Uuid,
        response_id: Uuid,
        feedback_type: FeedbackType,
        correction: Option<String>,
        user_id: Uuid,
    },
}
```

#### AgentAction - 输出动作

```rust
pub enum AgentAction {
    /// 文本回复
    ReplyText { content: String, emotion: EmotionState },
    
    /// 语音回复
    ReplyAudio { text: String, voice_id: Option<String> },
    
    /// 图片回复
    ReplyImage { url: String, caption: Option<String> },
    
    /// 表情包回复
    ReplySticker { sticker_id: String },
    
    /// 调用工具
    CallTool { name: String, args: Value, request_id: String },
    
    /// 生成总结
    Summarize { message_count: usize },
    
    /// 忽略 (不回复)
    Ignore,
    
    /// 思考 (内部)
    Think { thought: String },
}
```

### 2. AgentCore Trait (`core/agent_core.rs`)

```rust
#[async_trait]
pub trait AgentCore: Send + Sync {
    /// 处理事件，返回动作列表
    async fn process(
        &self,
        event: AgentEvent,
        memory: &dyn MemoryStore,
        persona: &Persona,
    ) -> Result<Vec<AgentAction>, AIAgentError>;

    /// 判断是否应该介入
    async fn should_intervene(
        &self,
        context: &MessageContext,
        persona: &Persona,
    ) -> Probability;

    /// 分析情绪
    async fn analyze_emotion(&self, context: &MessageContext) -> EmotionState;
}
```

### 3. 人设系统 (`persona/mod.rs`)

```rust
pub struct Persona {
    pub agent_id: Uuid,
    pub name: String,
    pub toxicity_level: i16,      // 毒舌度 0-10
    pub enthusiasm_level: i16,    // 热情度 0-10
    pub expertise_domains: Vec<String>,  // 专业领域
    pub speech_suffix: Option<String>,   // 口癖 (如 "喵~")
    pub trigger_keywords: Vec<String>,   // 触发关键词
    pub voice_id: String,         // TTS 语音 ID
    pub voice_speed: f32,         // 语速
    pub voice_pitch: f32,         // 音调
    pub forbidden_emotions: Vec<EmotionState>,  // 禁止的情绪
    pub proactive_enabled: bool,  // 是否启用主动消息
    pub learning_enabled: bool,   // 是否启用学习
}
```

#### Phase 4 新增: PersonaConfig 完整配置

```rust
pub struct PersonaConfig {
    // 基础设置
    pub name: String,
    pub toxicity_level: i16,
    pub enthusiasm_level: i16,
    
    // 情绪边界 (Phase 4)
    pub emotion_boundaries: EmotionBoundaries,
    
    // 贴纸偏好 (Phase 4)
    pub sticker_preferences: StickerPreferences,
    
    // 语音设置 (Phase 4)
    pub voice_settings: VoiceSettings,
    
    // 回复风格 (Phase 4)
    pub response_style: ResponseStyle,
    pub response_length: ResponseLength,
}
```

#### 预设人设

```rust
// 使用预设
let persona = PersonaPresets::friendly_assistant();
let persona = PersonaPresets::sarcastic_bot();
let persona = PersonaPresets::professional_expert();
let persona = PersonaPresets::cute_mascot();
```

### 4. 情绪分析 (`emotion/mod.rs`)

```rust
pub enum EmotionState {
    Neutral,    // 平静
    Happy,      // 开心
    Excited,    // 兴奋
    Sad,        // 难过
    Angry,      // 生气
    Empathetic, // 共情
    Supportive, // 支持
    Curious,    // 好奇
    Playful,    // 俏皮
    Anxious,    // 焦虑
    Confused,   // 困惑
}

// 使用规则分析器 (快速，无 API 调用)
let analyzer = RuleBasedEmotionAnalyzer::new();
let analysis = analyzer.analyze("太棒了！我好开心！").await;
// analysis.primary_emotion == EmotionState::Happy

// 使用混合分析器 (规则 + LLM)
let analyzer = HybridEmotionAnalyzer::new(rule_analyzer, llm_analyzer);
```

### 5. 工具系统 (`tools/mod.rs`)

#### 定义工具

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> Result<String, ToolError>;
}
```

#### 内置工具

| 工具 | 描述 |
|------|------|
| `CalculatorTool` | 数学计算 |
| `CurrentTimeTool` | 获取当前时间 |
| `WeatherTool` | 天气查询 (Mock) |

#### 注册工具

```rust
let mut registry = ToolRegistry::new();
registry.register(Arc::new(CalculatorTool::new()));
registry.register(Arc::new(CurrentTimeTool::new()));

// 获取 LLM function definitions
let functions = registry.to_function_definitions();
```

### 6. 语音服务 (`speech/`)

#### STT (语音转文字)

```rust
#[async_trait]
pub trait STTService: Send + Sync {
    async fn transcribe(&self, audio_url: &str) -> Result<Transcription, AIAgentError>;
    async fn transcribe_bytes(&self, audio: &[u8], format: &str) -> Result<Transcription, AIAgentError>;
}

// 使用 Whisper
let stt = WhisperSTT::new(api_key);
let transcription = stt.transcribe("https://example.com/audio.mp3").await?;
```

#### TTS (文字转语音)

```rust
#[async_trait]
pub trait TTSService: Send + Sync {
    async fn synthesize(&self, text: &str, config: &VoiceConfig) -> Result<AudioData, AIAgentError>;
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AIAgentError>;
}

// 使用 Edge TTS (免费)
let tts = EdgeTTS::new();
let config = VoiceConfig::new("zh-CN-XiaoxiaoNeural")
    .with_speed(1.1)
    .with_pitch(1.0);
let audio = tts.synthesize("你好，世界！", &config).await?;
```

### 7. 记忆系统 (`memory/mod.rs`)

```rust
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store(&self, memory: Memory) -> Result<(), MemoryError>;
    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError>;
    async fn get_context(&self, group_id: Uuid, limit: usize) -> Result<Vec<ContextMessage>, MemoryError>;
}

// 使用内存向量存储
let store = InMemoryVectorStore::new();

// 使用嵌入服务的记忆存储
let store = EmbeddingMemoryStore::new(embedding_service);
```

### 8. 配额管理 (`quota/mod.rs`)

```rust
#[async_trait]
pub trait QuotaManager: Send + Sync {
    async fn check_quota(&self, user_id: Uuid, service: ServiceType) -> Result<QuotaStatus, QuotaError>;
    async fn record_usage(&self, user_id: Uuid, service: ServiceType, amount: i64) -> Result<(), QuotaError>;
    async fn get_usage_stats(&self, user_id: Uuid) -> Result<Vec<UsageRecord>, QuotaError>;
}

pub enum ServiceType {
    LLM,
    Embedding,
    STT,
    TTS,
    ImageGen,
    Vision,
}
```

### 9. 语义搜索 (`search/mod.rs`)

```rust
#[async_trait]
pub trait SemanticSearchService: Send + Sync {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError>;
    async fn queue_embedding(&self, message_id: Uuid, content: &str) -> Result<(), SearchError>;
    async fn process_pending_embeddings(&self) -> Result<usize, SearchError>;
}

let request = SearchRequest {
    query: "关于项目进度的讨论".to_string(),
    group_id: Some(group_id),
    semantic: true,
    limit: 10,
    min_score: Some(0.7),
    ..Default::default()
};
let results = search_service.search(request).await?;
```

### 10. 消息总结 (`summary/mod.rs`)

```rust
#[async_trait]
pub trait SummarizeService: Send + Sync {
    async fn summarize(&self, request: SummaryRequest) -> Result<Summary, SummaryError>;
    async fn get_cached(&self, group_id: Uuid) -> Option<Summary>;
}

pub enum SummaryStyle {
    Brief,      // 简短
    Detailed,   // 详细
    Bullet,     // 要点
    Narrative,  // 叙述
}
```

---

## 快速开始

### 1. 创建基本 Agent

```rust
use fechat_server::services::ai::*;

// 1. 创建 LLM 客户端
let llm_client = SiliconFlowClient::new(
    "your-api-key",
    "Qwen/Qwen2.5-7B-Instruct",
);

// 2. 创建记忆存储
let memory_store = InMemoryVectorStore::new();

// 3. 创建 Agent Core
let agent = DefaultAgentCore::new(
    Arc::new(llm_client),
    Arc::new(memory_store),
);

// 4. 定义人设
let persona = Persona::new("小助手")
    .with_toxicity(3)
    .with_enthusiasm(7)
    .with_suffix("~");

// 5. 处理消息
let event = AgentEvent::UserMessage(MessageContext::new(
    "你好！",
    user_id,
    "张三",
    group_id,
));

let actions = agent.process(event, &memory_store, &persona).await?;
```

### 2. 添加工具支持

```rust
// 创建工具注册表
let mut registry = ToolRegistry::new();
registry.register(Arc::new(CalculatorTool::new()));
registry.register(Arc::new(CurrentTimeTool::new()));

// 创建带工具的 Agent
let agent = DefaultAgentCore::new(llm_client, memory_store)
    .with_tool_registry(Arc::new(registry));
```

### 3. 添加情绪分析

```rust
let emotion_analyzer = RuleBasedEmotionAnalyzer::new();
let agent = DefaultAgentCore::new(llm_client, memory_store)
    .with_emotion_analyzer(Arc::new(emotion_analyzer));
```

---

## 配置指南

### 环境变量

```env
# LLM 服务商 (国内优先)
SILICONFLOW_API_KEY=sk-xxx
SILICONFLOW_MODEL=Qwen/Qwen2.5-7B-Instruct

ZHIPU_API_KEY=xxx
ZHIPU_MODEL=glm-4

MOONSHOT_API_KEY=sk-xxx
MOONSHOT_MODEL=moonshot-v1-8k

# 国际备选
OPENAI_API_KEY=sk-xxx
OPENAI_MODEL=gpt-3.5-turbo

# 语音服务
WHISPER_API_KEY=sk-xxx
TTS_PROVIDER=edge  # edge, openai, ali

# 向量数据库
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=xxx

# 配额设置
DEFAULT_DAILY_TOKEN_LIMIT=100000
DEFAULT_MONTHLY_TOKEN_LIMIT=3000000
```

### Provider 优先级

```rust
// 国内服务商优先级 (数字越小优先级越高)
pub const PROVIDER_PRIORITIES: &[(&str, u8)] = &[
    ("siliconflow", 1),   // 硅基流动
    ("zhipu", 2),         // 智谱 AI
    ("moonshot", 3),      // 月之暗面
    ("tongyi", 4),        // 通义千问
    ("wenxin", 5),        // 文心一言
    ("openai", 10),       // OpenAI
    ("anthropic", 11),    // Anthropic
];
```

---

## API 参考

### REST API 端点

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | `/api/ai/groups/:group_id/summary` | 生成消息总结 |
| POST | `/api/ai/messages/:message_id/feedback` | 提交反馈 |
| GET | `/api/ai/groups/:group_id/feedback/stats` | 获取反馈统计 |
| GET | `/api/ai/agents` | 获取 Agent 列表 |
| POST | `/api/ai/agents` | 创建 Agent |
| PUT | `/api/ai/agents/:id` | 更新 Agent |
| DELETE | `/api/ai/agents/:id` | 删除 Agent |

### 总结 API 示例

```bash
curl -X POST http://localhost:8080/api/ai/groups/{group_id}/summary \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "message_count": 50,
    "style": "bullet",
    "use_cache": true
  }'
```

---

## 扩展开发

### 自定义工具

```rust
pub struct MyCustomTool {
    // 工具配置
}

#[async_trait]
impl Tool for MyCustomTool {
    fn name(&self) -> &str {
        "my_custom_tool"
    }

    fn description(&self) -> &str {
        "我的自定义工具"
    }

    fn parameters(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "param1": {
                    "type": "string",
                    "description": "参数1"
                }
            },
            "required": ["param1"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let param1 = args["param1"].as_str()
            .ok_or(ToolError::InvalidArguments("missing param1".to_string()))?;
        
        // 执行逻辑
        Ok(format!("结果: {}", param1))
    }
}
```

### 自定义 LLM 客户端

```rust
#[async_trait]
impl LLMClient for MyLLMClient {
    async fn generate_response(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, AIAgentError> {
        // 实现 LLM 调用
    }

    async fn generate_with_context(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
    ) -> Result<String, AIAgentError> {
        // 实现带上下文的 LLM 调用
    }
}
```

### 自定义情绪分析器

```rust
#[async_trait]
impl EmotionAnalyzer for MyEmotionAnalyzer {
    async fn analyze(&self, text: &str) -> EmotionAnalysis {
        // 实现情绪分析
    }

    async fn analyze_context(&self, messages: &[ChatMessage]) -> EmotionState {
        // 实现上下文情绪分析
    }
}
```

---

## 最佳实践

### 1. 错误处理

```rust
// 使用 Result 类型
let result = agent.process(event, &memory, &persona).await;
match result {
    Ok(actions) => {
        for action in actions {
            execute_action(action).await?;
        }
    }
    Err(AIAgentError::LLMError(msg)) => {
        // LLM 错误，可能需要重试
        tracing::warn!("LLM error: {}", msg);
    }
    Err(AIAgentError::QuotaExceeded) => {
        // 配额超限，通知用户
        send_quota_warning().await;
    }
    Err(e) => {
        tracing::error!("Agent error: {}", e);
    }
}
```

### 2. 性能优化

```rust
// 使用缓存
let summary = if let Some(cached) = summary_service.get_cached(group_id).await {
    cached
} else {
    summary_service.summarize(request).await?
};

// 批量处理嵌入
search_service.process_pending_embeddings().await?;
```

### 3. 测试

```rust
// 使用 Mock 进行测试
let mock_llm = MockLLMClient::new("test response");
let mock_memory = MockMemoryStore::new();
let agent = DefaultAgentCore::new(
    Arc::new(mock_llm),
    Arc::new(mock_memory),
);

// 属性测试
proptest! {
    #[test]
    fn test_probability_in_range(value in 0.0f64..=1.0) {
        let prob = Probability::new(value);
        prop_assert!(prob.value() >= 0.0);
        prop_assert!(prob.value() <= 1.0);
    }
}
```

### 4. 日志记录

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self), fields(agent_id = %persona.agent_id))]
async fn process_message(&self, event: AgentEvent, persona: &Persona) {
    debug!("Processing event: {:?}", event);
    
    // 处理逻辑
    
    info!(actions_count = actions.len(), "Generated actions");
}
```

---

## 数据库表

Phase 3 新增的数据库表：

| 表名 | 描述 |
|------|------|
| `agent_emotion_history` | 情绪历史记录 |
| `learning_patterns` | 学习模式 |
| `user_feedback` | 用户反馈 |
| `summary_cache` | 总结缓存 |
| `provider_usage` | 服务商使用记录 |

Phase 4 新增的数据库表：

| 表名 | 描述 |
|------|------|
| `agent_user_profiles` | 用户画像 (关系级别、亲密度、兴趣等) |
| `user_interaction_events` | 用户互动事件记录 |

---

## 测试覆盖

| 模块 | 测试数 |
|------|--------|
| Core Agent | 14 |
| Provider Manager | 34 |
| STT Service | 13 |
| TTS Service | 27 |
| Summary Service | 8 |
| Search Service | 8 |
| Tool System | 12 |
| Emotion Analysis | 29 |
| Trigger Evaluation | 18 |
| Memory System | 7 |
| Proactive Messaging | 8 |
| Quota Management | 10 |
| Streaming | 8 |
| Multimodal | 18 |
| Persona | 11 |
| Learning Engine | 12 |
| Integration Tests | 14 |
| **Phase 4 新增** | |
| User Profile | 8 |
| Intent Classifier | 12 |
| CoT Processor | 6 |
| Proactive (Enhanced) | 10 |
| **总计** | **393** |

---

## 相关文档

- [项目状态](../PROJECT_STATUS.md)
- [API 参考](../archive/docs/API_REFERENCE.md)
- [Phase 3 需求文档](../.kiro/specs/phase3-ai-agent-evolution/requirements.md)
- [Phase 3 设计文档](../.kiro/specs/phase3-ai-agent-evolution/design.md)

---

**版本**: v1.2.0  
**更新日期**: 2025-12-11  
**维护者**: FeChat Team

### 版本历史

| 版本 | 日期 | 重要更新 |
|------|------|------|
| v1.2.0 | 2025-12-11 | Phase 4 完成: 用户自定义人设、思维链、用户画像、意图分类、多模态增强、主动对话 |
| v1.1.0 | 2025-12-09 | Phase 3 完成: 三层架构、多服务商支持、语音交互 |
| v1.0.0 | 2025-11-01 | 初始版本 |
