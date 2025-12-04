use crate::middleware::UserContext;
use crate::services::presence::{PresenceService, PresenceUpdate};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Connection manager for tracking active WebSocket connections
#[derive(Clone)]
pub struct ConnectionManager {
    /// Map of user_id to list of connection shutdown senders
    connections: Arc<RwLock<HashMap<Uuid, Vec<broadcast::Sender<()>>>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection for a user
    pub async fn register_connection(&self, user_id: Uuid) -> broadcast::Receiver<()> {
        let (tx, rx) = broadcast::channel(1);
        let mut connections = self.connections.write().await;
        connections.entry(user_id).or_insert_with(Vec::new).push(tx);
        rx
    }

    /// Unregister a connection for a user
    pub async fn unregister_connection(&self, user_id: &Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(user_connections) = connections.get_mut(user_id) {
            // Remove closed channels
            user_connections.retain(|tx| tx.receiver_count() > 0);
            if user_connections.is_empty() {
                connections.remove(user_id);
            }
        }
    }

    /// Close all connections for a user (e.g., on logout)
    pub async fn close_user_connections(&self, user_id: &Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(user_connections) = connections.remove(user_id) {
            // Send shutdown signal to all connections
            for tx in user_connections {
                let _ = tx.send(());
            }
        }
    }

    /// Get the number of active connections for a user
    pub async fn connection_count(&self, user_id: &Uuid) -> usize {
        let connections = self.connections.read().await;
        connections
            .get(user_id)
            .map(|conns| conns.iter().filter(|tx| tx.receiver_count() > 0).count())
            .unwrap_or(0)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket application state
#[derive(Clone)]
pub struct WsState {
    pub presence_service: Arc<PresenceService>,
    pub presence_tx: broadcast::Sender<PresenceUpdate>,
    pub connection_manager: Arc<ConnectionManager>,
}

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Heartbeat ping
    Ping,
    /// Heartbeat pong
    Pong,
    /// Presence update notification
    PresenceUpdate(PresenceUpdate),
    /// Subscribe to users' presence
    Subscribe { user_ids: Vec<Uuid> },
    /// Unsubscribe from users' presence
    Unsubscribe { user_ids: Vec<Uuid> },
    /// Error message
    Error { message: String },
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
    Extension(user_context): Extension<UserContext>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, user_context))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: WsState, user_context: UserContext) {
    let user_id = user_context.user_id;
    
    // Register this connection with the connection manager
    let mut shutdown_rx = state.connection_manager.register_connection(user_id).await;
    
    // Mark user as online
    if let Err(e) = state.presence_service.user_connected(user_id).await {
        tracing::error!("Failed to mark user {} as online: {}", user_id, e);
        state.connection_manager.unregister_connection(&user_id).await;
        return;
    }

    // Broadcast presence update
    let update = PresenceUpdate {
        user_id,
        status: crate::models::user::OnlineStatus::Online,
        timestamp: chrono::Utc::now(),
    };
    let _ = state.presence_tx.send(update);

    tracing::info!("WebSocket connected: user {}", user_id);

    let (mut sender, mut receiver) = socket.split();
    let mut presence_rx = state.presence_tx.subscribe();

    // Spawn task to forward presence updates to client
    let send_task = tokio::spawn(async move {
        while let Ok(update) = presence_rx.recv().await {
            let msg = WsMessage::PresenceUpdate(update);
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Ping => {
                                // Update heartbeat
                                let _ = state_clone.presence_service.heartbeat(user_id).await;
                            }
                            _ => {}
                        }
                    }
                }
                Message::Ping(_) => {
                    let _ = state_clone.presence_service.heartbeat(user_id).await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete or shutdown signal
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
        _ = shutdown_rx.recv() => {
            tracing::info!("WebSocket connection closed by server (logout): user {}", user_id);
        },
    }

    // Unregister this connection
    state.connection_manager.unregister_connection(&user_id).await;

    // Mark user as offline
    if let Ok(Some(update)) = state.presence_service.user_disconnected(user_id).await {
        let _ = state.presence_tx.send(update);
    }

    tracing::info!("WebSocket disconnected: user {}", user_id);
}
