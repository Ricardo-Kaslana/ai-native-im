use crate::models::{GroupContext, TriggerDecision, TriggerType};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Trigger evaluator trait
#[async_trait]
pub trait TriggerEvaluator: Send + Sync {
    async fn evaluate(
        &self,
        message_content: &str,
        sender_id: &str,
        context: &GroupContext,
    ) -> TriggerDecision;
}

/// Configuration for trigger evaluation
#[derive(Debug, Clone)]
pub struct TriggerConfig {
    /// Cooldown period after responding (in seconds)
    pub cooldown_secs: u64,
    /// Probability boost for keyword matches (multiplier)
    pub keyword_boost: f32,
    /// Probability boost for expertise domain matches (multiplier)
    pub expertise_boost: f32,
    /// Base random probability
    pub base_random_probability: f32,
    /// Silence threshold in hours
    pub silence_threshold_hours: i64,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            cooldown_secs: 30,
            keyword_boost: 1.2,
            expertise_boost: 1.3,
            base_random_probability: 0.03,
            silence_threshold_hours: 2,
        }
    }
}

/// Default trigger evaluator implementation
/// 
/// **Feature: phase3-ai-agent-evolution**
/// **Requirements: 9.1, 9.2, 9.3, 9.4, 9.5**
pub struct DefaultTriggerEvaluator {
    mention_pattern: Regex,
    config: TriggerConfig,
    /// Track last response time per group for cooldown
    last_response_times: RwLock<HashMap<Uuid, DateTime<Utc>>>,
}

impl DefaultTriggerEvaluator {
    pub fn new() -> Self {
        Self {
            mention_pattern: Regex::new(r"@(\w+)").unwrap(),
            config: TriggerConfig::default(),
            last_response_times: RwLock::new(HashMap::new()),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TriggerConfig) -> Self {
        Self {
            mention_pattern: Regex::new(r"@(\w+)").unwrap(),
            config,
            last_response_times: RwLock::new(HashMap::new()),
        }
    }

    /// Record that the agent responded in a group
    pub fn record_response(&self, group_id: Uuid) {
        if let Ok(mut times) = self.last_response_times.write() {
            times.insert(group_id, Utc::now());
        }
    }

    /// Check if the agent is in cooldown for a group
    fn is_in_cooldown(&self, group_id: Uuid) -> bool {
        if let Ok(times) = self.last_response_times.read() {
            if let Some(last_time) = times.get(&group_id) {
                let elapsed = Utc::now() - *last_time;
                return elapsed < Duration::seconds(self.config.cooldown_secs as i64);
            }
        }
        false
    }

    /// Check if message mentions the agent
    fn check_mention(&self, message: &str, agent_name: &str) -> bool {
        let lower_message = message.to_lowercase();
        let lower_name = agent_name.to_lowercase();
        
        // Check for @name pattern
        for cap in self.mention_pattern.captures_iter(&lower_message) {
            if let Some(name) = cap.get(1) {
                if name.as_str() == lower_name {
                    return true;
                }
            }
        }
        
        // Also check for direct name mention
        lower_message.contains(&format!("@{}", lower_name))
    }

    /// Check if message matches any trigger keywords
    fn check_keywords(&self, message: &str, keywords: &[String]) -> bool {
        let lower_message = message.to_lowercase();
        keywords.iter().any(|kw| {
            let lower_kw = kw.to_lowercase();
            lower_message.contains(&lower_kw)
        })
    }

    /// Check if group has been silent for configured threshold
    fn check_silence(&self, context: &GroupContext) -> bool {
        if let Some(last_time) = context.last_message_time {
            let silence_duration = Utc::now() - last_time;
            silence_duration > Duration::hours(self.config.silence_threshold_hours)
        } else {
            false
        }
    }

    /// Check if message matches any expertise domains
    fn check_expertise_domains(&self, message: &str, domains: &[String]) -> bool {
        if domains.is_empty() {
            return false;
        }
        let lower_message = message.to_lowercase();
        domains.iter().any(|domain| {
            let lower_domain = domain.to_lowercase();
            lower_message.contains(&lower_domain)
        })
    }

    /// Generate random probability based on trigger type
    fn get_probability(&self, trigger_type: TriggerType) -> f32 {
        let mut rng = rand::thread_rng();
        match trigger_type {
            TriggerType::Mention => 1.0,
            TriggerType::Keyword => rng.gen_range(0.6..=0.8),
            TriggerType::ContextSimilarity => rng.gen_range(0.4..=0.6),
            TriggerType::Random => rng.gen_range(0.01..=0.05),
            TriggerType::SilenceBreaker => rng.gen_range(0.1..=0.2),
        }
    }

    /// Decide if should respond based on probability
    fn should_respond(&self, probability: f32) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < probability
    }
}

impl Default for DefaultTriggerEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TriggerEvaluator for DefaultTriggerEvaluator {
    async fn evaluate(
        &self,
        message_content: &str,
        _sender_id: &str,
        context: &GroupContext,
    ) -> TriggerDecision {
        let agent = &context.agent;

        // Check cooldown first (except for mentions and replies)
        let in_cooldown = self.is_in_cooldown(context.group_id);

        // Priority 0: Check for direct reply to agent (95% trigger, ignores cooldown)
        if context.is_reply_to_agent {
            return TriggerDecision {
                should_respond: true,
                probability: 0.95,
                trigger_type: TriggerType::Mention, // Treat as mention-level priority
            };
        }

        // Priority 1: Check for @mention (100% trigger, ignores cooldown)
        if self.check_mention(message_content, &agent.name) {
            return TriggerDecision {
                should_respond: true,
                probability: 1.0,
                trigger_type: TriggerType::Mention,
            };
        }

        // Apply cooldown for lower priority triggers
        if in_cooldown {
            return TriggerDecision {
                should_respond: false,
                probability: 0.0,
                trigger_type: TriggerType::Random,
            };
        }

        // Priority 2: Check for keyword match (60-80% trigger with boost)
        if self.check_keywords(message_content, &agent.trigger_keywords) {
            let base_probability = self.get_probability(TriggerType::Keyword);
            let probability = (base_probability * self.config.keyword_boost).min(1.0);
            return TriggerDecision {
                should_respond: self.should_respond(probability),
                probability,
                trigger_type: TriggerType::Keyword,
            };
        }

        // Priority 2.5: Check for expertise domain match
        if self.check_expertise_domains(message_content, &agent.expertise_domains) {
            let base_probability = self.get_probability(TriggerType::ContextSimilarity);
            let probability = (base_probability * self.config.expertise_boost).min(1.0);
            return TriggerDecision {
                should_respond: self.should_respond(probability),
                probability,
                trigger_type: TriggerType::ContextSimilarity,
            };
        }

        // Priority 3: Check for silence breaker (10-20% trigger)
        if self.check_silence(context) {
            let probability = self.get_probability(TriggerType::SilenceBreaker);
            return TriggerDecision {
                should_respond: self.should_respond(probability),
                probability,
                trigger_type: TriggerType::SilenceBreaker,
            };
        }

        // Priority 4: Random interjection (configurable base probability)
        let probability = self.config.base_random_probability;
        TriggerDecision {
            should_respond: self.should_respond(probability),
            probability,
            trigger_type: TriggerType::Random,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AIAgent;
    use uuid::Uuid;

    fn create_test_agent(name: &str, keywords: Vec<String>) -> AIAgent {
        AIAgent {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            name: name.to_string(),
            toxicity_level: 5,
            enthusiasm_level: 5,
            expertise_domains: vec![],
            speech_suffix: None,
            trigger_keywords: keywords,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_context(agent: AIAgent, last_message_time: Option<chrono::DateTime<Utc>>) -> GroupContext {
        GroupContext {
            group_id: agent.group_id,
            last_message_time,
            agent,
            is_reply_to_agent: false,
        }
    }

    fn create_test_context_with_reply(agent: AIAgent, is_reply: bool) -> GroupContext {
        GroupContext {
            group_id: agent.group_id,
            last_message_time: Some(Utc::now()),
            agent,
            is_reply_to_agent: is_reply,
        }
    }

    #[test]
    fn test_mention_detection() {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent("TestBot", vec![]);
        
        assert!(evaluator.check_mention("Hey @TestBot how are you?", &agent.name));
        assert!(evaluator.check_mention("@testbot help me", &agent.name));
        assert!(!evaluator.check_mention("Hello everyone", &agent.name));
    }

    #[test]
    fn test_keyword_detection() {
        let evaluator = DefaultTriggerEvaluator::new();
        let keywords = vec!["help".to_string(), "question".to_string()];
        
        assert!(evaluator.check_keywords("I need help with this", &keywords));
        assert!(evaluator.check_keywords("I have a QUESTION", &keywords));
        assert!(!evaluator.check_keywords("Hello world", &keywords));
    }

    #[test]
    fn test_silence_detection() {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent("Bot", vec![]);
        
        // No last message
        let context = create_test_context(agent.clone(), None);
        assert!(!evaluator.check_silence(&context));
        
        // Recent message (1 hour ago)
        let context = create_test_context(agent.clone(), Some(Utc::now() - Duration::hours(1)));
        assert!(!evaluator.check_silence(&context));
        
        // Old message (3 hours ago)
        let context = create_test_context(agent, Some(Utc::now() - Duration::hours(3)));
        assert!(evaluator.check_silence(&context));
    }

    #[tokio::test]
    async fn test_mention_always_triggers() {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent("Bot", vec![]);
        let context = create_test_context(agent, Some(Utc::now()));
        
        let decision = evaluator.evaluate("@Bot hello", "user1", &context).await;
        
        assert!(decision.should_respond);
        assert_eq!(decision.probability, 1.0);
        assert_eq!(decision.trigger_type, TriggerType::Mention);
    }

    #[test]
    fn test_probability_ranges() {
        let evaluator = DefaultTriggerEvaluator::new();
        
        // Test multiple times to verify ranges
        for _ in 0..100 {
            let p = evaluator.get_probability(TriggerType::Mention);
            assert_eq!(p, 1.0);
            
            let p = evaluator.get_probability(TriggerType::Keyword);
            assert!(p >= 0.6 && p <= 0.8);
            
            let p = evaluator.get_probability(TriggerType::ContextSimilarity);
            assert!(p >= 0.4 && p <= 0.6);
            
            let p = evaluator.get_probability(TriggerType::Random);
            assert!(p >= 0.01 && p <= 0.05);
            
            let p = evaluator.get_probability(TriggerType::SilenceBreaker);
            assert!(p >= 0.1 && p <= 0.2);
        }
    }

    #[tokio::test]
    async fn test_reply_triggers() {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent("Bot", vec![]);
        let context = create_test_context_with_reply(agent, true);
        
        let decision = evaluator.evaluate("hello", "user1", &context).await;
        
        assert!(decision.should_respond);
        assert!(decision.probability >= 0.9);
    }

    #[tokio::test]
    async fn test_cooldown() {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent("Bot", vec!["help".to_string()]);
        let context = create_test_context(agent, Some(Utc::now()));
        
        // Record a response
        evaluator.record_response(context.group_id);
        
        // Keyword should not trigger during cooldown
        let decision = evaluator.evaluate("I need help", "user1", &context).await;
        assert!(!decision.should_respond);
        assert_eq!(decision.probability, 0.0);
    }

    #[tokio::test]
    async fn test_mention_ignores_cooldown() {
        let evaluator = DefaultTriggerEvaluator::new();
        let agent = create_test_agent("Bot", vec![]);
        let context = create_test_context(agent, Some(Utc::now()));
        
        // Record a response
        evaluator.record_response(context.group_id);
        
        // Mention should still trigger during cooldown
        let decision = evaluator.evaluate("@Bot hello", "user1", &context).await;
        assert!(decision.should_respond);
        assert_eq!(decision.probability, 1.0);
    }

    #[test]
    fn test_expertise_domain_detection() {
        let evaluator = DefaultTriggerEvaluator::new();
        let domains = vec!["rust".to_string(), "programming".to_string()];
        
        assert!(evaluator.check_expertise_domains("I'm learning Rust", &domains));
        assert!(evaluator.check_expertise_domains("programming is fun", &domains));
        assert!(!evaluator.check_expertise_domains("hello world", &domains));
        assert!(!evaluator.check_expertise_domains("test", &[]));
    }

    #[test]
    fn test_custom_config() {
        let config = TriggerConfig {
            cooldown_secs: 60,
            keyword_boost: 1.5,
            expertise_boost: 1.4,
            base_random_probability: 0.05,
            silence_threshold_hours: 4,
        };
        let evaluator = DefaultTriggerEvaluator::with_config(config.clone());
        
        assert_eq!(evaluator.config.cooldown_secs, 60);
        assert_eq!(evaluator.config.keyword_boost, 1.5);
    }
}
