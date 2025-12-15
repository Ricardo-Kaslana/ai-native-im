use crate::error::AppError;
use crate::middleware::UserContext;
use crate::response::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Conversation type
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ConversationType {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "group")]
    Group,
}

/// Conversation model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub conversation_type: String,
    pub name: Option<String>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Conversation participant
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationParticipant {
    pub conversation_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub last_read_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Create conversation request
#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub conversation_type: String,
    pub name: Option<String>,
    pub participant_ids: Vec<String>,
}

/// Conversation response with participants
#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: String,
    pub conversation_type: String,
    pub name: Option<String>,
    pub participants: Vec<ParticipantInfo>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ParticipantInfo {
    pub id: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

/// Conversations list data
#[derive(Debug, Serialize)]
pub struct ConversationsData {
    pub conversations: Vec<ConversationResponse>,
}

/// Single conversation data
#[derive(Debug, Serialize)]
pub struct ConversationData {
    pub conversation: ConversationResponse,
}

/// GET /conversations - Get user's conversations
async fn get_conversations(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<ApiResponse<ConversationsData>>, AppError> {
    let conversations = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT DISTINCT c.*
        FROM conversations c
        INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
        WHERE cp.user_id = $1
        ORDER BY c.updated_at DESC
        "#,
    )
    .bind(&user_ctx.user_id)
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::new();
    for conv in conversations {
        let participants = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
            r#"
            SELECT u.id, u.username, u.avatar_url
            FROM users u
            INNER JOIN conversation_participants cp ON u.id = cp.user_id
            WHERE cp.conversation_id = $1
            "#,
        )
        .bind(&conv.id)
        .fetch_all(&pool)
        .await?;

        let participant_infos: Vec<ParticipantInfo> = participants
            .into_iter()
            .map(|(id, username, avatar_url)| ParticipantInfo {
                id: id.to_string(),
                username,
                avatar_url,
            })
            .collect();

        result.push(ConversationResponse {
            id: conv.id.to_string(),
            conversation_type: conv.conversation_type,
            name: conv.name,
            participants: participant_infos,
            created_at: conv.created_at.to_rfc3339(),
            updated_at: conv.updated_at.to_rfc3339(),
        });
    }

    Ok(Json(ApiResponse::success(ConversationsData { conversations: result })))
}


/// POST /conversations - Create a new conversation
async fn create_conversation(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<ApiResponse<ConversationData>>, AppError> {
    // Validate participants
    for participant_id_str in &req.participant_ids {
        let participant_id = Uuid::parse_str(participant_id_str)
            .map_err(|_| AppError::Validation("Invalid participant ID format".to_string()))?;
        
        if participant_id == user_ctx.user_id {
            return Err(AppError::Validation("Cannot create conversation with yourself".to_string()));
        }
    }

    // For private conversations, check if conversation already exists
    if req.conversation_type == "private" && req.participant_ids.len() == 1 {
        let other_user_id = Uuid::parse_str(&req.participant_ids[0])
            .map_err(|_| AppError::Validation("Invalid participant ID format".to_string()))?;
        
        let existing_conversation = sqlx::query_as::<_, Conversation>(
            r#"
            SELECT DISTINCT c.*
            FROM conversations c
            INNER JOIN conversation_participants cp1 ON c.id = cp1.conversation_id
            INNER JOIN conversation_participants cp2 ON c.id = cp2.conversation_id
            WHERE c.conversation_type = 'private'
              AND cp1.user_id = $1
              AND cp2.user_id = $2
            LIMIT 1
            "#,
        )
        .bind(&user_ctx.user_id)
        .bind(&other_user_id)
        .fetch_optional(&pool)
        .await?;

        if let Some(existing_conv) = existing_conversation {
            let participants = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
                r#"
                SELECT u.id, u.username, u.avatar_url
                FROM users u
                INNER JOIN conversation_participants cp ON u.id = cp.user_id
                WHERE cp.conversation_id = $1
                "#,
            )
            .bind(&existing_conv.id)
            .fetch_all(&pool)
            .await?;

            let participant_infos: Vec<ParticipantInfo> = participants
                .into_iter()
                .map(|(id, username, avatar_url)| ParticipantInfo {
                    id: id.to_string(),
                    username,
                    avatar_url,
                })
                .collect();

            let response = ConversationResponse {
                id: existing_conv.id.to_string(),
                conversation_type: existing_conv.conversation_type,
                name: existing_conv.name,
                participants: participant_infos,
                created_at: existing_conv.created_at.to_rfc3339(),
                updated_at: existing_conv.updated_at.to_rfc3339(),
            };

            return Ok(Json(ApiResponse::success(ConversationData { conversation: response })));
        }
    }

    // Create conversation
    let conversation = sqlx::query_as::<_, Conversation>(
        r#"
        INSERT INTO conversations (conversation_type, name, created_by)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&req.conversation_type)
    .bind(&req.name)
    .bind(&user_ctx.user_id)
    .fetch_one(&pool)
    .await?;

    // Add creator as participant
    sqlx::query(
        r#"
        INSERT INTO conversation_participants (conversation_id, user_id)
        VALUES ($1, $2)
        "#,
    )
    .bind(&conversation.id)
    .bind(&user_ctx.user_id)
    .execute(&pool)
    .await?;

    // Add other participants
    for participant_id_str in &req.participant_ids {
        let participant_id = Uuid::parse_str(participant_id_str)
            .map_err(|_| AppError::Validation("Invalid participant ID format".to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO conversation_participants (conversation_id, user_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(&conversation.id)
        .bind(&participant_id)
        .execute(&pool)
        .await?;
    }

    // Get all participants for response
    let participants = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
        r#"
        SELECT u.id, u.username, u.avatar_url
        FROM users u
        INNER JOIN conversation_participants cp ON u.id = cp.user_id
        WHERE cp.conversation_id = $1
        "#,
    )
    .bind(&conversation.id)
    .fetch_all(&pool)
    .await?;

    let participant_infos: Vec<ParticipantInfo> = participants
        .into_iter()
        .map(|(id, username, avatar_url)| ParticipantInfo {
            id: id.to_string(),
            username,
            avatar_url,
        })
        .collect();

    let response = ConversationResponse {
        id: conversation.id.to_string(),
        conversation_type: conversation.conversation_type,
        name: conversation.name,
        participants: participant_infos,
        created_at: conversation.created_at.to_rfc3339(),
        updated_at: conversation.updated_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(ConversationData { conversation: response })))
}


/// Message model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub msg_type: i16,
    pub content: String,
    pub timestamp: i64,
    pub is_ai_generated: bool,
    pub delivered: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Message response
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub sender_id: String,
    pub receiver_id: Option<String>,
    pub group_id: Option<String>,
    pub msg_type: String,
    pub content: String,
    pub timestamp: i64,
    pub is_ai_generated: bool,
    pub delivered: bool,
    pub created_at: String,
}

/// Messages list data
#[derive(Debug, Serialize)]
pub struct MessagesData {
    pub messages: Vec<MessageResponse>,
}

/// GET /conversations/:id/messages - Get messages for a conversation
async fn get_conversation_messages(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
    Path(conversation_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<MessagesData>>, AppError> {
    // Verify user is participant
    let is_participant = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM conversation_participants
            WHERE conversation_id = $1 AND user_id = $2
        )
        "#,
    )
    .bind(&conversation_id)
    .bind(&user_ctx.user_id)
    .fetch_one(&pool)
    .await?;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant of this conversation".to_string()));
    }

    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE id = $1",
    )
    .bind(&conversation_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Conversation not found".to_string()))?;

    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let messages = if conversation.conversation_type == "private" {
        let other_user_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT user_id FROM conversation_participants
            WHERE conversation_id = $1 AND user_id != $2
            LIMIT 1
            "#,
        )
        .bind(&conversation_id)
        .bind(&user_ctx.user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Other participant not found".to_string()))?;

        sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE (sender_id = $1 AND receiver_id = $2)
               OR (sender_id = $2 AND receiver_id = $1)
            ORDER BY timestamp DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&user_ctx.user_id)
        .bind(&other_user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT * FROM messages
            WHERE group_id = $1
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&conversation_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?
    };

    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|msg| {
            let msg_type_str = match msg.msg_type {
                0 => "text",
                1 => "image",
                2 => "video",
                3 => "emoji",
                _ => "text",
            };

            MessageResponse {
                id: msg.id.to_string(),
                sender_id: msg.sender_id.to_string(),
                receiver_id: msg.receiver_id.map(|id| id.to_string()),
                group_id: msg.group_id.map(|id| id.to_string()),
                msg_type: msg_type_str.to_string(),
                content: msg.content,
                timestamp: msg.timestamp,
                is_ai_generated: msg.is_ai_generated,
                delivered: msg.delivered,
                created_at: msg.created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(MessagesData { messages: message_responses })))
}


/// GET /conversations/:id - Get conversation details
async fn get_conversation(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ConversationData>>, AppError> {
    // Verify user is participant
    let is_participant = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM conversation_participants
            WHERE conversation_id = $1 AND user_id = $2
        )
        "#,
    )
    .bind(&conversation_id)
    .bind(&user_ctx.user_id)
    .fetch_one(&pool)
    .await?;

    if !is_participant {
        return Err(AppError::Forbidden("Not a participant of this conversation".to_string()));
    }

    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT * FROM conversations WHERE id = $1",
    )
    .bind(&conversation_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Conversation not found".to_string()))?;

    let participants = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
        r#"
        SELECT u.id, u.username, u.avatar_url
        FROM users u
        INNER JOIN conversation_participants cp ON u.id = cp.user_id
        WHERE cp.conversation_id = $1
        "#,
    )
    .bind(&conversation_id)
    .fetch_all(&pool)
    .await?;

    let participant_infos: Vec<ParticipantInfo> = participants
        .into_iter()
        .map(|(id, username, avatar_url)| ParticipantInfo {
            id: id.to_string(),
            username,
            avatar_url,
        })
        .collect();

    let response = ConversationResponse {
        id: conversation.id.to_string(),
        conversation_type: conversation.conversation_type,
        name: conversation.name,
        participants: participant_infos,
        created_at: conversation.created_at.to_rfc3339(),
        updated_at: conversation.updated_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(ConversationData { conversation: response })))
}

/// Create conversation routes
pub fn conversation_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/conversations", get(get_conversations).post(create_conversation))
        .route("/conversations/:id", get(get_conversation))
        .route("/conversations/:id/messages", get(get_conversation_messages))
        .with_state(pool)
}
