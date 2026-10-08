use crate::error::AppError;
use crate::middleware::UserContext;
use crate::response::ApiResponse;
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Contact model
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contact {
    pub user_id: Uuid,
    pub contact_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Contact response with user info
#[derive(Debug, Serialize)]
pub struct ContactResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub is_online: bool,
    pub created_at: String,
}

/// Add contact request
#[derive(Debug, Deserialize)]
pub struct AddContactRequest {
    pub contact_id: String,
}

/// Contacts list data
#[derive(Debug, Serialize)]
pub struct ContactsData {
    pub contacts: Vec<ContactResponse>,
}

/// Single contact data
#[derive(Debug, Serialize)]
pub struct ContactData {
    pub contact: ContactResponse,
}

/// GET /contacts - Get user's contacts
async fn get_contacts(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<ApiResponse<ContactsData>>, AppError> {
    let contacts = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Option<String>)>(
        r#"
        SELECT u.id, u.username, u.email, u.avatar_url, u.bio
        FROM users u
        INNER JOIN contacts c ON u.id = c.contact_id
        WHERE c.user_id = $1
        ORDER BY u.username
        "#,
    )
    .bind(&user_ctx.user_id)
    .fetch_all(&pool)
    .await?;

    let contact_responses: Vec<ContactResponse> = contacts
        .into_iter()
        .map(|(id, username, email, avatar_url, bio)| ContactResponse {
            id: id.to_string(),
            username,
            email,
            avatar_url,
            bio,
            is_online: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .collect();

    Ok(Json(ApiResponse::success(ContactsData { contacts: contact_responses })))
}

/// POST /contacts - Add a new contact
async fn add_contact(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
    Json(req): Json<AddContactRequest>,
) -> Result<Json<ApiResponse<ContactData>>, AppError> {
    let contact_id = Uuid::parse_str(&req.contact_id)
        .map_err(|_| AppError::Validation("Invalid contact ID format".to_string()))?;

    // Check if user exists
    let user_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
    )
    .bind(&contact_id)
    .fetch_one(&pool)
    .await?;

    if !user_exists {
        return Err(AppError::UserNotFound(contact_id.to_string()));
    }

    // Check if already a contact in this specific direction only
    // We allow bidirectional relationships to be created independently
    let already_contact = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM contacts WHERE user_id = $1 AND contact_id = $2)",
    )
    .bind(&user_ctx.user_id)
    .bind(&contact_id)
    .fetch_one(&pool)
    .await?;

    if already_contact {
        return Err(AppError::Conflict("Contact relationship already exists".to_string()));
    }

    // Add contact relationship in this direction
    sqlx::query(
        r#"
        INSERT INTO contacts (user_id, contact_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, contact_id) DO NOTHING
        "#,
    )
    .bind(&user_ctx.user_id)
    .bind(&contact_id)
    .execute(&pool)
    .await?;

    // Create bidirectional relationship - add current user as contact for the target user
    sqlx::query(
        r#"
        INSERT INTO contacts (user_id, contact_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, contact_id) DO NOTHING
        "#,
    )
    .bind(&contact_id)  // The target user becomes the user_id
    .bind(&user_ctx.user_id)  // Current user becomes the contact_id
    .execute(&pool)
    .await?;

    // Get contact info
    let contact = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        r#"
        SELECT username, email, avatar_url, bio
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(&contact_id)
    .fetch_one(&pool)
    .await?;

    let response = ContactResponse {
        id: contact_id.to_string(),
        username: contact.0,
        email: contact.1,
        avatar_url: contact.2,
        bio: contact.3,
        is_online: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(ContactData { contact: response })))
}

/// Message data for simple responses
#[derive(Debug, Serialize)]
pub struct MessageData {
    pub message: String,
}

/// DELETE /contacts/:id - Remove a contact
async fn remove_contact(
    State(pool): State<PgPool>,
    Extension(user_ctx): Extension<UserContext>,
    Path(contact_id): Path<Uuid>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    let result = sqlx::query(
        "DELETE FROM contacts WHERE user_id = $1 AND contact_id = $2",
    )
    .bind(&user_ctx.user_id)
    .bind(&contact_id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Contact not found".to_string()));
    }

    Ok(Json(ApiResponse::success(MessageData { message: "Contact removed".to_string() })))
}

/// Create contact routes
pub fn contact_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/contacts", get(get_contacts).post(add_contact))
        .route("/contacts/:id", delete(remove_contact))
        .with_state(pool)
}
