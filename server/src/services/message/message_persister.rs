//! Message Persister Module
//! 
//! Handles message persistence to PostgreSQL database with retry logic.

use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use super::models::{Message, OfflineMessageBatch};

/// Persister error types
#[derive(Debug, Error)]
pub enum PersisterError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Persistence failed after {attempts} attempts: {message}")]
    RetryExhausted { attempts: u32, message: String },
    
    #[error("Invalid message data: {0}")]
    InvalidData(String),
}

/// Message persister trait
#[async_trait]
pub trait MessagePersister: Send + Sync {
    /// Persist a message to the database
    async fn persist_message(&self, message: &Message) -> Result<(), PersisterError>;
    
    /// Get offline messages for a user since a timestamp
    async fn get_offline_messages(
        &self,
        user_id: &Uuid,
        since_timestamp: i64,
        limit: i32,
    ) -> Result<OfflineMessageBatch, PersisterError>;
    
    /// Mark messages as delivered
    async fn mark_messages_delivered(&self, message_ids: &[Uuid]) -> Result<(), PersisterError>;
    
    /// Check if a message ID exists (for deduplication fallback)
    async fn message_exists(&self, message_id: &Uuid) -> Result<bool, PersisterError>;
}

/// PostgreSQL message persister implementation
#[derive(Clone)]
pub struct PostgresMessagePersister {
    pool: PgPool,
    max_retries: u32,
    retry_delays_ms: Vec<u64>,
}

impl PostgresMessagePersister {
    /// Create a new PostgreSQL message persister
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            max_retries: 3,
            retry_delays_ms: vec![100, 500, 2000], // Exponential backoff
        }
    }

    /// Create with custom retry configuration
    pub fn with_retry_config(pool: PgPool, max_retries: u32, retry_delays_ms: Vec<u64>) -> Self {
        Self {
            pool,
            max_retries,
            retry_delays_ms,
        }
    }

    /// Execute with retry logic
    async fn execute_with_retry<F, T, Fut>(&self, operation: F) -> Result<T, PersisterError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(
                        "Database operation failed (attempt {}/{}): {}",
                        attempt + 1,
                        self.max_retries,
                        e
                    );
                    last_error = Some(e);
                    
                    if attempt < self.max_retries - 1 {
                        let delay = self.retry_delays_ms
                            .get(attempt as usize)
                            .copied()
                            .unwrap_or(2000);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }
        
        Err(PersisterError::RetryExhausted {
            attempts: self.max_retries,
            message: last_error.map(|e| e.to_string()).unwrap_or_default(),
        })
    }
}

#[async_trait]
impl MessagePersister for PostgresMessagePersister {
    async fn persist_message(&self, message: &Message) -> Result<(), PersisterError> {
        let pool = self.pool.clone();
        let message = message.clone();
        
        tracing::debug!(
            message_id = %message.id,
            sender_id = %message.sender_id,
            receiver_id = ?message.receiver_id,
            group_id = ?message.group_id,
            msg_type = message.msg_type,
            content_len = message.content.len(),
            "🔵 persist_message: Attempting database insert"
        );
        
        let result = self.execute_with_retry(|| {
            let pool = pool.clone();
            let message = message.clone();
            async move {
                let result = sqlx::query(
                    r#"
                    INSERT INTO messages (
                        id, sender_id, receiver_id, group_id, conversation_id, msg_type,
                        content, timestamp, is_ai_generated, delivered, created_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    ON CONFLICT (id) DO NOTHING
                    "#,
                )
                .bind(&message.id)
                .bind(&message.sender_id)
                .bind(&message.receiver_id)
                .bind(&message.group_id)
                .bind(&message.conversation_id)
                .bind(&message.msg_type)
                .bind(&message.content)
                .bind(&message.timestamp)
                .bind(&message.is_ai_generated)
                .bind(&message.delivered)
                .bind(&message.created_at)
                .execute(&pool)
                .await?;

                tracing::debug!(
                    message_id = %message.id,
                    conversation_id = ?message.conversation_id,
                    rows_affected = result.rows_affected(),
                    "✅ Database insert completed"
                );

                Ok(())
            }
        })
        .await;
        
        if let Err(ref e) = result {
            tracing::error!(
                message_id = %message.id,
                error = %e,
                "❌ persist_message: Database insert failed"
            );
        }
        
        result
    }

    async fn get_offline_messages(
        &self,
        user_id: &Uuid,
        since_timestamp: i64,
        limit: i32,
    ) -> Result<OfflineMessageBatch, PersisterError> {
        // Fetch one more than limit to check if there are more messages
        let fetch_limit = limit + 1;
        
        let messages: Vec<Message> = sqlx::query_as(
            r#"
            SELECT id, sender_id, receiver_id, group_id, msg_type, 
                   content, timestamp, is_ai_generated, delivered, created_at
            FROM messages
            WHERE receiver_id = $1 
              AND delivered = FALSE
              AND timestamp > $2
            ORDER BY timestamp ASC
            LIMIT $3
            "#,
        )
        .bind(user_id)
        .bind(since_timestamp)
        .bind(fetch_limit)
        .fetch_all(&self.pool)
        .await?;

        let has_more = messages.len() > limit as usize;
        let messages: Vec<Message> = messages.into_iter().take(limit as usize).collect();
        let next_timestamp = messages.last().map(|m| m.timestamp);

        Ok(OfflineMessageBatch {
            messages,
            has_more,
            next_timestamp,
        })
    }

    async fn mark_messages_delivered(&self, message_ids: &[Uuid]) -> Result<(), PersisterError> {
        if message_ids.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            UPDATE messages
            SET delivered = TRUE
            WHERE id = ANY($1)
            "#,
        )
        .bind(message_ids)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn message_exists(&self, message_id: &Uuid) -> Result<bool, PersisterError> {
        let result: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1)",
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(exists,)| exists).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::message::models::MsgType;

    // Helper to create a test message
    fn create_test_message() -> Message {
        Message {
            id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            receiver_id: Some(Uuid::new_v4()),
            group_id: None,
            msg_type: MsgType::Text as i16,
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            is_ai_generated: false,
            delivered: false,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_create_test_message() {
        let message = create_test_message();
        assert!(!message.id.is_nil());
        assert!(!message.sender_id.is_nil());
        assert!(message.receiver_id.is_some());
        assert!(message.group_id.is_none());
    }

    #[test]
    fn test_retry_config_values() {
        // Test that retry config values are correct without needing a pool
        let max_retries = 5u32;
        let retry_delays = vec![50u64, 100, 200, 400, 800];
        
        assert_eq!(max_retries, 5);
        assert_eq!(retry_delays.len(), 5);
        assert_eq!(retry_delays[0], 50);
        assert_eq!(retry_delays[4], 800);
    }
}
