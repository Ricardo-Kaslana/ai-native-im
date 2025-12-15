//! Vector store service for semantic search using Qdrant

use async_trait::async_trait;
use qdrant_client::qdrant::{
    point_id::PointIdOptions, CreateCollectionBuilder, DeletePointsBuilder, Distance, PointStruct,
    SearchPointsBuilder, UpsertPointsBuilder, Value as QdrantValue, VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

use super::AIAgentError;
use crate::models::HistoricalMessage;

/// Default vector dimension for embeddings
/// - OpenAI text-embedding-3-small: 1536
/// - OpenAI text-embedding-ada-002: 1536
/// - SiliconFlow BAAI/bge-large-zh-v1.5: 1024
/// - SiliconFlow BAAI/bge-m3: 1024
/// 
/// Can be overridden via EMBEDDING_DIMENSION environment variable
pub fn get_embedding_dimension() -> u64 {
    std::env::var("EMBEDDING_DIMENSION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024) // Default to 1024 for SiliconFlow compatibility
}

/// Legacy constant for backward compatibility
pub const EMBEDDING_DIMENSION: u64 = 1024;

/// Collection name for message embeddings
pub const MESSAGES_COLLECTION: &str = "chat_messages";

/// Vector store trait for semantic search
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store a message embedding
    async fn store_message(
        &self,
        group_id: Uuid,
        message_id: Uuid,
        sender_id: Uuid,
        content: &str,
        embedding: Vec<f32>,
        timestamp: i64,
    ) -> Result<(), AIAgentError>;

    /// Search for similar messages
    async fn search_similar(
        &self,
        group_id: Uuid,
        query_embedding: Vec<f32>,
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<HistoricalMessage>, AIAgentError>;

    /// Delete messages for a group
    async fn delete_group_messages(&self, group_id: Uuid) -> Result<(), AIAgentError>;
}

/// Qdrant vector store implementation
pub struct QdrantVectorStore {
    client: Qdrant,
    collection_name: String,
}

impl QdrantVectorStore {
    /// Create a new Qdrant vector store
    /// 
    /// URL format:
    /// - For gRPC: "http://localhost:6334" or "localhost:6334"
    /// - The qdrant-client library uses gRPC by default
    pub async fn new(url: &str) -> Result<Self, AIAgentError> {
        // qdrant-client uses gRPC protocol
        // Port 6334 is the default gRPC port for Qdrant
        // Port 6333 is HTTP REST (not supported by this client)
        let client = Qdrant::from_url(url)
            .api_key(std::env::var("QDRANT_API_KEY").ok())
            .skip_compatibility_check() // Skip version check to avoid connection issues
            .build()
            .map_err(|e| AIAgentError::VectorDBError(format!("Failed to connect to Qdrant: {}", e)))?;

        let store = Self {
            client,
            collection_name: MESSAGES_COLLECTION.to_string(),
        };

        // Ensure collection exists
        store.ensure_collection().await?;

        Ok(store)
    }

    /// Ensure the collection exists with proper configuration
    async fn ensure_collection(&self) -> Result<(), AIAgentError> {
        let dimension = get_embedding_dimension();
        
        let exists = self
            .client
            .collection_exists(&self.collection_name)
            .await
            .map_err(|e| AIAgentError::VectorDBError(format!("Failed to check collection: {}", e)))?;

        if !exists {
            info!("Creating Qdrant collection: {} with dimension {}", self.collection_name, dimension);

            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParamsBuilder::new(dimension, Distance::Cosine)),
                )
                .await
                .map_err(|e| AIAgentError::VectorDBError(format!("Failed to create collection: {}", e)))?;

            info!("Created Qdrant collection: {} with dimension {}", self.collection_name, dimension);
        }

        Ok(())
    }

    /// Calculate time decay weight (exponential decay over 30 days)
    fn calculate_time_decay(timestamp: i64) -> f32 {
        let now = chrono::Utc::now().timestamp();
        let age_days = (now - timestamp) as f32 / 86400.0;
        let decay_rate = 0.1; // 10% decay per day
        (-decay_rate * age_days).exp().max(0.1) // Minimum weight of 0.1
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn store_message(
        &self,
        group_id: Uuid,
        message_id: Uuid,
        sender_id: Uuid,
        content: &str,
        embedding: Vec<f32>,
        timestamp: i64,
    ) -> Result<(), AIAgentError> {
        let point_id = message_id.to_string();

        let mut payload = HashMap::new();
        payload.insert("group_id".to_string(), QdrantValue::from(group_id.to_string()));
        payload.insert("sender_id".to_string(), QdrantValue::from(sender_id.to_string()));
        payload.insert("content".to_string(), QdrantValue::from(content.to_string()));
        payload.insert("timestamp".to_string(), QdrantValue::from(timestamp));

        let point = PointStruct::new(point_id, embedding, payload);

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, vec![point]))
            .await
            .map_err(|e| AIAgentError::VectorDBError(format!("Failed to store message: {}", e)))?;

        debug!("Stored message {} in vector store", message_id);
        Ok(())
    }

    async fn search_similar(
        &self,
        group_id: Uuid,
        query_embedding: Vec<f32>,
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<HistoricalMessage>, AIAgentError> {
        let search_result = self
            .client
            .search_points(
                SearchPointsBuilder::new(&self.collection_name, query_embedding, limit as u64)
                    .filter(qdrant_client::qdrant::Filter::must([
                        qdrant_client::qdrant::Condition::matches("group_id", group_id.to_string()),
                    ]))
                    .with_payload(true)
                    .score_threshold(min_score),
            )
            .await
            .map_err(|e| AIAgentError::VectorDBError(format!("Search failed: {}", e)))?;

        let messages: Vec<HistoricalMessage> = search_result
            .result
            .into_iter()
            .filter_map(|point| {
                let payload = point.payload;
                let message_id = point.id?.point_id_options?;
                let message_id_str = match message_id {
                    PointIdOptions::Uuid(s) => s,
                    PointIdOptions::Num(n) => n.to_string(),
                };

                let content = payload.get("content")?.as_str()?.to_string();
                let sender_id = payload.get("sender_id")?.as_str()?.to_string();
                let timestamp = payload.get("timestamp")?.as_integer()?;

                let time_decay = Self::calculate_time_decay(timestamp);

                Some(HistoricalMessage {
                    message_id: message_id_str,
                    content,
                    sender_id,
                    timestamp,
                    similarity_score: point.score,
                    time_decay_weight: time_decay,
                })
            })
            .collect();

        debug!("Found {} similar messages for group {}", messages.len(), group_id);
        Ok(messages)
    }

    async fn delete_group_messages(&self, group_id: Uuid) -> Result<(), AIAgentError> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection_name).points(
                    qdrant_client::qdrant::Filter::must([
                        qdrant_client::qdrant::Condition::matches("group_id", group_id.to_string()),
                    ]),
                ),
            )
            .await
            .map_err(|e| AIAgentError::VectorDBError(format!("Failed to delete messages: {}", e)))?;

        info!("Deleted all messages for group {} from vector store", group_id);
        Ok(())
    }
}


/// Mock vector store for testing
pub struct MockVectorStore {
    messages: std::sync::Mutex<Vec<StoredMessage>>,
}

#[derive(Clone)]
struct StoredMessage {
    group_id: Uuid,
    message_id: Uuid,
    sender_id: Uuid,
    content: String,
    embedding: Vec<f32>,
    timestamp: i64,
}

impl MockVectorStore {
    pub fn new() -> Self {
        Self {
            messages: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

impl Default for MockVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for MockVectorStore {
    async fn store_message(
        &self,
        group_id: Uuid,
        message_id: Uuid,
        sender_id: Uuid,
        content: &str,
        embedding: Vec<f32>,
        timestamp: i64,
    ) -> Result<(), AIAgentError> {
        let mut messages = self.messages.lock().unwrap();
        messages.push(StoredMessage {
            group_id,
            message_id,
            sender_id,
            content: content.to_string(),
            embedding,
            timestamp,
        });
        Ok(())
    }

    async fn search_similar(
        &self,
        group_id: Uuid,
        query_embedding: Vec<f32>,
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<HistoricalMessage>, AIAgentError> {
        let messages = self.messages.lock().unwrap();

        let mut results: Vec<_> = messages
            .iter()
            .filter(|m| m.group_id == group_id)
            .map(|m| {
                let score = Self::cosine_similarity(&query_embedding, &m.embedding);
                (m, score)
            })
            .filter(|(_, score)| *score >= min_score)
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        let now = chrono::Utc::now().timestamp();
        Ok(results
            .into_iter()
            .map(|(m, score)| {
                let age_days = (now - m.timestamp) as f32 / 86400.0;
                let time_decay = (-0.1 * age_days).exp().max(0.1);

                HistoricalMessage {
                    message_id: m.message_id.to_string(),
                    content: m.content.clone(),
                    sender_id: m.sender_id.to_string(),
                    timestamp: m.timestamp,
                    similarity_score: score,
                    time_decay_weight: time_decay,
                }
            })
            .collect())
    }

    async fn delete_group_messages(&self, group_id: Uuid) -> Result<(), AIAgentError> {
        let mut messages = self.messages.lock().unwrap();
        messages.retain(|m| m.group_id != group_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_vector_store_basic() {
        let store = MockVectorStore::new();
        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        // Store a message
        let embedding = vec![1.0; 1536];
        store
            .store_message(group_id, message_id, sender_id, "Hello world", embedding.clone(), 0)
            .await
            .unwrap();

        // Search for similar
        let results = store.search_similar(group_id, embedding, 10, 0.5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello world");
    }

    #[tokio::test]
    async fn test_mock_vector_store_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((MockVectorStore::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(MockVectorStore::cosine_similarity(&a, &c).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mock_vector_store_delete() {
        let store = MockVectorStore::new();
        let group_id = Uuid::new_v4();
        let message_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let embedding = vec![1.0; 1536];
        store
            .store_message(group_id, message_id, sender_id, "Test", embedding.clone(), 0)
            .await
            .unwrap();

        store.delete_group_messages(group_id).await.unwrap();

        let results = store.search_similar(group_id, embedding, 10, 0.0).await.unwrap();
        assert!(results.is_empty());
    }
}
