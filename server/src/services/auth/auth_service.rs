use crate::models::user::{CreateUser, User};
use crate::services::auth::{
    AuditError, AuditService, PasswordService, RateLimiter, RateLimiterError, TokenPair,
    TokenService, UserError, UserService,
};
use crate::services::email::{EmailError, EmailService, EmailTemplate};
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Auth service errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account is locked")]
    AccountLocked,
    #[error("User not found")]
    UserNotFound,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Email already exists")]
    EmailExists,
    #[error("Invalid password format")]
    InvalidPassword,
    #[error("Current password is incorrect")]
    IncorrectCurrentPassword,
    #[error("User error: {0}")]
    UserError(#[from] UserError),
    #[error("Rate limiter error: {0}")]
    RateLimiterError(#[from] RateLimiterError),
    #[error("Audit error: {0}")]
    AuditError(#[from] AuditError),
    #[error("Email error: {0}")]
    EmailError(#[from] EmailError),
    #[error("Password error: {0}")]
    PasswordError(String),
    #[error("Token error: {0}")]
    TokenError(String),
}

/// Registration data
#[derive(Debug, Clone)]
pub struct RegisterData {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Login data
#[derive(Debug, Clone)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

/// Password change data
#[derive(Debug, Clone)]
pub struct ChangePasswordData {
    pub current_password: String,
    pub new_password: String,
}

/// Auth service for handling authentication operations
#[derive(Clone)]
pub struct AuthService {
    pub(crate) user_service: UserService,
    password_service: PasswordService,
    token_service: TokenService,
    pub(crate) rate_limiter: RateLimiter,
    audit_service: AuditService,
    email_service: Arc<dyn EmailService>,
}

impl AuthService {
    /// Create a new auth service
    pub fn new(
        pool: PgPool,
        redis: redis::aio::ConnectionManager,
        jwt_secret: String,
        email_service: Arc<dyn EmailService>,
    ) -> Self {
        Self {
            user_service: UserService::with_redis(pool.clone(), redis.clone()),
            password_service: PasswordService::new(),
            token_service: TokenService::with_redis(jwt_secret, redis.clone()),
            rate_limiter: RateLimiter::with_defaults(redis),
            audit_service: AuditService::new(pool),
            email_service,
        }
    }

    /// Create a new auth service with custom bcrypt cost (for testing)
    pub fn with_bcrypt_cost(
        pool: PgPool,
        redis: redis::aio::ConnectionManager,
        jwt_secret: String,
        email_service: Arc<dyn EmailService>,
        bcrypt_cost: u32,
    ) -> Self {
        Self {
            user_service: UserService::with_redis(pool.clone(), redis.clone()),
            password_service: PasswordService::with_cost(bcrypt_cost),
            token_service: TokenService::with_redis(jwt_secret, redis.clone()),
            rate_limiter: RateLimiter::with_defaults(redis),
            audit_service: AuditService::new(pool),
            email_service,
        }
    }

    /// Get reference to the database pool (for testing)
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn pool(&self) -> &PgPool {
        &self.user_service.pool
    }

    /// Get reference to user service (for testing)
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn user_service(&self) -> &UserService {
        &self.user_service
    }

    /// Get reference to rate limiter (for testing)
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn rate_limiter(&mut self) -> &mut RateLimiter {
        &mut self.rate_limiter
    }

    /// Get reference to token service (for testing)
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn token_service(&self) -> &TokenService {
        &self.token_service
    }

    /// Register a new user
    pub async fn register(&self, data: RegisterData) -> Result<(User, TokenPair), AuthError> {
        let ip_address = None;
        let user_agent = None;
        // Validate password strength
        self.password_service
            .validate_strength(&data.password)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        // Hash password
        let password_hash = self
            .password_service
            .hash_password(&data.password)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        // Create user
        let create_data = CreateUser {
            username: data.username.clone(),
            email: data.email,
            password_hash,
        };

        let user = self.user_service.create_user(create_data).await?;

        // Generate token pair
        let tokens = self
            .token_service
            .generate_token_pair(&user.id, &user.username)
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        // Log registration (treated as login success)
        self.audit_service
            .log_login_success(user.id, &user.username, ip_address, user_agent)
            .await?;

        // Send welcome email (don't fail registration if email fails)
        let welcome_template = EmailTemplate::Welcome {
            username: user.username.clone(),
        };
        
        if let Err(e) = self.email_service.send_template(&user.email, welcome_template).await {
            tracing::warn!("Failed to send welcome email: {}", e);
        }

        Ok((user, tokens))
    }

    /// Login a user
    pub async fn login(&mut self, data: LoginData) -> Result<(User, TokenPair), AuthError> {
        let ip_address = None;
        let user_agent = None;
        // Check if account is locked
        if self.rate_limiter.is_locked(&data.username).await? {
            // Log account locked
            self.audit_service
                .log_account_locked(&data.username, ip_address, user_agent)
                .await?;
            return Err(AuthError::AccountLocked);
        }

        // Get user by username
        let user = self
            .user_service
            .get_user_by_username(&data.username)
            .await
            .map_err(|_| {
                // Don't reveal if user exists or not
                AuthError::InvalidCredentials
            })?;

        // Verify password
        let password_valid = self
            .password_service
            .verify_password(&data.password, &user.password_hash)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        if !password_valid {
            // Record failed attempt
            self.rate_limiter
                .record_failed_attempt(&data.username)
                .await?;

            // Log failed login
            self.audit_service
                .log_login_failed(&data.username, "invalid_password", ip_address, user_agent)
                .await?;

            return Err(AuthError::InvalidCredentials);
        }

        // Reset failed attempts on successful login
        self.rate_limiter.reset_attempts(&data.username).await?;

        // Update online status
        self.user_service
            .update_online_status(&user.id, crate::models::user::OnlineStatus::Online)
            .await?;

        // Generate token pair
        let tokens = self
            .token_service
            .generate_token_pair(&user.id, &user.username)
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        // Log successful login
        self.audit_service
            .log_login_success(user.id, &user.username, ip_address, user_agent)
            .await?;

        Ok((user, tokens))
    }

    /// Logout a user
    pub async fn logout(&self, user_id: &Uuid, token: Option<&str>) -> Result<(), AuthError> {
        let ip_address = None;
        let user_agent = None;
        // Get user for logging
        let user = self.user_service.get_user_by_id(user_id).await?;

        // Update online status to offline
        self.user_service
            .update_online_status(user_id, crate::models::user::OnlineStatus::Offline)
            .await?;

        // Add token to blacklist if provided
        if let Some(token) = token {
            self.token_service
                .blacklist_token(token)
                .await
                .map_err(|e| AuthError::TokenError(e.to_string()))?;
        }

        // Log logout
        self.audit_service
            .log_logout(user.id, &user.username, ip_address, user_agent)
            .await?;

        Ok(())
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: &Uuid,
        data: ChangePasswordData,
    ) -> Result<(), AuthError> {
        let ip_address = None;
        let user_agent = None;
        // Get user
        let user = self.user_service.get_user_by_id(user_id).await?;

        // Verify current password
        let current_valid = self
            .password_service
            .verify_password(&data.current_password, &user.password_hash)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        if !current_valid {
            return Err(AuthError::IncorrectCurrentPassword);
        }

        // Validate new password strength
        self.password_service
            .validate_strength(&data.new_password)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        // Hash new password
        let new_password_hash = self
            .password_service
            .hash_password(&data.new_password)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        // Update password in database
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_password_hash)
            .bind(user_id)
            .execute(&self.user_service.pool)
            .await
            .map_err(|e| AuthError::UserError(UserError::DatabaseError(e)))?;

        // Log password change
        self.audit_service
            .log_password_changed(user.id, &user.username, ip_address, user_agent)
            .await?;

        // Send password changed notification email (don't fail password change if email fails)
        let notification_template = EmailTemplate::PasswordChanged {
            username: user.username.clone(),
        };
        
        if let Err(e) = self.email_service.send_template(&user.email, notification_template).await {
            tracing::warn!("Failed to send password change notification: {}", e);
        }

        // Invalidate all existing tokens
        self.token_service
            .blacklist_user_tokens(user_id)
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        Ok(())
    }

    /// Request password reset
    pub async fn request_password_reset(&self, email: &str) -> Result<String, AuthError> {
        let ip_address = None;
        let user_agent = None;
        // Get user by email
        let user = self
            .user_service
            .get_user_by_email(email)
            .await
            .map_err(|_| AuthError::UserNotFound)?;

        // Generate reset token
        let reset_token = self.password_service.generate_reset_token();

        // Store reset token in database (expires in 1 hour)
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (user_id, token, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&user.id)
        .bind(&reset_token)
        .bind(expires_at)
        .execute(&self.user_service.pool)
        .await
        .map_err(|e| AuthError::UserError(UserError::DatabaseError(e)))?;

        // Log password reset request
        self.audit_service
            .log_password_reset_requested(user.id, &user.username, email, ip_address, user_agent)
            .await?;

        // Send password reset email
        let reset_url = format!("https://example.com/reset-password?token={}", reset_token);
        let email_template = EmailTemplate::PasswordReset {
            username: user.username.clone(),
            reset_token: reset_token.clone(),
            reset_url,
        };
        
        self.email_service
            .send_template(email, email_template)
            .await?;

        Ok(reset_token)
    }

    /// Complete password reset
    pub async fn complete_password_reset(
        &self,
        token: &str,
        new_password: &str,
    ) -> Result<(), AuthError> {
        // Validate new password strength
        self.password_service
            .validate_strength(new_password)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        // Find valid reset token
        let reset_token: Option<(Uuid, Uuid)> = sqlx::query_as(
            r#"
            SELECT id, user_id
            FROM password_reset_tokens
            WHERE token = $1
            AND used_at IS NULL
            AND expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_optional(&self.user_service.pool)
        .await
        .map_err(|e| AuthError::UserError(UserError::DatabaseError(e)))?;

        let (token_id, user_id) = reset_token.ok_or(AuthError::InvalidCredentials)?;

        // Hash new password
        let new_password_hash = self
            .password_service
            .hash_password(new_password)
            .map_err(|e| AuthError::PasswordError(e.to_string()))?;

        // Update password
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_password_hash)
            .bind(&user_id)
            .execute(&self.user_service.pool)
            .await
            .map_err(|e| AuthError::UserError(UserError::DatabaseError(e)))?;

        // Mark token as used
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
            .bind(&token_id)
            .execute(&self.user_service.pool)
            .await
            .map_err(|e| AuthError::UserError(UserError::DatabaseError(e)))?;

        // Invalidate all existing tokens
        self.token_service
            .blacklist_user_tokens(&user_id)
            .await
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        Ok(())
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        let ip_address = None;
        let user_agent = None;
        // Verify refresh token
        let claims = self
            .token_service
            .verify_token(refresh_token)
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        // Check if it's a refresh token
        if claims.token_type != crate::services::auth::TokenType::Refresh {
            return Err(AuthError::TokenError("Not a refresh token".to_string()));
        }

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::TokenError("Invalid user ID in token".to_string()))?;

        // Verify user still exists
        let user = self.user_service.get_user_by_id(&user_id).await?;

        // Generate new token pair
        let tokens = self
            .token_service
            .generate_token_pair(&user.id, &user.username)
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        // Log token refresh
        self.audit_service
            .log_token_refreshed(user.id, &user.username, ip_address, user_agent)
            .await?;

        Ok(tokens)
    }
}
