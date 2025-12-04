use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Token service errors
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token encoding failed: {0}")]
    EncodingFailed(#[from] jsonwebtoken::errors::Error),
    #[error("Token is expired")]
    Expired,
    #[error("Token is invalid")]
    Invalid,
    #[error("Token is blacklisted")]
    Blacklisted,
    #[error("Redis error: {0}")]
    RedisError(String),
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// Token type (access or refresh)
    pub token_type: TokenType,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// JWT ID (unique identifier for this token)
    pub jti: String,
}

/// Token type enum
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Token pair returned after authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: i64,
    pub refresh_expires_at: i64,
}

/// Token service for JWT generation and verification
#[derive(Clone)]
pub struct TokenService {
    secret: String,
    access_expiry_seconds: i64,
    refresh_expiry_seconds: i64,
    redis: Option<redis::aio::ConnectionManager>,
}

impl TokenService {
    /// Create a new token service
    pub fn new(secret: String, access_expiry_seconds: i64, refresh_expiry_seconds: i64) -> Self {
        Self {
            secret,
            access_expiry_seconds,
            refresh_expiry_seconds,
            redis: None,
        }
    }

    /// Create with default expiry times (15 min access, 7 days refresh)
    pub fn with_defaults(secret: String) -> Self {
        Self::new(secret, 900, 604800)
    }

    /// Create with Redis support for token blacklist
    pub fn with_redis(secret: String, redis: redis::aio::ConnectionManager) -> Self {
        Self {
            secret,
            access_expiry_seconds: 900,
            refresh_expiry_seconds: 604800,
            redis: Some(redis),
        }
    }

    /// Generate a token pair (access + refresh)
    pub fn generate_token_pair(
        &self,
        user_id: &Uuid,
        username: &str,
    ) -> Result<TokenPair, TokenError> {
        let now = Utc::now();
        let access_exp = now + Duration::seconds(self.access_expiry_seconds);
        let refresh_exp = now + Duration::seconds(self.refresh_expiry_seconds);

        let access_token =
            self.generate_token(user_id, username, TokenType::Access, access_exp.timestamp())?;

        let refresh_token = self.generate_token(
            user_id,
            username,
            TokenType::Refresh,
            refresh_exp.timestamp(),
        )?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            access_expires_at: access_exp.timestamp(),
            refresh_expires_at: refresh_exp.timestamp(),
        })
    }

    /// Generate a single token
    fn generate_token(
        &self,
        user_id: &Uuid,
        username: &str,
        token_type: TokenType,
        exp: i64,
    ) -> Result<String, TokenError> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            token_type,
            iat: now,
            exp,
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(TokenError::EncodingFailed)
    }

    /// Verify and decode a token
    pub fn verify_token(&self, token: &str) -> Result<Claims, TokenError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let token_data: TokenData<Claims> = decode(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
            _ => TokenError::Invalid,
        })?;

        Ok(token_data.claims)
    }

    /// Extract claims without verification (for debugging)
    pub fn decode_without_verification(&self, token: &str) -> Result<Claims, TokenError> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data: TokenData<Claims> = decode(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|_| TokenError::Invalid)?;

        Ok(token_data.claims)
    }

    /// Check if token is expired
    pub fn is_expired(&self, claims: &Claims) -> bool {
        Utc::now().timestamp() > claims.exp
    }

    /// Get access token expiry in seconds
    pub fn access_expiry_seconds(&self) -> i64 {
        self.access_expiry_seconds
    }

    /// Get refresh token expiry in seconds
    pub fn refresh_expiry_seconds(&self) -> i64 {
        self.refresh_expiry_seconds
    }

    /// Add a token to the blacklist
    pub async fn blacklist_token(&self, token: &str) -> Result<(), TokenError> {
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| TokenError::RedisError("Redis not configured".to_string()))?;

        // Extract claims to get expiry time
        let claims = self.decode_without_verification(token)?;
        let now = Utc::now().timestamp();
        let ttl = (claims.exp - now).max(0);

        if ttl > 0 {
            let key = format!("blacklist:token:{}", claims.jti);
            let mut conn = redis.clone();

            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl as usize)
                .arg("1")
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| TokenError::RedisError(e.to_string()))?;
        }

        Ok(())
    }

    /// Check if a token is blacklisted
    pub async fn is_blacklisted(&self, token: &str) -> Result<bool, TokenError> {
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| TokenError::RedisError("Redis not configured".to_string()))?;

        let claims = self.decode_without_verification(token)?;
        let key = format!("blacklist:token:{}", claims.jti);
        let mut conn = redis.clone();

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| TokenError::RedisError(e.to_string()))?;

        Ok(exists)
    }

    /// Blacklist all tokens for a user issued before now
    pub async fn blacklist_user_tokens(&self, user_id: &Uuid) -> Result<(), TokenError> {
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| TokenError::RedisError("Redis not configured".to_string()))?;

        let key = format!("blacklist:user:{}", user_id);
        let mut conn = redis.clone();
        let now = Utc::now().timestamp();

        // Store the timestamp when blacklist was created
        // Tokens issued before this time will be rejected
        redis::cmd("SETEX")
            .arg(&key)
            .arg(self.refresh_expiry_seconds as usize)
            .arg(now.to_string())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| TokenError::RedisError(e.to_string()))?;

        Ok(())
    }

    /// Check if a token was issued before user blacklist timestamp
    pub async fn is_user_blacklisted(&self, user_id: &Uuid, token_iat: i64) -> Result<bool, TokenError> {
        let redis = self
            .redis
            .as_ref()
            .ok_or_else(|| TokenError::RedisError("Redis not configured".to_string()))?;

        let key = format!("blacklist:user:{}", user_id);
        let mut conn = redis.clone();

        let blacklist_timestamp: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| TokenError::RedisError(e.to_string()))?;

        match blacklist_timestamp {
            Some(ts_str) => {
                let blacklist_ts = ts_str.parse::<i64>()
                    .map_err(|_| TokenError::RedisError("Invalid timestamp in blacklist".to_string()))?;
                // Token is blacklisted if it was issued before the blacklist timestamp
                Ok(token_iat < blacklist_ts)
            }
            None => Ok(false),
        }
    }

    /// Verify token and check blacklist
    pub async fn verify_token_with_blacklist(&self, token: &str) -> Result<Claims, TokenError> {
        // First verify the token signature and expiry
        let claims = self.verify_token(token)?;

        // Check if token is blacklisted
        if self.is_blacklisted(token).await? {
            return Err(TokenError::Blacklisted);
        }

        // Check if user's tokens issued before a certain time are blacklisted
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| TokenError::Invalid)?;
        
        if self.is_user_blacklisted(&user_id, claims.iat).await? {
            return Err(TokenError::Blacklisted);
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    fn create_test_service() -> TokenService {
        TokenService::new("test_secret_key_12345".to_string(), 2, 10)
    }

    // Unit tests
    #[test]
    fn test_generate_token_pair() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let username = "testuser";

        let pair = service.generate_token_pair(&user_id, username).unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_ne!(pair.access_token, pair.refresh_token);
    }

    #[test]
    fn test_verify_valid_token() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();
        let username = "testuser";

        let pair = service.generate_token_pair(&user_id, username).unwrap();
        let claims = service.verify_token(&pair.access_token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_is_expired() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let pair = service.generate_token_pair(&user_id, "testuser").unwrap();
        let claims = service.verify_token(&pair.access_token).unwrap();

        // Token should not be expired immediately
        assert!(!service.is_expired(&claims));

        // Create expired claims manually
        let expired_claims = Claims {
            sub: user_id.to_string(),
            username: "testuser".to_string(),
            token_type: TokenType::Access,
            iat: Utc::now().timestamp() - 100,
            exp: Utc::now().timestamp() - 50,
            jti: Uuid::new_v4().to_string(),
        };
        assert!(service.is_expired(&expired_claims));
    }

    #[test]
    fn test_verify_invalid_token() {
        let service = create_test_service();
        let result = service.verify_token("invalid.token.here");
        assert!(matches!(result, Err(TokenError::Invalid)));
    }

    #[test]
    fn test_token_types() {
        let service = create_test_service();
        let user_id = Uuid::new_v4();

        let pair = service.generate_token_pair(&user_id, "testuser").unwrap();

        let access_claims = service.verify_token(&pair.access_token).unwrap();
        let refresh_claims = service.verify_token(&pair.refresh_token).unwrap();

        assert_eq!(access_claims.token_type, TokenType::Access);
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    // Property-based tests

    /// **Feature: user-auth-system, Property 8: Token validity periods**
    /// **Validates: Requirements 2.3, 3.3**
    /// *For any* user, access token expires before refresh token.
    proptest! {
        #[test]
        fn prop_token_validity_periods(
            user_id in "[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}",
            username in "[a-z]{3,20}"
        ) {
            let service = create_test_service();
            let uuid = Uuid::parse_str(&user_id).unwrap();

            let pair = service.generate_token_pair(&uuid, &username).unwrap();

            prop_assert!(pair.access_expires_at < pair.refresh_expires_at);
        }
    }

    /// **Feature: user-auth-system, Property 9: JWT payload completeness**
    /// **Validates: Requirements 2.4**
    /// *For any* generated token, claims contain user_id, username, token_type, iat, exp, jti.
    proptest! {
        #[test]
        fn prop_jwt_payload_completeness(
            username in "[a-z]{3,20}"
        ) {
            let service = create_test_service();
            let user_id = Uuid::new_v4();

            let pair = service.generate_token_pair(&user_id, &username).unwrap();
            let claims = service.verify_token(&pair.access_token).unwrap();

            prop_assert!(!claims.sub.is_empty());
            prop_assert_eq!(claims.username, username);
            prop_assert!(claims.iat > 0);
            prop_assert!(claims.exp > claims.iat);
            prop_assert!(!claims.jti.is_empty());
        }
    }

    /// **Feature: user-auth-system, Property 36: Token signature verification**
    /// **Validates: Requirements 9.1**
    /// *For any* token, modifying it invalidates the signature.
    proptest! {
        #[test]
        fn prop_token_signature_verification(
            username in "[a-z]{3,20}"
        ) {
            let service = create_test_service();
            let user_id = Uuid::new_v4();

            let pair = service.generate_token_pair(&user_id, &username).unwrap();

            // Modify the token
            let mut modified = pair.access_token.clone();
            modified.push('x');

            let result = service.verify_token(&modified);
            prop_assert!(result.is_err());
        }
    }

    /// **Feature: user-auth-system, Property 38: Token claims extraction**
    /// **Validates: Requirements 9.3**
    /// *For any* valid token, claims can be extracted and match original values.
    proptest! {
        #[test]
        fn prop_token_claims_extraction(
            username in "[a-z]{3,20}"
        ) {
            let service = create_test_service();
            let user_id = Uuid::new_v4();

            let pair = service.generate_token_pair(&user_id, &username).unwrap();
            let claims = service.verify_token(&pair.access_token).unwrap();

            prop_assert_eq!(claims.sub, user_id.to_string());
            prop_assert_eq!(claims.username, username);
        }
    }
}
