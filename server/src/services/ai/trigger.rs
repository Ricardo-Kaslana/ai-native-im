use crate::models::{GroupContext, TriggerDecision, TriggerType};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::Rng;
use regex::Regex;

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

/// Default trigger evaluator implementation
pub struct DefaultTriggerEvaluator {
    mention_pattern: Regex,
}

impl DefaultTriggerEvaluator {
    pub fn new() -> Self {
        Self {
            mention_pattern: Regex::new(r"@(\w+)").unwrap(),
        }
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

    /// Check if group has been silent for more than 2 hours
    fn check_silence(&self, context: &GroupContext) -> bool {
        if let Some(last_time) = context.last_message_time {
            let silence_duration = Utc::now() - last_time;
            silence_duration > Duration::hours(2)
        } else {
            false
        }
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

        // Priority 1: Check for @mention (100% trigger)
        if self.check_mention(message_content, &agent.name) {
            return TriggerDecision {
                should_respond: true,
                probability: 1.0,
                trigger_type: TriggerType::Mention,
            };
        }

        // Priority 2: Check for keyword match (60-80% trigger)
        if self.check_keywords(message_content, &agent.trigger_keywords) {
            let probability = self.get_probability(TriggerType::Keyword);
            return TriggerDecision {
                should_respond: self.should_respond(probability),
                probability,
                trigger_type: TriggerType::Keyword,
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

        // Priority 4: Random interjection (1-5% trigger)
        let probability = self.get_probability(TriggerType::Random);
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
}
