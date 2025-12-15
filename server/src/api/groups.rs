use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::UserContext;
use crate::models::{
    CreateGroup, Group, GroupDetails, GroupMemberInfo, GroupSearchResult, InviteCode,
    MemberRole, UpdateGroupSettings,
};
use crate::response::ApiResponse;
use crate::services::{GroupService, InviteService, MemberService};

/// Pagination query params
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

/// Search query params
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

/// Invite code generation request
#[derive(Debug, Deserialize)]
pub struct GenerateInviteCodeRequest {
    pub expires_in_hours: Option<i64>,
    pub max_uses: Option<i32>,
}

/// Role update request
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: MemberRole,
}

/// Ban request
#[derive(Debug, Deserialize)]
pub struct BanRequest {
    pub reason: Option<String>,
}

/// Join via code request
#[derive(Debug, Deserialize)]
pub struct JoinViaCodeRequest {
    pub code: String,
}

/// Add member request
#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

/// Message response for simple operations
#[derive(Debug, serde::Serialize)]
pub struct MessageData {
    pub message: String,
}

/// Create group routes
pub fn group_routes(pool: PgPool) -> Router {
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    let invite_service = InviteService::new(pool.clone());

    Router::new()
        // Group CRUD
        .route("/", post(create_group))
        .route("/search", get(search_groups))
        .route("/my", get(get_my_groups))
        .route("/join", post(join_via_code))
        .route("/:group_id", get(get_group_details))
        .route("/:group_id", put(update_group))
        .route("/:group_id", delete(dissolve_group))
        // Members
        .route("/:group_id/members", get(get_members))
        .route("/:group_id/members", post(add_member))
        .route("/:group_id/members/:user_id", delete(remove_member))
        .route("/:group_id/members/:user_id/role", put(update_role))
        .route("/:group_id/members/:user_id/mute", post(mute_member))
        .route("/:group_id/members/:user_id/unmute", post(unmute_member))
        .route("/:group_id/members/:user_id/ban", post(ban_member))
        .route("/:group_id/members/:user_id/unban", post(unban_member))
        .route("/:group_id/leave", post(leave_group))
        // Invite codes
        .route("/:group_id/invite-codes", get(get_invite_codes))
        .route("/:group_id/invite-codes", post(generate_invite_code))
        .route("/:group_id/invite-codes/:code", delete(revoke_invite_code))
        .with_state((group_service, member_service, invite_service))
}

type GroupState = (GroupService, MemberService, InviteService);

/// Create a new group
async fn create_group(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(request): Json<CreateGroup>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let group = group_service.create_group(user_ctx.user_id, request).await?;
    Ok(Json(ApiResponse::success(group)))
}

/// Get group details
async fn get_group_details(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<GroupDetails>>, AppError> {
    let details = group_service.get_group_details(group_id, Some(user_ctx.user_id)).await?;
    Ok(Json(ApiResponse::success(details)))
}

/// Update group settings
async fn update_group(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<UpdateGroupSettings>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let group = group_service.update_group_settings(group_id, user_ctx.user_id, request).await?;
    Ok(Json(ApiResponse::success(group)))
}

/// Dissolve a group
async fn dissolve_group(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    group_service.dissolve_group(group_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Group dissolved".to_string() })))
}

/// Search public groups
async fn search_groups(
    State((group_service, _, _)): State<GroupState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<GroupSearchResult>>>, AppError> {
    let limit = query.limit.unwrap_or(20);
    let groups = group_service.search_groups(&query.q, limit).await?;
    Ok(Json(ApiResponse::success(groups)))
}

/// Get user's groups
async fn get_my_groups(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<ApiResponse<Vec<Group>>>, AppError> {
    let groups = group_service.get_user_groups(user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(groups)))
}


/// Get group members
async fn get_members(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<GroupMemberInfo>>>, AppError> {
    let page = pagination.page.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(100);
    let members = member_service.get_group_members(group_id, user_ctx.user_id, page, limit).await?;
    Ok(Json(ApiResponse::success(members)))
}

/// Add a member to group
async fn add_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<AddMemberRequest>,
) -> Result<Json<ApiResponse<crate::models::GroupMember>>, AppError> {
    let member = member_service.add_member(group_id, request.user_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(member)))
}

/// Remove a member from group
async fn remove_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    member_service.remove_member(group_id, target_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Member removed".to_string() })))
}

/// Update member role
async fn update_role(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<ApiResponse<crate::models::GroupMember>>, AppError> {
    let member = member_service.update_member_role(group_id, target_id, request.role, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(member)))
}

/// Mute a member
async fn mute_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    member_service.mute_member(group_id, target_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Member muted".to_string() })))
}

/// Unmute a member
async fn unmute_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    member_service.unmute_member(group_id, target_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Member unmuted".to_string() })))
}

/// Ban a member
async fn ban_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<BanRequest>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    member_service.ban_member(group_id, target_id, user_ctx.user_id, request.reason).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Member banned".to_string() })))
}

/// Unban a member
async fn unban_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    member_service.unban_member(group_id, target_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Member unbanned".to_string() })))
}

/// Leave a group
async fn leave_group(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    member_service.leave_group(group_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Left group".to_string() })))
}


/// Get invite codes for a group
async fn get_invite_codes(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<InviteCode>>>, AppError> {
    let codes = invite_service.get_group_invite_codes(group_id, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(codes)))
}

/// Generate an invite code
async fn generate_invite_code(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<GenerateInviteCodeRequest>,
) -> Result<Json<ApiResponse<InviteCode>>, AppError> {
    let code = invite_service
        .generate_invite_code(group_id, user_ctx.user_id, request.expires_in_hours, request.max_uses)
        .await?;
    Ok(Json(ApiResponse::success(code)))
}

/// Revoke an invite code
async fn revoke_invite_code(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Path((group_id, code)): Path<(Uuid, String)>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    let _ = group_id; // kept for REST consistency
    invite_service.revoke_invite_code(&code, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(MessageData { message: "Invite code revoked".to_string() })))
}

/// Join a group via invite code
async fn join_via_code(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(request): Json<JoinViaCodeRequest>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let group = invite_service.join_via_invite_code(&request.code, user_ctx.user_id).await?;
    Ok(Json(ApiResponse::success(group)))
}
