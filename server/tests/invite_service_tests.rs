//! Invite service tests

mod common;

use ai_im_server::models::CreateGroup;
use ai_im_server::services::{GroupError, GroupService, InviteService, MemberService};
use uuid::Uuid;

async fn create_test_user(pool: &sqlx::PgPool, username: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, 'test_hash')"#,
        user_id, username, format!("{}@test.com", username)
    ).execute(pool).await.expect("Failed to create test user");
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
async fn test_generate_invite_code() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let invite_service = InviteService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None,
    }).await.unwrap();
    
    let result = invite_service.generate_invite_code(group.id, owner_id, None, None).await;
    assert!(result.is_ok());
    
    let code = result.unwrap();
    assert_eq!(code.code.len(), 8);
    assert!(code.code.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(code.group_id, group.id);
    assert_eq!(code.usage_count, 0);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_join_via_invite_code() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let invite_service = InviteService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let joiner_id = create_test_user(&pool, &format!("joiner_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None,
    }).await.unwrap();
    
    let code = invite_service.generate_invite_code(group.id, owner_id, None, None).await.unwrap();
    let result = invite_service.join_via_invite_code(&code.code, joiner_id).await;
    assert!(result.is_ok());
    
    assert!(member_service.permission_service().is_member(group.id, joiner_id).await.unwrap());
    
    let codes = invite_service.get_group_invite_codes(group.id, owner_id).await.unwrap();
    assert_eq!(codes[0].usage_count, 1);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, joiner_id).await;
}

#[tokio::test]
async fn test_join_via_invalid_code() {
    let pool = common::setup_test_db().await;
    let invite_service = InviteService::new(pool.clone());
    
    let joiner_id = create_test_user(&pool, &format!("joiner_{}", Uuid::new_v4())).await;
    
    let result = invite_service.join_via_invite_code("INVALID1", joiner_id).await;
    assert!(matches!(result, Err(GroupError::InvalidInviteCode)));
    
    cleanup_test_user(&pool, joiner_id).await;
}

#[tokio::test]
async fn test_join_via_code_max_uses_reached() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let invite_service = InviteService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let joiner1_id = create_test_user(&pool, &format!("joiner1_{}", Uuid::new_v4())).await;
    let joiner2_id = create_test_user(&pool, &format!("joiner2_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None,
    }).await.unwrap();
    
    let code = invite_service.generate_invite_code(group.id, owner_id, None, Some(1)).await.unwrap();
    invite_service.join_via_invite_code(&code.code, joiner1_id).await.unwrap();
    
    let result = invite_service.join_via_invite_code(&code.code, joiner2_id).await;
    assert!(matches!(result, Err(GroupError::InviteCodeMaxUsesReached)));
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, joiner1_id).await;
    cleanup_test_user(&pool, joiner2_id).await;
}

#[tokio::test]
async fn test_revoke_invite_code() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let invite_service = InviteService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None,
    }).await.unwrap();
    
    let code = invite_service.generate_invite_code(group.id, owner_id, None, None).await.unwrap();
    invite_service.revoke_invite_code(&code.code, owner_id).await.unwrap();
    
    let codes = invite_service.get_group_invite_codes(group.id, owner_id).await.unwrap();
    assert!(codes.is_empty());
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
}
