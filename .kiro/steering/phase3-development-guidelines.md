# Phase 3 AI Agent Evolution - 开发规范

## 概述

本文档为 Phase 3 AI Agent Evolution 的开发提供详细的编码规范和最佳实践指南。所有开发人员在实现 Phase 3 功能时必须遵循这些规范。

## 项目结构规范

### 后端目录结构

```
server/src/
├── api/                    # API 路由层 (HTTP handlers)
│   └── ai_agents.rs        # AI Agent 相关 API
├── middleware/             # 中间件 (JWT, 日志等)
├── models/                 # 数据模型 (数据库实体)
│   └── ai_agent.rs         # AI Agent 模型定义
├── services/               # 业务逻辑层
│   └── ai/                 # AI 服务模块
│       ├── core/           # [新增] 核心抽象层
│       │   ├── mod.rs
│       │   ├── events.rs       # AgentEvent, AgentAction 枚举
│       │   ├── agent_core.rs   # AgentCore trait
│       │   └── default_agent.rs # 默认实现
│       ├── providers/      # [新增] LLM 服务商管理
│       │   ├── mod.rs
│       │   ├── provider_manager.rs
│       │   ├── zhipu.rs        # 智谱 AI
│       │   ├── moonshot.rs     # 月之暗面
│       │   ├── tongyi.rs       # 通义千问
│       │   └── wenxin.rs       # 文心一言
│       ├── speech/         # [新增] 语音服务
│       │   ├── mod.rs
│       │   ├── stt.rs          # STT trait
│       │   ├── tts.rs          # TTS trait
│       │   ├── whisper.rs      # Whisper 实现
│       │   ├── openai_tts.rs   # OpenAI TTS
│       │   ├── edge_tts.rs     # Edge TTS
│       │   └── ali_speech.rs   # 阿里语音
│       ├── emotion/        # [新增] 情绪分析
│       │   ├── mod.rs
│       │   ├── analyzer.rs     # EmotionAnalyzer trait
│       │   ├── rule_based.rs   # 规则分析器
│       │   └── llm_based.rs    # LLM 分析器
│       ├── memory/         # [新增] 记忆系统
│       │   ├── mod.rs
│       │   └── vector_store.rs # 向量存储
│       ├── learning/       # [新增] 学习引擎
│       │   ├── mod.rs
│       │   └── engine.rs
│       ├── tools/          # [新增] 工具系统
│       │   ├── mod.rs
│       │   ├── registry.rs
│       │   ├── weather.rs
│       │   └── calculator.rs
│       ├── summary/        # [新增] 消息总结
│       │   ├── mod.rs
│       │   └── service.rs
│       ├── quota/          # [新增] 配额管理
│       │   └── mod.rs
│       ├── proactive/      # [新增] 主动消息
│       │   └── mod.rs
│       ├── mod.rs          # 模块导出
│       ├── error.rs        # AI 错误类型
│       ├── llm_client.rs   # LLM 客户端 trait
│       ├── embedding.rs    # 嵌入服务
│       ├── trigger.rs      # 触发评估器
│       ├── personality.rs  # 人格管理
│       └── property_tests.rs # 属性测试
├── error.rs                # 统一错误处理
├── response.rs             # 统一响应格式
└── main.rs
```

## 编码规范

### 1. Trait 定义规范

所有核心功能必须先定义 trait，再提供默认实现：

```rust
// ✅ 正确：先定义 trait
#[async_trait]
pub trait EmotionAnalyzer: Send + Sync {
    /// 分析文本情绪
    async fn analyze(&self, text: &str) -> EmotionAnalysis;
    
    /// 分析对话上下文情绪
    async fn analyze_context(&self, messages: &[ChatMessage]) -> EmotionState;
}

// ✅ 正确：提供默认实现
pub struct RuleBasedEmotionAnalyzer { /* ... */ }

impl EmotionAnalyzer for RuleBasedEmotionAnalyzer {
    // ...
}

// ❌ 错误：直接实现具体类型而不定义 trait
pub struct EmotionAnalyzer {
    pub fn analyze(&self, text: &str) -> EmotionAnalysis { /* ... */ }
}
```

### 2. 错误处理规范

#### 2.1 使用 thiserror 定义错误

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpeechError {
    #[error("STT transcription failed: {0}")]
    STTError(String),
    
    #[error("TTS synthesis failed: {0}")]
    TTSError(String),
    
    #[error("Audio format not supported: {0}")]
    UnsupportedFormat(String),
    
    #[error("Audio too long: {duration_secs}s (max: {max_secs}s)")]
    AudioTooLong { duration_secs: u64, max_secs: u64 },
}
```

#### 2.2 错误转换到 AppError

所有服务层错误必须实现 `From<ServiceError> for AppError`：

```rust
impl From<SpeechError> for AppError {
    fn from(err: SpeechError) -> Self {
        match err {
            SpeechError::STTError(msg) => AppError::AIServiceUnavailable(msg),
            SpeechError::TTSError(msg) => AppError::AIServiceUnavailable(msg),
            SpeechError::UnsupportedFormat(fmt) => AppError::BadRequest(format!("Unsupported audio format: {}", fmt)),
            SpeechError::AudioTooLong { duration_secs, max_secs } => {
                AppError::BadRequest(format!("Audio too long: {}s (max: {}s)", duration_secs, max_secs))
            }
        }
    }
}
```

#### 2.3 错误码规范

Phase 3 新增错误码范围：4007-4018

| 错误码 | 名称 | 描述 |
|--------|------|------|
| 4007 | ALL_PROVIDERS_FAILED | 所有服务商均失败 |
| 4008 | STT_ERROR | 语音识别失败 |
| 4009 | TTS_ERROR | 语音合成失败 |
| 4010 | QUOTA_EXCEEDED | 配额已用尽 |
| 4011 | TOOL_NOT_FOUND | 工具不存在 |
| 4012 | TOOL_EXECUTION_ERROR | 工具执行失败 |
| 4013 | MEMORY_ERROR | 记忆系统错误 |
| 4014 | SUMMARY_ERROR | 消息总结失败 |
| 4015 | EMOTION_ANALYSIS_ERROR | 情绪分析失败 |
| 4016 | LEARNING_ERROR | 学习系统错误 |
| 4017 | INVALID_PERSONA | 人设配置无效 |
| 4018 | STREAMING_INTERRUPTED | 流式响应中断 |

### 3. API 响应规范

#### 3.1 统一响应格式

```rust
// 成功响应
{
    "success": true,
    "data": { /* 业务数据 */ }
}

// 错误响应
{
    "success": false,
    "error": {
        "code": 4008,
        "message": "STT transcription failed",
        "details": { /* 可选详情 */ }
    }
}
```

#### 3.2 API Handler 规范

```rust
/// 生成消息总结
async fn generate_summary(
    State(service): State<SummarizeService>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<SummaryRequest>,
) -> Result<Json<ApiResponse<Summary>>, AppError> {
    // 1. 权限检查
    // 2. 参数验证
    // 3. 调用服务
    let summary = service.summarize(request).await?;
    
    // 4. 返回统一格式
    Ok(Json(ApiResponse::success(summary)))
}
```

### 4. 服务层规范

#### 4.1 Builder 模式配置

```rust
pub struct AIAgentService {
    pool: PgPool,
    llm_client: Option<Arc<dyn LLMClient>>,
    emotion_analyzer: Option<Arc<dyn EmotionAnalyzer>>,
    // ...
}

impl AIAgentService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            llm_client: None,
            emotion_analyzer: None,
        }
    }
    
    pub fn with_llm_client(mut self, client: Arc<dyn LLMClient>) -> Self {
        self.llm_client = Some(client);
        self
    }
    
    pub fn with_emotion_analyzer(mut self, analyzer: Arc<dyn EmotionAnalyzer>) -> Self {
        self.emotion_analyzer = Some(analyzer);
        self
    }
}
```

#### 4.2 依赖注入

```rust
// ✅ 正确：通过 trait 注入依赖
pub struct DefaultAgentCore {
    llm_client: Arc<dyn LLMClient>,
    memory_store: Arc<dyn MemoryStore>,
    emotion_analyzer: Arc<dyn EmotionAnalyzer>,
}

// ❌ 错误：直接依赖具体实现
pub struct DefaultAgentCore {
    llm_client: OpenAIClient,  // 不要这样做
}
```

### 5. 数据库规范

#### 5.1 Migration 命名

```
migrations/
├── 20251208_001_create_agent_emotion_history.sql
├── 20251208_002_create_learning_patterns.sql
├── 20251208_003_create_user_feedback.sql
├── 20251208_004_create_summary_cache.sql
└── 20251208_005_create_provider_usage.sql
```

#### 5.2 SQL 查询规范

```rust
// ✅ 正确：使用 sqlx::query_as! 宏
let agent = sqlx::query_as!(
    AIAgent,
    r#"
    SELECT id, group_id, name, toxicity_level, enthusiasm_level, 
           expertise_domains, speech_suffix, trigger_keywords, 
           is_active, created_at, updated_at
    FROM ai_agents 
    WHERE id = $1
    "#,
    agent_id
)
.fetch_optional(&self.pool)
.await?
.ok_or(AIAgentError::AgentNotFound(agent_id))?;

// ❌ 错误：使用字符串拼接
let query = format!("SELECT * FROM ai_agents WHERE id = '{}'", agent_id);
```

### 6. 测试规范

#### 6.1 属性测试 (Property-Based Testing)

使用 `proptest` 框架，每个正确性属性必须有对应的测试：

```rust
use proptest::prelude::*;

/// **Feature: phase3-ai-agent-evolution, Property 14: Trigger Probability Calculation**
/// **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5**
proptest! {
    #[test]
    fn test_trigger_probability_in_range(
        message in ".*",
        agent_name in "[a-zA-Z]{3,20}",
    ) {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent(&agent_name, vec![]);
        let context = create_test_context(agent, None);
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let decision = rt.block_on(evaluator.evaluate(&message, "user1", &context));
        
        // 属性：概率必须在 [0.0, 1.0] 范围内
        prop_assert!(decision.probability >= 0.0);
        prop_assert!(decision.probability <= 1.0);
    }
}

/// **Feature: phase3-ai-agent-evolution, Property 21: Provider Fallback Chain**
/// **Validates: Requirements 16.3**
proptest! {
    #[test]
    fn test_provider_fallback_chain(
        num_providers in 1usize..5,
        fail_indices in prop::collection::vec(any::<bool>(), 1..5),
    ) {
        // 测试 provider 失败时的 fallback 行为
    }
}
```

#### 6.2 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_state_transition() {
        let analyzer = RuleBasedEmotionAnalyzer::new();
        
        // 测试开心情绪检测
        let result = analyzer.analyze_sync("太棒了！我很高兴！");
        assert_eq!(result.primary_emotion, EmotionState::Happy);
        
        // 测试悲伤情绪检测
        let result = analyzer.analyze_sync("我很难过...");
        assert_eq!(result.primary_emotion, EmotionState::Empathetic);
    }

    #[tokio::test]
    async fn test_provider_manager_fallback() {
        let manager = ProviderManager::new(vec![
            mock_failing_provider(),
            mock_successful_provider(),
        ]);
        
        let result = manager.execute_with_fallback(|client| {
            Box::pin(async move { client.generate_response("test", "test").await })
        }).await;
        
        assert!(result.is_ok());
    }
}
```

### 7. 日志规范

```rust
use tracing::{debug, info, warn, error, instrument};

impl AIAgentService {
    #[instrument(skip(self), fields(agent_id = %agent.id))]
    pub async fn generate_response(
        &self,
        agent: &AIAgent,
        message: &str,
    ) -> Result<String, AIAgentError> {
        debug!("Generating response for message: {}", message);
        
        let response = self.llm_client
            .as_ref()
            .ok_or_else(|| {
                error!("LLM client not configured");
                AIAgentError::Internal("LLM client not configured".to_string())
            })?
            .generate_response(&system_prompt, message)
            .await?;
        
        info!(response_length = response.len(), "Response generated successfully");
        Ok(response)
    }
}
```

### 8. 国内服务商优先级规范

```rust
/// Provider 优先级配置
/// 数字越小优先级越高
pub const PROVIDER_PRIORITIES: &[(&str, u8)] = &[
    ("siliconflow", 1),   // 硅基流动 - 最高优先级
    ("zhipu", 2),         // 智谱 AI
    ("moonshot", 3),      // 月之暗面
    ("tongyi", 4),        // 通义千问
    ("wenxin", 5),        // 文心一言
    ("openai", 10),       // OpenAI - 国际备选
    ("anthropic", 11),    // Anthropic - 国际备选
];

impl ProviderManager {
    /// 创建带国内优先级的 Provider Manager
    pub fn with_domestic_priority(configs: Vec<ProviderConfig>) -> Self {
        let mut sorted = configs;
        sorted.sort_by_key(|c| c.priority);
        // 确保国内服务商在前
        Self { providers: sorted, /* ... */ }
    }
}
```

### 9. 中文支持规范

#### 9.1 情绪关键词

```rust
impl RuleBasedEmotionAnalyzer {
    fn default_chinese_keywords() -> EmotionKeywords {
        EmotionKeywords {
            happy: vec![
                "开心", "高兴", "太棒了", "哈哈", "😄", "🎉", "好耶",
                "太好了", "真棒", "厉害", "赞", "👍"
            ],
            sad: vec![
                "难过", "伤心", "😢", "唉", "哭了", "失望", "郁闷"
            ],
            angry: vec![
                "生气", "愤怒", "😠", "烦死了", "讨厌", "气死"
            ],
            // ...
        }
    }
}
```

#### 9.2 用户提示消息

```rust
impl AgentError {
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

## 代码审查清单

在提交代码前，请确保：

- [ ] 所有新功能都有对应的 trait 定义
- [ ] 错误类型已添加到 `AIAgentError` 或创建新的错误枚举
- [ ] 错误已正确转换到 `AppError`
- [ ] API 使用统一响应格式 `ApiResponse<T>`
- [ ] 数据库查询使用 `sqlx::query_as!` 宏
- [ ] 添加了属性测试验证正确性属性
- [ ] 添加了单元测试覆盖边界情况
- [ ] 使用 `tracing` 添加了适当的日志
- [ ] 中文用户提示消息已添加
- [ ] 国内服务商优先级已正确配置
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 代码通过 `cargo fmt` 格式化

## 参考文档

- [Requirements Document](.kiro/specs/phase3-ai-agent-evolution/requirements.md)
- [Design Document](.kiro/specs/phase3-ai-agent-evolution/design.md)
- [Tasks Document](.kiro/specs/phase3-ai-agent-evolution/tasks.md)
- [Error Codes Reference](archive/refactor/ERROR_CODES.md)
