//! AI Agent Message Sender Implementation
//!
//! Provides concrete implementations for sending AI-generated messages back to the chat.

use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::{AgentMessageSender, AIAgentError};
use crate::services::message::message_router::MessageRouter;
use crate::services::message::models::{CreateMessage, MsgType};

/// Message sender that uses the message router to send AI responses
pub struct RouterMessageSender<R: MessageRouter> {
    router: Arc<R>,
    /// Virtual user ID for the AI agent (used as sender_id)
    /// In production, each agent should have its own user record
    agent_user_id_resolver: Arc<dyn AgentUserIdResolver>,
}

/// Trait for resolving agent IDs to user IDs
#[async_trait::async_trait]
pub trait AgentUserIdResolver: Send + Sync {
    /// Get the user ID for an agent
    /// Returns None if the agent doesn't have a user record
    async fn get_user_id(&self, agent_id: Uuid) -> Option<Uuid>;
}

/// Simple resolver that uses the agent ID as the user ID
pub struct DirectAgentIdResolver;

#[async_trait::async_trait]
impl AgentUserIdResolver for DirectAgentIdResolver {
    async fn get_user_id(&self, agent_id: Uuid) -> Option<Uuid> {
        Some(agent_id)
    }
}

/// Database-backed resolver that looks up agent user IDs
pub struct DatabaseAgentIdResolver {
    pool: sqlx::PgPool,
}

impl DatabaseAgentIdResolver {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AgentUserIdResolver for DatabaseAgentIdResolver {
    async fn get_user_id(&self, agent_id: Uuid) -> Option<Uuid> {
        // For now, we use the agent_id directly
        // In a full implementation, agents would have associated user records
        // that could be looked up here
        Some(agent_id)
    }
}

impl<R: MessageRouter> RouterMessageSender<R> {
    /// Create a new router message sender
    pub fn new(router: Arc<R>, resolver: Arc<dyn AgentUserIdResolver>) -> Self {
        Self {
            router,
            agent_user_id_resolver: resolver,
        }
    }

    /// Create with direct agent ID resolver (agent_id = user_id)
    pub fn with_direct_resolver(router: Arc<R>) -> Self {
        Self {
            router,
            agent_user_id_resolver: Arc::new(DirectAgentIdResolver),
        }
    }
}

#[async_trait::async_trait]
impl<R: MessageRouter + 'static> AgentMessageSender for RouterMessageSender<R> {
    async fn send_message(
        &self,
        group_id: Uuid,
        agent_id: Uuid,
        agent_name: &str,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Uuid, AIAgentError> {
        // Resolve agent to user ID
        let sender_id = self
            .agent_user_id_resolver
            .get_user_id(agent_id)
            .await
            .ok_or_else(|| {
                AIAgentError::Internal(format!("Could not resolve user ID for agent {}", agent_id))
            })?;

        let message_id = Uuid::new_v4();

        // Create the message
        let create_message = CreateMessage {
            id: Some(message_id),
            sender_id,
            receiver_id: None,
            group_id: Some(group_id),
            msg_type: MsgType::Text,
            content: content.to_string(),
            is_ai_generated: true,
        };

        debug!(
            "Agent {} ({}) sending message {} to group {}",
            agent_name, agent_id, message_id, group_id
        );

        // Route the message
        match self.router.route_message(create_message).await {
            Ok(ack) => {
                info!(
                    "Agent {} sent message {} to group {} (ack: {:?})",
                    agent_name, ack.message_id, group_id, ack.status
                );
                Ok(ack.message_id)
            }
            Err(e) => {
                error!(
                    "Agent {} failed to send message to group {}: {}",
                    agent_name, group_id, e
                );
                Err(AIAgentError::Internal(format!(
                    "Failed to route message: {}",
                    e
                )))
            }
        }
    }
}

/// Message sender that broadcasts through WebSocket connections
pub struct WebSocketMessageSender {
    connection_manager: Arc<crate::services::message::connection_manager::InMemoryConnectionManager>,
    pool: sqlx::PgPool,
}

impl WebSocketMessageSender {
    pub fn new(
        connection_manager: Arc<crate::services::message::connection_manager::InMemoryConnectionManager>,
        pool: sqlx::PgPool,
    ) -> Self {
        Self {
            connection_manager,
            pool,
        }
    }

    /// Get all member IDs for a group
    async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<Uuid>, AIAgentError> {
        let members = sqlx::query_scalar!(
            r#"SELECT user_id FROM group_members WHERE group_id = $1"#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AIAgentError::Database(e))?;

        Ok(members)
    }
}

#[async_trait::async_trait]
impl AgentMessageSender for WebSocketMessageSender {
    async fn send_message(
        &self,
        group_id: Uuid,
        agent_id: Uuid,
        agent_name: &str,
        content: &str,
        reply_to: Option<Uuid>,
    ) -> Result<Uuid, AIAgentError> {
        let message_id = Uuid::new_v4();

        // Create message struct for broadcasting
        let message = crate::services::message::models::Message {
            id: message_id,
            sender_id: agent_id,
            receiver_id: None,
            group_id: Some(group_id),
            msg_type: MsgType::Text as i16,
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            is_ai_generated: true,
            delivered: false,
            created_at: chrono::Utc::now(),
        };

        // Get group members
        let members = self.get_group_members(group_id).await?;

        debug!(
            "Agent {} broadcasting message {} to {} members in group {}",
            agent_name,
            message_id,
            members.len(),
            group_id
        );

        // Broadcast to all connected members
        let mut delivered_count = 0;
        for member_id in members {
            let connections = self
                .connection_manager
                .get_user_connections_async(&member_id)
                .await;

            for conn_id in connections {
                if let Some(sender) = self.connection_manager.get_sender_async(&conn_id).await {
                    if sender.send(message.clone()).is_ok() {
                        delivered_count += 1;
                    }
                }
            }
        }

        info!(
            "Agent {} sent message {} to group {}, delivered to {} connections",
            agent_name, message_id, group_id, delivered_count
        );

        Ok(message_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::message::models::MessageAck;
    use crate::services::message::message_router::RouterError;

    struct MockRouter {
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl MessageRouter for MockRouter {
        async fn route_message(
            &self,
            message: CreateMessage,
        ) -> Result<MessageAck, RouterError> {
            if self.should_fail {
                Err(RouterError::DeliveryFailed("Mock failure".to_string()))
            } else {
                Ok(MessageAck::success(message.id.unwrap_or_else(Uuid::new_v4)))
            }
        }

        async fn route_to_user(
            &self,
            _user_id: &Uuid,
            _message: &crate::services::message::models::Message,
        ) -> Result<bool, RouterError> {
            Ok(true)
        }

        async fn route_to_group(
            &self,
            _group_id: &Uuid,
            _message: &crate::services::message::models::Message,
        ) -> Result<u32, RouterError> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_router_message_sender_success() {
        let router = Arc::new(MockRouter { should_fail: false });
        let sender = RouterMessageSender::with_direct_resolver(router);

        let result = sender
            .send_message(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "TestBot",
                "Hello!",
                None,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_router_message_sender_failure() {
        let router = Arc::new(MockRouter { should_fail: true });
        let sender = RouterMessageSender::with_direct_resolver(router);

        let result = sender
            .send_message(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "TestBot",
                "Hello!",
                None,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_direct_agent_id_resolver() {
        let resolver = DirectAgentIdResolver;
        let agent_id = Uuid::new_v4();

        let user_id = resolver.get_user_id(agent_id).await;
        assert_eq!(user_id, Some(agent_id));
    }
}
