//! Group service tests

mod common;

use ai_im_server::models::{CreateGroup, GroupStatus, MemberRole, UpdateGroupSettings};
use ai_im_server::services::{GroupError, GroupService, MemberService};
use uuid::Uuid;

async fn create_test_user(pool: &sqlx::PgPool, username: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, 'test_hash')"#,
        user_id,
        username,
        format!("{}@test.com", username)
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");
    user_id
}

async fn cleanup_test_data(pool: &sqlx::PgPool, group_id: Uuid) {
    let _ = sqlx::query!("DELETE FROM group_audit_log WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM group_invite_codes WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM group_bans WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM group_members WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM groups WHERE id = $1", group_id).execute(pool).await;
}

async fn cleanup_test_user(pool: &sqlx::PgPool, user_id: Uuid) {
    let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id).execute(pool).await;
}

#[tokio::test]
async fn test_create_group_success() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let request = CreateGroup {
        name: "Test Group".to_string(),
        description: Some("A test group".to_string()),
        avatar: None,
        is_public: Some(false),
        member_ids: None,
    };
    
    let result = service.create_group(owner_id, request).await;
    assert!(result.is_ok());
    
    let group = result.unwrap();
    assert_eq!(group.name, "Test Group");
    assert_eq!(group.owner_id, owner_id);
    assert_eq!(group.member_count, 1);
    assert_eq!(group.max_capacity, 500);
    assert_eq!(group.status, GroupStatus::Active);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_create_group_invalid_name_empty() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let request = CreateGroup { name: "".to_string(), description: None, avatar: None, is_public: None, member_ids: None };
    let result = service.create_group(owner_id, request).await;
    assert!(matches!(result, Err(GroupError::InvalidGroupName)));
    
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_create_group_invalid_name_too_long() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let request = CreateGroup { name: "A".repeat(51), description: None, avatar: None, is_public: None, member_ids: None };
    let result = service.create_group(owner_id, request).await;
    assert!(matches!(result, Err(GroupError::InvalidGroupName)));
    
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_update_group_settings() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let group = service.create_group(owner_id, CreateGroup {
        name: "Original".to_string(), description: None, avatar: None, is_public: Some(false), member_ids: None,
    }).await.unwrap();
    
    let updates = UpdateGroupSettings {
        name: Some("Updated".to_string()),
        description: Some("New desc".to_string()),
        avatar: None,
        is_public: Some(true),
    };
    
    let updated = service.update_group_settings(group.id, owner_id, updates).await.unwrap();
    assert_eq!(updated.name, "Updated");
    assert!(updated.is_public);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_dissolve_group() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let group = service.create_group(owner_id, CreateGroup {
        name: "To Dissolve".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    service.dissolve_group(group.id, owner_id).await.unwrap();
    let dissolved = service.get_group(group.id).await.unwrap();
    assert_eq!(dissolved.status, GroupStatus::Dissolved);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_dissolve_group_non_owner() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let admin_id = create_test_user(&pool, &format!("admin_{}", Uuid::new_v4())).await;
    
    let group = service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, admin_id, owner_id).await.unwrap();
    member_service.update_member_role(group.id, admin_id, MemberRole::Admin, owner_id).await.unwrap();
    
    let result = service.dissolve_group(group.id, admin_id).await;
    assert!(matches!(result, Err(GroupError::OwnerOnly)));
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, admin_id).await;
}

#[tokio::test]
async fn test_search_groups() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let group = service.create_group(owner_id, CreateGroup {
        name: "Searchable XYZ123".to_string(), description: None, avatar: None, is_public: Some(true), member_ids: None,
    }).await.unwrap();
    
    let results = service.search_groups("XYZ123", 20).await.unwrap();
    assert!(results.iter().any(|g| g.id == group.id));
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_search_groups_query_too_short() {
    let pool = common::setup_test_db().await;
    let service = GroupService::new(pool.clone());
    let result = service.search_groups("A", 20).await;
    assert!(matches!(result, Err(GroupError::SearchQueryTooShort)));
}
