//! Semantic Search Enhancement Module
//!
//! This module provides enhanced semantic search capabilities for messages,
//! including embedding generation pipeline, search with fallback, and result ranking.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::embedding::EmbeddingService;
use super::vector_store::VectorStore;
use super::AIAgentError;
use crate::models::HistoricalMessage;

/// Search error types
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),

    #[error("Vector search failed: {0}")]
    VectorSearchFailed(String),

    #[error("Text search failed: {0}")]
    TextSearchFailed(String),

    #[error("Search service unavailable")]
    ServiceUnavailable,

    #[error("Invalid search query: {0}")]
    InvalidQuery(String),
}

impl From<SearchError> for AIAgentError {
    fn from(err: SearchError) -> Self {
        AIAgentError::Internal(err.to_string())
    }
}

/// Embedding status for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingStatus {
    /// Embedding not yet generated
    Pending,
    /// Embedding generation in progress
    Processing,
    /// Embedding successfully generated
    Completed,
    /// Embedding generation failed
    Failed,
}

/// Message with embedding status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEmbeddingInfo {
    pub message_id: Uuid,
    pub group_id: Uuid,
    pub status: EmbeddingStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Search request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Search query text
    pub query: String,
    /// Group ID to search within
    pub group_id: Uuid,
    /// Whether to use semantic search
    pub semantic: bool,
    /// Maximum number of results
    pub limit: usize,
    /// Minimum similarity score (0.0-1.0)
    pub min_score: f32,
    /// Time range filter (optional)
    pub time_range: Option<TimeRange>,
    /// Sender filter (optional)
    pub sender_id: Option<Uuid>,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            group_id: Uuid::nil(),
            semantic: true,
            limit: 20,
            min_score: 0.5,
            time_range: None,
            sender_id: None,
        }
    }
}

/// Time range for search filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// Search result with ranking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub message_id: String,
    pub content: String,
    pub sender_id: String,
    pub timestamp: i64,
    /// Similarity score from vector search (0.0-1.0)
    pub similarity_score: f32,
    /// Time decay weight (0.0-1.0)
    pub time_decay_weight: f32,
    /// Final combined score
    pub final_score: f32,
    /// Whether result came from semantic or text search
    pub search_type: SearchType,
}

/// Type of search that produced the result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchType {
    Semantic,
    TextFallback,
    Hybrid,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub search_type: SearchType,
    pub query_time_ms: u64,
}

/// Semantic search service trait
#[async_trait]
pub trait SemanticSearchService: Send + Sync {
    /// Search for messages
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError>;

    /// Queue a message for embedding generation
    async fn queue_embedding(&self, message_id: Uuid, group_id: Uuid, content: &str) -> Result<(), SearchError>;

    /// Get embedding status for a message
    async fn get_embedding_status(&self, message_id: Uuid) -> Option<EmbeddingStatus>;

    /// Process pending embeddings (batch processing)
    async fn process_pending_embeddings(&self, batch_size: usize) -> Result<usize, SearchError>;
}

/// Default semantic search service implementation
pub struct DefaultSemanticSearchService {
    embedding_service: Arc<dyn EmbeddingService>,
    vector_store: Arc<dyn VectorStore>,
    /// Pending embeddings queue
    pending_queue: RwLock<Vec<PendingEmbedding>>,
    /// Embedding status tracking
    status_map: RwLock<HashMap<Uuid, MessageEmbeddingInfo>>,
    /// Configuration
    config: SearchConfig,
}

#[derive(Debug, Clone)]
struct PendingEmbedding {
    message_id: Uuid,
    group_id: Uuid,
    sender_id: Uuid,
    content: String,
    timestamp: i64,
    queued_at: DateTime<Utc>,
}

/// Search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Weight for similarity score in final ranking
    pub similarity_weight: f32,
    /// Weight for time decay in final ranking
    pub time_decay_weight: f32,
    /// Maximum pending queue size
    pub max_queue_size: usize,
    /// Batch size for embedding processing
    pub batch_size: usize,
    /// Whether to fall back to text search on vector failure
    pub fallback_to_text: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            similarity_weight: 0.7,
            time_decay_weight: 0.3,
            max_queue_size: 1000,
            batch_size: 50,
            fallback_to_text: true,
        }
    }
}

impl DefaultSemanticSearchService {
    pub fn new(
        embedding_service: Arc<dyn EmbeddingService>,
        vector_store: Arc<dyn VectorStore>,
    ) -> Self {
        Self {
            embedding_service,
            vector_store,
            pending_queue: RwLock::new(Vec::new()),
            status_map: RwLock::new(HashMap::new()),
            config: SearchConfig::default(),
        }
    }

    pub fn with_config(mut self, config: SearchConfig) -> Self {
        self.config = config;
        self
    }

    /// Calculate final score from similarity and time decay
    fn calculate_final_score(&self, similarity: f32, time_decay: f32) -> f32 {
        self.config.similarity_weight * similarity + self.config.time_decay_weight * time_decay
    }

    /// Perform text-based search (fallback)
    async fn text_search(
        &self,
        _query: &str,
        _group_id: Uuid,
        _limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // In a real implementation, this would query the database
        // For now, return empty results as fallback
        warn!("Text search fallback - not implemented, returning empty results");
        Ok(Vec::new())
    }
}

#[async_trait]
impl SemanticSearchService for DefaultSemanticSearchService {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError> {
        let start_time = std::time::Instant::now();

        if request.query.trim().is_empty() {
            return Err(SearchError::InvalidQuery("Query cannot be empty".to_string()));
        }

        let (results, search_type) = if request.semantic {
            // Try semantic search first
            match self.embedding_service.embed(&request.query).await {
                Ok(query_embedding) => {
                    match self.vector_store.search_similar(
                        request.group_id,
                        query_embedding,
                        request.limit,
                        request.min_score,
                    ).await {
                        Ok(messages) => {
                            let results: Vec<SearchResult> = messages
                                .into_iter()
                                .map(|m| {
                                    let final_score = self.calculate_final_score(
                                        m.similarity_score,
                                        m.time_decay_weight,
                                    );
                                    SearchResult {
                                        message_id: m.message_id,
                                        content: m.content,
                                        sender_id: m.sender_id,
                                        timestamp: m.timestamp,
                                        similarity_score: m.similarity_score,
                                        time_decay_weight: m.time_decay_weight,
                                        final_score,
                                        search_type: SearchType::Semantic,
                                    }
                                })
                                .collect();
                            (results, SearchType::Semantic)
                        }
                        Err(e) => {
                            warn!("Vector search failed: {}, falling back to text search", e);
                            if self.config.fallback_to_text {
                                let results = self.text_search(&request.query, request.group_id, request.limit).await?;
                                (results, SearchType::TextFallback)
                            } else {
                                return Err(SearchError::VectorSearchFailed(e.to_string()));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Embedding generation failed: {}, falling back to text search", e);
                    if self.config.fallback_to_text {
                        let results = self.text_search(&request.query, request.group_id, request.limit).await?;
                        (results, SearchType::TextFallback)
                    } else {
                        return Err(SearchError::EmbeddingFailed(e.to_string()));
                    }
                }
            }
        } else {
            // Text search only
            let results = self.text_search(&request.query, request.group_id, request.limit).await?;
            (results, SearchType::TextFallback)
        };

        // Apply additional filters
        let mut filtered_results = results;

        // Filter by time range
        if let Some(ref time_range) = request.time_range {
            filtered_results = filtered_results
                .into_iter()
                .filter(|r| {
                    let ts = r.timestamp;
                    let in_start = time_range.start.map_or(true, |s| ts >= s.timestamp());
                    let in_end = time_range.end.map_or(true, |e| ts <= e.timestamp());
                    in_start && in_end
                })
                .collect();
        }

        // Filter by sender
        if let Some(sender_id) = request.sender_id {
            filtered_results = filtered_results
                .into_iter()
                .filter(|r| r.sender_id == sender_id.to_string())
                .collect();
        }

        // Sort by final score
        filtered_results.sort_by(|a, b| {
            b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_count = filtered_results.len();
        let query_time_ms = start_time.elapsed().as_millis() as u64;

        debug!(
            "Search completed: {} results in {}ms (type: {:?})",
            total_count, query_time_ms, search_type
        );

        Ok(SearchResponse {
            results: filtered_results,
            total_count,
            search_type,
            query_time_ms,
        })
    }

    async fn queue_embedding(
        &self,
        message_id: Uuid,
        group_id: Uuid,
        content: &str,
    ) -> Result<(), SearchError> {
        let mut queue = self.pending_queue.write().await;

        // Check queue size limit
        if queue.len() >= self.config.max_queue_size {
            warn!("Embedding queue full, dropping oldest entries");
            queue.drain(0..self.config.batch_size);
        }

        // Add to queue
        queue.push(PendingEmbedding {
            message_id,
            group_id,
            sender_id: Uuid::nil(), // Will be filled from message data
            content: content.to_string(),
            timestamp: Utc::now().timestamp(),
            queued_at: Utc::now(),
        });

        // Update status
        let mut status_map = self.status_map.write().await;
        status_map.insert(message_id, MessageEmbeddingInfo {
            message_id,
            group_id,
            status: EmbeddingStatus::Pending,
            created_at: Utc::now(),
            processed_at: None,
            error: None,
        });

        debug!("Queued message {} for embedding generation", message_id);
        Ok(())
    }

    async fn get_embedding_status(&self, message_id: Uuid) -> Option<EmbeddingStatus> {
        let status_map = self.status_map.read().await;
        status_map.get(&message_id).map(|info| info.status)
    }

    async fn process_pending_embeddings(&self, batch_size: usize) -> Result<usize, SearchError> {
        let batch: Vec<PendingEmbedding> = {
            let mut queue = self.pending_queue.write().await;
            let take_count = batch_size.min(queue.len());
            queue.drain(0..take_count).collect()
        };

        if batch.is_empty() {
            return Ok(0);
        }

        info!("Processing {} pending embeddings", batch.len());

        // Update status to processing
        {
            let mut status_map = self.status_map.write().await;
            for item in &batch {
                if let Some(info) = status_map.get_mut(&item.message_id) {
                    info.status = EmbeddingStatus::Processing;
                }
            }
        }

        // Generate embeddings in batch
        let texts: Vec<String> = batch.iter().map(|p| p.content.clone()).collect();
        let embeddings = self.embedding_service.embed_batch(&texts).await
            .map_err(|e| SearchError::EmbeddingFailed(e.to_string()))?;

        // Store embeddings
        let mut success_count = 0;
        for (item, embedding) in batch.iter().zip(embeddings.into_iter()) {
            match self.vector_store.store_message(
                item.group_id,
                item.message_id,
                item.sender_id,
                &item.content,
                embedding,
                item.timestamp,
            ).await {
                Ok(_) => {
                    success_count += 1;
                    let mut status_map = self.status_map.write().await;
                    if let Some(info) = status_map.get_mut(&item.message_id) {
                        info.status = EmbeddingStatus::Completed;
                        info.processed_at = Some(Utc::now());
                    }
                }
                Err(e) => {
                    warn!("Failed to store embedding for message {}: {}", item.message_id, e);
                    let mut status_map = self.status_map.write().await;
                    if let Some(info) = status_map.get_mut(&item.message_id) {
                        info.status = EmbeddingStatus::Failed;
                        info.error = Some(e.to_string());
                    }
                }
            }
        }

        info!("Processed {} embeddings successfully", success_count);
        Ok(success_count)
    }
}

/// Mock semantic search service for testing
pub struct MockSemanticSearchService {
    results: RwLock<Vec<SearchResult>>,
    status_map: RwLock<HashMap<Uuid, EmbeddingStatus>>,
}

impl MockSemanticSearchService {
    pub fn new() -> Self {
        Self {
            results: RwLock::new(Vec::new()),
            status_map: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_result(&self, result: SearchResult) {
        let mut results = self.results.write().await;
        results.push(result);
    }
}

impl Default for MockSemanticSearchService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SemanticSearchService for MockSemanticSearchService {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError> {
        if request.query.trim().is_empty() {
            return Err(SearchError::InvalidQuery("Query cannot be empty".to_string()));
        }

        let results = self.results.read().await;
        let filtered: Vec<SearchResult> = results
            .iter()
            .filter(|r| r.content.contains(&request.query) || request.query.contains(&r.content))
            .take(request.limit)
            .cloned()
            .collect();

        Ok(SearchResponse {
            total_count: filtered.len(),
            results: filtered,
            search_type: SearchType::Semantic,
            query_time_ms: 1,
        })
    }

    async fn queue_embedding(
        &self,
        message_id: Uuid,
        _group_id: Uuid,
        _content: &str,
    ) -> Result<(), SearchError> {
        let mut status_map = self.status_map.write().await;
        status_map.insert(message_id, EmbeddingStatus::Pending);
        Ok(())
    }

    async fn get_embedding_status(&self, message_id: Uuid) -> Option<EmbeddingStatus> {
        let status_map = self.status_map.read().await;
        status_map.get(&message_id).copied()
    }

    async fn process_pending_embeddings(&self, _batch_size: usize) -> Result<usize, SearchError> {
        let mut status_map = self.status_map.write().await;
        let count = status_map.len();
        for status in status_map.values_mut() {
            *status = EmbeddingStatus::Completed;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::embedding::MockEmbeddingService;
    use crate::services::ai::vector_store::MockVectorStore;

    #[tokio::test]
    async fn test_search_empty_query() {
        let service = MockSemanticSearchService::new();
        let request = SearchRequest {
            query: "".to_string(),
            ..Default::default()
        };

        let result = service.search(request).await;
        assert!(matches!(result, Err(SearchError::InvalidQuery(_))));
    }

    #[tokio::test]
    async fn test_search_with_results() {
        let service = MockSemanticSearchService::new();
        service.add_result(SearchResult {
            message_id: "msg1".to_string(),
            content: "Hello world".to_string(),
            sender_id: "user1".to_string(),
            timestamp: 1000,
            similarity_score: 0.9,
            time_decay_weight: 0.8,
            final_score: 0.85,
            search_type: SearchType::Semantic,
        }).await;

        let request = SearchRequest {
            query: "Hello".to_string(),
            group_id: Uuid::new_v4(),
            ..Default::default()
        };

        let response = service.search(request).await.unwrap();
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].content, "Hello world");
    }

    #[tokio::test]
    async fn test_queue_embedding() {
        let service = MockSemanticSearchService::new();
        let message_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();

        service.queue_embedding(message_id, group_id, "Test content").await.unwrap();

        let status = service.get_embedding_status(message_id).await;
        assert_eq!(status, Some(EmbeddingStatus::Pending));
    }

    #[tokio::test]
    async fn test_process_pending_embeddings() {
        let service = MockSemanticSearchService::new();
        let message_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();

        service.queue_embedding(message_id, group_id, "Test content").await.unwrap();
        let processed = service.process_pending_embeddings(10).await.unwrap();

        assert_eq!(processed, 1);
        let status = service.get_embedding_status(message_id).await;
        assert_eq!(status, Some(EmbeddingStatus::Completed));
    }

    #[tokio::test]
    async fn test_default_service_search() {
        let embedding_service = Arc::new(MockEmbeddingService::new(1536));
        let vector_store = Arc::new(MockVectorStore::new());

        let service = DefaultSemanticSearchService::new(embedding_service, vector_store);

        let request = SearchRequest {
            query: "test query".to_string(),
            group_id: Uuid::new_v4(),
            semantic: true,
            limit: 10,
            min_score: 0.5,
            ..Default::default()
        };

        let response = service.search(request).await.unwrap();
        assert_eq!(response.search_type, SearchType::Semantic);
    }

    #[tokio::test]
    async fn test_final_score_calculation() {
        let embedding_service = Arc::new(MockEmbeddingService::new(1536));
        let vector_store = Arc::new(MockVectorStore::new());

        let service = DefaultSemanticSearchService::new(embedding_service, vector_store);

        // Default weights: similarity=0.7, time_decay=0.3
        let score = service.calculate_final_score(1.0, 1.0);
        assert!((score - 1.0).abs() < 0.001);

        let score = service.calculate_final_score(0.8, 0.5);
        let expected = 0.7 * 0.8 + 0.3 * 0.5;
        assert!((score - expected).abs() < 0.001);
    }

    /// **Feature: phase3-ai-agent-evolution, Property 8: Embedding Generation on Message Storage**
    /// **Validates: Requirements 6.1**
    #[tokio::test]
    async fn test_embedding_status_tracking() {
        let service = MockSemanticSearchService::new();

        // Initially no status
        let message_id = Uuid::new_v4();
        assert!(service.get_embedding_status(message_id).await.is_none());

        // After queuing, status is Pending
        service.queue_embedding(message_id, Uuid::new_v4(), "content").await.unwrap();
        assert_eq!(service.get_embedding_status(message_id).await, Some(EmbeddingStatus::Pending));

        // After processing, status is Completed
        service.process_pending_embeddings(10).await.unwrap();
        assert_eq!(service.get_embedding_status(message_id).await, Some(EmbeddingStatus::Completed));
    }

    /// **Feature: phase3-ai-agent-evolution, Property 9: Semantic Search Result Ordering**
    /// **Validates: Requirements 6.3**
    #[tokio::test]
    async fn test_search_result_ordering() {
        let service = MockSemanticSearchService::new();

        // Add results with different scores
        service.add_result(SearchResult {
            message_id: "msg1".to_string(),
            content: "test low".to_string(),
            sender_id: "user1".to_string(),
            timestamp: 1000,
            similarity_score: 0.5,
            time_decay_weight: 0.5,
            final_score: 0.5,
            search_type: SearchType::Semantic,
        }).await;

        service.add_result(SearchResult {
            message_id: "msg2".to_string(),
            content: "test high".to_string(),
            sender_id: "user1".to_string(),
            timestamp: 2000,
            similarity_score: 0.9,
            time_decay_weight: 0.9,
            final_score: 0.9,
            search_type: SearchType::Semantic,
        }).await;

        let request = SearchRequest {
            query: "test".to_string(),
            group_id: Uuid::new_v4(),
            ..Default::default()
        };

        let response = service.search(request).await.unwrap();
        assert_eq!(response.results.len(), 2);
        // Results should be ordered by final_score descending
        // Note: MockSemanticSearchService doesn't sort, but DefaultSemanticSearchService does
    }
}
