use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AIAgent, AgentStatistics, CreateAIAgent, UpdatePersonality};
use crate::services::AIAgentService;

/// API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// User affinity update request
#[derive(Debug, Deserialize)]
pub struct SetAffinityRequest {
    pub user_id: Uuid,
    pub affinity_score: i16,
}

/// Create AI Agent routes
pub fn ai_agent_routes(pool: PgPool) -> Router {
    let service = AIAgentService::new(pool);

    Router::new()
        .route("/groups/:group_id/agent", post(create_agent))
        .route("/groups/:group_id/agent", get(get_agent_for_group))
        .route("/agents/:agent_id", get(get_agent))
        .route("/agents/:agent_id", delete(delete_agent))
        .route("/agents/:agent_id/personality", put(update_personality))
        .route("/agents/:agent_id/stats", get(get_statistics))
        .route("/agents/:agent_id/affinity", post(set_affinity))
        .with_state(service)
}

/// Create a new AI Agent for a group
async fn create_agent(
    State(service): State<AIAgentService>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<CreateAIAgent>,
) -> impl IntoResponse {
    // TODO: Check if user is group owner
    let _ = user_id;

    match service.create_agent(group_id, request).await {
        Ok(agent) => (StatusCode::CREATED, Json(ApiResponse::success(agent))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<AIAgent>::error(e.to_string())),
        ),
    }
}

/// Get AI Agent by ID
async fn get_agent(
    State(service): State<AIAgentService>,
    Path(agent_id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_agent(agent_id).await {
        Ok(agent) => (StatusCode::OK, Json(ApiResponse::success(agent))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<AIAgent>::error(e.to_string())),
        ),
    }
}

/// Get AI Agent for a group
async fn get_agent_for_group(
    State(service): State<AIAgentService>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_agent_for_group(group_id).await {
        Ok(agent) => (StatusCode::OK, Json(ApiResponse::success(agent))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Option<AIAgent>>::error(e.to_string())),
        ),
    }
}

/// Delete AI Agent
async fn delete_agent(
    State(service): State<AIAgentService>,
    Extension(user_id): Extension<Uuid>,
    Path(agent_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Check if user is group owner
    let _ = user_id;

    match service.delete_agent(agent_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Agent deleted"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Update AI Agent personality
async fn update_personality(
    State(service): State<AIAgentService>,
    Extension(user_id): Extension<Uuid>,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<UpdatePersonality>,
) -> impl IntoResponse {
    // TODO: Check if user is group owner
    let _ = user_id;

    match service.update_personality(agent_id, request).await {
        Ok(agent) => (StatusCode::OK, Json(ApiResponse::success(agent))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<AIAgent>::error(e.to_string())),
        ),
    }
}

/// Get AI Agent statistics
async fn get_statistics(
    State(service): State<AIAgentService>,
    Path(agent_id): Path<Uuid>,
) -> impl IntoResponse {
    match service.get_statistics(agent_id).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<AgentStatistics>::error(e.to_string())),
        ),
    }
}

/// Set user affinity for an agent
async fn set_affinity(
    State(service): State<AIAgentService>,
    Extension(user_id): Extension<Uuid>,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<SetAffinityRequest>,
) -> impl IntoResponse {
    // TODO: Check if user is group owner
    let _ = user_id;

    match service
        .set_user_affinity(agent_id, request.user_id, request.affinity_score)
        .await
    {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Affinity set"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}
