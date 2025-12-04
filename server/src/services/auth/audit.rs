use serde_json::Value;
use sqlx::PgPool;
use std::net::IpAddr;
use thiserror::Error;
use uuid::Uuid;

/// Audit service errors
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Security event types
#[derive(Debug, Clone)]
pub enum SecurityEvent {
    LoginSuccess,
    LoginFailed,
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,
    AccountLocked,
    Logout,
    TokenRefreshed,
}

impl SecurityEvent {
    fn as_str(&self) -> &'static str {
        match self {
            SecurityEvent::LoginSuccess => "login_success",
            SecurityEvent::LoginFailed => "login_failed",
            SecurityEvent::PasswordChanged => "password_changed",
            SecurityEvent::PasswordResetRequested => "password_reset_requested",
            SecurityEvent::PasswordResetCompleted => "password_reset_completed",
            SecurityEvent::AccountLocked => "account_locked",
            SecurityEvent::Logout => "logout",
            SecurityEvent::TokenRefreshed => "token_refreshed",
        }
    }
}

/// Audit service for security event logging
#[derive(Clone)]
pub struct AuditService {
    pool: PgPool,
}

impl AuditService {
    /// Create a new audit service
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Log a security event
    pub async fn log_event(
        &self,
        user_id: Option<Uuid>,
        username: Option<&str>,
        event: SecurityEvent,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
        details: Option<Value>,
    ) -> Result<(), AuditError> {
        sqlx::query(
            r#"
            INSERT INTO security_audit_log (user_id, username, event_type, ip_address, user_agent, details)
            VALUES ($1, $2, $3, $4::text::inet, $5, $6)
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind(event.as_str())
        .bind(ip_address.map(|ip| ip.to_string()))
        .bind(user_agent)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Log successful login
    pub async fn log_login_success(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "method": "password"
        });

        self.log_event(
            Some(user_id),
            Some(username),
            SecurityEvent::LoginSuccess,
            ip_address,
            user_agent,
            Some(details),
        )
        .await
    }

    /// Log failed login
    pub async fn log_login_failed(
        &self,
        username: &str,
        reason: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "reason": reason
        });

        self.log_event(
            None,
            Some(username),
            SecurityEvent::LoginFailed,
            ip_address,
            user_agent,
            Some(details),
        )
        .await
    }

    /// Log password change
    pub async fn log_password_changed(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        self.log_event(
            Some(user_id),
            Some(username),
            SecurityEvent::PasswordChanged,
            ip_address,
            user_agent,
            None,
        )
        .await
    }

    /// Log password reset request
    pub async fn log_password_reset_requested(
        &self,
        user_id: Uuid,
        username: &str,
        email: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "email": email
        });

        self.log_event(
            Some(user_id),
            Some(username),
            SecurityEvent::PasswordResetRequested,
            ip_address,
            user_agent,
            Some(details),
        )
        .await
    }

    /// Log account locked
    pub async fn log_account_locked(
        &self,
        username: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "reason": "too_many_failed_attempts"
        });

        self.log_event(
            None,
            Some(username),
            SecurityEvent::AccountLocked,
            ip_address,
            user_agent,
            Some(details),
        )
        .await
    }

    /// Log logout
    pub async fn log_logout(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        self.log_event(
            Some(user_id),
            Some(username),
            SecurityEvent::Logout,
            ip_address,
            user_agent,
            None,
        )
        .await
    }

    /// Log token refresh
    pub async fn log_token_refreshed(
        &self,
        user_id: Uuid,
        username: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AuditError> {
        self.log_event(
            Some(user_id),
            Some(username),
            SecurityEvent::TokenRefreshed,
            ip_address,
            user_agent,
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_as_str() {
        assert_eq!(SecurityEvent::LoginSuccess.as_str(), "login_success");
        assert_eq!(SecurityEvent::LoginFailed.as_str(), "login_failed");
        assert_eq!(
            SecurityEvent::PasswordChanged.as_str(),
            "password_changed"
        );
        assert_eq!(SecurityEvent::Logout.as_str(), "logout");
    }
}
