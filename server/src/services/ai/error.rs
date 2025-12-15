use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AIAgentError {
    #[error("AI Agent not found: {0}")]
    AgentNotFound(Uuid),

    #[error("Group already has an AI Agent")]
    AgentAlreadyExists,

    #[error("Invalid personality parameter: {0}")]
    InvalidPersonality(String),

    #[error("Agent name must be between 1 and 100 characters")]
    InvalidAgentName,

    #[error("Toxicity level must be between 0 and 10")]
    InvalidToxicityLevel,

    #[error("Enthusiasm level must be between 0 and 10")]
    InvalidEnthusiasmLevel,

    #[error("LLM API error: {0}")]
    LLMError(String),

    #[error("LLM API timeout after {0} seconds")]
    LLMTimeout(u64),

    #[error("LLM API rate limited, retry after {0} seconds")]
    LLMRateLimited(u64),

    #[error("Invalid LLM response: {0}")]
    InvalidLLMResponse(String),

    #[error("Vector database error: {0}")]
    VectorDBError(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),

    // Phase 3 新增错误类型
    #[error("All LLM providers failed")]
    AllProvidersFailed,

    #[error("STT transcription failed: {0}")]
    STTError(String),

    #[error("TTS synthesis failed: {0}")]
    TTSError(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionError(String),

    #[error("Memory system error: {0}")]
    MemoryError(String),

    #[error("Summary generation failed: {0}")]
    SummaryError(String),

    #[error("Emotion analysis failed: {0}")]
    EmotionAnalysisError(String),

    #[error("Learning system error: {0}")]
    LearningError(String),

    #[error("Invalid persona configuration: {0}")]
    InvalidPersona(String),

    #[error("Streaming response interrupted")]
    StreamingInterrupted,
}

impl AIAgentError {
    pub fn status_code(&self) -> u16 {
        match self {
            AIAgentError::AgentNotFound(_) => 404,
            AIAgentError::AgentAlreadyExists
            | AIAgentError::InvalidPersonality(_)
            | AIAgentError::InvalidAgentName
            | AIAgentError::InvalidToxicityLevel
            | AIAgentError::InvalidEnthusiasmLevel
            | AIAgentError::InvalidPersona(_) => 400,
            AIAgentError::QuotaExceeded(_) => 429,
            AIAgentError::ToolNotFound(_) => 404,
            AIAgentError::LLMError(_)
            | AIAgentError::LLMTimeout(_)
            | AIAgentError::LLMRateLimited(_)
            | AIAgentError::InvalidLLMResponse(_)
            | AIAgentError::VectorDBError(_)
            | AIAgentError::EmbeddingError(_)
            | AIAgentError::Database(_)
            | AIAgentError::Redis(_)
            | AIAgentError::Serialization(_)
            | AIAgentError::Internal(_)
            | AIAgentError::AllProvidersFailed
            | AIAgentError::STTError(_)
            | AIAgentError::TTSError(_)
            | AIAgentError::ToolExecutionError(_)
            | AIAgentError::MemoryError(_)
            | AIAgentError::SummaryError(_)
            | AIAgentError::EmotionAnalysisError(_)
            | AIAgentError::LearningError(_)
            | AIAgentError::StreamingInterrupted => 500,
        }
    }

    /// Check if error is transient and can be retried
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            AIAgentError::LLMTimeout(_)
                | AIAgentError::LLMRateLimited(_)
                | AIAgentError::LLMError(_)
                | AIAgentError::VectorDBError(_)
                | AIAgentError::Redis(_)
                | AIAgentError::AllProvidersFailed
                | AIAgentError::StreamingInterrupted
        )
    }

    /// Get user-friendly error message in Chinese
    /// 获取用户友好的中文错误消息
    pub fn user_message(&self) -> String {
        match self {
            AIAgentError::QuotaExceeded(_) => {
                "您今日的 AI 对话次数已用完，明天再来吧！".to_string()
            }
            AIAgentError::AllProvidersFailed => {
                "AI 服务暂时不可用，请稍后再试。".to_string()
            }
            AIAgentError::STTError(_) => {
                "语音识别失败，请重新发送或使用文字。".to_string()
            }
            AIAgentError::TTSError(_) => {
                "语音合成失败，已为您显示文字回复。".to_string()
            }
            AIAgentError::LLMTimeout(_) => {
                "AI 响应超时，请稍后再试。".to_string()
            }
            AIAgentError::LLMRateLimited(_) => {
                "请求太频繁，请稍后再试。".to_string()
            }
            _ => "处理您的请求时出现问题，请稍后再试。".to_string()
        }
    }
}
