use crate::models::{
    AIAgent, AgentMessageStats, AgentStatistics, CreateAIAgent, GroupContext, HistoricalMessage,
    TriggerCounts, TriggerDecision, TriggerType, UpdatePersonality,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use super::{
    load_personality_config, AIAgentError, DefaultPersonalityManager, DefaultTriggerEvaluator,
    DefaultTypingSimulator, EmbeddingService, LLMClient, PersonalityManager, TriggerEvaluator,
    TypingSimulator, VectorStore,
};

/// AI Agent service for managing agents and generating responses
#[derive(Clone)]
pub struct AIAgentService {
    pool: PgPool,
    trigger_evaluator: Arc<dyn TriggerEvaluator>,
    personality_manager: Arc<dyn PersonalityManager>,
    typing_simulator: Arc<dyn TypingSimulator>,
    llm_client: Option<Arc<dyn LLMClient>>,
    vector_store: Option<Arc<dyn VectorStore>>,
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

impl AIAgentService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            trigger_evaluator: Arc::new(DefaultTriggerEvaluator::new()),
            personality_manager: Arc::new(DefaultPersonalityManager::new()),
            typing_simulator: Arc::new(DefaultTypingSimulator::new()),
            llm_client: None,
            vector_store: None,
            embedding_service: None,
        }
    }

    pub fn with_llm_client(mut self, client: Arc<dyn LLMClient>) -> Self {
        self.llm_client = Some(client);
        self
    }

    pub fn with_trigger_evaluator(mut self, evaluator: Arc<dyn TriggerEvaluator>) -> Self {
        self.trigger_evaluator = evaluator;
        self
    }

    pub fn with_typing_simulator(mut self, simulator: Arc<dyn TypingSimulator>) -> Self {
        self.typing_simulator = simulator;
        self
    }

    pub fn with_vector_store(mut self, store: Arc<dyn VectorStore>) -> Self {
        self.vector_store = Some(store);
        self
    }

    pub fn with_embedding_service(mut self, service: Arc<dyn EmbeddingService>) -> Self {
        self.embedding_service = Some(service);
        self
    }

    /// Validate agent name
    fn validate_name(name: &str) -> Result<(), AIAgentError> {
        if name.is_empty() || name.len() > 100 {
            return Err(AIAgentError::InvalidAgentName);
        }
        Ok(())
    }

    /// Validate personality levels
    fn validate_levels(toxicity: i16, enthusiasm: i16) -> Result<(), AIAgentError> {
        if !(0..=10).contains(&toxicity) {
            return Err(AIAgentError::InvalidToxicityLevel);
        }
        if !(0..=10).contains(&enthusiasm) {
            return Err(AIAgentError::InvalidEnthusiasmLevel);
        }
        Ok(())
    }

    /// Create a new AI Agent for a group
    pub async fn create_agent(
        &self,
        group_id: Uuid,
        request: CreateAIAgent,
    ) -> Result<AIAgent, AIAgentError> {
        Self::validate_name(&request.name)?;

        let toxicity = request.toxicity_level.unwrap_or(5);
        let enthusiasm = request.enthusiasm_level.unwrap_or(5);
        Self::validate_levels(toxicity, enthusiasm)?;

        // Check if group already has an agent
        let existing = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM ai_agents WHERE group_id = $1) as "exists!""#,
            group_id
        )
        .fetch_one(&self.pool)
        .await?;

        if existing {
            return Err(AIAgentError::AgentAlreadyExists);
        }

        let agent = sqlx::query_as!(
            AIAgent,
            r#"
            INSERT INTO ai_agents (group_id, name, toxicity_level, enthusiasm_level, expertise_domains, speech_suffix, trigger_keywords)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, group_id, name, toxicity_level, enthusiasm_level, expertise_domains, speech_suffix, trigger_keywords, is_active, created_at, updated_at
            "#,
            group_id,
            request.name,
            toxicity,
            enthusiasm,
            &request.expertise_domains.unwrap_or_default(),
            request.speech_suffix,
            &request.trigger_keywords.unwrap_or_default()
        )
        .fetch_one(&self.pool)
        .await?;

        // Initialize message stats
        sqlx::query!(
            r#"INSERT INTO agent_message_stats (agent_id) VALUES ($1)"#,
            agent.id
        )
        .execute(&self.pool)
        .await?;

        info!("Created AI Agent {} for group {}", agent.id, group_id);
        Ok(agent)
    }

    /// Get AI Agent by ID
    pub async fn get_agent(&self, agent_id: Uuid) -> Result<AIAgent, AIAgentError> {
        sqlx::query_as!(
            AIAgent,
            r#"
            SELECT id, group_id, name, toxicity_level, enthusiasm_level, expertise_domains, speech_suffix, trigger_keywords, is_active, created_at, updated_at
            FROM ai_agents WHERE id = $1
            "#,
            agent_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AIAgentError::AgentNotFound(agent_id))
    }

    /// Get AI Agent for a group
    pub async fn get_agent_for_group(&self, group_id: Uuid) -> Result<Option<AIAgent>, AIAgentError> {
        let agent = sqlx::query_as!(
            AIAgent,
            r#"
            SELECT id, group_id, name, toxicity_level, enthusiasm_level, expertise_domains, speech_suffix, trigger_keywords, is_active, created_at, updated_at
            FROM ai_agents WHERE group_id = $1 AND is_active = TRUE
            "#,
            group_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(agent)
    }


    /// Update AI Agent personality
    pub async fn update_personality(
        &self,
        agent_id: Uuid,
        updates: UpdatePersonality,
    ) -> Result<AIAgent, AIAgentError> {
        // Validate if provided
        if let Some(ref name) = updates.name {
            Self::validate_name(name)?;
        }
        if let Some(toxicity) = updates.toxicity_level {
            if !(0..=10).contains(&toxicity) {
                return Err(AIAgentError::InvalidToxicityLevel);
            }
        }
        if let Some(enthusiasm) = updates.enthusiasm_level {
            if !(0..=10).contains(&enthusiasm) {
                return Err(AIAgentError::InvalidEnthusiasmLevel);
            }
        }

        let agent = sqlx::query_as!(
            AIAgent,
            r#"
            UPDATE ai_agents
            SET name = COALESCE($2, name),
                toxicity_level = COALESCE($3, toxicity_level),
                enthusiasm_level = COALESCE($4, enthusiasm_level),
                expertise_domains = COALESCE($5, expertise_domains),
                speech_suffix = COALESCE($6, speech_suffix),
                trigger_keywords = COALESCE($7, trigger_keywords),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, group_id, name, toxicity_level, enthusiasm_level, expertise_domains, speech_suffix, trigger_keywords, is_active, created_at, updated_at
            "#,
            agent_id,
            updates.name,
            updates.toxicity_level,
            updates.enthusiasm_level,
            updates.expertise_domains.as_deref(),
            updates.speech_suffix,
            updates.trigger_keywords.as_deref()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AIAgentError::AgentNotFound(agent_id))?;

        info!("Updated AI Agent {} personality", agent_id);
        Ok(agent)
    }

    /// Delete AI Agent
    pub async fn delete_agent(&self, agent_id: Uuid) -> Result<(), AIAgentError> {
        let result = sqlx::query!(
            r#"UPDATE ai_agents SET is_active = FALSE WHERE id = $1"#,
            agent_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AIAgentError::AgentNotFound(agent_id));
        }

        info!("Deactivated AI Agent {}", agent_id);
        Ok(())
    }

    /// Evaluate if agent should respond to a message
    pub async fn evaluate_trigger(
        &self,
        agent: &AIAgent,
        message_content: &str,
        sender_id: &str,
        last_message_time: Option<chrono::DateTime<Utc>>,
    ) -> TriggerDecision {
        let context = GroupContext {
            group_id: agent.group_id,
            last_message_time,
            agent: agent.clone(),
        };

        self.trigger_evaluator
            .evaluate(message_content, sender_id, &context)
            .await
    }

    /// Log trigger evaluation
    pub async fn log_trigger(
        &self,
        agent_id: Uuid,
        group_id: Uuid,
        message_id: Uuid,
        decision: &TriggerDecision,
    ) -> Result<(), AIAgentError> {
        sqlx::query!(
            r#"
            INSERT INTO agent_trigger_logs (agent_id, group_id, message_id, trigger_type, did_respond, probability)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            agent_id,
            group_id,
            message_id,
            decision.trigger_type.to_string(),
            decision.should_respond,
            decision.probability
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Generate AI response
    pub async fn generate_response(
        &self,
        agent: &AIAgent,
        message_content: &str,
        recent_messages: &[String],
        historical_context: &[HistoricalMessage],
    ) -> Result<String, AIAgentError> {
        let llm_client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| AIAgentError::Internal("LLM client not configured".to_string()))?;

        // Load personality config
        let personality = load_personality_config(&self.pool, agent).await?;

        // Generate system prompt
        let system_prompt = self.personality_manager.generate_system_prompt(
            &personality,
            recent_messages,
            historical_context,
        );

        debug!("Generating response for agent {}", agent.id);

        // Call LLM
        let response = llm_client
            .generate_response(&system_prompt, message_content)
            .await?;

        // Apply speech suffix if configured
        let response = if let Some(ref suffix) = agent.speech_suffix {
            // Apply suffix to ~70% of messages
            if rand::random::<f32>() < 0.7 {
                format!("{}{}", response.trim_end(), suffix)
            } else {
                response
            }
        } else {
            response
        };

        Ok(response)
    }

    /// Simulate typing and return the delay
    pub fn calculate_typing_delay(&self, message: &str) -> std::time::Duration {
        self.typing_simulator.calculate_delay(message)
    }

    /// Simulate typing (async wait)
    pub async fn simulate_typing(&self, message: &str) {
        self.typing_simulator.simulate_typing(message).await;
    }

    /// Update message statistics
    pub async fn update_message_stats(
        &self,
        agent_id: Uuid,
        response_time_ms: i32,
    ) -> Result<(), AIAgentError> {
        // Use a simpler update that doesn't require complex arithmetic
        sqlx::query!(
            r#"
            UPDATE agent_message_stats
            SET total_messages = total_messages + 1,
                last_message_at = NOW(),
                avg_response_time_ms = $2
            WHERE agent_id = $1
            "#,
            agent_id,
            response_time_ms
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get agent statistics
    pub async fn get_statistics(&self, agent_id: Uuid) -> Result<AgentStatistics, AIAgentError> {
        // Get message stats
        let stats = sqlx::query_as!(
            AgentMessageStats,
            r#"
            SELECT agent_id, total_messages, last_message_at, avg_response_time_ms
            FROM agent_message_stats WHERE agent_id = $1
            "#,
            agent_id
        )
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(AgentMessageStats {
            agent_id,
            total_messages: 0,
            last_message_at: None,
            avg_response_time_ms: None,
        });

        // Get trigger counts for last 30 days
        let trigger_counts = self.get_trigger_counts(agent_id).await?;

        Ok(AgentStatistics {
            agent_id,
            total_messages: stats.total_messages,
            last_message_at: stats.last_message_at,
            avg_response_time_ms: stats.avg_response_time_ms,
            trigger_counts,
        })
    }

    /// Get trigger counts by type
    async fn get_trigger_counts(&self, agent_id: Uuid) -> Result<TriggerCounts, AIAgentError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                trigger_type,
                COUNT(*) as count,
                SUM(CASE WHEN did_respond THEN 1 ELSE 0 END) as responded
            FROM agent_trigger_logs
            WHERE agent_id = $1 AND created_at > NOW() - INTERVAL '30 days'
            GROUP BY trigger_type
            "#,
            agent_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut counts = TriggerCounts::default();
        for row in rows {
            let count = row.count.unwrap_or(0);
            let responded = row.responded.unwrap_or(0);
            counts.total_evaluations += count;
            counts.total_responses += responded;

            match row.trigger_type.as_str() {
                "mention" => counts.mention = count,
                "keyword" => counts.keyword = count,
                "context_similarity" => counts.context_similarity = count,
                "random" => counts.random = count,
                "silence_breaker" => counts.silence_breaker = count,
                _ => {}
            }
        }

        Ok(counts)
    }

    /// Set user affinity for an agent
    pub async fn set_user_affinity(
        &self,
        agent_id: Uuid,
        user_id: Uuid,
        affinity_score: i16,
    ) -> Result<(), AIAgentError> {
        if !(-10..=10).contains(&affinity_score) {
            return Err(AIAgentError::InvalidPersonality(
                "Affinity score must be between -10 and 10".to_string(),
            ));
        }

        sqlx::query!(
            r#"
            INSERT INTO agent_user_affinities (agent_id, user_id, affinity_score)
            VALUES ($1, $2, $3)
            ON CONFLICT (agent_id, user_id) DO UPDATE SET affinity_score = $3
            "#,
            agent_id,
            user_id,
            affinity_score
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ============== Vector Store Methods ==============

    /// Store a message in the vector database for semantic search
    pub async fn store_message_embedding(
        &self,
        group_id: Uuid,
        message_id: Uuid,
        sender_id: Uuid,
        content: &str,
    ) -> Result<(), AIAgentError> {
        let embedding_service = self.embedding_service.as_ref().ok_or_else(|| {
            AIAgentError::Internal("Embedding service not configured".to_string())
        })?;

        let vector_store = self.vector_store.as_ref().ok_or_else(|| {
            AIAgentError::Internal("Vector store not configured".to_string())
        })?;

        // Generate embedding
        let embedding = embedding_service.embed(content).await?;

        // Store in vector database
        let timestamp = Utc::now().timestamp();
        vector_store
            .store_message(group_id, message_id, sender_id, content, embedding, timestamp)
            .await?;

        debug!("Stored message {} embedding in vector store", message_id);
        Ok(())
    }

    /// Search for similar historical messages
    pub async fn search_similar_messages(
        &self,
        group_id: Uuid,
        query: &str,
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<HistoricalMessage>, AIAgentError> {
        let embedding_service = self.embedding_service.as_ref().ok_or_else(|| {
            AIAgentError::Internal("Embedding service not configured".to_string())
        })?;

        let vector_store = self.vector_store.as_ref().ok_or_else(|| {
            AIAgentError::Internal("Vector store not configured".to_string())
        })?;

        // Generate query embedding
        let query_embedding = embedding_service.embed(query).await?;

        // Search vector store
        let results = vector_store
            .search_similar(group_id, query_embedding, limit, min_score)
            .await?;

        Ok(results)
    }

    /// Evaluate trigger with context similarity check
    pub async fn evaluate_trigger_with_context(
        &self,
        agent: &AIAgent,
        message_content: &str,
        sender_id: &str,
        last_message_time: Option<chrono::DateTime<Utc>>,
    ) -> TriggerDecision {
        // First check basic triggers
        let basic_decision = self
            .evaluate_trigger(agent, message_content, sender_id, last_message_time)
            .await;

        // If already triggered by mention or keyword, return immediately
        if basic_decision.should_respond
            && matches!(
                basic_decision.trigger_type,
                TriggerType::Mention | TriggerType::Keyword
            )
        {
            return basic_decision;
        }

        // Try context similarity if vector store is available
        if self.vector_store.is_some() && self.embedding_service.is_some() {
            if let Ok(similar_messages) = self
                .search_similar_messages(agent.group_id, message_content, 5, 0.8)
                .await
            {
                if !similar_messages.is_empty() {
                    // Found similar historical context - 40-60% trigger probability
                    let probability = rand::random::<f32>() * 0.2 + 0.4;
                    let should_respond = rand::random::<f32>() < probability;

                    return TriggerDecision {
                        should_respond,
                        probability,
                        trigger_type: TriggerType::ContextSimilarity,
                    };
                }
            }
        }

        // Fall back to basic decision
        basic_decision
    }

    /// Process incoming message - store embedding and evaluate trigger
    pub async fn process_message(
        &self,
        agent: &AIAgent,
        message_id: Uuid,
        sender_id: Uuid,
        content: &str,
        last_message_time: Option<chrono::DateTime<Utc>>,
    ) -> Result<(TriggerDecision, Vec<HistoricalMessage>), AIAgentError> {
        // Store message embedding (if configured)
        if self.vector_store.is_some() && self.embedding_service.is_some() {
            if let Err(e) = self
                .store_message_embedding(agent.group_id, message_id, sender_id, content)
                .await
            {
                // Log but don't fail - embedding storage is not critical
                debug!("Failed to store message embedding: {}", e);
            }
        }

        // Evaluate trigger with context
        let decision = self
            .evaluate_trigger_with_context(
                agent,
                content,
                &sender_id.to_string(),
                last_message_time,
            )
            .await;

        // Get historical context if we should respond
        let historical_context = if decision.should_respond {
            self.search_similar_messages(agent.group_id, content, 5, 0.6)
                .await
                .unwrap_or_default()
        } else {
            vec![]
        };

        Ok((decision, historical_context))
    }

    /// Delete all message embeddings for a group
    pub async fn delete_group_embeddings(&self, group_id: Uuid) -> Result<(), AIAgentError> {
        if let Some(vector_store) = &self.vector_store {
            vector_store.delete_group_messages(group_id).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(AIAgentService::validate_name("Bot").is_ok());
        assert!(AIAgentService::validate_name("A".repeat(100).as_str()).is_ok());
        assert!(AIAgentService::validate_name("").is_err());
        assert!(AIAgentService::validate_name("A".repeat(101).as_str()).is_err());
    }

    #[test]
    fn test_validate_levels() {
        assert!(AIAgentService::validate_levels(0, 0).is_ok());
        assert!(AIAgentService::validate_levels(10, 10).is_ok());
        assert!(AIAgentService::validate_levels(5, 5).is_ok());
        assert!(AIAgentService::validate_levels(-1, 5).is_err());
        assert!(AIAgentService::validate_levels(11, 5).is_err());
        assert!(AIAgentService::validate_levels(5, -1).is_err());
        assert!(AIAgentService::validate_levels(5, 11).is_err());
    }
}
