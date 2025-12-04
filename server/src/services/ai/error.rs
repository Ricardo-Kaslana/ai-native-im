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
}

impl AIAgentError {
    pub fn status_code(&self) -> u16 {
        match self {
            AIAgentError::AgentNotFound(_) => 404,
            AIAgentError::AgentAlreadyExists
            | AIAgentError::InvalidPersonality(_)
            | AIAgentError::InvalidAgentName
            | AIAgentError::InvalidToxicityLevel
            | AIAgentError::InvalidEnthusiasmLevel => 400,
            AIAgentError::LLMError(_)
            | AIAgentError::LLMTimeout(_)
            | AIAgentError::LLMRateLimited(_)
            | AIAgentError::InvalidLLMResponse(_)
            | AIAgentError::VectorDBError(_)
            | AIAgentError::EmbeddingError(_)
            | AIAgentError::Database(_)
            | AIAgentError::Redis(_)
            | AIAgentError::Serialization(_)
            | AIAgentError::Internal(_) => 500,
        }
    }

    /// Check if error is transient and can be retried
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            AIAgentError::LLMTimeout(_)
                | AIAgentError::LLMRateLimited(_)
                | AIAgentError::VectorDBError(_)
                | AIAgentError::Redis(_)
        )
    }
}
