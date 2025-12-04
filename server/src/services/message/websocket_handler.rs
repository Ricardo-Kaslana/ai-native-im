//! WebSocket Handler Module
//! 
//! Manages WebSocket connections, authentication, and message processing.

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::connection_manager::InMemoryConnectionManager;
use super::message_router::{MessageRouter, MessageRouterImpl};
use super::models::{CreateMessage, Message, MsgType};
use crate::services::auth::TokenService;

/// WebSocket handler error types
#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Authentication timeout")]
    AuthenticationTimeout,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error("Send error: {0}")]
    SendError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub conn_id: String,
    pub user_id: Uuid,
    pub connected_at: Instant,
    pub last_heartbeat: Instant,
    pub authenticated: bool,
}

impl ConnectionState {
    pub fn new(user_id: Uuid) -> Self {
        let now = Instant::now();
        Self {
            conn_id: Uuid::new_v4().to_string(),
            user_id,
            connected_at: now,
            last_heartbeat: now,
            authenticated: true,
        }
    }
}

/// WebSocket handler configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Authentication timeout in seconds
    pub auth_timeout_secs: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Heartbeat timeout in seconds (connection considered dead if no pong)
    pub heartbeat_timeout_secs: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            auth_timeout_secs: 3,
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 10,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

/// WebSocket handler for managing connections
pub struct WebSocketHandler {
    connection_manager: Arc<InMemoryConnectionManager>,
    message_router: Arc<MessageRouterImpl>,
    token_service: Arc<TokenService>,
    config: WebSocketConfig,
}

impl WebSocketHandler {
    /// Create a new WebSocket handler
    pub fn new(
        connection_manager: Arc<InMemoryConnectionManager>,
        message_router: Arc<MessageRouterImpl>,
        token_service: Arc<TokenService>,
    ) -> Self {
        Self {
            connection_manager,
            message_router,
            token_service,
            config: WebSocketConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        connection_manager: Arc<InMemoryConnectionManager>,
        message_router: Arc<MessageRouterImpl>,
        token_service: Arc<TokenService>,
        config: WebSocketConfig,
    ) -> Self {
        Self {
            connection_manager,
            message_router,
            token_service,
            config,
        }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(
        &self,
        socket: WebSocket,
        token: Option<String>,
    ) -> Result<(), WebSocketError> {
        // Authenticate user
        let user_id = self.authenticate(token).await?;
        
        // Create connection state
        let state = ConnectionState::new(user_id);
        let conn_id = state.conn_id.clone();
        
        tracing::info!(
            user_id = %user_id,
            conn_id = %conn_id,
            "WebSocket connection established"
        );

        // Split socket into sender and receiver
        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Create channel for outgoing messages
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        // Register connection
        self.connection_manager
            .register_connection_async(user_id, conn_id.clone(), tx)
            .await;

        // Spawn task to forward messages to WebSocket
        let conn_id_clone = conn_id.clone();
        let sender_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let json = match serde_json::to_string(&message) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };
                
                if ws_sender.send(WsMessage::Text(json)).await.is_err() {
                    tracing::warn!(conn_id = %conn_id_clone, "Failed to send message to WebSocket");
                    break;
                }
            }
        });

        // Handle incoming messages
        let heartbeat_timeout = Duration::from_secs(self.config.heartbeat_timeout_secs);
        let mut last_heartbeat = Instant::now();

        loop {
            tokio::select! {
                // Check for incoming messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            last_heartbeat = Instant::now();
                            if let Err(e) = self.handle_text_message(&text, user_id).await {
                                tracing::warn!(
                                    user_id = %user_id,
                                    error = %e,
                                    "Error handling message"
                                );
                            }
                        }
                        Some(Ok(WsMessage::Binary(data))) => {
                            last_heartbeat = Instant::now();
                            if let Err(e) = self.handle_binary_message(&data, user_id).await {
                                tracing::warn!(
                                    user_id = %user_id,
                                    error = %e,
                                    "Error handling binary message"
                                );
                            }
                        }
                        Some(Ok(WsMessage::Ping(_))) => {
                            last_heartbeat = Instant::now();
                        }
                        Some(Ok(WsMessage::Pong(_))) => {
                            last_heartbeat = Instant::now();
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            tracing::info!(
                                user_id = %user_id,
                                conn_id = %conn_id,
                                "WebSocket connection closed by client"
                            );
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!(
                                user_id = %user_id,
                                error = %e,
                                "WebSocket error"
                            );
                            break;
                        }
                        None => {
                            tracing::info!(
                                user_id = %user_id,
                                conn_id = %conn_id,
                                "WebSocket stream ended"
                            );
                            break;
                        }
                    }
                }
                // Check heartbeat timeout
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    if last_heartbeat.elapsed() > heartbeat_timeout {
                        tracing::warn!(
                            user_id = %user_id,
                            conn_id = %conn_id,
                            "Heartbeat timeout, closing connection"
                        );
                        break;
                    }
                }
            }
        }

        // Cleanup
        sender_task.abort();
        self.connection_manager.unregister_connection_async(&conn_id).await;
        
        tracing::info!(
            user_id = %user_id,
            conn_id = %conn_id,
            "WebSocket connection cleaned up"
        );

        Ok(())
    }

    /// Authenticate user from token
    async fn authenticate(&self, token: Option<String>) -> Result<Uuid, WebSocketError> {
        let token = token.ok_or_else(|| {
            WebSocketError::AuthenticationFailed("No token provided".to_string())
        })?;

        // Verify token with timeout
        let auth_timeout = Duration::from_secs(self.config.auth_timeout_secs);
        
        let result = tokio::time::timeout(auth_timeout, async {
            self.token_service.verify_token(&token)
        })
        .await
        .map_err(|_| WebSocketError::AuthenticationTimeout)?;

        let claims = result.map_err(|e| {
            WebSocketError::AuthenticationFailed(e.to_string())
        })?;

        Uuid::parse_str(&claims.sub).map_err(|e| {
            WebSocketError::AuthenticationFailed(format!("Invalid user ID: {}", e))
        })
    }

    /// Handle text message
    async fn handle_text_message(&self, text: &str, sender_id: Uuid) -> Result<(), WebSocketError> {
        // Try to parse as JSON message
        let msg: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| WebSocketError::InvalidMessage(e.to_string()))?;

        // Check message type
        if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
            match msg_type {
                "heartbeat" => {
                    // Heartbeat received, nothing to do
                    Ok(())
                }
                "message" => {
                    self.handle_chat_message(&msg, sender_id).await
                }
                "sync" => {
                    // Handle sync request
                    self.handle_sync_request(&msg, sender_id).await
                }
                _ => {
                    Err(WebSocketError::InvalidMessage(format!("Unknown message type: {}", msg_type)))
                }
            }
        } else {
            // Assume it's a chat message
            self.handle_chat_message(&msg, sender_id).await
        }
    }

    /// Handle binary message (protobuf)
    async fn handle_binary_message(&self, _data: &[u8], _sender_id: Uuid) -> Result<(), WebSocketError> {
        // TODO: Implement protobuf deserialization
        // For now, we only support JSON
        Err(WebSocketError::InvalidMessage("Binary messages not yet supported".to_string()))
    }

    /// Handle chat message
    async fn handle_chat_message(&self, msg: &serde_json::Value, sender_id: Uuid) -> Result<(), WebSocketError> {
        // Extract message fields
        let receiver_id = msg.get("receiver_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let group_id = msg.get("group_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        let content = msg.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WebSocketError::InvalidMessage("Missing content".to_string()))?;
        
        let msg_type = msg.get("msg_type")
            .and_then(|v| v.as_i64())
            .map(|t| MsgType::from(t as i32))
            .unwrap_or(MsgType::Text);

        let message_id = msg.get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        // Create message
        let create_msg = CreateMessage {
            id: message_id,
            sender_id,
            receiver_id,
            group_id,
            msg_type,
            content: content.to_string(),
            is_ai_generated: false,
        };

        // Route message
        match self.message_router.route_message(create_msg).await {
            Ok(ack) => {
                tracing::debug!(
                    message_id = ?ack.message_id,
                    status = ?ack.status,
                    "Message routed successfully"
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to route message");
                Err(WebSocketError::InternalError(e.to_string()))
            }
        }
    }

    /// Handle sync request
    async fn handle_sync_request(&self, _msg: &serde_json::Value, _user_id: Uuid) -> Result<(), WebSocketError> {
        // TODO: Implement offline message sync
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_new() {
        let user_id = Uuid::new_v4();
        let state = ConnectionState::new(user_id);
        
        assert_eq!(state.user_id, user_id);
        assert!(state.authenticated);
        assert!(!state.conn_id.is_empty());
    }

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        
        assert_eq!(config.auth_timeout_secs, 3);
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.heartbeat_timeout_secs, 10);
        assert_eq!(config.max_message_size, 1024 * 1024);
    }

    #[test]
    fn test_connection_state_unique_ids() {
        let user_id = Uuid::new_v4();
        let state1 = ConnectionState::new(user_id);
        let state2 = ConnectionState::new(user_id);
        
        // Same user, different connection IDs
        assert_eq!(state1.user_id, state2.user_id);
        assert_ne!(state1.conn_id, state2.conn_id);
    }
}
