//! User Profile Module
//!
//! Manages user personality profiles and relationship scores for AI agent personalization.
//! The AI agent learns about users through interactions and adapts its responses accordingly.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{debug, info};
use uuid::Uuid;

use crate::services::ai::AIAgentError;

/// User profile learned by an AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub user_id: Uuid,
    
    // Relationship scores (-10 to 10)
    pub intimacy_score: i16,
    pub respect_score: i16,
    pub trust_score: i16,
    
    // Personality traits
    pub traits: Vec<String>,
    
    // Communication preferences
    pub preferred_tone: String,
    pub preferred_response_length: String,
    
    // Interests
    pub interests: Vec<String>,
    pub sensitive_topics: Vec<String>,
    
    // Interaction stats
    pub total_interactions: i32,
    pub positive_interactions: i32,
    pub negative_interactions: i32,
    pub last_interaction_at: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id: Uuid::nil(),
            user_id: Uuid::nil(),
            intimacy_score: 0,
            respect_score: 0,
            trust_score: 0,
            traits: Vec::new(),
            preferred_tone: "neutral".to_string(),
            preferred_response_length: "medium".to_string(),
            interests: Vec::new(),
            sensitive_topics: Vec::new(),
            total_interactions: 0,
            positive_interactions: 0,
            negative_interactions: 0,
            last_interaction_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Relationship level description
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipLevel {
    Stranger,    // -10 to -6
    Distant,     // -5 to -2
    Neutral,     // -1 to 1
    Acquaintance, // 2 to 4
    Friend,      // 5 to 7
    CloseFriend, // 8 to 10
}

impl RelationshipLevel {
    pub fn from_score(score: i16) -> Self {
        match score {
            -10..=-6 => RelationshipLevel::Stranger,
            -5..=-2 => RelationshipLevel::Distant,
            -1..=1 => RelationshipLevel::Neutral,
            2..=4 => RelationshipLevel::Acquaintance,
            5..=7 => RelationshipLevel::Friend,
            8..=10 => RelationshipLevel::CloseFriend,
            _ => RelationshipLevel::Neutral,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RelationshipLevel::Stranger => "陌生人，保持距离",
            RelationshipLevel::Distant => "关系疏远，谨慎对待",
            RelationshipLevel::Neutral => "普通关系，正常交流",
            RelationshipLevel::Acquaintance => "熟人，可以开些玩笑",
            RelationshipLevel::Friend => "朋友，可以畅所欲言",
            RelationshipLevel::CloseFriend => "密友，无话不谈",
        }
    }

    pub fn tone_adjustment(&self) -> &'static str {
        match self {
            RelationshipLevel::Stranger => "正式、礼貌",
            RelationshipLevel::Distant => "客气、保守",
            RelationshipLevel::Neutral => "友好、适度",
            RelationshipLevel::Acquaintance => "轻松、亲切",
            RelationshipLevel::Friend => "随意、直接",
            RelationshipLevel::CloseFriend => "亲昵、打趣",
        }
    }
}

/// User profile update request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserProfile {
    pub intimacy_delta: Option<i16>,
    pub respect_delta: Option<i16>,
    pub trust_delta: Option<i16>,
    pub add_traits: Option<Vec<String>>,
    pub remove_traits: Option<Vec<String>>,
    pub add_interests: Option<Vec<String>>,
    pub remove_interests: Option<Vec<String>>,
    pub preferred_tone: Option<String>,
    pub preferred_response_length: Option<String>,
}

/// Interaction event for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub event_type: InteractionType,
    pub sentiment: Sentiment,
    pub context_summary: Option<String>,
    pub detected_traits: Vec<String>,
    pub detected_interests: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Message,
    Reaction,
    Mention,
    Feedback,
}

impl std::fmt::Display for InteractionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractionType::Message => write!(f, "message"),
            InteractionType::Reaction => write!(f, "reaction"),
            InteractionType::Mention => write!(f, "mention"),
            InteractionType::Feedback => write!(f, "feedback"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

impl std::fmt::Display for Sentiment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sentiment::Positive => write!(f, "positive"),
            Sentiment::Negative => write!(f, "negative"),
            Sentiment::Neutral => write!(f, "neutral"),
        }
    }
}

/// User Profile Service
pub struct UserProfileService {
    pool: PgPool,
}

impl UserProfileService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get or create a user profile
    pub async fn get_or_create_profile(
        &self,
        agent_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserProfile, AIAgentError> {
        // Try to get existing profile using dynamic query
        let existing: Option<UserProfileRow> = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT id, agent_id, user_id, intimacy_score, respect_score, trust_score,
                   traits, preferred_tone, preferred_response_length, interests,
                   sensitive_topics, total_interactions, positive_interactions,
                   negative_interactions, last_interaction_at, created_at, updated_at
            FROM agent_user_profiles
            WHERE agent_id = $1 AND user_id = $2
            "#
        )
        .bind(agent_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = existing {
            return Ok(row.into());
        }

        // Create new profile
        let profile: UserProfileRow = sqlx::query_as::<_, UserProfileRow>(
            r#"
            INSERT INTO agent_user_profiles (agent_id, user_id)
            VALUES ($1, $2)
            RETURNING id, agent_id, user_id, intimacy_score, respect_score, trust_score,
                      traits, preferred_tone, preferred_response_length, interests,
                      sensitive_topics, total_interactions, positive_interactions,
                      negative_interactions, last_interaction_at, created_at, updated_at
            "#
        )
        .bind(agent_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        info!("Created new user profile for agent {} user {}", agent_id, user_id);
        Ok(profile.into())
    }

    /// Record an interaction event and update profile
    pub async fn record_interaction(
        &self,
        agent_id: Uuid,
        user_id: Uuid,
        event: InteractionEvent,
        message_id: Option<Uuid>,
    ) -> Result<UserProfile, AIAgentError> {
        // Record the event using dynamic query
        sqlx::query(
            r#"
            INSERT INTO user_interaction_events 
            (agent_id, user_id, event_type, sentiment, context_summary, detected_traits, detected_interests, message_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(agent_id)
        .bind(user_id)
        .bind(event.event_type.to_string())
        .bind(event.sentiment.to_string())
        .bind(&event.context_summary)
        .bind(serde_json::to_value(&event.detected_traits).unwrap_or_default())
        .bind(serde_json::to_value(&event.detected_interests).unwrap_or_default())
        .bind(message_id)
        .execute(&self.pool)
        .await?;

        // Update profile based on sentiment
        let (intimacy_delta, positive_delta, negative_delta) = match event.sentiment {
            Sentiment::Positive => (1, 1, 0),
            Sentiment::Negative => (-1, 0, 1),
            Sentiment::Neutral => (0, 0, 0),
        };

        // Update the profile using dynamic query
        let profile: Option<UserProfileRow> = sqlx::query_as::<_, UserProfileRow>(
            r#"
            UPDATE agent_user_profiles
            SET intimacy_score = LEAST(10, GREATEST(-10, intimacy_score + $3)),
                total_interactions = total_interactions + 1,
                positive_interactions = positive_interactions + $4,
                negative_interactions = negative_interactions + $5,
                last_interaction_at = NOW(),
                updated_at = NOW()
            WHERE agent_id = $1 AND user_id = $2
            RETURNING id, agent_id, user_id, intimacy_score, respect_score, trust_score,
                      traits, preferred_tone, preferred_response_length, interests,
                      sensitive_topics, total_interactions, positive_interactions,
                      negative_interactions, last_interaction_at, created_at, updated_at
            "#
        )
        .bind(agent_id)
        .bind(user_id)
        .bind(intimacy_delta as i16)
        .bind(positive_delta)
        .bind(negative_delta)
        .fetch_optional(&self.pool)
        .await?;

        match profile {
            Some(p) => Ok(p.into()),
            None => self.get_or_create_profile(agent_id, user_id).await,
        }
    }

    /// Update user profile manually
    pub async fn update_profile(
        &self,
        agent_id: Uuid,
        user_id: Uuid,
        update: UpdateUserProfile,
    ) -> Result<UserProfile, AIAgentError> {
        // Get current profile
        let current = self.get_or_create_profile(agent_id, user_id).await?;

        // Calculate new scores
        let new_intimacy = (current.intimacy_score + update.intimacy_delta.unwrap_or(0))
            .clamp(-10, 10);
        let new_respect = (current.respect_score + update.respect_delta.unwrap_or(0))
            .clamp(-10, 10);
        let new_trust = (current.trust_score + update.trust_delta.unwrap_or(0))
            .clamp(-10, 10);

        // Update traits
        let mut traits = current.traits;
        if let Some(add) = update.add_traits {
            for t in add {
                if !traits.contains(&t) {
                    traits.push(t);
                }
            }
        }
        if let Some(remove) = update.remove_traits {
            traits.retain(|t| !remove.contains(t));
        }

        // Update interests
        let mut interests = current.interests;
        if let Some(add) = update.add_interests {
            for i in add {
                if !interests.contains(&i) {
                    interests.push(i);
                }
            }
        }
        if let Some(remove) = update.remove_interests {
            interests.retain(|i| !remove.contains(i));
        }

        let profile: UserProfileRow = sqlx::query_as::<_, UserProfileRow>(
            r#"
            UPDATE agent_user_profiles
            SET intimacy_score = $3,
                respect_score = $4,
                trust_score = $5,
                traits = $6,
                interests = $7,
                preferred_tone = COALESCE($8, preferred_tone),
                preferred_response_length = COALESCE($9, preferred_response_length),
                updated_at = NOW()
            WHERE agent_id = $1 AND user_id = $2
            RETURNING id, agent_id, user_id, intimacy_score, respect_score, trust_score,
                      traits, preferred_tone, preferred_response_length, interests,
                      sensitive_topics, total_interactions, positive_interactions,
                      negative_interactions, last_interaction_at, created_at, updated_at
            "#
        )
        .bind(agent_id)
        .bind(user_id)
        .bind(new_intimacy)
        .bind(new_respect)
        .bind(new_trust)
        .bind(serde_json::to_value(&traits).unwrap_or_default())
        .bind(serde_json::to_value(&interests).unwrap_or_default())
        .bind(&update.preferred_tone)
        .bind(&update.preferred_response_length)
        .fetch_one(&self.pool)
        .await?;

        debug!("Updated user profile for agent {} user {}", agent_id, user_id);
        Ok(profile.into())
    }

    /// Get all profiles for an agent
    pub async fn get_profiles_for_agent(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<UserProfile>, AIAgentError> {
        let profiles: Vec<UserProfileRow> = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT id, agent_id, user_id, intimacy_score, respect_score, trust_score,
                   traits, preferred_tone, preferred_response_length, interests,
                   sensitive_topics, total_interactions, positive_interactions,
                   negative_interactions, last_interaction_at, created_at, updated_at
            FROM agent_user_profiles
            WHERE agent_id = $1
            ORDER BY last_interaction_at DESC NULLS LAST
            "#
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(profiles.into_iter().map(|r| r.into()).collect())
    }

    /// Generate personalization hints for prompt
    pub fn generate_user_context(&self, profile: &UserProfile, username: &str) -> String {
        let relationship = RelationshipLevel::from_score(profile.intimacy_score);
        
        let mut context = format!(
            "\n与用户 {} 的关系：\n- 关系级别：{} ({})\n- 建议语气：{}\n",
            username,
            relationship.description(),
            profile.intimacy_score,
            relationship.tone_adjustment()
        );

        if !profile.traits.is_empty() {
            context.push_str(&format!(
                "- 用户特点：{}\n",
                profile.traits.join("、")
            ));
        }

        if !profile.interests.is_empty() {
            context.push_str(&format!(
                "- 用户兴趣：{}\n",
                profile.interests.join("、")
            ));
        }

        if !profile.sensitive_topics.is_empty() {
            context.push_str(&format!(
                "- 敏感话题（谨慎提及）：{}\n",
                profile.sensitive_topics.join("、")
            ));
        }

        context.push_str(&format!(
            "- 总交互次数：{}\n",
            profile.total_interactions
        ));

        context
    }
}

// Internal row type for database mapping
#[derive(Debug, sqlx::FromRow)]
struct UserProfileRow {
    id: Uuid,
    agent_id: Uuid,
    user_id: Uuid,
    intimacy_score: i16,
    respect_score: i16,
    trust_score: i16,
    traits: Option<serde_json::Value>,
    preferred_tone: Option<String>,
    preferred_response_length: Option<String>,
    interests: Option<serde_json::Value>,
    sensitive_topics: Option<serde_json::Value>,
    total_interactions: i32,
    positive_interactions: i32,
    negative_interactions: i32,
    last_interaction_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserProfileRow> for UserProfile {
    fn from(row: UserProfileRow) -> Self {
        Self {
            id: row.id,
            agent_id: row.agent_id,
            user_id: row.user_id,
            intimacy_score: row.intimacy_score,
            respect_score: row.respect_score,
            trust_score: row.trust_score,
            traits: row.traits
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            preferred_tone: row.preferred_tone.unwrap_or_else(|| "neutral".to_string()),
            preferred_response_length: row.preferred_response_length.unwrap_or_else(|| "medium".to_string()),
            interests: row.interests
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            sensitive_topics: row.sensitive_topics
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default(),
            total_interactions: row.total_interactions,
            positive_interactions: row.positive_interactions,
            negative_interactions: row.negative_interactions,
            last_interaction_at: row.last_interaction_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_level() {
        assert_eq!(RelationshipLevel::from_score(-10), RelationshipLevel::Stranger);
        assert_eq!(RelationshipLevel::from_score(0), RelationshipLevel::Neutral);
        assert_eq!(RelationshipLevel::from_score(5), RelationshipLevel::Friend);
        assert_eq!(RelationshipLevel::from_score(10), RelationshipLevel::CloseFriend);
    }

    #[test]
    fn test_generate_user_context() {
        let service = UserProfileService::new(PgPool::connect_lazy("postgres://").unwrap());
        let profile = UserProfile {
            intimacy_score: 7,
            traits: vec!["幽默".to_string(), "技术宅".to_string()],
            interests: vec!["编程".to_string(), "游戏".to_string()],
            total_interactions: 42,
            ..Default::default()
        };

        let context = service.generate_user_context(&profile, "张三");
        
        assert!(context.contains("张三"));
        assert!(context.contains("朋友"));
        assert!(context.contains("幽默"));
        assert!(context.contains("编程"));
        assert!(context.contains("42"));
    }
}
