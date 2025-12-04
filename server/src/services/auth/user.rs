use crate::models::user::{CreateUser, OnlineStatus, UpdateUser, User, UserProfile, UserRole};
use sqlx::{PgPool, Postgres, Acquire};
use thiserror::Error;
use uuid::Uuid;

/// User service errors
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
    #[error("Username already exists")]
    UsernameExists,
    #[error("Email already exists")]
    EmailExists,
    #[error("Invalid username format")]
    InvalidUsername,
    #[error("Invalid avatar URL")]
    InvalidAvatarUrl,
    #[error("Bio too long (max 500 characters)")]
    BioTooLong,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// User service for managing users
#[derive(Clone)]
pub struct UserService {
    pub(crate) pool: PgPool,
    redis: Option<redis::aio::ConnectionManager>,
}

impl UserService {
    /// Create a new user service
    pub fn new(pool: PgPool) -> Self {
        Self { pool, redis: None }
    }

    /// Create a new user service with Redis caching
    pub fn with_redis(pool: PgPool, redis: redis::aio::ConnectionManager) -> Self {
        Self {
            pool,
            redis: Some(redis),
        }
    }

    /// Create a user service that uses a transaction
    /// This is useful for testing with transaction rollback
    pub fn with_transaction(tx: sqlx::Transaction<'static, Postgres>) -> UserServiceWithTransaction {
        UserServiceWithTransaction { tx }
    }

    /// Validate username format (3-50 chars, alphanumeric and underscore)
    pub fn validate_username(username: &str) -> Result<(), UserError> {
        if username.len() < 3 || username.len() > 50 {
            return Err(UserError::InvalidUsername);
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(UserError::InvalidUsername);
        }
        Ok(())
    }

    /// Validate avatar URL
    pub fn validate_avatar_url(url: &str) -> Result<(), UserError> {
        if url.len() > 500 {
            return Err(UserError::InvalidAvatarUrl);
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(UserError::InvalidAvatarUrl);
        }
        Ok(())
    }

    /// Validate bio length
    pub fn validate_bio(bio: &str) -> Result<(), UserError> {
        if bio.len() > 500 {
            return Err(UserError::BioTooLong);
        }
        Ok(())
    }

    /// Create a new user
    pub async fn create_user(&self, data: CreateUser) -> Result<User, UserError> {
        Self::validate_username(&data.username)?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&data.username)
        .bind(&data.email)
        .bind(&data.password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key") {
                    return UserError::UsernameExists;
                }
                if db_err.constraint() == Some("users_email_key") {
                    return UserError::EmailExists;
                }
            }
            UserError::DatabaseError(e)
        })?;

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1 AND account_status != 'deleted'
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE username = $1 AND account_status != 'deleted'
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE email = $1 AND account_status != 'deleted'
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    /// Get user profile by ID (public data only)
    /// Uses Redis cache with 5-minute TTL if available
    pub async fn get_profile_by_id(&self, user_id: &Uuid) -> Result<UserProfile, UserError> {
        // Try to get from cache first
        if let Some(ref redis) = self.redis {
            match self.get_cached_profile(user_id, redis.clone()).await {
                Ok(Some(profile)) => {
                    tracing::debug!("Cache hit for user profile: {}", user_id);
                    return Ok(profile);
                }
                Ok(None) => {
                    tracing::debug!("Cache miss for user profile: {}", user_id);
                }
                Err(e) => {
                    tracing::warn!("Redis cache error, falling back to database: {}", e);
                }
            }
        }

        // Get from database
        let user = self.get_user_by_id(user_id).await?;
        let profile: UserProfile = user.into();

        // Cache the profile (best effort, don't fail if caching fails)
        if let Some(ref redis) = self.redis {
            if let Err(e) = self.cache_profile(&profile, redis.clone()).await {
                tracing::warn!("Failed to cache user profile: {}", e);
            }
        }

        Ok(profile)
    }

    /// Get profile from Redis cache
    async fn get_cached_profile(
        &self,
        user_id: &Uuid,
        mut redis: redis::aio::ConnectionManager,
    ) -> Result<Option<UserProfile>, String> {
        let key = format!("user_profile:{}", user_id);
        
        let cached: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut redis)
            .await
            .map_err(|e| e.to_string())?;

        match cached {
            Some(json) => {
                let profile: UserProfile = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to deserialize profile: {}", e))?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    /// Cache profile in Redis with 5-minute TTL
    async fn cache_profile(
        &self,
        profile: &UserProfile,
        mut redis: redis::aio::ConnectionManager,
    ) -> Result<(), String> {
        let key = format!("user_profile:{}", profile.id);
        let json = serde_json::to_string(profile)
            .map_err(|e| format!("Failed to serialize profile: {}", e))?;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(300) // 5 minutes TTL
            .arg(&json)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Invalidate cached profile
    async fn invalidate_cache(&self, user_id: &Uuid) -> Result<(), String> {
        if let Some(ref redis) = self.redis {
            let key = format!("user_profile:{}", user_id);
            let mut redis = redis.clone();
            
            redis::cmd("DEL")
                .arg(&key)
                .query_async::<()>(&mut redis)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Get user role by ID
    pub async fn get_user_role(&self, user_id: &Uuid) -> Result<UserRole, UserError> {
        let user = self.get_user_by_id(user_id).await?;
        Ok(user.role)
    }

    /// Get multiple user profiles by IDs (batch retrieval, max 100)
    pub async fn get_profiles_by_ids(
        &self,
        user_ids: &[Uuid],
    ) -> Result<Vec<UserProfile>, UserError> {
        if user_ids.len() > 100 {
            return Ok(vec![]);
        }

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = ANY($1) AND account_status != 'deleted'
            "#,
        )
        .bind(user_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(users.into_iter().map(UserProfile::from).collect())
    }

    /// Search users by username (case-insensitive, max 50 results)
    pub async fn search_users(&self, query: &str) -> Result<Vec<UserProfile>, UserError> {
        if query.len() < 2 {
            return Ok(vec![]);
        }

        let search_pattern = format!("%{}%", query.to_lowercase());

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE LOWER(username) LIKE $1
            AND account_status = 'active'
            ORDER BY username
            LIMIT 50
            "#,
        )
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await?;

        Ok(users.into_iter().map(UserProfile::from).collect())
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: &Uuid,
        data: UpdateUser,
    ) -> Result<UserProfile, UserError> {
        // Validate inputs
        if let Some(ref username) = data.username {
            Self::validate_username(username)?;
        }
        if let Some(ref avatar_url) = data.avatar_url {
            if !avatar_url.is_empty() {
                Self::validate_avatar_url(avatar_url)?;
            }
        }
        if let Some(ref bio) = data.bio {
            Self::validate_bio(bio)?;
        }

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut bind_count = 1;

        if data.username.is_some() {
            query.push_str(&format!(", username = ${}", bind_count));
            bind_count += 1;
        }
        if data.avatar_url.is_some() {
            query.push_str(&format!(", avatar_url = ${}", bind_count));
            bind_count += 1;
        }
        if data.bio.is_some() {
            query.push_str(&format!(", bio = ${}", bind_count));
            bind_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(username) = data.username {
            query_builder = query_builder.bind(username);
        }
        if let Some(avatar_url) = data.avatar_url {
            query_builder = query_builder.bind(avatar_url);
        }
        if let Some(bio) = data.bio {
            query_builder = query_builder.bind(bio);
        }

        query_builder = query_builder.bind(user_id);

        let user = query_builder.fetch_one(&self.pool).await.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key") {
                    return UserError::UsernameExists;
                }
            }
            UserError::DatabaseError(e)
        })?;

        // Invalidate cache after profile update
        if let Err(e) = self.invalidate_cache(user_id).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        }

        Ok(user.into())
    }

    /// Update online status
    pub async fn update_online_status(
        &self,
        user_id: &Uuid,
        status: OnlineStatus,
    ) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET online_status = $1, last_seen_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&status)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Invalidate cache after status update
        if let Err(e) = self.invalidate_cache(user_id).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        }

        Ok(())
    }

    /// Suspend user account
    pub async fn suspend_account(&self, user_id: &Uuid) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET account_status = 'suspended'
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Invalidate cache after account status change
        if let Err(e) = self.invalidate_cache(user_id).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        }

        Ok(())
    }

    /// Reactivate user account
    pub async fn reactivate_account(&self, user_id: &Uuid) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET account_status = 'active'
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Invalidate cache after account reactivation
        if let Err(e) = self.invalidate_cache(user_id).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        }

        Ok(())
    }

    /// Delete user account (soft delete with anonymization)
    pub async fn delete_account(&self, user_id: &Uuid) -> Result<(), UserError> {
        let anonymous_username = format!("deleted_{}", Uuid::new_v4());
        let anonymous_email = format!("{}@deleted.local", Uuid::new_v4());

        sqlx::query(
            r#"
            UPDATE users
            SET account_status = 'deleted',
                deleted_at = NOW(),
                username = $1,
                email = $2,
                avatar_url = NULL,
                bio = NULL
            WHERE id = $3
            "#,
        )
        .bind(&anonymous_username)
        .bind(&anonymous_email)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Invalidate cache after account deletion
        if let Err(e) = self.invalidate_cache(user_id).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(UserService::validate_username("valid_user123").is_ok());
        assert!(UserService::validate_username("ab").is_err()); // Too short
        assert!(UserService::validate_username(&"a".repeat(51)).is_err()); // Too long
        assert!(UserService::validate_username("invalid-user").is_err()); // Invalid char
        assert!(UserService::validate_username("invalid user").is_err()); // Space
    }

    #[test]
    fn test_validate_avatar_url() {
        assert!(UserService::validate_avatar_url("https://example.com/avatar.jpg").is_ok());
        assert!(UserService::validate_avatar_url("http://example.com/avatar.jpg").is_ok());
        assert!(UserService::validate_avatar_url("ftp://example.com/avatar.jpg").is_err());
        let long_url = format!("https://{}", "a".repeat(500));
        assert!(UserService::validate_avatar_url(&long_url).is_err());
    }

    #[test]
    fn test_validate_bio() {
        assert!(UserService::validate_bio("This is a valid bio").is_ok());
        assert!(UserService::validate_bio(&"a".repeat(500)).is_ok());
        assert!(UserService::validate_bio(&"a".repeat(501)).is_err());
    }
}

/// User service that uses a transaction
/// This is primarily used for testing with transaction rollback
pub struct UserServiceWithTransaction {
    tx: sqlx::Transaction<'static, Postgres>,
}

impl UserServiceWithTransaction {
    /// Get a mutable reference to the transaction for manual control (e.g., rollback)
    pub fn transaction_mut(&mut self) -> &mut sqlx::Transaction<'static, Postgres> {
        &mut self.tx
    }
    
    /// Consume the service and return the transaction
    pub fn into_transaction(self) -> sqlx::Transaction<'static, Postgres> {
        self.tx
    }

    /// Create a new user
    pub async fn create_user(&mut self, data: CreateUser) -> Result<User, UserError> {
        UserService::validate_username(&data.username)?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&data.username)
        .bind(&data.email)
        .bind(&data.password_hash)
        .fetch_one(&mut *self.tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key") {
                    return UserError::UsernameExists;
                }
                if db_err.constraint() == Some("users_email_key") {
                    return UserError::EmailExists;
                }
            }
            UserError::DatabaseError(e)
        })?;

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&mut self, user_id: &Uuid) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1 AND account_status != 'deleted'
            "#,
        )
        .bind(user_id)
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    /// Get user by username
    pub async fn get_user_by_username(&mut self, username: &str) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE username = $1 AND account_status != 'deleted'
            "#,
        )
        .bind(username)
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    /// Get user by email
    pub async fn get_user_by_email(&mut self, email: &str) -> Result<User, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE email = $1 AND account_status != 'deleted'
            "#,
        )
        .bind(email)
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(UserError::NotFound)?;

        Ok(user)
    }

    /// Get user profile by ID (public data only)
    pub async fn get_profile_by_id(&mut self, user_id: &Uuid) -> Result<UserProfile, UserError> {
        let user = self.get_user_by_id(user_id).await?;
        Ok(user.into())
    }

    /// Get multiple user profiles by IDs (batch retrieval, max 100)
    pub async fn get_profiles_by_ids(
        &mut self,
        user_ids: &[Uuid],
    ) -> Result<Vec<UserProfile>, UserError> {
        if user_ids.len() > 100 {
            return Ok(vec![]);
        }

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = ANY($1) AND account_status != 'deleted'
            "#,
        )
        .bind(user_ids)
        .fetch_all(&mut *self.tx)
        .await?;

        Ok(users.into_iter().map(UserProfile::from).collect())
    }

    /// Search users by username (case-insensitive, max 50 results)
    pub async fn search_users(&mut self, query: &str) -> Result<Vec<UserProfile>, UserError> {
        if query.len() < 2 {
            return Ok(vec![]);
        }

        let search_pattern = format!("%{}%", query.to_lowercase());

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE LOWER(username) LIKE $1
            AND account_status = 'active'
            ORDER BY username
            LIMIT 50
            "#,
        )
        .bind(&search_pattern)
        .fetch_all(&mut *self.tx)
        .await?;

        Ok(users.into_iter().map(UserProfile::from).collect())
    }

    /// Update user profile
    pub async fn update_profile(
        &mut self,
        user_id: &Uuid,
        data: UpdateUser,
    ) -> Result<UserProfile, UserError> {
        // Validate inputs
        if let Some(ref username) = data.username {
            UserService::validate_username(username)?;
        }
        if let Some(ref avatar_url) = data.avatar_url {
            if !avatar_url.is_empty() {
                UserService::validate_avatar_url(avatar_url)?;
            }
        }
        if let Some(ref bio) = data.bio {
            UserService::validate_bio(bio)?;
        }

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut bind_count = 1;

        if data.username.is_some() {
            query.push_str(&format!(", username = ${}", bind_count));
            bind_count += 1;
        }
        if data.avatar_url.is_some() {
            query.push_str(&format!(", avatar_url = ${}", bind_count));
            bind_count += 1;
        }
        if data.bio.is_some() {
            query.push_str(&format!(", bio = ${}", bind_count));
            bind_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(username) = data.username {
            query_builder = query_builder.bind(username);
        }
        if let Some(avatar_url) = data.avatar_url {
            query_builder = query_builder.bind(avatar_url);
        }
        if let Some(bio) = data.bio {
            query_builder = query_builder.bind(bio);
        }

        query_builder = query_builder.bind(user_id);

        let user = query_builder.fetch_one(&mut *self.tx).await.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key") {
                    return UserError::UsernameExists;
                }
            }
            UserError::DatabaseError(e)
        })?;

        Ok(user.into())
    }

    /// Update online status
    pub async fn update_online_status(
        &mut self,
        user_id: &Uuid,
        status: OnlineStatus,
    ) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET online_status = $1, last_seen_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(&status)
        .bind(user_id)
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    /// Suspend user account
    pub async fn suspend_account(&mut self, user_id: &Uuid) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET account_status = 'suspended'
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    /// Reactivate user account
    pub async fn reactivate_account(&mut self, user_id: &Uuid) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE users
            SET account_status = 'active'
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }

    /// Delete user account (soft delete with anonymization)
    pub async fn delete_account(&mut self, user_id: &Uuid) -> Result<(), UserError> {
        let anonymous_username = format!("deleted_{}", Uuid::new_v4());
        let anonymous_email = format!("{}@deleted.local", Uuid::new_v4());

        sqlx::query(
            r#"
            UPDATE users
            SET account_status = 'deleted',
                deleted_at = NOW(),
                username = $1,
                email = $2,
                avatar_url = NULL,
                bio = NULL
            WHERE id = $3
            "#,
        )
        .bind(&anonymous_username)
        .bind(&anonymous_email)
        .bind(user_id)
        .execute(&mut *self.tx)
        .await?;

        Ok(())
    }
}
