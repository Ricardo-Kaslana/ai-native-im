//! Member service tests

mod common;

use ai_im_server::models::{CreateGroup, MemberRole};
use ai_im_server::services::{GroupError, GroupService, MemberService};
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
async fn test_add_member_success() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    let result = member_service.add_member(group.id, member_id, owner_id).await;
    assert!(result.is_ok());
    
    let member = result.unwrap();
    assert_eq!(member.user_id, member_id);
    assert_eq!(member.role, MemberRole::Member);
    
    let updated_group = group_service.get_group(group.id).await.unwrap();
    assert_eq!(updated_group.member_count, 2);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_add_member_already_member() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    let result = member_service.add_member(group.id, member_id, owner_id).await;
    assert!(matches!(result, Err(GroupError::AlreadyMember(_))));
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_remove_member_success() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    member_service.remove_member(group.id, member_id, owner_id).await.unwrap();
    
    let updated_group = group_service.get_group(group.id).await.unwrap();
    assert_eq!(updated_group.member_count, 1);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_admin_cannot_kick_owner() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let admin_id = create_test_user(&pool, &format!("admin_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, admin_id, owner_id).await.unwrap();
    member_service.update_member_role(group.id, admin_id, MemberRole::Admin, owner_id).await.unwrap();
    
    let result = member_service.remove_member(group.id, owner_id, admin_id).await;
    assert!(matches!(result, Err(GroupError::CannotRemoveOwner)));
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, admin_id).await;
}

#[tokio::test]
async fn test_promote_to_admin() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    let result = member_service.update_member_role(group.id, member_id, MemberRole::Admin, owner_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().role, MemberRole::Admin);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_mute_member() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    member_service.mute_member(group.id, member_id, owner_id).await.unwrap();
    
    assert!(member_service.permission_service().is_muted(group.id, member_id).await.unwrap());
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_ban_member() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    member_service.ban_member(group.id, member_id, owner_id, Some("Spam".to_string())).await.unwrap();
    
    assert!(member_service.permission_service().is_banned(group.id, member_id).await.unwrap());
    assert!(!member_service.permission_service().is_member(group.id, member_id).await.unwrap());
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_banned_user_cannot_rejoin() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    member_service.ban_member(group.id, member_id, owner_id, None).await.unwrap();
    
    let result = member_service.add_member(group.id, member_id, owner_id).await;
    assert!(matches!(result, Err(GroupError::UserBanned(_))));
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, member_id).await;
}

#[tokio::test]
async fn test_get_group_members() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let member_service = MemberService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let admin_id = create_test_user(&pool, &format!("admin_{}", Uuid::new_v4())).await;
    let member_id = create_test_user(&pool, &format!("member_{}", Uuid::new_v4())).await;
    
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test".to_string(), description: None, avatar: None, is_public: None, member_ids: None,
    }).await.unwrap();
    
    member_service.add_member(group.id, admin_id, owner_id).await.unwrap();
    member_service.add_member(group.id, member_id, owner_id).await.unwrap();
    member_service.update_member_role(group.id, admin_id, MemberRole::Admin, owner_id).await.unwrap();
    
    let members = member_service.get_group_members(group.id, owner_id, 0, 100).await.unwrap();
    assert_eq!(members.len(), 3);
    assert_eq!(members[0].role, MemberRole::Owner);
    assert_eq!(members[1].role, MemberRole::Admin);
    assert_eq!(members[2].role, MemberRole::Member);
    
    cleanup_test_data(&pool, group.id).await;
    cleanup_test_user(&pool, owner_id).await;
    cleanup_test_user(&pool, admin_id).await;
    cleanup_test_user(&pool, member_id).await;
}
