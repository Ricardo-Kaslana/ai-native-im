use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::UserContext;
use crate::models::{
    AIAgent, AgentStatistics, CreateAIAgent, UpdatePersonality,
    TestPersonaRequest, TestPersonaResponse,
};
use crate::response::ApiResponse;
use crate::services::ai::persona::{PersonaPresets, PresetInfo};
use crate::services::ai::user_profile::{
    UserProfile, UserProfileService, UpdateUserProfile, InteractionEvent,
    InteractionType, Sentiment,
};
use crate::services::ai::summary::{
    DefaultSummarizeService, Summary, SummaryRequest, SummaryStyle, SummarizeService,
};
use crate::services::ai::learning::{
    FeedbackStats, FeedbackType, InMemoryLearningEngine, LearningConfig,
    LearningEngine, LearningError,
};
use crate::services::AIAgentService;

/// User affinity update request
#[derive(Debug, Deserialize)]
pub struct SetAffinityRequest {
    pub user_id: Uuid,
    pub affinity_score: i16,
}

/// Summary request for API
#[derive(Debug, Deserialize)]
pub struct SummaryApiRequest {
    /// Maximum number of messages to include (default: 100)
    #[serde(default = "default_message_count")]
    pub message_count: usize,
    /// Summary style (brief, detailed, topics, action_items)
    #[serde(default)]
    pub style: SummaryStyle,
    /// Whether to use cache (default: true)
    #[serde(default = "default_use_cache")]
    pub use_cache: bool,
}

fn default_message_count() -> usize {
    100
}

fn default_use_cache() -> bool {
    true
}

/// Feedback request for API
#[derive(Debug, Deserialize)]
pub struct FeedbackApiRequest {
    /// The response message ID that feedback is for
    pub response_id: Uuid,
    /// Type of feedback
    pub feedback_type: String,
    /// Optional correction text
    pub correction: Option<String>,
}

/// Feedback response
#[derive(Debug, Serialize)]
pub struct FeedbackResponse {
    pub recorded: bool,
    pub message: String,
}

/// Message response for simple operations
#[derive(Debug, Serialize)]
pub struct MessageData {
    pub message: String,
}

/// AI Agent state type
type AIAgentState = (AIAgentService, Arc<dyn SummarizeService>, Arc<dyn LearningEngine>, PgPool);

/// Create AI Agent routes
pub fn ai_agent_routes(pool: PgPool) -> Router {
    let agent_service = AIAgentService::new(pool.clone());
    
    // Create summary service with a mock LLM client for now
    // In production, this should be configured with a real LLM client
    let llm_client: Arc<dyn crate::services::ai::LLMClient> = Arc::new(
        crate::services::MockLLMClient::new("消息总结功能已启用。".to_string())
    );
    let summary_service: Arc<dyn SummarizeService> = Arc::new(
        DefaultSummarizeService::new(llm_client)
    );
    
    // Create learning engine
    let learning_engine: Arc<dyn LearningEngine> = Arc::new(
        InMemoryLearningEngine::new(LearningConfig::default())
    );

    Router::new()
        // Agent CRUD
        .route("/groups/:group_id/agent", post(create_agent))
        .route("/groups/:group_id/agent", get(get_agent_for_group))
        .route("/agents/:agent_id", get(get_agent))
        .route("/agents/:agent_id", delete(delete_agent))
        .route("/agents/:agent_id/personality", put(update_personality))
        .route("/agents/:agent_id/stats", get(get_statistics))
        .route("/agents/:agent_id/affinity", post(set_affinity))
        // Summary
        .route("/groups/:group_id/summary", post(generate_summary))
        // Feedback
        .route("/messages/:message_id/feedback", post(record_feedback))
        .route("/groups/:group_id/feedback/stats", get(get_feedback_stats))
        // Persona customization
        .route("/agents/presets", get(get_persona_presets))
        .route("/agents/:agent_id/test", post(test_persona))
        // User profiles
        .route("/agents/:agent_id/profiles/:user_id", get(get_user_profile))
        .route("/agents/:agent_id/profiles/:user_id", put(update_user_profile))
        .route("/agents/:agent_id/profiles", get(list_user_profiles))
        .with_state((agent_service, summary_service, learning_engine, pool))
}

/// Create AI Agent routes with custom LLM client
pub fn ai_agent_routes_with_llm(pool: PgPool, llm_client: Arc<dyn crate::services::ai::LLMClient>) -> Router {
    let agent_service = AIAgentService::new(pool.clone());
    let summary_service: Arc<dyn SummarizeService> = Arc::new(
        DefaultSummarizeService::new(llm_client)
    );
    
    // Create learning engine
    let learning_engine: Arc<dyn LearningEngine> = Arc::new(
        InMemoryLearningEngine::new(LearningConfig::default())
    );

    Router::new()
        // Agent CRUD
        .route("/groups/:group_id/agent", post(create_agent))
        .route("/groups/:group_id/agent", get(get_agent_for_group))
        .route("/agents/:agent_id", get(get_agent))
        .route("/agents/:agent_id", delete(delete_agent))
        .route("/agents/:agent_id/personality", put(update_personality))
        .route("/agents/:agent_id/stats", get(get_statistics))
        .route("/agents/:agent_id/affinity", post(set_affinity))
        // Summary
        .route("/groups/:group_id/summary", post(generate_summary))
        // Feedback
        .route("/messages/:message_id/feedback", post(record_feedback))
        .route("/groups/:group_id/feedback/stats", get(get_feedback_stats))
        // Persona customization
        .route("/agents/presets", get(get_persona_presets))
        .route("/agents/:agent_id/test", post(test_persona))
        // User profiles
        .route("/agents/:agent_id/profiles/:user_id", get(get_user_profile))
        .route("/agents/:agent_id/profiles/:user_id", put(update_user_profile))
        .route("/agents/:agent_id/profiles", get(list_user_profiles))
        .with_state((agent_service, summary_service, learning_engine, pool))
}

/// Create a new AI Agent for a group
async fn create_agent(
    State((service, _, _, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<CreateAIAgent>,
) -> Result<Json<ApiResponse<AIAgent>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group owner
    let agent = service.create_agent(group_id, request).await?;
    Ok(Json(ApiResponse::success(agent)))
}


/// Get AI Agent by ID
async fn get_agent(
    State((service, _, _, _)): State<AIAgentState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<ApiResponse<AIAgent>>, AppError> {
    let agent = service.get_agent(agent_id).await?;
    Ok(Json(ApiResponse::success(agent)))
}

/// Get AI Agent for a group
async fn get_agent_for_group(
    State((service, _, _, _)): State<AIAgentState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Option<AIAgent>>>, AppError> {
    let agent = service.get_agent_for_group(group_id).await?;
    Ok(Json(ApiResponse::success(agent)))
}

/// Delete AI Agent
async fn delete_agent(
    State((service, _, _, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group owner
    service.delete_agent(agent_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Agent deleted".to_string() })))
}

/// Update AI Agent personality
async fn update_personality(
    State((service, _, _, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<UpdatePersonality>,
) -> Result<Json<ApiResponse<AIAgent>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group owner
    let agent = service.update_personality(agent_id, request).await?;
    Ok(Json(ApiResponse::success(agent)))
}

/// Get AI Agent statistics
async fn get_statistics(
    State((service, _, _, _)): State<AIAgentState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<ApiResponse<AgentStatistics>>, AppError> {
    let stats = service.get_statistics(agent_id).await?;
    Ok(Json(ApiResponse::success(stats)))
}

// ============= User Profile API Endpoints =============

/// User profile path parameters
#[derive(Debug, Deserialize)]
struct ProfilePathParams {
    agent_id: Uuid,
    user_id: Uuid,
}

/// Get user profile for an agent
/// 
/// GET /api/ai/agents/:agent_id/profiles/:user_id
async fn get_user_profile(
    State((_, _, _, pool)): State<AIAgentState>,
    Extension(_user_ctx): Extension<UserContext>,
    Path((agent_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let service = UserProfileService::new(pool);
    let profile = service.get_or_create_profile(agent_id, user_id).await
        .map_err(|e| AppError::Internal(format!("Failed to get user profile: {}", e)))?;
    Ok(Json(ApiResponse::success(profile)))
}

/// Update user profile for an agent
/// 
/// PUT /api/ai/agents/:agent_id/profiles/:user_id
async fn update_user_profile(
    State((_, _, _, pool)): State<AIAgentState>,
    Extension(_user_ctx): Extension<UserContext>,
    Path((agent_id, user_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateUserProfile>,
) -> Result<Json<ApiResponse<UserProfile>>, AppError> {
    let service = UserProfileService::new(pool);
    let profile = service.update_profile(agent_id, user_id, request).await
        .map_err(|e| AppError::Internal(format!("Failed to update user profile: {}", e)))?;
    Ok(Json(ApiResponse::success(profile)))
}

/// List all user profiles for an agent
/// 
/// GET /api/ai/agents/:agent_id/profiles
async fn list_user_profiles(
    State((_, _, _, pool)): State<AIAgentState>,
    Extension(_user_ctx): Extension<UserContext>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<UserProfile>>>, AppError> {
    let service = UserProfileService::new(pool);
    let profiles = service.get_profiles_for_agent(agent_id).await
        .map_err(|e| AppError::Internal(format!("Failed to list user profiles: {}", e)))?;
    Ok(Json(ApiResponse::success(profiles)))
}

/// Get all available persona presets
/// 
/// GET /api/ai/agents/presets
/// 
/// Returns a list of predefined persona templates that users can apply.
async fn get_persona_presets() -> Result<Json<ApiResponse<Vec<PresetInfo>>>, AppError> {
    let presets = PersonaPresets::list();
    Ok(Json(ApiResponse::success(presets)))
}

/// Test a persona configuration with a sample message
/// 
/// POST /api/ai/agents/:agent_id/test
/// 
/// Allows users to preview how their configured persona would respond.
async fn test_persona(
    State((service, _, _, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<TestPersonaRequest>,
) -> Result<Json<ApiResponse<TestPersonaResponse>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group owner
    let start = std::time::Instant::now();
    
    // Get the agent to validate it exists
    let agent = service.get_agent(agent_id).await?;
    
    // Generate a test response based on persona settings
    let response = generate_test_response(&agent, &request.message, request.sender_name.as_deref());
    
    Ok(Json(ApiResponse::success(TestPersonaResponse {
        response,
        emotion: "neutral".to_string(),
        processing_time_ms: start.elapsed().as_millis() as u64,
    })))
}

/// Generate a test response based on agent persona
fn generate_test_response(agent: &AIAgent, message: &str, sender_name: Option<&str>) -> String {
    let sender = sender_name.unwrap_or("用户");
    
    // Build response based on personality traits
    let tone = match agent.toxicity_level {
        0..=2 => "亲切",
        3..=5 => "随和",
        6..=8 => "毒舌",
        _ => "犯贱",
    };
    
    let energy = match agent.enthusiasm_level {
        0..=3 => "冷淡",
        4..=6 => "温和",
        7..=10 => "热情",
        _ => "正常",
    };
    
    let suffix = agent.speech_suffix.as_deref().unwrap_or("");
    
    format!(
        "[测试响应] 我是{}，性格{}且{}。\n收到{}的消息：\"{}\"\n这是一个人设预览，实际对话中我会更加自然{}",
        agent.name, tone, energy, sender, message, suffix
    )
}

/// Set user affinity for an agent
async fn set_affinity(
    State((service, _, _, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(agent_id): Path<Uuid>,
    Json(request): Json<SetAffinityRequest>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group owner
    service.set_user_affinity(agent_id, request.user_id, request.affinity_score).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Affinity set".to_string() })))
}

/// Generate a summary for a group's messages
/// 
/// POST /api/ai/groups/:group_id/summary
/// 
/// **Feature: phase3-ai-agent-evolution**
/// **Requirements: 5.1**
async fn generate_summary(
    State((_, summary_service, _, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<SummaryApiRequest>,
) -> Result<Json<ApiResponse<Summary>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group member
    
    let summary_request = SummaryRequest {
        group_id,
        message_count: request.message_count,
        since: None,
        style: request.style,
        use_cache: request.use_cache,
    };
    
    let summary = summary_service.summarize(summary_request).await?;
    Ok(Json(ApiResponse::success(summary)))
}

/// Record feedback for a message response
/// 
/// POST /api/ai/messages/:message_id/feedback
/// 
/// **Feature: phase3-ai-agent-evolution**
/// **Requirements: 18.1, 18.2**
async fn record_feedback(
    State((_, _, learning_engine, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(message_id): Path<Uuid>,
    Json(request): Json<FeedbackApiRequest>,
) -> Result<Json<ApiResponse<FeedbackResponse>>, AppError> {
    let feedback_type = match request.feedback_type.to_lowercase().as_str() {
        "positive" => FeedbackType::Positive,
        "negative" => FeedbackType::Negative,
        "correction" => FeedbackType::Correction,
        "ignored" => FeedbackType::Ignored,
        _ => return Err(AppError::BadRequest(
            "Invalid feedback type. Must be: positive, negative, correction, or ignored".to_string()
        )),
    };
    
    // For now, we use a placeholder group_id since we don't have message lookup
    // In production, this should look up the message to get the group_id
    let group_id = Uuid::nil(); // TODO: Look up message to get group_id
    
    learning_engine
        .record_feedback(
            message_id,
            request.response_id,
            user_ctx.user_id,
            group_id,
            feedback_type,
            request.correction,
        )
        .await
        .map_err(|e| match e {
            LearningError::LearningDisabled => AppError::BadRequest("Learning is disabled".to_string()),
            _ => AppError::Internal(format!("Failed to record feedback: {}", e)),
        })?;
    
    Ok(Json(ApiResponse::success(FeedbackResponse {
        recorded: true,
        message: "反馈已记录，感谢您的帮助！".to_string(),
    })))
}

/// Get feedback statistics for a group
/// 
/// GET /api/ai/groups/:group_id/feedback/stats
/// 
/// **Feature: phase3-ai-agent-evolution**
/// **Requirements: 18.3**
async fn get_feedback_stats(
    State((_, _, learning_engine, _)): State<AIAgentState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<FeedbackStats>>, AppError> {
    let _ = user_ctx.user_id; // TODO: Check if user is group member
    
    let stats = learning_engine
        .get_feedback_stats(group_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get feedback stats: {}", e)))?;
    
    Ok(Json(ApiResponse::success(stats)))
}
