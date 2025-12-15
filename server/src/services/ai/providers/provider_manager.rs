//! Provider Manager Implementation
//!
//! Manages multiple LLM providers with automatic fallback and domestic priority.
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 16.1, 16.2, 16.3**

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::services::ai::{AIAgentError, LLMClient, LLMProvider};

// ============================================================================
// Provider Configuration
// ============================================================================

/// Provider configuration with priority and health settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider identifier
    pub provider: LLMProvider,
    /// API key for the provider
    pub api_key: String,
    /// Optional API base URL override
    pub api_base: Option<String>,
    /// Model to use
    pub model: String,
    /// Priority (lower = higher priority, 1 = highest)
    pub priority: u8,
    /// Whether this provider is enabled
    pub enabled: bool,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Maximum retries for this provider
    pub max_retries: Option<u32>,
}

impl ProviderConfig {
    /// Create a new provider config
    pub fn new(provider: LLMProvider, api_key: String, model: String) -> Self {
        Self {
            provider,
            api_key,
            api_base: None,
            model,
            priority: 10, // Default priority
            enabled: true,
            timeout_secs: Some(30),
            max_retries: Some(2),
        }
    }

    /// Set priority (lower = higher priority)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Set API base URL
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = Some(api_base);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Enable or disable the provider
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Default priorities for providers (domestic first)
/// 国内服务商优先级配置
pub mod priorities {
    pub const SILICONFLOW: u8 = 1;  // 硅基流动 - 最高优先级
    pub const ZHIPU: u8 = 2;        // 智谱 AI
    pub const MOONSHOT: u8 = 3;     // 月之暗面
    pub const TONGYI: u8 = 4;       // 通义千问
    pub const WENXIN: u8 = 5;       // 文心一言
    pub const OPENAI: u8 = 10;      // OpenAI - 国际备选
    pub const ANTHROPIC: u8 = 11;   // Anthropic - 国际备选
    pub const GOOGLE: u8 = 12;      // Google - 国际备选
}

// ============================================================================
// Provider Health Tracking
// ============================================================================

/// Health status for a provider
#[derive(Debug)]
pub struct ProviderHealth {
    /// Number of successful requests
    pub success_count: AtomicU64,
    /// Number of failed requests
    pub failure_count: AtomicU64,
    /// Last successful request time
    pub last_success: RwLock<Option<Instant>>,
    /// Last failure time
    pub last_failure: RwLock<Option<Instant>>,
    /// Whether the provider is currently healthy
    pub is_healthy: RwLock<bool>,
}

impl Default for ProviderHealth {
    fn default() -> Self {
        Self {
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
            last_success: RwLock::new(None),
            last_failure: RwLock::new(None),
            is_healthy: RwLock::new(true),
        }
    }
}

impl ProviderHealth {
    /// Record a successful request
    pub async fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        *self.last_success.write().await = Some(Instant::now());
        *self.is_healthy.write().await = true;
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_failure.write().await = Some(Instant::now());
        
        // Mark as unhealthy if too many recent failures
        let failures = self.failure_count.load(Ordering::Relaxed);
        let successes = self.success_count.load(Ordering::Relaxed);
        
        if failures > 3 && failures > successes / 2 {
            *self.is_healthy.write().await = false;
        }
    }

    /// Check if provider should be tried (healthy or enough time passed)
    pub async fn should_try(&self, cooldown_secs: u64) -> bool {
        let is_healthy = *self.is_healthy.read().await;
        if is_healthy {
            return true;
        }

        // Check if cooldown period has passed
        if let Some(last_failure) = *self.last_failure.read().await {
            last_failure.elapsed() > Duration::from_secs(cooldown_secs)
        } else {
            true
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let successes = self.success_count.load(Ordering::Relaxed);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let total = successes + failures;
        
        if total == 0 {
            1.0 // Assume healthy if no data
        } else {
            successes as f64 / total as f64
        }
    }
}

// ============================================================================
// Provider Manager
// ============================================================================

/// Manages multiple LLM providers with fallback support
///
/// **Feature: phase3-ai-agent-evolution, Property 21: Provider Fallback Chain**
/// **Feature: phase3-ai-agent-evolution, Property 22: Domestic Provider Priority**
pub struct ProviderManager {
    /// Providers sorted by priority
    providers: Vec<(ProviderConfig, Arc<dyn LLMClient>, Arc<ProviderHealth>)>,
    /// Cooldown period for unhealthy providers (seconds)
    unhealthy_cooldown_secs: u64,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            unhealthy_cooldown_secs: 60,
        }
    }

    /// Add a provider with its client
    pub fn add_provider(
        mut self,
        config: ProviderConfig,
        client: Arc<dyn LLMClient>,
    ) -> Self {
        self.providers.push((config, client, Arc::new(ProviderHealth::default())));
        // Sort by priority (lower = higher priority)
        self.providers.sort_by_key(|(c, _, _)| c.priority);
        self
    }

    /// Set cooldown period for unhealthy providers
    pub fn with_cooldown(mut self, secs: u64) -> Self {
        self.unhealthy_cooldown_secs = secs;
        self
    }

    /// Get the number of providers
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    /// Get enabled provider count
    pub fn enabled_count(&self) -> usize {
        self.providers.iter().filter(|(c, _, _)| c.enabled).count()
    }

    /// Execute an operation with automatic fallback
    ///
    /// Tries providers in priority order, falling back on transient errors.
    ///
    /// **Property 21: Provider Fallback Chain**
    /// For any sequence of provider failures, the system tries the next provider
    /// in priority order until success or all providers exhausted.
    pub async fn execute_with_fallback<F, T>(&self, operation: F) -> Result<T, AIAgentError>
    where
        F: Fn(Arc<dyn LLMClient>) -> BoxFuture<'static, Result<T, AIAgentError>> + Send + Sync,
    {
        let mut last_error: Option<AIAgentError> = None;
        let mut tried_count = 0;

        for (config, client, health) in &self.providers {
            // Skip disabled providers
            if !config.enabled {
                debug!(provider = %config.provider, "Skipping disabled provider");
                continue;
            }

            // Skip unhealthy providers unless cooldown passed
            if !health.should_try(self.unhealthy_cooldown_secs).await {
                debug!(
                    provider = %config.provider,
                    "Skipping unhealthy provider (in cooldown)"
                );
                continue;
            }

            tried_count += 1;
            debug!(
                provider = %config.provider,
                priority = config.priority,
                "Trying provider"
            );

            // Execute with timeout
            let timeout = Duration::from_secs(config.timeout_secs.unwrap_or(30));
            let result = tokio::time::timeout(timeout, operation(client.clone())).await;

            match result {
                Ok(Ok(value)) => {
                    info!(
                        provider = %config.provider,
                        "Provider succeeded"
                    );
                    health.record_success().await;
                    return Ok(value);
                }
                Ok(Err(e)) => {
                    // Check if error is transient (should try next provider)
                    if e.is_transient() {
                        warn!(
                            provider = %config.provider,
                            error = %e,
                            "Provider failed with transient error, trying next"
                        );
                        health.record_failure().await;
                        last_error = Some(e);
                        continue;
                    } else {
                        // Non-transient error, don't try other providers
                        error!(
                            provider = %config.provider,
                            error = %e,
                            "Provider failed with non-transient error"
                        );
                        health.record_failure().await;
                        return Err(e);
                    }
                }
                Err(_) => {
                    // Timeout
                    warn!(
                        provider = %config.provider,
                        timeout_secs = config.timeout_secs.unwrap_or(30),
                        "Provider timed out, trying next"
                    );
                    health.record_failure().await;
                    last_error = Some(AIAgentError::LLMTimeout(
                        config.timeout_secs.unwrap_or(30),
                    ));
                    continue;
                }
            }
        }

        // All providers failed
        if tried_count == 0 {
            error!("No providers available (all disabled or unhealthy)");
            Err(AIAgentError::AllProvidersFailed)
        } else {
            error!(
                tried_count = tried_count,
                "All providers failed"
            );
            Err(last_error.unwrap_or(AIAgentError::AllProvidersFailed))
        }
    }

    /// Get health statistics for all providers
    pub fn get_health_stats(&self) -> Vec<ProviderHealthStats> {
        self.providers
            .iter()
            .map(|(config, _, health)| ProviderHealthStats {
                provider: config.provider,
                priority: config.priority,
                enabled: config.enabled,
                success_rate: health.success_rate(),
                success_count: health.success_count.load(Ordering::Relaxed),
                failure_count: health.failure_count.load(Ordering::Relaxed),
            })
            .collect()
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Health statistics for a provider
#[derive(Debug, Clone, Serialize)]
pub struct ProviderHealthStats {
    pub provider: LLMProvider,
    pub priority: u8,
    pub enabled: bool,
    pub success_rate: f64,
    pub success_count: u64,
    pub failure_count: u64,
}

// ============================================================================
// LLMClient Implementation for ProviderManager
// ============================================================================

#[async_trait]
impl LLMClient for ProviderManager {
    async fn generate_response(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, AIAgentError> {
        let system = system_prompt.to_string();
        let user = user_message.to_string();
        
        self.execute_with_fallback(move |client| {
            let s = system.clone();
            let u = user.clone();
            Box::pin(async move { client.generate_response(&s, &u).await })
        })
        .await
    }

    async fn generate_with_history(
        &self,
        messages: &[crate::services::ai::llm_client::ChatMessage],
    ) -> Result<String, AIAgentError> {
        let msgs = messages.to_vec();
        
        self.execute_with_fallback(move |client| {
            let m = msgs.clone();
            Box::pin(async move { client.generate_with_history(&m).await })
        })
        .await
    }

    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<crate::services::ai::StreamChunk>, AIAgentError> {
        // For streaming, just use the first available provider
        // (fallback during streaming is complex)
        for (config, client, health) in &self.providers {
            if config.enabled && health.should_try(self.unhealthy_cooldown_secs).await {
                return client.generate_stream(system_prompt, user_message).await;
            }
        }
        Err(AIAgentError::AllProvidersFailed)
    }

    fn provider(&self) -> LLMProvider {
        // Return the highest priority enabled provider
        self.providers
            .iter()
            .find(|(c, _, _)| c.enabled)
            .map(|(c, _, _)| c.provider)
            .unwrap_or(LLMProvider::Custom)
    }

    fn model(&self) -> &str {
        // Return the model of the highest priority enabled provider
        self.providers
            .iter()
            .find(|(c, _, _)| c.enabled)
            .map(|(c, _, _)| c.model.as_str())
            .unwrap_or("unknown")
    }

    fn config(&self) -> &crate::services::ai::LLMConfig {
        // This is a bit awkward - return a static config
        // In practice, callers should use get_health_stats() instead
        static DEFAULT_CONFIG: std::sync::OnceLock<crate::services::ai::LLMConfig> = 
            std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(|| {
            crate::services::ai::LLMConfig::custom(
                "provider-manager".to_string(),
                "multi-provider".to_string(),
                None,
            )
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::MockLLMClient;

    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new(
            LLMProvider::SiliconFlow,
            "test-key".to_string(),
            "test-model".to_string(),
        )
        .with_priority(priorities::SILICONFLOW)
        .with_timeout(60);

        assert_eq!(config.priority, 1);
        assert_eq!(config.timeout_secs, Some(60));
        assert!(config.enabled);
    }

    #[test]
    fn test_provider_manager_sorting() {
        let manager = ProviderManager::new()
            .add_provider(
                ProviderConfig::new(LLMProvider::OpenAI, "key".to_string(), "model".to_string())
                    .with_priority(priorities::OPENAI),
                Arc::new(MockLLMClient::new("openai".to_string())),
            )
            .add_provider(
                ProviderConfig::new(LLMProvider::SiliconFlow, "key".to_string(), "model".to_string())
                    .with_priority(priorities::SILICONFLOW),
                Arc::new(MockLLMClient::new("siliconflow".to_string())),
            );

        // SiliconFlow should be first (priority 1)
        assert_eq!(manager.providers[0].0.provider, LLMProvider::SiliconFlow);
        assert_eq!(manager.providers[1].0.provider, LLMProvider::OpenAI);
    }

    #[tokio::test]
    async fn test_execute_with_fallback_success() {
        let manager = ProviderManager::new()
            .add_provider(
                ProviderConfig::new(LLMProvider::SiliconFlow, "key".to_string(), "model".to_string())
                    .with_priority(1),
                Arc::new(MockLLMClient::new("success".to_string())),
            );

        let result = manager
            .generate_response("system", "user")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_execute_with_fallback_on_failure() {
        let manager = ProviderManager::new()
            .add_provider(
                ProviderConfig::new(LLMProvider::SiliconFlow, "key".to_string(), "model".to_string())
                    .with_priority(1),
                Arc::new(MockLLMClient::failing()),
            )
            .add_provider(
                ProviderConfig::new(LLMProvider::OpenAI, "key".to_string(), "model".to_string())
                    .with_priority(10),
                Arc::new(MockLLMClient::new("fallback".to_string())),
            );

        let result = manager
            .generate_response("system", "user")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback");
    }

    #[tokio::test]
    async fn test_disabled_provider_skipped() {
        let manager = ProviderManager::new()
            .add_provider(
                ProviderConfig::new(LLMProvider::SiliconFlow, "key".to_string(), "model".to_string())
                    .with_priority(1)
                    .with_enabled(false),
                Arc::new(MockLLMClient::new("disabled".to_string())),
            )
            .add_provider(
                ProviderConfig::new(LLMProvider::OpenAI, "key".to_string(), "model".to_string())
                    .with_priority(10),
                Arc::new(MockLLMClient::new("enabled".to_string())),
            );

        let result = manager
            .generate_response("system", "user")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "enabled");
    }

    #[tokio::test]
    async fn test_all_providers_fail() {
        let manager = ProviderManager::new()
            .add_provider(
                ProviderConfig::new(LLMProvider::SiliconFlow, "key".to_string(), "model".to_string())
                    .with_priority(1),
                Arc::new(MockLLMClient::failing()),
            )
            .add_provider(
                ProviderConfig::new(LLMProvider::OpenAI, "key".to_string(), "model".to_string())
                    .with_priority(10),
                Arc::new(MockLLMClient::failing()),
            );

        let result = manager
            .generate_response("system", "user")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_tracking() {
        let health = ProviderHealth::default();
        
        // Initially healthy
        assert!(health.should_try(60).await);
        assert_eq!(health.success_rate(), 1.0);

        // Record some successes
        health.record_success().await;
        health.record_success().await;
        assert_eq!(health.success_count.load(Ordering::Relaxed), 2);

        // Record a failure
        health.record_failure().await;
        assert_eq!(health.failure_count.load(Ordering::Relaxed), 1);
        
        // Success rate should be 2/3
        let rate = health.success_rate();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_domestic_priority_order() {
        // Verify domestic providers have lower priority numbers (higher priority)
        assert!(priorities::SILICONFLOW < priorities::OPENAI);
        assert!(priorities::ZHIPU < priorities::OPENAI);
        assert!(priorities::MOONSHOT < priorities::OPENAI);
        assert!(priorities::TONGYI < priorities::OPENAI);
        assert!(priorities::WENXIN < priorities::OPENAI);
    }
}
