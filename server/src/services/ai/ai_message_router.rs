//! AI-Enhanced Message Router
//!
//! Extends the message router to automatically trigger AI Agent responses for group messages.

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{AgentMessage, AgentMessageHandler, AgentResponse, AIAgentError, AIAgentService};
use crate::services::message::models::{CreateMessage, Message};
use crate::services::message::message_router::{MessageRouter, RouterError};

/// AI-enhanced message router that triggers AI Agent responses
pub struct AIMessageRouter<R: MessageRouter> {
    inner_router: Arc<R>,
    agent_handler: Option<Arc<AgentMessageHandler>>,
    response_tx: Option<mpsc::UnboundedSender<AgentResponse>>,
    /// Recent messages cache per group (group_id -> messages)
    recent_messages: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Vec<String>>>>,
    /// Maximum recent messages to keep per group
    max_recent_messages: usize,
}

impl<R: MessageRouter> Clone for AIMessageRouter<R> {
    fn clone(&self) -> Self {
        Self {
            inner_router: self.inner_router.clone(),
            agent_handler: self.agent_handler.clone(),
            response_tx: self.response_tx.clone(),
            recent_messages: self.recent_messages.clone(),
            max_recent_messages: self.max_recent_messages,
        }
    }
}

impl<R: MessageRouter> AIMessageRouter<R> {
    /// Create a new AI message router
    pub fn new(inner_router: Arc<R>) -> Self {
        Self {
            inner_router,
            agent_handler: None,
            response_tx: None,
            recent_messages: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            max_recent_messages: 20,
        }
    }

    /// Set the agent handler
    pub fn with_agent_handler(mut self, handler: Arc<AgentMessageHandler>) -> Self {
        self.agent_handler = Some(handler);
        self
    }

    /// Set the response channel for AI responses
    pub fn with_response_channel(mut self, tx: mpsc::UnboundedSender<AgentResponse>) -> Self {
        self.response_tx = Some(tx);
        self
    }

    /// Set maximum recent messages to keep
    pub fn with_max_recent_messages(mut self, max: usize) -> Self {
        self.max_recent_messages = max;
        self
    }

    /// Add a message to recent messages cache
    async fn add_to_recent(&self, group_id: Uuid, sender_name: &str, content: &str) {
        let mut cache = self.recent_messages.write().await;
        let messages = cache.entry(group_id).or_insert_with(Vec::new);
        
        let formatted = format!("{}: {}", sender_name, content);
        messages.push(formatted);
        
        // Keep only the most recent messages
        if messages.len() > self.max_recent_messages {
            messages.remove(0);
        }
    }

    /// Get recent messages for a group
    async fn get_recent_messages(&self, group_id: Uuid) -> Vec<String> {
        let cache = self.recent_messages.read().await;
        cache.get(&group_id).cloned().unwrap_or_default()
    }

    /// Process AI Agent response for a group message
    async fn process_ai_response(
        &self,
        message: &Message,
        sender_name: String,
    ) -> Result<Option<AgentResponse>, AIAgentError> {
        let handler = match &self.agent_handler {
            Some(h) => h,
            None => return Ok(None),
        };

        let group_id = match message.group_id {
            Some(id) => id,
            None => return Ok(None), // Not a group message
        };

        // Skip AI-generated messages to prevent loops
        if message.is_ai_generated {
            debug!("Skipping AI-generated message");
            return Ok(None);
        }

        // Get recent messages for context
        let recent_messages = self.get_recent_messages(group_id).await;

        // Create agent message
        let agent_message = AgentMessage {
            message_id: message.id,
            group_id,
            sender_id: message.sender_id,
            sender_name: sender_name.clone(),
            content: message.content.clone(),
            timestamp: message.timestamp,
        };

        // Process through agent handler
        // NOTE: AgentMessageHandler.process_message() already sends the response
        // via its internal response_tx channel, so we don't need to send it again here.
        let response = handler.process_message(agent_message, &recent_messages).await?;

        Ok(response)
    }
}

#[async_trait::async_trait]
impl<R: MessageRouter + 'static> MessageRouter for AIMessageRouter<R> {
    async fn route_message(
        &self,
        message: CreateMessage,
    ) -> Result<crate::services::message::models::MessageAck, RouterError> {
        // First, route through the inner router
        let ack = self.inner_router.route_message(message.clone()).await?;

        // If it's a group message and we have an agent handler, process AI response
        if let Some(group_id) = message.group_id {
            // Add to recent messages (use sender_id as name for now)
            let sender_name = message.sender_id.to_string();
            self.add_to_recent(group_id, &sender_name, &message.content).await;

            // Create a Message struct for AI processing
            let msg = Message {
                id: ack.message_id,
                sender_id: message.sender_id,
                receiver_id: message.receiver_id,
                group_id: Some(group_id),
                conversation_id: Some(group_id),  // For group messages, use group_id as conversation_id
                msg_type: message.msg_type as i16,
                content: message.content,
                timestamp: chrono::Utc::now().timestamp_millis(),
                is_ai_generated: message.is_ai_generated,
                delivered: false,
                created_at: chrono::Utc::now(),
            };

            // Spawn AI processing in background (don't block message routing)
            let handler = self.agent_handler.clone();
            let response_tx = self.response_tx.clone();
            let recent_messages = self.get_recent_messages(group_id).await;

            if handler.is_some() {
                tokio::spawn(async move {
                    if let Some(handler) = handler {
                        let agent_message = AgentMessage {
                            message_id: msg.id,
                            group_id,
                            sender_id: msg.sender_id,
                            sender_name: msg.sender_id.to_string(),
                            content: msg.content.clone(),
                            timestamp: msg.timestamp,
                        };

                        match handler.process_message(agent_message, &recent_messages).await {
                            Ok(Some(response)) => {
                                info!(
                                    "AI Agent {} will respond to message in group {}",
                                    response.agent_name, group_id
                                );
                                // NOTE: Response is already sent by AgentMessageHandler.process_message()
                                // via its internal response_tx channel. Do NOT send again here
                                // to avoid duplicate messages.
                            }
                            Ok(None) => {
                                debug!("AI Agent decided not to respond");
                            }
                            Err(e) => {
                                warn!("AI Agent processing error: {}", e);
                            }
                        }
                    }
                });
            }
        }

        Ok(ack)
    }

    async fn route_to_user(
        &self,
        user_id: &Uuid,
        message: &Message,
    ) -> Result<bool, RouterError> {
        self.inner_router.route_to_user(user_id, message).await
    }

    async fn route_to_group(
        &self,
        group_id: &Uuid,
        message: &Message,
    ) -> Result<u32, RouterError> {
        self.inner_router.route_to_group(group_id, message).await
    }
}

/// Builder for creating AI-enhanced message routing system
pub struct AIMessageRouterBuilder<R: MessageRouter> {
    inner_router: Arc<R>,
    agent_service: Option<AIAgentService>,
    max_recent_messages: usize,
}

impl<R: MessageRouter + 'static> AIMessageRouterBuilder<R> {
    pub fn new(inner_router: Arc<R>) -> Self {
        Self {
            inner_router,
            agent_service: None,
            max_recent_messages: 20,
        }
    }

    pub fn with_agent_service(mut self, service: AIAgentService) -> Self {
        self.agent_service = Some(service);
        self
    }

    pub fn with_max_recent_messages(mut self, max: usize) -> Self {
        self.max_recent_messages = max;
        self
    }

    /// Build the AI message router and response channel
    pub fn build(self) -> (AIMessageRouter<R>, Option<mpsc::UnboundedReceiver<AgentResponse>>) {
        let (tx, rx) = mpsc::unbounded_channel();

        let agent_handler = self.agent_service.map(|service| {
            Arc::new(AgentMessageHandler::new(service).with_response_channel(tx.clone()))
        });

        let router = AIMessageRouter {
            inner_router: self.inner_router,
            agent_handler,
            response_tx: Some(tx),
            recent_messages: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            max_recent_messages: self.max_recent_messages,
        };

        (router, Some(rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::message::models::{MessageAck, MsgType};

    // Mock router for testing
    struct MockRouter;

    #[async_trait::async_trait]
    impl MessageRouter for MockRouter {
        async fn route_message(&self, message: CreateMessage) -> Result<MessageAck, RouterError> {
            Ok(MessageAck::success(message.id.unwrap_or_else(Uuid::new_v4)))
        }

        async fn route_to_user(&self, _user_id: &Uuid, _message: &Message) -> Result<bool, RouterError> {
            Ok(true)
        }

        async fn route_to_group(&self, _group_id: &Uuid, _message: &Message) -> Result<u32, RouterError> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn test_ai_router_routes_messages() {
        let inner = Arc::new(MockRouter);
        let router = AIMessageRouter::new(inner);

        let message = CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id: Uuid::new_v4(),
            receiver_id: Some(Uuid::new_v4()),
            group_id: None,
            msg_type: MsgType::Text,
            content: "Hello".to_string(),
            is_ai_generated: false,
        };

        let result = router.route_message(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ai_router_caches_recent_messages() {
        let inner = Arc::new(MockRouter);
        let router = AIMessageRouter::new(inner).with_max_recent_messages(5);

        let group_id = Uuid::new_v4();

        // Add messages
        for i in 0..10 {
            router.add_to_recent(group_id, "User", &format!("Message {}", i)).await;
        }

        // Should only keep last 5
        let recent = router.get_recent_messages(group_id).await;
        assert_eq!(recent.len(), 5);
        assert!(recent[0].contains("Message 5"));
        assert!(recent[4].contains("Message 9"));
    }

    #[tokio::test]
    async fn test_ai_router_builder() {
        let inner = Arc::new(MockRouter);
        let builder = AIMessageRouterBuilder::new(inner)
            .with_max_recent_messages(10);

        let (router, rx) = builder.build();
        assert!(rx.is_some());
    }
}
