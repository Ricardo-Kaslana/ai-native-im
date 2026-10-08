//! Unified Error Handling Module
//!
//! This module provides a centralized error handling system for the AI-Native-IM backend.
//! All API endpoints should use `AppError` as their error type to ensure consistent
//! error responses across the application.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Error code ranges:
/// - 1000-1999: General errors
/// - 2000-2999: Authentication errors
/// - 3000-3999: Business logic errors
/// - 4000-4999: AI service errors
pub mod error_codes {
    // 1000-1999: General errors
    pub const INTERNAL_ERROR: i32 = 1000;
    pub const VALIDATION_ERROR: i32 = 1001;
    pub const NOT_FOUND: i32 = 1002;
    pub const CONFLICT: i32 = 1003;
    pub const BAD_REQUEST: i32 = 1004;
    pub const TIMEOUT: i32 = 1005;

    // 2000-2999: Authentication errors
    pub const UNAUTHORIZED: i32 = 2000;
    pub const FORBIDDEN: i32 = 2001;
    pub const TOKEN_EXPIRED: i32 = 2002;
    pub const INVALID_CREDENTIALS: i32 = 2003;
    pub const ACCOUNT_LOCKED: i32 = 2004;
    pub const ACCOUNT_SUSPENDED: i32 = 2005;

    // 3000-3999: Business logic errors
    pub const USER_NOT_FOUND: i32 = 3000;
    pub const GROUP_NOT_FOUND: i32 = 3001;
    pub const MESSAGE_NOT_FOUND: i32 = 3002;
    pub const PERMISSION_DENIED: i32 = 3003;
    pub const ALREADY_EXISTS: i32 = 3004;
    pub const GROUP_FULL: i32 = 3005;
    pub const INVITE_EXPIRED: i32 = 3006;
    pub const MEDIA_NOT_FOUND: i32 = 3007;
    pub const MEDIA_TOO_LARGE: i32 = 3008;
    pub const UNSUPPORTED_MEDIA_TYPE: i32 = 3009;

    // 4000-4999: AI service errors
    pub const AI_SERVICE_UNAVAILABLE: i32 = 4000;
    pub const AI_RATE_LIMITED: i32 = 4001;
    pub const AI_TIMEOUT: i32 = 4002;
    pub const AI_INVALID_RESPONSE: i32 = 4003;
    pub const AI_AGENT_NOT_FOUND: i32 = 4004;
    pub const EMBEDDING_ERROR: i32 = 4005;
    pub const VECTOR_DB_ERROR: i32 = 4006;
}

/// Unified application error type
#[derive(Debug, Error)]
pub enum AppError {
    // General errors (1000-1999)
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Request timeout")]
    Timeout,

    // Authentication errors (2000-2999)
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account locked")]
    AccountLocked,

    #[error("Account suspended")]
    AccountSuspended,

    // Business logic errors (3000-3999)
    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("Group is full")]
    GroupFull,

    #[error("Invite code expired or invalid")]
    InviteExpired,

    #[error("Media not found: {0}")]
    MediaNotFound(String),

    #[error("Media file too large: {size} bytes (max: {max} bytes)")]
    MediaTooLarge { size: u64, max: u64 },

    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    // AI service errors (4000-4999)
    #[error("AI service unavailable: {0}")]
    AIServiceUnavailable(String),

    #[error("AI service rate limited")]
    AIRateLimited,

    #[error("AI service timeout")]
    AITimeout,

    #[error("Invalid AI response: {0}")]
    AIInvalidResponse(String),

    #[error("AI agent not found: {0}")]
    AIAgentNotFound(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),

    #[error("Vector database error: {0}")]
    VectorDBError(String),

    // Database error (maps to internal)
    #[error("Database error: {0}")]
    Database(String),
}

impl AppError {
    /// Get the error code for this error
    pub fn error_code(&self) -> i32 {
        match self {
            // General errors
            AppError::Internal(_) => error_codes::INTERNAL_ERROR,
            AppError::Validation(_) => error_codes::VALIDATION_ERROR,
            AppError::NotFound(_) => error_codes::NOT_FOUND,
            AppError::Conflict(_) => error_codes::CONFLICT,
            AppError::BadRequest(_) => error_codes::BAD_REQUEST,
            AppError::Timeout => error_codes::TIMEOUT,

            // Authentication errors
            AppError::Unauthorized => error_codes::UNAUTHORIZED,
            AppError::Forbidden(_) => error_codes::FORBIDDEN,
            AppError::TokenExpired => error_codes::TOKEN_EXPIRED,
            AppError::InvalidCredentials => error_codes::INVALID_CREDENTIALS,
            AppError::AccountLocked => error_codes::ACCOUNT_LOCKED,
            AppError::AccountSuspended => error_codes::ACCOUNT_SUSPENDED,

            // Business logic errors
            AppError::UserNotFound(_) => error_codes::USER_NOT_FOUND,
            AppError::GroupNotFound(_) => error_codes::GROUP_NOT_FOUND,
            AppError::MessageNotFound(_) => error_codes::MESSAGE_NOT_FOUND,
            AppError::PermissionDenied(_) => error_codes::PERMISSION_DENIED,
            AppError::AlreadyExists(_) => error_codes::ALREADY_EXISTS,
            AppError::GroupFull => error_codes::GROUP_FULL,
            AppError::InviteExpired => error_codes::INVITE_EXPIRED,
            AppError::MediaNotFound(_) => error_codes::MEDIA_NOT_FOUND,
            AppError::MediaTooLarge { .. } => error_codes::MEDIA_TOO_LARGE,
            AppError::UnsupportedMediaType(_) => error_codes::UNSUPPORTED_MEDIA_TYPE,

            // AI service errors
            AppError::AIServiceUnavailable(_) => error_codes::AI_SERVICE_UNAVAILABLE,
            AppError::AIRateLimited => error_codes::AI_RATE_LIMITED,
            AppError::AITimeout => error_codes::AI_TIMEOUT,
            AppError::AIInvalidResponse(_) => error_codes::AI_INVALID_RESPONSE,
            AppError::AIAgentNotFound(_) => error_codes::AI_AGENT_NOT_FOUND,
            AppError::EmbeddingError(_) => error_codes::EMBEDDING_ERROR,
            AppError::VectorDBError(_) => error_codes::VECTOR_DB_ERROR,

            // Database error maps to internal
            AppError::Database(_) => error_codes::INTERNAL_ERROR,
        }
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 500 Internal Server Error
            AppError::Internal(_) | AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,

            // 400 Bad Request
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,

            // 404 Not Found
            AppError::NotFound(_)
            | AppError::UserNotFound(_)
            | AppError::GroupNotFound(_)
            | AppError::MessageNotFound(_)
            | AppError::MediaNotFound(_)
            | AppError::AIAgentNotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            AppError::Conflict(_) | AppError::AlreadyExists(_) => StatusCode::CONFLICT,

            // 408 Request Timeout
            AppError::Timeout => StatusCode::REQUEST_TIMEOUT,

            // 401 Unauthorized
            AppError::Unauthorized
            | AppError::TokenExpired
            | AppError::InvalidCredentials
            | AppError::AccountLocked
            | AppError::AccountSuspended => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            AppError::Forbidden(_) | AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,

            // 400 Bad Request (business logic)
            AppError::GroupFull
            | AppError::InviteExpired
            | AppError::MediaTooLarge { .. }
            | AppError::UnsupportedMediaType(_) => StatusCode::BAD_REQUEST,

            // 503 Service Unavailable (AI errors)
            AppError::AIServiceUnavailable(_)
            | AppError::AIRateLimited
            | AppError::AITimeout
            | AppError::AIInvalidResponse(_)
            | AppError::EmbeddingError(_)
            | AppError::VectorDBError(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Check if this error is transient and can be retried
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            AppError::Timeout
                | AppError::AIRateLimited
                | AppError::AITimeout
                | AppError::AIServiceUnavailable(_)
        )
    }
}

/// Error response structure for JSON serialization
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
}

/// Error detail structure
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            success: false,
            error: ErrorDetail {
                code: self.error_code(),
                message: self.to_string(),
                details: None,
            },
        };

        (status, Json(error_response)).into_response()
    }
}

// Type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_ranges() {
        // General errors: 1000-1999
        assert!(AppError::Internal("test".into()).error_code() >= 1000);
        assert!(AppError::Internal("test".into()).error_code() < 2000);
        assert!(AppError::Validation("test".into()).error_code() >= 1000);
        assert!(AppError::Validation("test".into()).error_code() < 2000);

        // Auth errors: 2000-2999
        assert!(AppError::Unauthorized.error_code() >= 2000);
        assert!(AppError::Unauthorized.error_code() < 3000);
        assert!(AppError::TokenExpired.error_code() >= 2000);
        assert!(AppError::TokenExpired.error_code() < 3000);

        // Business errors: 3000-3999
        assert!(AppError::UserNotFound("test".into()).error_code() >= 3000);
        assert!(AppError::UserNotFound("test".into()).error_code() < 4000);
        assert!(AppError::GroupFull.error_code() >= 3000);
        assert!(AppError::GroupFull.error_code() < 4000);

        // AI errors: 4000-4999
        assert!(AppError::AIServiceUnavailable("test".into()).error_code() >= 4000);
        assert!(AppError::AIServiceUnavailable("test".into()).error_code() < 5000);
        assert!(AppError::AIRateLimited.error_code() >= 4000);
        assert!(AppError::AIRateLimited.error_code() < 5000);
    }

    #[test]
    fn test_status_code_mapping() {
        assert_eq!(
            AppError::Internal("test".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Validation("test".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::NotFound("test".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            AppError::Forbidden("test".into()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            AppError::AIServiceUnavailable("test".into()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn test_is_transient() {
        assert!(AppError::Timeout.is_transient());
        assert!(AppError::AIRateLimited.is_transient());
        assert!(AppError::AITimeout.is_transient());
        assert!(!AppError::Unauthorized.is_transient());
        assert!(!AppError::Validation("test".into()).is_transient());
    }
}


// ============================================================================
// From trait implementations for error conversion
// ============================================================================

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violations
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        // PostgreSQL unique violation
                        return AppError::Conflict("Resource already exists".to_string());
                    }
                }
                AppError::Database(err.to_string())
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}

impl From<crate::services::ai::AIAgentError> for AppError {
    fn from(err: crate::services::ai::AIAgentError) -> Self {
        use crate::services::ai::AIAgentError;
        match err {
            AIAgentError::AgentNotFound(id) => AppError::AIAgentNotFound(id.to_string()),
            AIAgentError::AgentAlreadyExists => {
                AppError::AlreadyExists("AI Agent already exists for this group".to_string())
            }
            AIAgentError::InvalidPersonality(msg) => AppError::Validation(msg),
            AIAgentError::InvalidAgentName => {
                AppError::Validation("Agent name must be between 1 and 100 characters".to_string())
            }
            AIAgentError::InvalidToxicityLevel => {
                AppError::Validation("Toxicity level must be between 0 and 10".to_string())
            }
            AIAgentError::InvalidEnthusiasmLevel => {
                AppError::Validation("Enthusiasm level must be between 0 and 10".to_string())
            }
            AIAgentError::LLMError(msg) => AppError::AIServiceUnavailable(msg),
            AIAgentError::LLMTimeout(_) => AppError::AITimeout,
            AIAgentError::LLMRateLimited(_) => AppError::AIRateLimited,
            AIAgentError::InvalidLLMResponse(msg) => AppError::AIInvalidResponse(msg),
            AIAgentError::VectorDBError(msg) => AppError::VectorDBError(msg),
            AIAgentError::EmbeddingError(msg) => AppError::EmbeddingError(msg),
            AIAgentError::Database(err) => AppError::Database(err.to_string()),
            AIAgentError::Redis(msg) => AppError::Internal(format!("Redis error: {}", msg)),
            AIAgentError::Serialization(msg) => {
                AppError::Internal(format!("Serialization error: {}", msg))
            }
            AIAgentError::Internal(msg) => AppError::Internal(msg),
            // Phase 3 新增错误类型
            AIAgentError::AllProvidersFailed => {
                AppError::AIServiceUnavailable("All AI providers failed".to_string())
            }
            AIAgentError::STTError(msg) => {
                AppError::AIServiceUnavailable(format!("Speech-to-text error: {}", msg))
            }
            AIAgentError::TTSError(msg) => {
                AppError::AIServiceUnavailable(format!("Text-to-speech error: {}", msg))
            }
            AIAgentError::QuotaExceeded(msg) => AppError::AIRateLimited,
            AIAgentError::ToolNotFound(name) => {
                AppError::NotFound(format!("Tool not found: {}", name))
            }
            AIAgentError::ToolExecutionError(msg) => {
                AppError::Internal(format!("Tool execution error: {}", msg))
            }
            AIAgentError::MemoryError(msg) => {
                AppError::Internal(format!("Memory system error: {}", msg))
            }
            AIAgentError::SummaryError(msg) => {
                AppError::Internal(format!("Summary error: {}", msg))
            }
            AIAgentError::EmotionAnalysisError(msg) => {
                AppError::Internal(format!("Emotion analysis error: {}", msg))
            }
            AIAgentError::LearningError(msg) => {
                AppError::Internal(format!("Learning system error: {}", msg))
            }
            AIAgentError::InvalidPersona(msg) => AppError::Validation(msg),
            AIAgentError::StreamingInterrupted => {
                AppError::Internal("Streaming response interrupted".to_string())
            }
        }
    }
}

impl From<crate::services::group::GroupError> for AppError {
    fn from(err: crate::services::group::GroupError) -> Self {
        use crate::services::group::GroupError;
        match err {
            GroupError::GroupNotFound(id) => AppError::GroupNotFound(id.to_string()),
            GroupError::InvalidGroupName => {
                AppError::Validation("Group name must be between 1 and 50 characters".to_string())
            }
            GroupError::InvalidDescription => {
                AppError::Validation("Group description must not exceed 500 characters".to_string())
            }
            GroupError::InvalidAvatarUrl => AppError::Validation("Invalid avatar URL".to_string()),
            GroupError::GroupFull(_) => AppError::GroupFull,
            GroupError::AlreadyMember(id) => {
                AppError::Conflict(format!("User {} is already a member", id))
            }
            GroupError::NotMember(id) => {
                AppError::Forbidden(format!("User {} is not a member of this group", id))
            }
            GroupError::UserBanned(id) => {
                AppError::Forbidden(format!("User {} is banned from this group", id))
            }
            GroupError::PermissionDenied(msg) => AppError::PermissionDenied(msg),
            GroupError::CannotRemoveOwner => {
                AppError::Forbidden("Cannot remove group owner".to_string())
            }
            GroupError::CannotRemoveAdmin => {
                AppError::Forbidden("Admins cannot remove other admins".to_string())
            }
            GroupError::OwnerOnly => {
                AppError::Forbidden("Only the group owner can perform this action".to_string())
            }
            GroupError::GroupDissolved => AppError::NotFound("Group has been dissolved".to_string()),
            GroupError::InvalidInviteCode => AppError::InviteExpired,
            GroupError::InviteCodeExpired => AppError::InviteExpired,
            GroupError::InviteCodeMaxUsesReached => AppError::InviteExpired,
            GroupError::SearchQueryTooShort => {
                AppError::Validation("Search query must be at least 2 characters".to_string())
            }
            GroupError::AlreadyMuted(id) => {
                AppError::Conflict(format!("User {} is already muted", id))
            }
            GroupError::NotMuted(id) => {
                AppError::BadRequest(format!("User {} is not muted", id))
            }
            GroupError::NotBanned(id) => {
                AppError::BadRequest(format!("User {} is not banned", id))
            }
            GroupError::Database(err) => AppError::Database(err.to_string()),
            GroupError::Redis(msg) => AppError::Internal(format!("Redis error: {}", msg)),
            GroupError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl From<crate::services::media::MediaError> for AppError {
    fn from(err: crate::services::media::MediaError) -> Self {
        use crate::services::media::MediaError;
        match err {
            MediaError::FileSizeTooLarge(size, max) => AppError::MediaTooLarge { size, max },
            MediaError::UnsupportedMimeType(mime) => AppError::UnsupportedMediaType(mime),
            MediaError::InvalidFormat(msg) => AppError::Validation(msg),
            MediaError::FileNotFound(id) => AppError::MediaNotFound(id),
            MediaError::ProcessingFailed(msg) => AppError::Internal(msg),
            MediaError::StorageError(msg) => AppError::Internal(msg),
            MediaError::InvalidUploadToken => {
                AppError::Unauthorized
            }
            MediaError::MalwareDetected(msg) => {
                AppError::Forbidden(format!("Malware detected: {}", msg))
            }
            MediaError::ScanFailed(msg) => AppError::Internal(msg),
            MediaError::ScannerUnavailable => {
                AppError::Internal("Malware scanner unavailable".to_string())
            }
            MediaError::Unauthorized => AppError::Unauthorized,
            MediaError::FileStillReferenced => {
                AppError::Conflict("File is still referenced by messages".to_string())
            }
            MediaError::DatabaseError(err) => AppError::Database(err.to_string()),
            MediaError::RedisError(err) => AppError::Internal(format!("Redis error: {}", err)),
            MediaError::ImageError(err) => AppError::Internal(format!("Image error: {}", err)),
            MediaError::IoError(err) => AppError::Internal(format!("IO error: {}", err)),
            MediaError::InternalError(msg) => AppError::Internal(msg),
        }
    }
}

impl From<crate::services::auth::AuthError> for AppError {
    fn from(err: crate::services::auth::AuthError) -> Self {
        use crate::services::auth::AuthError;
        match err {
            AuthError::InvalidCredentials => AppError::InvalidCredentials,
            AuthError::AccountLocked => AppError::AccountLocked,
            AuthError::UserNotFound => AppError::UserNotFound("User not found".to_string()),
            AuthError::UsernameExists => AppError::AlreadyExists("Username already exists".to_string()),
            AuthError::EmailExists => AppError::AlreadyExists("Email already exists".to_string()),
            AuthError::InvalidPassword => AppError::Validation("Invalid password format".to_string()),
            AuthError::IncorrectCurrentPassword => AppError::Validation("Current password is incorrect".to_string()),
            AuthError::UserError(user_err) => {
                use crate::services::auth::UserError;
                match user_err {
                    UserError::UsernameExists => AppError::AlreadyExists("Username already exists".to_string()),
                    UserError::EmailExists => AppError::AlreadyExists("Email already exists".to_string()),
                    UserError::NotFound => AppError::UserNotFound("User not found".to_string()),
                    UserError::InvalidUsername => AppError::Validation("Invalid username".to_string()),
                    UserError::InvalidAvatarUrl => AppError::Validation("Invalid avatar URL".to_string()),
                    UserError::BioTooLong => AppError::Validation("Bio too long".to_string()),
                    _ => AppError::Internal(format!("User error: {}", user_err)),
                }
            }
            AuthError::RateLimiterError(_) => AppError::Forbidden("Too many requests, please try again later".to_string()),
            AuthError::AuditError(err) => AppError::Internal(format!("Audit error: {}", err)),
            AuthError::EmailError(err) => AppError::Internal(format!("Email error: {}", err)),
            AuthError::PasswordError(msg) => AppError::Validation(msg),
            AuthError::TokenError(msg) => AppError::Unauthorized,
        }
    }
}
