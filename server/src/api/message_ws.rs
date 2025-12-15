//! WebSocket API endpoint for real-time messaging

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::services::message::{MessageRouter, WebSocketHandler};

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// JWT token for authentication
    pub token: Option<String>,
}

/// Application state for WebSocket handler
#[derive(Clone)]
pub struct WsAppState<R: MessageRouter> {
    pub ws_handler: Arc<WebSocketHandler<R>>,
}



/// Handle WebSocket connection
async fn handle_socket<R: MessageRouter>(socket: WebSocket, token: Option<String>, handler: Arc<WebSocketHandler<R>>) {
    if let Err(e) = handler.handle_connection(socket, token).await {
        tracing::error!("WebSocket connection error: {}", e);
    }
}

/// Create WebSocket router
pub fn create_ws_router<R: MessageRouter + Clone + 'static>(ws_handler: Arc<WebSocketHandler<R>>) -> axum::Router {
    use axum::routing::get;
    
    let state = WsAppState { ws_handler };
    
    axum::Router::new()
        .route("/ws", get(ws_handler_endpoint::<R>))
        .with_state(state)
}

/// WebSocket endpoint handler
async fn ws_handler_endpoint<R: MessageRouter + 'static>(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<WsAppState<R>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, query.token, state.ws_handler))
}
