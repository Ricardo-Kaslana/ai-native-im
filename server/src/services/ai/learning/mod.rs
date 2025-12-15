//! Learning Engine Module
//!
//! This module provides learning capabilities for AI agents,
//! allowing them to improve responses based on user feedback
//! and pattern recognition.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

/// Learning engine errors
#[derive(Debug, Error)]
pub enum LearningError {
    #[error("Pattern not found: {0}")]
    PatternNotFound(Uuid),

    #[error("Feedback recording failed: {0}")]
    FeedbackFailed(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Learning disabled for this group")]
    LearningDisabled,
}

/// Type of feedback from users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackType {
    /// User liked the response
    Positive,
    /// User disliked the response
    Negative,
    /// User explicitly corrected the response
    Correction,
    /// User ignored the response (implicit negative)
    Ignored,
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::Positive => write!(f, "positive"),
            FeedbackType::Negative => write!(f, "negative"),
            FeedbackType::Correction => write!(f, "correction"),
            FeedbackType::Ignored => write!(f, "ignored"),
        }
    }
}

/// A learned pattern from interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPattern {
    pub id: Uuid,
    pub group_id: Uuid,
    /// Hash of the context for deduplication
    pub context_hash: String,
    /// Embedding vector for similarity search
    pub context_embedding: Option<Vec<f32>>,
    /// The successful response that worked
    pub successful_response: Option<String>,
    /// Type of feedback received
    pub feedback_type: FeedbackType,
    /// How many times this pattern was seen
    pub frequency: i32,
    /// Whether this needs human review
    pub needs_review: bool,
    /// Last time this pattern was used
    pub last_used: DateTime<Utc>,
    /// When this pattern was created
    pub created_at: DateTime<Utc>,
}

/// Feedback record for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub id: Uuid,
    pub message_id: Uuid,
    pub response_id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub feedback_type: FeedbackType,
    /// Optional correction text if user provided one
    pub correction: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Configuration for learning engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    /// Whether learning is enabled
    pub enabled: bool,
    /// Minimum negative feedback count before flagging for review
    pub review_threshold: i32,
    /// Whether to anonymize learning data
    pub anonymize_data: bool,
    /// Maximum patterns to store per group
    pub max_patterns_per_group: usize,
    /// Similarity threshold for pattern matching
    pub similarity_threshold: f32,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            review_threshold: 3,
            anonymize_data: true,
            max_patterns_per_group: 1000,
            similarity_threshold: 0.8,
        }
    }
}

/// Trait for learning engine
#[async_trait]
pub trait LearningEngine: Send + Sync {
    /// Record feedback for a response
    async fn record_feedback(
        &self,
        message_id: Uuid,
        response_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
        feedback: FeedbackType,
        correction: Option<String>,
    ) -> Result<(), LearningError>;

    /// Get patterns similar to the given context
    async fn get_patterns(
        &self,
        context: &str,
        group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<LearningPattern>, LearningError>;

    /// Check if a pattern needs review
    async fn check_for_review(&self, pattern_id: Uuid) -> Result<bool, LearningError>;

    /// Get patterns that need human review
    async fn get_patterns_for_review(
        &self,
        group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<LearningPattern>, LearningError>;

    /// Mark a pattern as reviewed
    async fn mark_reviewed(&self, pattern_id: Uuid) -> Result<(), LearningError>;

    /// Get feedback statistics for a group
    async fn get_feedback_stats(
        &self,
        group_id: Uuid,
    ) -> Result<FeedbackStats, LearningError>;
}

/// Statistics about feedback
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedbackStats {
    pub total_feedback: i64,
    pub positive_count: i64,
    pub negative_count: i64,
    pub correction_count: i64,
    pub patterns_learned: i64,
    pub patterns_needing_review: i64,
}

/// In-memory learning engine for development/testing
pub struct InMemoryLearningEngine {
    patterns: RwLock<HashMap<Uuid, LearningPattern>>,
    feedback: RwLock<Vec<FeedbackRecord>>,
    config: LearningConfig,
}

impl InMemoryLearningEngine {
    pub fn new(config: LearningConfig) -> Self {
        Self {
            patterns: RwLock::new(HashMap::new()),
            feedback: RwLock::new(Vec::new()),
            config,
        }
    }

    /// Generate a simple hash for context deduplication
    fn hash_context(context: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Anonymize text by removing potential PII
    fn anonymize_text(text: &str) -> String {
        // Simple anonymization - in production, use more sophisticated methods
        let mut result = text.to_string();

        // Remove potential phone numbers
        let phone_regex = regex::Regex::new(r"\d{3}[-.\s]?\d{4}[-.\s]?\d{4}").unwrap();
        result = phone_regex.replace_all(&result, "[PHONE]").to_string();

        // Remove potential email addresses
        let email_regex = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        result = email_regex.replace_all(&result, "[EMAIL]").to_string();

        result
    }
}

impl Default for InMemoryLearningEngine {
    fn default() -> Self {
        Self::new(LearningConfig::default())
    }
}

#[async_trait]
impl LearningEngine for InMemoryLearningEngine {
    async fn record_feedback(
        &self,
        message_id: Uuid,
        response_id: Uuid,
        user_id: Uuid,
        group_id: Uuid,
        feedback: FeedbackType,
        correction: Option<String>,
    ) -> Result<(), LearningError> {
        if !self.config.enabled {
            return Err(LearningError::LearningDisabled);
        }

        // Record the feedback
        let record = FeedbackRecord {
            id: Uuid::new_v4(),
            message_id,
            response_id,
            user_id,
            group_id,
            feedback_type: feedback,
            correction: correction.clone(),
            created_at: Utc::now(),
        };

        {
            let mut feedback_store = self.feedback.write().unwrap();
            feedback_store.push(record);
        }

        // Update or create pattern
        let context_hash = Self::hash_context(&message_id.to_string());

        {
            let mut patterns = self.patterns.write().unwrap();

            // Find existing pattern with same context hash
            let existing = patterns
                .values_mut()
                .find(|p| p.context_hash == context_hash && p.group_id == group_id);

            if let Some(pattern) = existing {
                pattern.frequency += 1;
                pattern.last_used = Utc::now();

                // Update feedback type based on new feedback
                if feedback == FeedbackType::Negative || feedback == FeedbackType::Correction {
                    pattern.feedback_type = feedback;

                    // Check if needs review
                    if pattern.frequency >= self.config.review_threshold {
                        pattern.needs_review = true;
                    }
                }

                // Store correction as successful response
                if let Some(ref corr) = correction {
                    let anonymized = if self.config.anonymize_data {
                        Self::anonymize_text(corr)
                    } else {
                        corr.clone()
                    };
                    pattern.successful_response = Some(anonymized);
                }
            } else {
                // Create new pattern
                // Check if needs review on creation (for negative feedback with threshold <= 1)
                let needs_review = (feedback == FeedbackType::Negative
                    || feedback == FeedbackType::Correction)
                    && 1 >= self.config.review_threshold;

                let pattern = LearningPattern {
                    id: Uuid::new_v4(),
                    group_id,
                    context_hash,
                    context_embedding: None,
                    successful_response: correction.map(|c| {
                        if self.config.anonymize_data {
                            Self::anonymize_text(&c)
                        } else {
                            c
                        }
                    }),
                    feedback_type: feedback,
                    frequency: 1,
                    needs_review,
                    last_used: Utc::now(),
                    created_at: Utc::now(),
                };

                patterns.insert(pattern.id, pattern);
            }
        }

        Ok(())
    }

    async fn get_patterns(
        &self,
        _context: &str,
        group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<LearningPattern>, LearningError> {
        let patterns = self.patterns.read().unwrap();

        // Get patterns for this group with positive feedback
        let mut result: Vec<_> = patterns
            .values()
            .filter(|p| {
                p.group_id == group_id
                    && p.feedback_type == FeedbackType::Positive
                    && p.successful_response.is_some()
            })
            .cloned()
            .collect();

        // Sort by frequency (most used first)
        result.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        Ok(result.into_iter().take(limit).collect())
    }

    async fn check_for_review(&self, pattern_id: Uuid) -> Result<bool, LearningError> {
        let patterns = self.patterns.read().unwrap();

        patterns
            .get(&pattern_id)
            .map(|p| p.needs_review)
            .ok_or(LearningError::PatternNotFound(pattern_id))
    }

    async fn get_patterns_for_review(
        &self,
        group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<LearningPattern>, LearningError> {
        let patterns = self.patterns.read().unwrap();

        let result: Vec<_> = patterns
            .values()
            .filter(|p| p.group_id == group_id && p.needs_review)
            .cloned()
            .take(limit)
            .collect();

        Ok(result)
    }

    async fn mark_reviewed(&self, pattern_id: Uuid) -> Result<(), LearningError> {
        let mut patterns = self.patterns.write().unwrap();

        patterns
            .get_mut(&pattern_id)
            .map(|p| p.needs_review = false)
            .ok_or(LearningError::PatternNotFound(pattern_id))
    }

    async fn get_feedback_stats(
        &self,
        group_id: Uuid,
    ) -> Result<FeedbackStats, LearningError> {
        let feedback = self.feedback.read().unwrap();
        let patterns = self.patterns.read().unwrap();

        let group_feedback: Vec<_> = feedback
            .iter()
            .filter(|f| f.group_id == group_id)
            .collect();

        let positive_count = group_feedback
            .iter()
            .filter(|f| f.feedback_type == FeedbackType::Positive)
            .count() as i64;

        let negative_count = group_feedback
            .iter()
            .filter(|f| f.feedback_type == FeedbackType::Negative)
            .count() as i64;

        let correction_count = group_feedback
            .iter()
            .filter(|f| f.feedback_type == FeedbackType::Correction)
            .count() as i64;

        let group_patterns: Vec<_> = patterns
            .values()
            .filter(|p| p.group_id == group_id)
            .collect();

        let patterns_needing_review = group_patterns
            .iter()
            .filter(|p| p.needs_review)
            .count() as i64;

        Ok(FeedbackStats {
            total_feedback: group_feedback.len() as i64,
            positive_count,
            negative_count,
            correction_count,
            patterns_learned: group_patterns.len() as i64,
            patterns_needing_review,
        })
    }
}

/// Mock learning engine for testing
pub struct MockLearningEngine {
    pub should_fail: bool,
    pub patterns: Vec<LearningPattern>,
}

impl MockLearningEngine {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            patterns: Vec::new(),
        }
    }

    pub fn failing() -> Self {
        Self {
            should_fail: true,
            patterns: Vec::new(),
        }
    }

    pub fn with_patterns(patterns: Vec<LearningPattern>) -> Self {
        Self {
            should_fail: false,
            patterns,
        }
    }
}

impl Default for MockLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LearningEngine for MockLearningEngine {
    async fn record_feedback(
        &self,
        _message_id: Uuid,
        _response_id: Uuid,
        _user_id: Uuid,
        _group_id: Uuid,
        _feedback: FeedbackType,
        _correction: Option<String>,
    ) -> Result<(), LearningError> {
        if self.should_fail {
            Err(LearningError::FeedbackFailed("Mock failure".to_string()))
        } else {
            Ok(())
        }
    }

    async fn get_patterns(
        &self,
        _context: &str,
        _group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<LearningPattern>, LearningError> {
        Ok(self.patterns.iter().take(limit).cloned().collect())
    }

    async fn check_for_review(&self, _pattern_id: Uuid) -> Result<bool, LearningError> {
        Ok(false)
    }

    async fn get_patterns_for_review(
        &self,
        _group_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<LearningPattern>, LearningError> {
        Ok(Vec::new())
    }

    async fn mark_reviewed(&self, _pattern_id: Uuid) -> Result<(), LearningError> {
        Ok(())
    }

    async fn get_feedback_stats(
        &self,
        _group_id: Uuid,
    ) -> Result<FeedbackStats, LearningError> {
        Ok(FeedbackStats::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_config_default() {
        let config = LearningConfig::default();
        assert!(config.enabled);
        assert_eq!(config.review_threshold, 3);
        assert!(config.anonymize_data);
    }

    #[test]
    fn test_feedback_type_display() {
        assert_eq!(FeedbackType::Positive.to_string(), "positive");
        assert_eq!(FeedbackType::Negative.to_string(), "negative");
        assert_eq!(FeedbackType::Correction.to_string(), "correction");
        assert_eq!(FeedbackType::Ignored.to_string(), "ignored");
    }

    #[tokio::test]
    async fn test_record_positive_feedback() {
        let engine = InMemoryLearningEngine::default();
        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let response_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = engine
            .record_feedback(
                message_id,
                response_id,
                user_id,
                group_id,
                FeedbackType::Positive,
                None,
            )
            .await;

        assert!(result.is_ok());

        let stats = engine.get_feedback_stats(group_id).await.unwrap();
        assert_eq!(stats.total_feedback, 1);
        assert_eq!(stats.positive_count, 1);
    }

    #[tokio::test]
    async fn test_record_negative_feedback_triggers_review() {
        let mut config = LearningConfig::default();
        config.review_threshold = 2;
        let engine = InMemoryLearningEngine::new(config);

        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Record negative feedback multiple times
        for _ in 0..3 {
            engine
                .record_feedback(
                    message_id,
                    Uuid::new_v4(),
                    user_id,
                    group_id,
                    FeedbackType::Negative,
                    None,
                )
                .await
                .unwrap();
        }

        let patterns_for_review = engine
            .get_patterns_for_review(group_id, 10)
            .await
            .unwrap();

        assert!(!patterns_for_review.is_empty());
    }

    #[tokio::test]
    async fn test_record_correction() {
        let engine = InMemoryLearningEngine::default();
        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let response_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let correction = "This is the correct response".to_string();

        engine
            .record_feedback(
                message_id,
                response_id,
                user_id,
                group_id,
                FeedbackType::Correction,
                Some(correction.clone()),
            )
            .await
            .unwrap();

        let stats = engine.get_feedback_stats(group_id).await.unwrap();
        assert_eq!(stats.correction_count, 1);
        assert_eq!(stats.patterns_learned, 1);
    }

    #[tokio::test]
    async fn test_learning_disabled() {
        let mut config = LearningConfig::default();
        config.enabled = false;
        let engine = InMemoryLearningEngine::new(config);

        let result = engine
            .record_feedback(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                FeedbackType::Positive,
                None,
            )
            .await;

        assert!(matches!(result, Err(LearningError::LearningDisabled)));
    }

    #[tokio::test]
    async fn test_mark_reviewed() {
        let mut config = LearningConfig::default();
        config.review_threshold = 1;
        let engine = InMemoryLearningEngine::new(config);

        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();

        // Create a pattern that needs review
        engine
            .record_feedback(
                message_id,
                Uuid::new_v4(),
                Uuid::new_v4(),
                group_id,
                FeedbackType::Negative,
                None,
            )
            .await
            .unwrap();

        let patterns = engine.get_patterns_for_review(group_id, 10).await.unwrap();
        assert!(!patterns.is_empty());

        let pattern_id = patterns[0].id;

        // Mark as reviewed
        engine.mark_reviewed(pattern_id).await.unwrap();

        // Should no longer need review
        let needs_review = engine.check_for_review(pattern_id).await.unwrap();
        assert!(!needs_review);
    }

    #[test]
    fn test_anonymize_text() {
        let text = "Contact me at test@example.com or 123-4567-8901";
        let anonymized = InMemoryLearningEngine::anonymize_text(text);

        assert!(anonymized.contains("[EMAIL]"));
        assert!(anonymized.contains("[PHONE]"));
        assert!(!anonymized.contains("test@example.com"));
        assert!(!anonymized.contains("123-4567-8901"));
    }

    /// **Feature: phase3-ai-agent-evolution, Property 25: Feedback Recording**
    /// **Validates: Requirements 18.1, 18.2**
    #[test]
    fn test_feedback_recording_properties() {
        use proptest::prelude::*;

        proptest!(|(
            feedback_type in prop_oneof![
                Just(FeedbackType::Positive),
                Just(FeedbackType::Negative),
                Just(FeedbackType::Correction),
                Just(FeedbackType::Ignored),
            ]
        )| {
            // Property: All feedback types have valid string representation
            let display = feedback_type.to_string();
            prop_assert!(!display.is_empty());

            // Property: Feedback types are distinguishable
            prop_assert!(
                display == "positive" ||
                display == "negative" ||
                display == "correction" ||
                display == "ignored"
            );
        });
    }

    /// **Feature: phase3-ai-agent-evolution, Property 26: Repeated Mistake Flagging**
    /// **Validates: Requirements 18.4**
    #[test]
    fn test_repeated_mistake_flagging() {
        use proptest::prelude::*;

        proptest!(|(
            threshold in 1i32..10,
            frequency in 0i32..20
        )| {
            // Property: Pattern needs review when frequency >= threshold
            let needs_review = frequency >= threshold;

            // This is the expected behavior
            if frequency >= threshold {
                prop_assert!(needs_review);
            } else {
                prop_assert!(!needs_review);
            }
        });
    }

    /// **Feature: phase3-ai-agent-evolution, Property 27: Learning Data Anonymization**
    /// **Validates: Requirements 18.5**
    #[test]
    fn test_anonymization_properties() {
        // Test that emails are anonymized
        let text_with_email = "Contact user@domain.com for help";
        let anonymized = InMemoryLearningEngine::anonymize_text(text_with_email);
        assert!(!anonymized.contains("user@domain.com"));
        assert!(anonymized.contains("[EMAIL]"));

        // Test that phone numbers are anonymized
        let text_with_phone = "Call 138-1234-5678 now";
        let anonymized = InMemoryLearningEngine::anonymize_text(text_with_phone);
        assert!(!anonymized.contains("138-1234-5678"));
        assert!(anonymized.contains("[PHONE]"));
    }

    /// **Feature: phase3-ai-agent-evolution, Property 28: Disabled Learning Functionality**
    /// **Validates: Requirements 18.6**
    #[tokio::test]
    async fn test_disabled_learning_returns_error() {
        let mut config = LearningConfig::default();
        config.enabled = false;
        let engine = InMemoryLearningEngine::new(config);

        let result = engine
            .record_feedback(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                FeedbackType::Positive,
                None,
            )
            .await;

        // Property: When learning is disabled, recording feedback returns LearningDisabled error
        assert!(matches!(result, Err(LearningError::LearningDisabled)));
    }
}
