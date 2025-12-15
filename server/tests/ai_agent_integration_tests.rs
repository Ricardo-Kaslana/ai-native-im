//! AI Agent integration tests with vector store and embedding

mod common;

use ai_im_server::models::{CreateAIAgent, CreateGroup, TriggerType};
use ai_im_server::services::{
    AIAgentService, GroupService, MockEmbeddingService, MockVectorStore,
};
use std::sync::Arc;
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

async fn cleanup_agent(pool: &sqlx::PgPool, agent_id: Uuid) {
    let _ = sqlx::query!("DELETE FROM agent_trigger_logs WHERE agent_id = $1", agent_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM agent_user_affinities WHERE agent_id = $1", agent_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM agent_message_stats WHERE agent_id = $1", agent_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM ai_agents WHERE id = $1", agent_id)
        .execute(pool)
        .await;
}

async fn cleanup_group(pool: &sqlx::PgPool, group_id: Uuid) {
    let agents: Vec<Uuid> =
        sqlx::query_scalar!("SELECT id FROM ai_agents WHERE group_id = $1", group_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    for agent_id in agents {
        cleanup_agent(pool, agent_id).await;
    }

    let _ = sqlx::query!("DELETE FROM group_audit_log WHERE group_id = $1", group_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM group_invite_codes WHERE group_id = $1", group_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM group_bans WHERE group_id = $1", group_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM group_members WHERE group_id = $1", group_id)
        .execute(pool)
        .await;
    let _ = sqlx::query!("DELETE FROM groups WHERE id = $1", group_id)
        .execute(pool)
        .await;
}

async fn cleanup_user(pool: &sqlx::PgPool, user_id: Uuid) {
    let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(pool)
        .await;
}

// ============== Integration Tests ==============

#[tokio::test]
async fn test_agent_with_vector_store() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    // Create mock services
    let vector_store = Arc::new(MockVectorStore::new());
    let embedding_service = Arc::new(MockEmbeddingService::default());

    let agent_service = AIAgentService::new(pool.clone())
        .with_vector_store(vector_store.clone())
        .with_embedding_service(embedding_service.clone());

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    let agent = agent_service
        .create_agent(
            group.id,
            CreateAIAgent {
                name: "TestBot".to_string(),
                toxicity_level: Some(5),
                enthusiasm_level: Some(7),
                expertise_domains: Some(vec!["programming".to_string()]),
                speech_suffix: None,
                trigger_keywords: Some(vec!["help".to_string()]),
            },
        )
        .await
        .unwrap();

    // Store a message embedding
    let message_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    agent_service
        .store_message_embedding(group.id, message_id, sender_id, "How do I write Rust code?")
        .await
        .unwrap();

    // Search for similar messages
    let results = agent_service
        .search_similar_messages(group.id, "How do I write Rust code?", 5, 0.5)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].similarity_score >= 0.99); // Same text should have high similarity

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_process_message_with_mention() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    let vector_store = Arc::new(MockVectorStore::new());
    let embedding_service = Arc::new(MockEmbeddingService::default());

    let agent_service = AIAgentService::new(pool.clone())
        .with_vector_store(vector_store)
        .with_embedding_service(embedding_service);

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    let agent = agent_service
        .create_agent(
            group.id,
            CreateAIAgent {
                name: "Bot".to_string(),
                toxicity_level: None,
                enthusiasm_level: None,
                expertise_domains: None,
                speech_suffix: None,
                trigger_keywords: None,
            },
        )
        .await
        .unwrap();

    // Process a message with @mention
    let message_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let (decision, _historical) = agent_service
        .process_message(&agent, message_id, sender_id, "@Bot hello!", None)
        .await
        .unwrap();

    assert!(decision.should_respond);
    assert_eq!(decision.trigger_type, TriggerType::Mention);
    assert_eq!(decision.probability, 1.0);

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_process_message_with_keyword() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    // Don't use vector store for this test to ensure keyword trigger is used
    let agent_service = AIAgentService::new(pool.clone());

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    let agent = agent_service
        .create_agent(
            group.id,
            CreateAIAgent {
                name: "Bot".to_string(),
                toxicity_level: None,
                enthusiasm_level: None,
                expertise_domains: None,
                speech_suffix: None,
                trigger_keywords: Some(vec!["help".to_string(), "question".to_string()]),
            },
        )
        .await
        .unwrap();

    // Process a message with keyword (without vector store, so no context similarity)
    let message_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let (decision, _) = agent_service
        .process_message(&agent, message_id, sender_id, "I need help with this", None)
        .await
        .unwrap();

    assert_eq!(decision.trigger_type, TriggerType::Keyword);
    assert!(decision.probability >= 0.6 && decision.probability <= 0.8);

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_delete_group_embeddings() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    let vector_store = Arc::new(MockVectorStore::new());
    let embedding_service = Arc::new(MockEmbeddingService::default());

    let agent_service = AIAgentService::new(pool.clone())
        .with_vector_store(vector_store.clone())
        .with_embedding_service(embedding_service);

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    let _agent = agent_service
        .create_agent(
            group.id,
            CreateAIAgent {
                name: "Bot".to_string(),
                toxicity_level: None,
                enthusiasm_level: None,
                expertise_domains: None,
                speech_suffix: None,
                trigger_keywords: None,
            },
        )
        .await
        .unwrap();

    // Store some messages
    for i in 0..5 {
        agent_service
            .store_message_embedding(
                group.id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                &format!("Test message {}", i),
            )
            .await
            .unwrap();
    }

    // Verify messages exist
    let results = agent_service
        .search_similar_messages(group.id, "Test message", 10, 0.0)
        .await
        .unwrap();
    assert_eq!(results.len(), 5);

    // Delete all embeddings
    agent_service.delete_group_embeddings(group.id).await.unwrap();

    // Verify messages are deleted
    let results = agent_service
        .search_similar_messages(group.id, "Test message", 10, 0.0)
        .await
        .unwrap();
    assert!(results.is_empty());

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}


// ============== Agent Message Handler Tests ==============

use ai_im_server::services::{AgentMessage, AgentMessageHandler, AgentMessageSender, MockLLMClient};
use std::sync::atomic::{AtomicU32, Ordering};

struct TestMessageSender {
    send_count: AtomicU32,
}

impl TestMessageSender {
    fn new() -> Self {
        Self {
            send_count: AtomicU32::new(0),
        }
    }

    fn get_send_count(&self) -> u32 {
        self.send_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl AgentMessageSender for TestMessageSender {
    async fn send_message(
        &self,
        _group_id: Uuid,
        _agent_id: Uuid,
        _agent_name: &str,
        _content: &str,
        _reply_to: Option<Uuid>,
    ) -> Result<Uuid, ai_im_server::services::AIAgentError> {
        self.send_count.fetch_add(1, Ordering::SeqCst);
        Ok(Uuid::new_v4())
    }
}

#[tokio::test]
async fn test_agent_message_handler_with_mention() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    // Create agent service with mock LLM
    let llm_client = Arc::new(MockLLMClient::new("Hello! How can I help you?".to_string()));
    let agent_service = AIAgentService::new(pool.clone()).with_llm_client(llm_client);

    let handler = AgentMessageHandler::new(agent_service);

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    // Create agent
    handler
        .agent_service()
        .create_agent(
            group.id,
            CreateAIAgent {
                name: "TestBot".to_string(),
                toxicity_level: Some(5),
                enthusiasm_level: Some(7),
                expertise_domains: None,
                speech_suffix: None,
                trigger_keywords: None,
            },
        )
        .await
        .unwrap();

    // Process a message with @mention
    let message = AgentMessage {
        message_id: Uuid::new_v4(),
        group_id: group.id,
        sender_id: Uuid::new_v4(),
        sender_name: "TestUser".to_string(),
        content: "@TestBot hello!".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    let result = handler.process_message(message, &[]).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.is_some());

    let response = response.unwrap();
    assert_eq!(response.agent_name, "TestBot");
    assert!(!response.content.is_empty());

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_agent_message_handler_no_agent() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    let agent_service = AIAgentService::new(pool.clone());
    let handler = AgentMessageHandler::new(agent_service);

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    // Process a message without any agent in the group
    let message = AgentMessage {
        message_id: Uuid::new_v4(),
        group_id: group.id,
        sender_id: Uuid::new_v4(),
        sender_name: "TestUser".to_string(),
        content: "Hello everyone!".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    let result = handler.process_message(message, &[]).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // No response because no agent

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}

#[tokio::test]
async fn test_agent_message_handler_random_message() {
    let pool = common::setup_test_db().await;
    let group_service = GroupService::new(pool.clone());

    let llm_client = Arc::new(MockLLMClient::new("Interesting!".to_string()));
    let agent_service = AIAgentService::new(pool.clone()).with_llm_client(llm_client);

    let handler = AgentMessageHandler::new(agent_service);

    let owner_id = create_test_user(&pool, &format!("owner_{}", Uuid::new_v4())).await;
    let group = group_service
        .create_group(
            owner_id,
            CreateGroup {
                name: "Test Group".to_string(),
                description: None,
                avatar: None,
                is_public: None,
                member_ids: None,
            },
        )
        .await
        .unwrap();

    // Create agent
    handler
        .agent_service()
        .create_agent(
            group.id,
            CreateAIAgent {
                name: "Bot".to_string(),
                toxicity_level: None,
                enthusiasm_level: None,
                expertise_domains: None,
                speech_suffix: None,
                trigger_keywords: None,
            },
        )
        .await
        .unwrap();

    // Process a random message (low probability of response)
    let message = AgentMessage {
        message_id: Uuid::new_v4(),
        group_id: group.id,
        sender_id: Uuid::new_v4(),
        sender_name: "TestUser".to_string(),
        content: "Just chatting about random stuff".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    // This might or might not respond due to random trigger
    let result = handler.process_message(message, &[]).await;
    assert!(result.is_ok());
    // We don't assert on the response because it's probabilistic

    cleanup_group(&pool, group.id).await;
    cleanup_user(&pool, owner_id).await;
}
