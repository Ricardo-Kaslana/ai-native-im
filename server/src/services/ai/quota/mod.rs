//! Quota Management Module
//!
//! This module provides rate limiting and quota management for AI services,
//! tracking usage per user and group with daily/monthly limits.

use async_trait::async_trait;
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

/// Quota management errors
#[derive(Debug, Error)]
pub enum QuotaError {
    #[error("Daily quota exceeded for user {user_id}: used {used}/{limit}")]
    DailyQuotaExceeded {
        user_id: Uuid,
        used: i64,
        limit: i64,
    },

    #[error("Monthly quota exceeded for user {user_id}: used {used}/{limit}")]
    MonthlyQuotaExceeded {
        user_id: Uuid,
        used: i64,
        limit: i64,
    },

    #[error("Group quota exceeded for group {group_id}: used {used}/{limit}")]
    GroupQuotaExceeded {
        group_id: Uuid,
        used: i64,
        limit: i64,
    },

    #[error("Service quota exceeded for {service}: used {used}/{limit}")]
    ServiceQuotaExceeded {
        service: String,
        used: i64,
        limit: i64,
    },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Redis error: {0}")]
    RedisError(String),
}

/// Service types for quota tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    /// LLM text generation
    LLM,
    /// Text embedding generation
    Embedding,
    /// Speech-to-text
    STT,
    /// Text-to-speech
    TTS,
    /// Image generation
    ImageGen,
    /// Vision/image analysis
    Vision,
}

impl ServiceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceType::LLM => "llm",
            ServiceType::Embedding => "embedding",
            ServiceType::STT => "stt",
            ServiceType::TTS => "tts",
            ServiceType::ImageGen => "image_gen",
            ServiceType::Vision => "vision",
        }
    }

    /// Get cost per unit (approximate USD)
    pub fn cost_per_unit(&self) -> f64 {
        match self {
            ServiceType::LLM => 0.000002,      // $0.002 per 1K tokens
            ServiceType::Embedding => 0.0000001, // $0.0001 per 1K tokens
            ServiceType::STT => 0.0001,        // $0.006 per minute
            ServiceType::TTS => 0.000015,      // $0.015 per 1K characters
            ServiceType::ImageGen => 0.02,     // $0.02 per image
            ServiceType::Vision => 0.001,      // $0.001 per image
        }
    }
}

/// Quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Daily token limit for LLM
    pub daily_token_limit: i64,
    /// Monthly token limit for LLM
    pub monthly_token_limit: i64,
    /// Daily audio seconds limit for STT/TTS
    pub daily_audio_seconds: i64,
    /// Monthly audio seconds limit
    pub monthly_audio_seconds: i64,
    /// Daily image generation limit
    pub daily_image_limit: i64,
    /// Whether to enforce hard limits (vs soft warnings)
    pub enforce_hard_limits: bool,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            daily_token_limit: 100_000,
            monthly_token_limit: 3_000_000,
            daily_audio_seconds: 600,      // 10 minutes
            monthly_audio_seconds: 18_000, // 5 hours
            daily_image_limit: 10,
            enforce_hard_limits: true,
        }
    }
}

/// Current quota status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaStatus {
    /// Amount used in current period
    pub used: i64,
    /// Total limit for period
    pub limit: i64,
    /// Remaining quota
    pub remaining: i64,
    /// When quota resets
    pub reset_at: DateTime<Utc>,
    /// Percentage used (0-100)
    pub percentage_used: f32,
}

impl QuotaStatus {
    pub fn new(used: i64, limit: i64, reset_at: DateTime<Utc>) -> Self {
        let remaining = (limit - used).max(0);
        let percentage_used = if limit > 0 {
            (used as f32 / limit as f32 * 100.0).min(100.0)
        } else {
            0.0
        };

        Self {
            used,
            limit,
            remaining,
            reset_at,
            percentage_used,
        }
    }

    pub fn is_exceeded(&self) -> bool {
        self.used >= self.limit
    }

    pub fn is_warning(&self) -> bool {
        self.percentage_used >= 80.0
    }
}

/// Usage record for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub user_id: Uuid,
    pub group_id: Option<Uuid>,
    pub service_type: ServiceType,
    pub provider: String,
    pub tokens_used: i64,
    pub audio_seconds: f64,
    pub cost_estimate: f64,
    pub timestamp: DateTime<Utc>,
}

/// Trait for quota management
#[async_trait]
pub trait QuotaManager: Send + Sync {
    /// Check if user has quota available
    async fn check_quota(
        &self,
        user_id: Uuid,
        service_type: ServiceType,
        config: &QuotaConfig,
    ) -> Result<QuotaStatus, QuotaError>;

    /// Record usage
    async fn record_usage(&self, record: UsageRecord) -> Result<(), QuotaError>;

    /// Get user's current usage summary
    async fn get_usage_summary(
        &self,
        user_id: Uuid,
    ) -> Result<HashMap<ServiceType, QuotaStatus>, QuotaError>;

    /// Get group's current usage
    async fn get_group_usage(
        &self,
        group_id: Uuid,
    ) -> Result<HashMap<ServiceType, QuotaStatus>, QuotaError>;

    /// Reset daily quotas (called by scheduler)
    async fn reset_daily_quotas(&self) -> Result<(), QuotaError>;

    /// Get cost estimate for period
    async fn get_cost_estimate(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<f64, QuotaError>;
}

/// In-memory quota manager for development/testing
pub struct InMemoryQuotaManager {
    /// Usage records: (user_id, date, service_type) -> count
    daily_usage: RwLock<HashMap<(Uuid, String, ServiceType), i64>>,
    /// Monthly usage: (user_id, month, service_type) -> count
    monthly_usage: RwLock<HashMap<(Uuid, String, ServiceType), i64>>,
    /// All usage records for cost tracking
    records: RwLock<Vec<UsageRecord>>,
}

impl InMemoryQuotaManager {
    pub fn new() -> Self {
        Self {
            daily_usage: RwLock::new(HashMap::new()),
            monthly_usage: RwLock::new(HashMap::new()),
            records: RwLock::new(Vec::new()),
        }
    }

    fn today_key() -> String {
        Utc::now().format("%Y-%m-%d").to_string()
    }

    fn month_key() -> String {
        Utc::now().format("%Y-%m").to_string()
    }

    fn next_day_reset() -> DateTime<Utc> {
        let now = Utc::now();
        let tomorrow = now.date_naive().succ_opt().unwrap();
        tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc()
    }

    fn next_month_reset() -> DateTime<Utc> {
        let now = Utc::now();
        let next_month = if now.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
        };
        next_month
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    fn get_limit_for_service(service_type: ServiceType, config: &QuotaConfig) -> i64 {
        match service_type {
            ServiceType::LLM | ServiceType::Embedding => config.daily_token_limit,
            ServiceType::STT | ServiceType::TTS => config.daily_audio_seconds,
            ServiceType::ImageGen | ServiceType::Vision => config.daily_image_limit,
        }
    }
}

impl Default for InMemoryQuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QuotaManager for InMemoryQuotaManager {
    async fn check_quota(
        &self,
        user_id: Uuid,
        service_type: ServiceType,
        config: &QuotaConfig,
    ) -> Result<QuotaStatus, QuotaError> {
        let today = Self::today_key();
        let key = (user_id, today, service_type);

        let daily_usage = self.daily_usage.read().unwrap();
        let used = *daily_usage.get(&key).unwrap_or(&0);
        let limit = Self::get_limit_for_service(service_type, config);

        let status = QuotaStatus::new(used, limit, Self::next_day_reset());

        if config.enforce_hard_limits && status.is_exceeded() {
            return Err(QuotaError::DailyQuotaExceeded {
                user_id,
                used,
                limit,
            });
        }

        Ok(status)
    }

    async fn record_usage(&self, record: UsageRecord) -> Result<(), QuotaError> {
        let today = Self::today_key();
        let month = Self::month_key();

        // Calculate units based on service type
        let units = match record.service_type {
            ServiceType::LLM | ServiceType::Embedding => record.tokens_used,
            ServiceType::STT | ServiceType::TTS => record.audio_seconds as i64,
            ServiceType::ImageGen | ServiceType::Vision => 1,
        };

        // Update daily usage
        {
            let mut daily = self.daily_usage.write().unwrap();
            let key = (record.user_id, today, record.service_type);
            *daily.entry(key).or_insert(0) += units;
        }

        // Update monthly usage
        {
            let mut monthly = self.monthly_usage.write().unwrap();
            let key = (record.user_id, month, record.service_type);
            *monthly.entry(key).or_insert(0) += units;
        }

        // Store record
        {
            let mut records = self.records.write().unwrap();
            records.push(record);
        }

        Ok(())
    }

    async fn get_usage_summary(
        &self,
        user_id: Uuid,
    ) -> Result<HashMap<ServiceType, QuotaStatus>, QuotaError> {
        let today = Self::today_key();
        let daily_usage = self.daily_usage.read().unwrap();
        let config = QuotaConfig::default();

        let mut summary = HashMap::new();

        for service_type in [
            ServiceType::LLM,
            ServiceType::Embedding,
            ServiceType::STT,
            ServiceType::TTS,
            ServiceType::ImageGen,
            ServiceType::Vision,
        ] {
            let key = (user_id, today.clone(), service_type);
            let used = *daily_usage.get(&key).unwrap_or(&0);
            let limit = Self::get_limit_for_service(service_type, &config);
            summary.insert(service_type, QuotaStatus::new(used, limit, Self::next_day_reset()));
        }

        Ok(summary)
    }

    async fn get_group_usage(
        &self,
        group_id: Uuid,
    ) -> Result<HashMap<ServiceType, QuotaStatus>, QuotaError> {
        let records = self.records.read().unwrap();
        let today = Self::today_key();
        let config = QuotaConfig::default();

        let mut usage_by_service: HashMap<ServiceType, i64> = HashMap::new();

        for record in records.iter() {
            if record.group_id == Some(group_id)
                && record.timestamp.format("%Y-%m-%d").to_string() == today
            {
                let units = match record.service_type {
                    ServiceType::LLM | ServiceType::Embedding => record.tokens_used,
                    ServiceType::STT | ServiceType::TTS => record.audio_seconds as i64,
                    ServiceType::ImageGen | ServiceType::Vision => 1,
                };
                *usage_by_service.entry(record.service_type).or_insert(0) += units;
            }
        }

        let mut summary = HashMap::new();
        for service_type in [
            ServiceType::LLM,
            ServiceType::Embedding,
            ServiceType::STT,
            ServiceType::TTS,
            ServiceType::ImageGen,
            ServiceType::Vision,
        ] {
            let used = *usage_by_service.get(&service_type).unwrap_or(&0);
            let limit = Self::get_limit_for_service(service_type, &config);
            summary.insert(service_type, QuotaStatus::new(used, limit, Self::next_day_reset()));
        }

        Ok(summary)
    }

    async fn reset_daily_quotas(&self) -> Result<(), QuotaError> {
        let mut daily = self.daily_usage.write().unwrap();
        daily.clear();
        Ok(())
    }

    async fn get_cost_estimate(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<f64, QuotaError> {
        let records = self.records.read().unwrap();

        let total_cost: f64 = records
            .iter()
            .filter(|r| r.user_id == user_id && r.timestamp >= start && r.timestamp <= end)
            .map(|r| r.cost_estimate)
            .sum();

        Ok(total_cost)
    }
}

/// Helper to check quota before operation
pub async fn check_and_record_usage<Q: QuotaManager>(
    quota_manager: &Q,
    user_id: Uuid,
    group_id: Option<Uuid>,
    service_type: ServiceType,
    provider: &str,
    tokens: i64,
    audio_seconds: f64,
    config: &QuotaConfig,
) -> Result<(), QuotaError> {
    // Check quota first
    quota_manager
        .check_quota(user_id, service_type, config)
        .await?;

    // Calculate cost estimate
    let cost = match service_type {
        ServiceType::LLM | ServiceType::Embedding => tokens as f64 * service_type.cost_per_unit(),
        ServiceType::STT | ServiceType::TTS => audio_seconds * service_type.cost_per_unit(),
        ServiceType::ImageGen | ServiceType::Vision => service_type.cost_per_unit(),
    };

    // Record usage
    let record = UsageRecord {
        user_id,
        group_id,
        service_type,
        provider: provider.to_string(),
        tokens_used: tokens,
        audio_seconds,
        cost_estimate: cost,
        timestamp: Utc::now(),
    };

    quota_manager.record_usage(record).await?;

    Ok(())
}

/// Mock quota manager for testing
pub struct MockQuotaManager {
    pub should_exceed: bool,
    pub usage: RwLock<HashMap<(Uuid, ServiceType), i64>>,
}

impl MockQuotaManager {
    pub fn new() -> Self {
        Self {
            should_exceed: false,
            usage: RwLock::new(HashMap::new()),
        }
    }

    pub fn exceeding() -> Self {
        Self {
            should_exceed: true,
            usage: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MockQuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QuotaManager for MockQuotaManager {
    async fn check_quota(
        &self,
        user_id: Uuid,
        service_type: ServiceType,
        config: &QuotaConfig,
    ) -> Result<QuotaStatus, QuotaError> {
        if self.should_exceed {
            return Err(QuotaError::DailyQuotaExceeded {
                user_id,
                used: config.daily_token_limit,
                limit: config.daily_token_limit,
            });
        }

        let usage = self.usage.read().unwrap();
        let used = *usage.get(&(user_id, service_type)).unwrap_or(&0);
        let limit = config.daily_token_limit;

        Ok(QuotaStatus::new(
            used,
            limit,
            InMemoryQuotaManager::next_day_reset(),
        ))
    }

    async fn record_usage(&self, record: UsageRecord) -> Result<(), QuotaError> {
        let mut usage = self.usage.write().unwrap();
        let key = (record.user_id, record.service_type);
        *usage.entry(key).or_insert(0) += record.tokens_used;
        Ok(())
    }

    async fn get_usage_summary(
        &self,
        _user_id: Uuid,
    ) -> Result<HashMap<ServiceType, QuotaStatus>, QuotaError> {
        Ok(HashMap::new())
    }

    async fn get_group_usage(
        &self,
        _group_id: Uuid,
    ) -> Result<HashMap<ServiceType, QuotaStatus>, QuotaError> {
        Ok(HashMap::new())
    }

    async fn reset_daily_quotas(&self) -> Result<(), QuotaError> {
        let mut usage = self.usage.write().unwrap();
        usage.clear();
        Ok(())
    }

    async fn get_cost_estimate(
        &self,
        _user_id: Uuid,
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> Result<f64, QuotaError> {
        Ok(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_quota_config_default() {
        let config = QuotaConfig::default();
        assert_eq!(config.daily_token_limit, 100_000);
        assert_eq!(config.monthly_token_limit, 3_000_000);
        assert!(config.enforce_hard_limits);
    }

    #[test]
    fn test_quota_status_calculation() {
        let status = QuotaStatus::new(50_000, 100_000, Utc::now());
        assert_eq!(status.remaining, 50_000);
        assert_eq!(status.percentage_used, 50.0);
        assert!(!status.is_exceeded());
        assert!(!status.is_warning());

        let status = QuotaStatus::new(85_000, 100_000, Utc::now());
        assert!(status.is_warning());
        assert!(!status.is_exceeded());

        let status = QuotaStatus::new(100_000, 100_000, Utc::now());
        assert!(status.is_exceeded());
    }

    #[tokio::test]
    async fn test_check_quota_within_limit() {
        let manager = InMemoryQuotaManager::new();
        let user_id = Uuid::new_v4();
        let config = QuotaConfig::default();

        let status = manager
            .check_quota(user_id, ServiceType::LLM, &config)
            .await
            .unwrap();

        assert_eq!(status.used, 0);
        assert_eq!(status.limit, config.daily_token_limit);
        assert!(!status.is_exceeded());
    }

    #[tokio::test]
    async fn test_record_and_check_usage() {
        let manager = InMemoryQuotaManager::new();
        let user_id = Uuid::new_v4();
        let config = QuotaConfig::default();

        // Record some usage
        let record = UsageRecord {
            user_id,
            group_id: None,
            service_type: ServiceType::LLM,
            provider: "test".to_string(),
            tokens_used: 1000,
            audio_seconds: 0.0,
            cost_estimate: 0.002,
            timestamp: Utc::now(),
        };

        manager.record_usage(record).await.unwrap();

        // Check quota
        let status = manager
            .check_quota(user_id, ServiceType::LLM, &config)
            .await
            .unwrap();

        assert_eq!(status.used, 1000);
    }

    #[tokio::test]
    async fn test_quota_exceeded() {
        let manager = InMemoryQuotaManager::new();
        let user_id = Uuid::new_v4();
        let mut config = QuotaConfig::default();
        config.daily_token_limit = 100;
        config.enforce_hard_limits = true;

        // Record usage that exceeds limit
        let record = UsageRecord {
            user_id,
            group_id: None,
            service_type: ServiceType::LLM,
            provider: "test".to_string(),
            tokens_used: 150,
            audio_seconds: 0.0,
            cost_estimate: 0.0003,
            timestamp: Utc::now(),
        };

        manager.record_usage(record).await.unwrap();

        // Check quota should fail
        let result = manager
            .check_quota(user_id, ServiceType::LLM, &config)
            .await;

        assert!(matches!(result, Err(QuotaError::DailyQuotaExceeded { .. })));
    }

    #[tokio::test]
    async fn test_usage_summary() {
        let manager = InMemoryQuotaManager::new();
        let user_id = Uuid::new_v4();

        // Record usage for different services
        for service_type in [ServiceType::LLM, ServiceType::STT] {
            let record = UsageRecord {
                user_id,
                group_id: None,
                service_type,
                provider: "test".to_string(),
                tokens_used: 500,
                audio_seconds: 30.0,
                cost_estimate: 0.001,
                timestamp: Utc::now(),
            };
            manager.record_usage(record).await.unwrap();
        }

        let summary = manager.get_usage_summary(user_id).await.unwrap();

        assert!(summary.contains_key(&ServiceType::LLM));
        assert!(summary.contains_key(&ServiceType::STT));
    }

    #[tokio::test]
    async fn test_cost_estimate() {
        let manager = InMemoryQuotaManager::new();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Record usage with cost
        let record = UsageRecord {
            user_id,
            group_id: None,
            service_type: ServiceType::LLM,
            provider: "test".to_string(),
            tokens_used: 1000,
            audio_seconds: 0.0,
            cost_estimate: 0.002,
            timestamp: now,
        };

        manager.record_usage(record).await.unwrap();

        let cost = manager
            .get_cost_estimate(user_id, now - Duration::hours(1), now + Duration::hours(1))
            .await
            .unwrap();

        assert!((cost - 0.002).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_reset_daily_quotas() {
        let manager = InMemoryQuotaManager::new();
        let user_id = Uuid::new_v4();

        // Record usage
        let record = UsageRecord {
            user_id,
            group_id: None,
            service_type: ServiceType::LLM,
            provider: "test".to_string(),
            tokens_used: 1000,
            audio_seconds: 0.0,
            cost_estimate: 0.002,
            timestamp: Utc::now(),
        };

        manager.record_usage(record).await.unwrap();

        // Reset
        manager.reset_daily_quotas().await.unwrap();

        // Check quota should be 0
        let config = QuotaConfig::default();
        let status = manager
            .check_quota(user_id, ServiceType::LLM, &config)
            .await
            .unwrap();

        assert_eq!(status.used, 0);
    }

    /// **Feature: phase3-ai-agent-evolution, Property 17: Quota Management**
    /// **Validates: Requirements 12.1, 12.2, 12.3**
    #[test]
    fn test_quota_management_properties() {
        use proptest::prelude::*;

        proptest!(|(
            used in 0i64..1_000_000,
            limit in 1i64..1_000_000
        )| {
            let status = QuotaStatus::new(used, limit, Utc::now());

            // Property 1: Remaining is always non-negative
            prop_assert!(status.remaining >= 0);

            // Property 2: Percentage is always 0-100
            prop_assert!(status.percentage_used >= 0.0);
            prop_assert!(status.percentage_used <= 100.0);

            // Property 3: is_exceeded is consistent with used >= limit
            prop_assert_eq!(status.is_exceeded(), used >= limit);

            // Property 4: remaining + used <= limit (when not exceeded)
            if !status.is_exceeded() {
                prop_assert_eq!(status.remaining + status.used, limit);
            }
        });
    }

    #[test]
    fn test_service_type_cost() {
        // All service types should have positive cost
        for service_type in [
            ServiceType::LLM,
            ServiceType::Embedding,
            ServiceType::STT,
            ServiceType::TTS,
            ServiceType::ImageGen,
            ServiceType::Vision,
        ] {
            assert!(service_type.cost_per_unit() > 0.0);
        }
    }
}
