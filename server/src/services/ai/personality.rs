use crate::models::{AIAgent, HistoricalMessage, PersonalityConfig, UserAffinityInfo};
use sqlx::PgPool;

use super::cot::{CoTConfig, CoTProcessor};
use super::intent::{IntentResult, generate_intent_prompt};
use super::user_profile::{UserProfile, UserProfileService};
use super::AIAgentError;

/// Personality manager trait
pub trait PersonalityManager: Send + Sync {
    fn generate_system_prompt(
        &self,
        config: &PersonalityConfig,
        recent_messages: &[String],
        historical_context: &[HistoricalMessage],
    ) -> String;
    
    /// Generate system prompt with Chain of Thought support
    fn generate_system_prompt_with_cot(
        &self,
        config: &PersonalityConfig,
        recent_messages: &[String],
        historical_context: &[HistoricalMessage],
        cot_config: Option<&CoTConfig>,
    ) -> String {
        let base_prompt = self.generate_system_prompt(config, recent_messages, historical_context);
        
        if let Some(cot_cfg) = cot_config {
            if cot_cfg.enabled {
                let processor = CoTProcessor::with_config(cot_cfg.clone());
                let cot_instruction = processor.generate_instruction();
                return format!("{}{}", base_prompt, cot_instruction);
            }
        }
        
        base_prompt
    }
    
    /// Generate system prompt with user profile context
    fn generate_system_prompt_with_user_profile(
        &self,
        config: &PersonalityConfig,
        recent_messages: &[String],
        historical_context: &[HistoricalMessage],
        user_profile: Option<&UserProfile>,
        username: &str,
        cot_config: Option<&CoTConfig>,
    ) -> String {
        let mut base_prompt = self.generate_system_prompt(config, recent_messages, historical_context);
        
        // Add user profile context
        if let Some(profile) = user_profile {
            let profile_service = UserProfileService::new(sqlx::postgres::PgPool::connect_lazy("postgres://").unwrap());
            let user_context = profile_service.generate_user_context(profile, username);
            base_prompt.push_str(&user_context);
        }
        
        // Add CoT instructions if enabled
        if let Some(cot_cfg) = cot_config {
            if cot_cfg.enabled {
                let processor = CoTProcessor::with_config(cot_cfg.clone());
                let cot_instruction = processor.generate_instruction();
                base_prompt.push_str(&cot_instruction);
            }
        }
        
        base_prompt
    }
    
    /// Generate comprehensive system prompt with all context
    /// Includes: user profile, intent analysis, and CoT
    fn generate_comprehensive_prompt(
        &self,
        config: &PersonalityConfig,
        recent_messages: &[String],
        historical_context: &[HistoricalMessage],
        user_profile: Option<&UserProfile>,
        username: &str,
        intent_result: Option<&IntentResult>,
        cot_config: Option<&CoTConfig>,
    ) -> String {
        let mut base_prompt = self.generate_system_prompt(config, recent_messages, historical_context);
        
        // Add user profile context
        if let Some(profile) = user_profile {
            let profile_service = UserProfileService::new(sqlx::postgres::PgPool::connect_lazy("postgres://").unwrap());
            let user_context = profile_service.generate_user_context(profile, username);
            base_prompt.push_str(&user_context);
        }
        
        // Add intent analysis context
        if let Some(intent) = intent_result {
            let intent_prompt = generate_intent_prompt(intent);
            base_prompt.push_str(&intent_prompt);
        }
        
        // Add CoT instructions if enabled
        if let Some(cot_cfg) = cot_config {
            if cot_cfg.enabled {
                let processor = CoTProcessor::with_config(cot_cfg.clone());
                let cot_instruction = processor.generate_instruction();
                base_prompt.push_str(&cot_instruction);
            }
        }
        
        base_prompt
    }
}

/// Default personality manager implementation
pub struct DefaultPersonalityManager;

impl DefaultPersonalityManager {
    pub fn new() -> Self {
        Self
    }

    /// Get toxicity description based on level
    fn toxicity_description(level: i16) -> &'static str {
        match level {
            0..=2 => "very polite and gentle",
            3..=4 => "friendly and considerate",
            5..=6 => "casual with occasional sarcasm",
            7..=8 => "sarcastic and witty",
            9..=10 => "very sarcastic and sharp-tongued",
            _ => "neutral",
        }
    }

    /// Get enthusiasm description based on level
    fn enthusiasm_description(level: i16) -> &'static str {
        match level {
            0..=2 => "reserved and calm",
            3..=4 => "moderately engaged",
            5..=6 => "friendly and responsive",
            7..=8 => "enthusiastic and energetic",
            9..=10 => "extremely excited and passionate",
            _ => "neutral",
        }
    }

    /// Get affinity description based on score
    fn affinity_description(score: i16) -> &'static str {
        match score {
            -10..=-7 => "strongly dislikes",
            -6..=-4 => "somewhat dislikes",
            -3..=-1 => "slightly cool towards",
            0 => "neutral towards",
            1..=3 => "slightly fond of",
            4..=6 => "likes",
            7..=10 => "really likes",
            _ => "neutral towards",
        }
    }
}

impl Default for DefaultPersonalityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonalityManager for DefaultPersonalityManager {
    fn generate_system_prompt(
        &self,
        config: &PersonalityConfig,
        recent_messages: &[String],
        historical_context: &[HistoricalMessage],
    ) -> String {
        let mut prompt = format!(
            r#"You are {name}, a member of this group chat with the following personality:

Personality Traits:
- Communication style: {toxicity_desc} (toxicity: {toxicity}/10)
- Energy level: {enthusiasm_desc} (enthusiasm: {enthusiasm}/10)
- Areas of expertise: {expertise}
"#,
            name = config.name,
            toxicity_desc = Self::toxicity_description(config.toxicity_level),
            toxicity = config.toxicity_level,
            enthusiasm_desc = Self::enthusiasm_description(config.enthusiasm_level),
            enthusiasm = config.enthusiasm_level,
            expertise = if config.expertise_domains.is_empty() {
                "general topics".to_string()
            } else {
                config.expertise_domains.join(", ")
            }
        );

        // Add speech suffix instruction
        if let Some(ref suffix) = config.speech_suffix {
            prompt.push_str(&format!(
                "- Speech pattern: End most of your messages with \"{}\"\n",
                suffix
            ));
        }

        // Add user relationships
        if !config.user_affinities.is_empty() {
            prompt.push_str("\nUser Relationships:\n");
            for affinity in &config.user_affinities {
                prompt.push_str(&format!(
                    "- {} {}: {}\n",
                    Self::affinity_description(affinity.affinity_score),
                    affinity.username,
                    affinity.username
                ));
            }
        }

        // Add recent conversation
        if !recent_messages.is_empty() {
            prompt.push_str("\nRecent Conversation:\n");
            for msg in recent_messages.iter().take(10) {
                prompt.push_str(&format!("{}\n", msg));
            }
        }

        // Add historical context
        if !historical_context.is_empty() {
            prompt.push_str("\nRelevant Historical Context:\n");
            for msg in historical_context.iter().take(5) {
                prompt.push_str(&format!(
                    "[{:.2} relevance] {}\n",
                    msg.similarity_score * msg.time_decay_weight,
                    msg.content
                ));
            }
        }

        // Add instructions
        prompt.push_str(&format!(
            r#"
Instructions:
- Respond naturally as {} would
- Reference historical context when relevant
- Don't mention you're an AI
- Keep responses concise (1-3 sentences typically)
- Match the group's conversation style
- Be helpful but maintain your personality
"#,
            config.name
        ));

        prompt
    }
}

/// Load personality config from database
pub async fn load_personality_config(
    pool: &PgPool,
    agent: &AIAgent,
) -> Result<PersonalityConfig, AIAgentError> {
    // Load user affinities
    let affinities = sqlx::query_as!(
        UserAffinityInfo,
        r#"
        SELECT 
            aua.user_id,
            u.username,
            aua.affinity_score
        FROM agent_user_affinities aua
        JOIN users u ON aua.user_id = u.id
        WHERE aua.agent_id = $1
        "#,
        agent.id
    )
    .fetch_all(pool)
    .await?;

    Ok(PersonalityConfig {
        agent_id: agent.id,
        name: agent.name.clone(),
        toxicity_level: agent.toxicity_level,
        enthusiasm_level: agent.enthusiasm_level,
        expertise_domains: agent.expertise_domains.clone(),
        speech_suffix: agent.speech_suffix.clone(),
        user_affinities: affinities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_toxicity_descriptions() {
        assert_eq!(DefaultPersonalityManager::toxicity_description(0), "very polite and gentle");
        assert_eq!(DefaultPersonalityManager::toxicity_description(5), "casual with occasional sarcasm");
        assert_eq!(DefaultPersonalityManager::toxicity_description(10), "very sarcastic and sharp-tongued");
    }

    #[test]
    fn test_enthusiasm_descriptions() {
        assert_eq!(DefaultPersonalityManager::enthusiasm_description(0), "reserved and calm");
        assert_eq!(DefaultPersonalityManager::enthusiasm_description(5), "friendly and responsive");
        assert_eq!(DefaultPersonalityManager::enthusiasm_description(10), "extremely excited and passionate");
    }

    #[test]
    fn test_generate_system_prompt() {
        let manager = DefaultPersonalityManager::new();
        let config = PersonalityConfig {
            agent_id: Uuid::new_v4(),
            name: "TestBot".to_string(),
            toxicity_level: 5,
            enthusiasm_level: 7,
            expertise_domains: vec!["programming".to_string(), "gaming".to_string()],
            speech_suffix: Some("~".to_string()),
            user_affinities: vec![],
        };

        let prompt = manager.generate_system_prompt(&config, &[], &[]);

        assert!(prompt.contains("TestBot"));
        assert!(prompt.contains("toxicity: 5/10"));
        assert!(prompt.contains("enthusiasm: 7/10"));
        assert!(prompt.contains("programming, gaming"));
        assert!(prompt.contains("End most of your messages with \"~\""));
    }

    #[test]
    fn test_generate_prompt_with_context() {
        let manager = DefaultPersonalityManager::new();
        let config = PersonalityConfig {
            agent_id: Uuid::new_v4(),
            name: "Bot".to_string(),
            toxicity_level: 3,
            enthusiasm_level: 5,
            expertise_domains: vec![],
            speech_suffix: None,
            user_affinities: vec![],
        };

        let recent = vec![
            "User1: Hello everyone!".to_string(),
            "User2: Hi there!".to_string(),
        ];

        let historical = vec![HistoricalMessage {
            message_id: "msg1".to_string(),
            content: "We talked about this before".to_string(),
            sender_id: "user1".to_string(),
            timestamp: 0,
            similarity_score: 0.9,
            time_decay_weight: 0.8,
        }];

        let prompt = manager.generate_system_prompt(&config, &recent, &historical);

        assert!(prompt.contains("Recent Conversation:"));
        assert!(prompt.contains("User1: Hello everyone!"));
        assert!(prompt.contains("Relevant Historical Context:"));
        assert!(prompt.contains("We talked about this before"));
    }
}
