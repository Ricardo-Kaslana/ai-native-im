use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateGroup, Group, GroupDetails, GroupMemberInfo, GroupSearchResult, InviteCode,
    MemberRole, UpdateGroupSettings,
};
use crate::services::{GroupService, InviteService, MemberService};

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
    Extension(user_id): Extension<Uuid>,
    Json(request): Json<CreateGroup>,
) -> impl IntoResponse {
    match group_service.create_group(user_id, request).await {
        Ok(group) => (StatusCode::CREATED, Json(ApiResponse::success(group))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Group>::error(e.to_string())),
        ),
    }
}

/// Get group details
async fn get_group_details(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match group_service.get_group_details(group_id, Some(user_id)).await {
        Ok(details) => (StatusCode::OK, Json(ApiResponse::success(details))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<GroupDetails>::error(e.to_string())),
        ),
    }
}


/// Update group settings
async fn update_group(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<UpdateGroupSettings>,
) -> impl IntoResponse {
    match group_service.update_group_settings(group_id, user_id, request).await {
        Ok(group) => (StatusCode::OK, Json(ApiResponse::success(group))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Group>::error(e.to_string())),
        ),
    }
}

/// Dissolve a group
async fn dissolve_group(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match group_service.dissolve_group(group_id, user_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Group dissolved"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Search public groups
async fn search_groups(
    State((group_service, _, _)): State<GroupState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    match group_service.search_groups(&query.q, limit).await {
        Ok(groups) => (StatusCode::OK, Json(ApiResponse::success(groups))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Vec<GroupSearchResult>>::error(e.to_string())),
        ),
    }
}

/// Get user's groups
async fn get_my_groups(
    State((group_service, _, _)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
) -> impl IntoResponse {
    match group_service.get_user_groups(user_id).await {
        Ok(groups) => (StatusCode::OK, Json(ApiResponse::success(groups))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Vec<Group>>::error(e.to_string())),
        ),
    }
}

/// Get group members
async fn get_members(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(100);
    match member_service.get_group_members(group_id, user_id, page, limit).await {
        Ok(members) => (StatusCode::OK, Json(ApiResponse::success(members))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Vec<GroupMemberInfo>>::error(e.to_string())),
        ),
    }
}

/// Add member request
#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

/// Add a member to group
async fn add_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<AddMemberRequest>,
) -> impl IntoResponse {
    match member_service.add_member(group_id, request.user_id, actor_id).await {
        Ok(member) => (StatusCode::CREATED, Json(ApiResponse::success(member))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<crate::models::GroupMember>::error(e.to_string())),
        ),
    }
}

/// Remove a member from group
async fn remove_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match member_service.remove_member(group_id, target_id, actor_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Member removed"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Update member role
async fn update_role(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    match member_service.update_member_role(group_id, target_id, request.role, actor_id).await {
        Ok(member) => (StatusCode::OK, Json(ApiResponse::success(member))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<crate::models::GroupMember>::error(e.to_string())),
        ),
    }
}

/// Mute a member
async fn mute_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match member_service.mute_member(group_id, target_id, actor_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Member muted"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Unmute a member
async fn unmute_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match member_service.unmute_member(group_id, target_id, actor_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Member unmuted"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}


/// Ban a member
async fn ban_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<BanRequest>,
) -> impl IntoResponse {
    match member_service.ban_member(group_id, target_id, actor_id, request.reason).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Member banned"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Unban a member
async fn unban_member(
    State((_, member_service, _)): State<GroupState>,
    Extension(actor_id): Extension<Uuid>,
    Path((group_id, target_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match member_service.unban_member(group_id, target_id, actor_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Member unbanned"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Leave a group
async fn leave_group(
    State((_, member_service, _)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match member_service.leave_group(group_id, user_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Left group"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Get invite codes for a group
async fn get_invite_codes(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
) -> impl IntoResponse {
    match invite_service.get_group_invite_codes(group_id, user_id).await {
        Ok(codes) => (StatusCode::OK, Json(ApiResponse::success(codes))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Vec<InviteCode>>::error(e.to_string())),
        ),
    }
}

/// Generate an invite code
async fn generate_invite_code(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path(group_id): Path<Uuid>,
    Json(request): Json<GenerateInviteCodeRequest>,
) -> impl IntoResponse {
    match invite_service
        .generate_invite_code(group_id, user_id, request.expires_in_hours, request.max_uses)
        .await
    {
        Ok(code) => (StatusCode::CREATED, Json(ApiResponse::success(code))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<InviteCode>::error(e.to_string())),
        ),
    }
}

/// Revoke an invite code
async fn revoke_invite_code(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Path((group_id, code)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    // Note: group_id is not used here but kept for REST consistency
    let _ = group_id;
    match invite_service.revoke_invite_code(&code, user_id).await {
        Ok(()) => (StatusCode::OK, Json(ApiResponse::success("Invite code revoked"))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<&str>::error(e.to_string())),
        ),
    }
}

/// Join a group via invite code
async fn join_via_code(
    State((_, _, invite_service)): State<GroupState>,
    Extension(user_id): Extension<Uuid>,
    Json(request): Json<JoinViaCodeRequest>,
) -> impl IntoResponse {
    match invite_service.join_via_invite_code(&request.code, user_id).await {
        Ok(group) => (StatusCode::OK, Json(ApiResponse::success(group))),
        Err(e) => (
            StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(ApiResponse::<Group>::error(e.to_string())),
        ),
    }
}
