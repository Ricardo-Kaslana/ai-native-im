//! AI Agent service tests

mod common;

use ai_im_server::models::{CreateAIAgent, CreateGroup, TriggerType, UpdatePersonality};
use ai_im_server::services::{AIAgentError, AIAgentService, GroupService};
use uuid::Uuid;

async fn create_test_user(pool: &sqlx::PgPool, username: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, 'test_hash')"#,
        user_id, username, format!("{}@test.com", username)
    ).execute(pool).await.expect("Failed to create test user");
    user_id
}

async fn cleanup_agent(pool: &sqlx::PgPool, agent_id: Uuid) {
    let _ = sqlx::query!("DELETE FROM agent_trigger_logs WHERE agent_id = $1", agent_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM agent_user_affinities WHERE agent_id = $1", agent_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM agent_message_stats WHERE agent_id = $1", agent_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM ai_agents WHERE id = $1", agent_id).execute(pool).await;
}

async fn cleanup_group(pool: &sqlx::PgPool, group_id: Uuid) {
    // First cleanup any agents
    let agents: Vec<Uuid> = sqlx::query_scalar!("SELECT id FROM ai_agents WHERE group_id = $1", group_id)
        .fetch_all(pool).await.unwrap_or_default();
    for agent_id in agents {
        cleanup_agent(pool, agent_id).await;
    }
    
    let _ = sqlx::query!("DELETE FROM group_audit_log WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM group_invite_codes WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM group_bans WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM group_members WHERE group_id = $1", group_id).execute(pool).await;
    let _ = sqlx::query!("DELETE FROM groups WHERE id = $1", group_id).execute(pool).await;
}

async fn cleanup_user(pool: &sqlx::PgPool, user_id: Uuid) {
    let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id).execute(pool).await;
}

// ============== Agent Creation Tests ==============

#[tokio::test]
async fn test_create_agent_success() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let request = CreateAIAgent {
        name: "TestBot".to_string(),
        toxicity_level: Some(5),
        enthusiasm_level: Some(7),
        expertise_domains: Some(vec!["programming".to_string()]),
        speech_suffix: Some("~".to_string()),
        trigger_keywords: Some(vec!["help".to_string()]),
    };
    
    let result = agent_service.create_agent(group.id, request).await;
    assert!(result.is_ok());
    
    let agent = result.unwrap();
    assert_eq!(agent.name, "TestBot");
    assert_eq!(agent.toxicity_level, 5);
    assert_eq!(agent.enthusiasm_level, 7);
    assert_eq!(agent.speech_suffix, Some("~".to_string()));
    assert!(agent.is_active);
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_create_agent_default_values() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let request = CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: None,
        enthusiasm_level: None,
        expertise_domains: None,
        speech_suffix: None,
        trigger_keywords: None,
    };
    
    let agent = agent_service.create_agent(group.id, request).await.unwrap();
    assert_eq!(agent.toxicity_level, 5); // default
    assert_eq!(agent.enthusiasm_level, 5); // default
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_create_agent_invalid_name() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    // Empty name
    let result = agent_service.create_agent(group.id, CreateAIAgent {
        name: "".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await;
    assert!(matches!(result, Err(AIAgentError::InvalidAgentName)));
    
    // Name too long
    let result = agent_service.create_agent(group.id, CreateAIAgent {
        name: "A".repeat(101),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await;
    assert!(matches!(result, Err(AIAgentError::InvalidAgentName)));
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_create_agent_invalid_levels() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    // Invalid toxicity
    let result = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: Some(11),
        enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await;
    assert!(matches!(result, Err(AIAgentError::InvalidToxicityLevel)));
    
    // Invalid enthusiasm
    let result = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: None,
        enthusiasm_level: Some(-1),
        expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await;
    assert!(matches!(result, Err(AIAgentError::InvalidEnthusiasmLevel)));
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_create_agent_already_exists() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    // Create first agent
    agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot1".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await.unwrap();
    
    // Try to create second agent
    let result = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot2".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await;
    assert!(matches!(result, Err(AIAgentError::AgentAlreadyExists)));
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

// ============== Personality Update Tests ==============

#[tokio::test]
async fn test_update_personality() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let agent = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: Some(5), enthusiasm_level: Some(5),
        expertise_domains: None, speech_suffix: None, trigger_keywords: None,
    }).await.unwrap();
    
    let updated = agent_service.update_personality(agent.id, UpdatePersonality {
        name: Some("NewBot".to_string()),
        toxicity_level: Some(8),
        enthusiasm_level: Some(3),
        expertise_domains: Some(vec!["gaming".to_string()]),
        speech_suffix: Some("meow".to_string()),
        trigger_keywords: None,
    }).await.unwrap();
    
    assert_eq!(updated.name, "NewBot");
    assert_eq!(updated.toxicity_level, 8);
    assert_eq!(updated.enthusiasm_level, 3);
    assert_eq!(updated.speech_suffix, Some("meow".to_string()));
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

// ============== Trigger Evaluation Tests ==============

#[tokio::test]
async fn test_trigger_mention() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let agent = agent_service.create_agent(group.id, CreateAIAgent {
        name: "TestBot".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await.unwrap();
    
    let decision = agent_service.evaluate_trigger(&agent, "@TestBot hello", "user1", None).await;
    
    assert!(decision.should_respond);
    assert_eq!(decision.probability, 1.0);
    assert_eq!(decision.trigger_type, TriggerType::Mention);
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_trigger_keyword() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let agent = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None,
        trigger_keywords: Some(vec!["help".to_string(), "question".to_string()]),
    }).await.unwrap();
    
    let decision = agent_service.evaluate_trigger(&agent, "I need help with this", "user1", None).await;
    
    assert_eq!(decision.trigger_type, TriggerType::Keyword);
    // With keyword boost (1.2x), probability can be up to 0.96
    assert!(decision.probability >= 0.6 && decision.probability <= 1.0);
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

// ============== Statistics Tests ==============

#[tokio::test]
async fn test_get_statistics() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let agent = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await.unwrap();
    
    let stats = agent_service.get_statistics(agent.id).await.unwrap();
    
    assert_eq!(stats.agent_id, agent.id);
    assert_eq!(stats.total_messages, 0);
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

// ============== User Affinity Tests ==============

#[tokio::test]
async fn test_set_user_affinity() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());
    let agent_service = AIAgentService::new(pool.clone());
    
    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let user_id = create_test_user(&pool, &format!("user_{}", Uuid::new_v4())).await;
    let group = group_service.create_group(owner_id, CreateGroup {
        name: "Test Group".to_string(), description: None, avatar: None, is_public: None, member_ids: None }).await.unwrap();
    
    let agent = agent_service.create_agent(group.id, CreateAIAgent {
        name: "Bot".to_string(),
        toxicity_level: None, enthusiasm_level: None, expertise_domains: None,
        speech_suffix: None, trigger_keywords: None,
    }).await.unwrap();
    
    // Set positive affinity
    agent_service.set_user_affinity(agent.id, user_id, 5).await.unwrap();
    
    // Update to negative
    agent_service.set_user_affinity(agent.id, user_id, -3).await.unwrap();
    
    // Invalid affinity
    let result = agent_service.set_user_affinity(agent.id, user_id, 15).await;
    assert!(result.is_err());
    
    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
    cleanup_user(&pool, user_id).await;
}
