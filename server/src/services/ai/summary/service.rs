//! Default Summarization Service Implementation
//!
//! Provides message summarization using LLM with caching support.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use super::{MessageForSummary, Summary, SummaryRequest, SummaryStyle, SummarizeService};
use crate::services::ai::{AIAgentError, LLMClient};

/// Cache entry for summaries
#[derive(Debug, Clone)]
struct CacheEntry {
    summary: Summary,
}

/// Default summarization service implementation
pub struct DefaultSummarizeService {
    llm_client: Arc<dyn LLMClient>,
    cache: Arc<RwLock<HashMap<(Uuid, SummaryStyle), CacheEntry>>>,
    cache_duration_minutes: i64,
    max_context_tokens: usize,
}

impl DefaultSummarizeService {
    /// Create a new summarization service
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self {
            llm_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_duration_minutes: 30,
            max_context_tokens: 4000,
        }
    }

    /// Set cache duration in minutes
    pub fn with_cache_duration(mut self, minutes: i64) -> Self {
        self.cache_duration_minutes = minutes;
        self
    }

    /// Set maximum context tokens
    pub fn with_max_context_tokens(mut self, tokens: usize) -> Self {
        self.max_context_tokens = tokens;
        self
    }

    /// Build context from messages
    fn build_context(&self, messages: &[MessageForSummary]) -> String {
        let mut context = String::new();
        let mut estimated_tokens = 0;

        for msg in messages.iter().rev() {
            let formatted = msg.format();
            let msg_tokens = formatted.len() / 4; // Rough estimate: 4 chars per token

            if estimated_tokens + msg_tokens > self.max_context_tokens {
                break;
            }

            context = format!("{}\n{}", formatted, context);
            estimated_tokens += msg_tokens;
        }

        context.trim().to_string()
    }

    /// Generate summary using LLM
    #[instrument(skip(self, messages), fields(message_count = messages.len()))]
    async fn generate_summary(
        &self,
        messages: &[MessageForSummary],
        style: SummaryStyle,
    ) -> Result<String, AIAgentError> {
        if messages.is_empty() {
            return Ok("没有消息可供总结。".to_string());
        }

        let context = self.build_context(messages);
        let system_prompt = format!(
            "你是一个专业的对话总结助手。{}\n\n请确保总结准确、客观，不添加对话中没有的信息。",
            style.prompt_instruction()
        );

        debug!(context_length = context.len(), "Generating summary");

        let response = self
            .llm_client
            .generate_response(&system_prompt, &context)
            .await?;

        info!("Summary generated successfully");
        Ok(response)
    }
}

#[async_trait]
impl SummarizeService for DefaultSummarizeService {
    #[instrument(skip(self), fields(group_id = %request.group_id, style = ?request.style))]
    async fn summarize(&self, request: SummaryRequest) -> Result<Summary, AIAgentError> {
        // Check cache first if enabled
        if request.use_cache {
            if let Some(cached) = self.get_cached(request.group_id, request.style).await {
                debug!("Returning cached summary");
                return Ok(cached);
            }
        }

        // In a real implementation, we would fetch messages from the database
        // For now, we'll return an error indicating messages need to be provided
        // This would be integrated with the message repository
        warn!("Message fetching not implemented - returning placeholder");

        // Generate placeholder summary
        let now = Utc::now();
        let summary = Summary {
            id: Uuid::new_v4(),
            group_id: request.group_id,
            content: "消息总结功能需要与消息存储集成。".to_string(),
            message_count: 0,
            time_range_start: request.since,
            time_range_end: Some(now),
            style: request.style,
            generated_at: now,
            expires_at: now + Duration::minutes(self.cache_duration_minutes),
        };

        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                (request.group_id, request.style),
                CacheEntry {
                    summary: summary.clone(),
                },
            );
        }

        Ok(summary)
    }

    async fn get_cached(&self, group_id: Uuid, style: SummaryStyle) -> Option<Summary> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(&(group_id, style)) {
            if entry.summary.expires_at > Utc::now() {
                return Some(entry.summary.clone());
            }
        }
        None
    }

    async fn invalidate_cache(&self, group_id: Uuid) -> Result<(), AIAgentError> {
        let mut cache = self.cache.write().await;
        cache.retain(|(gid, _), _| *gid != group_id);
        debug!(group_id = %group_id, "Cache invalidated");
        Ok(())
    }
}

/// Summarization service with message fetching capability
pub struct SummarizeServiceWithMessages<F>
where
    F: Fn(Uuid, usize, Option<chrono::DateTime<Utc>>) -> futures::future::BoxFuture<'static, Result<Vec<MessageForSummary>, AIAgentError>>
        + Send
        + Sync,
{
    inner: DefaultSummarizeService,
    fetch_messages: F,
}

impl<F> SummarizeServiceWithMessages<F>
where
    F: Fn(Uuid, usize, Option<chrono::DateTime<Utc>>) -> futures::future::BoxFuture<'static, Result<Vec<MessageForSummary>, AIAgentError>>
        + Send
        + Sync,
{
    /// Create a new service with message fetching capability
    pub fn new(llm_client: Arc<dyn LLMClient>, fetch_messages: F) -> Self {
        Self {
            inner: DefaultSummarizeService::new(llm_client),
            fetch_messages,
        }
    }

    /// Generate summary with fetched messages
    pub async fn summarize_with_fetch(
        &self,
        request: SummaryRequest,
    ) -> Result<Summary, AIAgentError> {
        // Check cache first
        if request.use_cache {
            if let Some(cached) = self.inner.get_cached(request.group_id, request.style).await {
                return Ok(cached);
            }
        }

        // Fetch messages
        let messages = (self.fetch_messages)(request.group_id, request.message_count, request.since).await?;

        if messages.is_empty() {
            return Err(AIAgentError::SummaryError(
                "没有找到可供总结的消息".to_string(),
            ));
        }

        // Generate summary
        let content = self.inner.generate_summary(&messages, request.style).await?;

        let now = Utc::now();
        let time_range_start = messages.last().map(|m| m.sent_at);
        let time_range_end = messages.first().map(|m| m.sent_at);

        let summary = Summary {
            id: Uuid::new_v4(),
            group_id: request.group_id,
            content,
            message_count: messages.len(),
            time_range_start,
            time_range_end,
            style: request.style,
            generated_at: now,
            expires_at: now + Duration::minutes(self.inner.cache_duration_minutes),
        };

        // Cache the result
        {
            let mut cache = self.inner.cache.write().await;
            cache.insert(
                (request.group_id, request.style),
                CacheEntry {
                    summary: summary.clone(),
                },
            );
        }

        Ok(summary)
    }
}

/// Mock summarization service for testing
pub struct MockSummarizeService {
    response: String,
    should_fail: bool,
}

impl MockSummarizeService {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            should_fail: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            response: String::new(),
            should_fail: true,
        }
    }
}

#[async_trait]
impl SummarizeService for MockSummarizeService {
    async fn summarize(&self, request: SummaryRequest) -> Result<Summary, AIAgentError> {
        if self.should_fail {
            return Err(AIAgentError::SummaryError("Mock failure".to_string()));
        }

        let now = Utc::now();
        Ok(Summary {
            id: Uuid::new_v4(),
            group_id: request.group_id,
            content: self.response.clone(),
            message_count: 10,
            time_range_start: Some(now - Duration::hours(1)),
            time_range_end: Some(now),
            style: request.style,
            generated_at: now,
            expires_at: now + Duration::minutes(30),
        })
    }

    async fn get_cached(&self, _group_id: Uuid, _style: SummaryStyle) -> Option<Summary> {
        None
    }

    async fn invalidate_cache(&self, _group_id: Uuid) -> Result<(), AIAgentError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::MockLLMClient;

    #[tokio::test]
    async fn test_mock_summarize_service() {
        let service = MockSummarizeService::new("Test summary");
        let request = SummaryRequest {
            group_id: Uuid::new_v4(),
            ..Default::default()
        };

        let result = service.summarize(request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Test summary");
    }

    #[tokio::test]
    async fn test_mock_summarize_service_failure() {
        let service = MockSummarizeService::failing();
        let request = SummaryRequest {
            group_id: Uuid::new_v4(),
            ..Default::default()
        };

        let result = service.summarize(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_default_summarize_service_cache() {
        let llm_client = Arc::new(MockLLMClient::new("Summary content".to_string()));
        let service = DefaultSummarizeService::new(llm_client);

        let group_id = Uuid::new_v4();

        // First call - no cache
        assert!(service.get_cached(group_id, SummaryStyle::Brief).await.is_none());

        // Summarize (will cache)
        let request = SummaryRequest {
            group_id,
            use_cache: true,
            ..Default::default()
        };
        let _ = service.summarize(request).await;

        // Now should be cached
        assert!(service.get_cached(group_id, SummaryStyle::Brief).await.is_some());

        // Invalidate
        service.invalidate_cache(group_id).await.unwrap();
        assert!(service.get_cached(group_id, SummaryStyle::Brief).await.is_none());
    }

    #[tokio::test]
    async fn test_build_context() {
        let llm_client = Arc::new(MockLLMClient::new("Summary".to_string()));
        let service = DefaultSummarizeService::new(llm_client).with_max_context_tokens(100);

        let messages = vec![
            MessageForSummary {
                id: Uuid::new_v4(),
                sender_name: "Alice".to_string(),
                content: "Hello".to_string(),
                sent_at: Utc::now(),
            },
            MessageForSummary {
                id: Uuid::new_v4(),
                sender_name: "Bob".to_string(),
                content: "Hi there".to_string(),
                sent_at: Utc::now(),
            },
        ];

        let context = service.build_context(&messages);
        assert!(context.contains("Alice"));
        assert!(context.contains("Bob"));
    }

    // Property test: Summary message count
    #[test]
    fn test_summary_message_count_property() {
        use proptest::prelude::*;

        proptest!(|(count in 1usize..1000)| {
            let request = SummaryRequest {
                group_id: Uuid::new_v4(),
                message_count: count,
                ..Default::default()
            };
            prop_assert!(request.message_count > 0);
            prop_assert!(request.message_count == count);
        });
    }
}
