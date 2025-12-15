//! Vector Memory Store Implementation
//!
//! Provides vector-based memory storage using embeddings for semantic search.
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 10.1, 10.2**

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

use super::{ContextMessage, ContextRole, Memory, MemoryError, MemoryMetadata, MemoryStore};
use crate::services::ai::embedding::EmbeddingService;
use std::sync::Arc;

/// In-memory vector store for development and testing
///
/// Uses cosine similarity for semantic search.
/// For production, use QdrantMemoryStore or similar.
pub struct InMemoryVectorStore {
    /// Stored memories by group
    memories: RwLock<HashMap<Uuid, Vec<Memory>>>,
    /// Conversation history by group
    conversations: RwLock<HashMap<Uuid, Vec<ContextMessage>>>,
    /// Maximum memories per group
    max_memories_per_group: usize,
    /// Maximum conversation history per group
    max_conversation_history: usize,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            memories: RwLock::new(HashMap::new()),
            conversations: RwLock::new(HashMap::new()),
            max_memories_per_group: 1000,
            max_conversation_history: 50,
        }
    }

    /// Set maximum memories per group
    pub fn with_max_memories(mut self, max: usize) -> Self {
        self.max_memories_per_group = max;
        self
    }

    /// Set maximum conversation history
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_conversation_history = max;
        self
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Add a conversation message
    pub fn add_conversation_message(&self, group_id: Uuid, message: ContextMessage) {
        if let Ok(mut conversations) = self.conversations.write() {
            let history = conversations.entry(group_id).or_insert_with(Vec::new);
            history.push(message);

            // Trim to max history
            if history.len() > self.max_conversation_history {
                let excess = history.len() - self.max_conversation_history;
                history.drain(0..excess);
            }
        }
    }

    /// Clear conversation history for a group
    pub fn clear_conversation(&self, group_id: Uuid) {
        if let Ok(mut conversations) = self.conversations.write() {
            conversations.remove(&group_id);
        }
    }

    /// Get memory count for a group
    pub fn memory_count(&self, group_id: Uuid) -> usize {
        self.memories
            .read()
            .map(|m| m.get(&group_id).map(|v| v.len()).unwrap_or(0))
            .unwrap_or(0)
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryStore for InMemoryVectorStore {
    async fn store(&self, memory: Memory) -> Result<(), MemoryError> {
        let mut memories = self
            .memories
            .write()
            .map_err(|e| MemoryError::StoreFailed(e.to_string()))?;

        let group_memories = memories.entry(memory.group_id).or_insert_with(Vec::new);

        // Check if we need to remove old memories
        if group_memories.len() >= self.max_memories_per_group {
            // Remove lowest importance memory
            if let Some(min_idx) = group_memories
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.importance.partial_cmp(&b.importance).unwrap())
                .map(|(i, _)| i)
            {
                group_memories.remove(min_idx);
            }
        }

        group_memories.push(memory);
        debug!("Stored memory, total: {}", group_memories.len());

        Ok(())
    }

    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError> {
        // For in-memory store without embedding service, do simple text matching
        let memories = self
            .memories
            .read()
            .map_err(|e| MemoryError::RetrieveFailed(e.to_string()))?;

        let query_lower = query.to_lowercase();
        let mut results: Vec<(f32, Memory)> = Vec::new();

        for group_memories in memories.values() {
            for memory in group_memories {
                // Simple text similarity score
                let score = if memory.content.to_lowercase().contains(&query_lower) {
                    0.8 + memory.importance * 0.2
                } else {
                    // Check for word overlap
                    let query_words: std::collections::HashSet<&str> =
                        query_lower.split_whitespace().collect();
                    let memory_lower = memory.content.to_lowercase();
                    let memory_words: std::collections::HashSet<&str> =
                        memory_lower.split_whitespace().collect();
                    let overlap = query_words
                        .iter()
                        .filter(|w| memory_words.contains(*w))
                        .count();
                    if overlap > 0 {
                        (overlap as f32 / query_words.len().max(1) as f32) * 0.5
                            + memory.importance * 0.2
                    } else {
                        0.0
                    }
                };

                if score > 0.1 {
                    results.push((score, memory.clone()));
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        Ok(results.into_iter().take(limit).map(|(_, m)| m).collect())
    }

    async fn get_context(
        &self,
        group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ContextMessage>, MemoryError> {
        let conversations = self
            .conversations
            .read()
            .map_err(|e| MemoryError::RetrieveFailed(e.to_string()))?;

        Ok(conversations
            .get(&group_id)
            .map(|history| {
                let start = history.len().saturating_sub(limit);
                history[start..].to_vec()
            })
            .unwrap_or_default())
    }
}

/// Vector memory store with embedding service
///
/// Uses an embedding service for semantic search.
pub struct EmbeddingMemoryStore {
    /// Base in-memory store
    inner: InMemoryVectorStore,
    /// Embedding service
    embedding_service: Arc<dyn EmbeddingService>,
}

impl EmbeddingMemoryStore {
    /// Create a new embedding memory store
    pub fn new(embedding_service: Arc<dyn EmbeddingService>) -> Self {
        Self {
            inner: InMemoryVectorStore::new(),
            embedding_service,
        }
    }

    /// Store a memory with automatic embedding generation
    pub async fn store_with_embedding(
        &self,
        group_id: Uuid,
        content: String,
        importance: f32,
        metadata: MemoryMetadata,
    ) -> Result<Memory, MemoryError> {
        // Generate embedding
        let embedding = self
            .embedding_service
            .embed(&content)
            .await
            .map_err(|e| MemoryError::StoreFailed(e.to_string()))?;

        let memory = Memory {
            id: Uuid::new_v4(),
            group_id,
            content,
            embedding,
            importance,
            created_at: Utc::now(),
            metadata,
        };

        self.inner.store(memory.clone()).await?;
        Ok(memory)
    }
}

#[async_trait]
impl MemoryStore for EmbeddingMemoryStore {
    async fn store(&self, memory: Memory) -> Result<(), MemoryError> {
        self.inner.store(memory).await
    }

    async fn retrieve(&self, query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError> {
        // Generate query embedding
        let query_embedding = match self.embedding_service.embed(query).await {
            Ok(emb) => emb,
            Err(e) => {
                warn!("Failed to generate query embedding: {}, falling back to text search", e);
                return self.inner.retrieve(query, limit).await;
            }
        };

        // Get all memories and compute similarity
        let memories = self
            .inner
            .memories
            .read()
            .map_err(|e| MemoryError::RetrieveFailed(e.to_string()))?;

        let mut results: Vec<(f32, Memory)> = Vec::new();

        for group_memories in memories.values() {
            for memory in group_memories {
                if !memory.embedding.is_empty() {
                    let similarity =
                        InMemoryVectorStore::cosine_similarity(&query_embedding, &memory.embedding);
                    // Combine similarity with importance
                    let score = similarity * 0.8 + memory.importance * 0.2;
                    if score > 0.3 {
                        results.push((score, memory.clone()));
                    }
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        Ok(results.into_iter().take(limit).map(|(_, m)| m).collect())
    }

    async fn get_context(
        &self,
        group_id: Uuid,
        limit: usize,
    ) -> Result<Vec<ContextMessage>, MemoryError> {
        self.inner.get_context(group_id, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory(group_id: Uuid, content: &str, importance: f32) -> Memory {
        Memory {
            id: Uuid::new_v4(),
            group_id,
            content: content.to_string(),
            embedding: vec![],
            importance,
            created_at: Utc::now(),
            metadata: MemoryMetadata::default(),
        }
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = InMemoryVectorStore::new();
        let group_id = Uuid::new_v4();

        let memory = create_test_memory(group_id, "Rust is a great programming language", 0.8);
        store.store(memory).await.unwrap();

        let results = store.retrieve("programming", 5).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("programming"));
    }

    #[tokio::test]
    async fn test_max_memories_limit() {
        let store = InMemoryVectorStore::new().with_max_memories(3);
        let group_id = Uuid::new_v4();

        // Store 4 memories
        for i in 0..4 {
            let memory = create_test_memory(group_id, &format!("Memory {}", i), i as f32 * 0.25);
            store.store(memory).await.unwrap();
        }

        // Should only have 3 memories (lowest importance removed)
        assert_eq!(store.memory_count(group_id), 3);
    }

    #[tokio::test]
    async fn test_conversation_context() {
        let store = InMemoryVectorStore::new();
        let group_id = Uuid::new_v4();

        store.add_conversation_message(
            group_id,
            ContextMessage {
                role: ContextRole::User,
                content: "Hello".to_string(),
            },
        );
        store.add_conversation_message(
            group_id,
            ContextMessage {
                role: ContextRole::Assistant,
                content: "Hi there!".to_string(),
            },
        );

        let context = store.get_context(group_id, 10).await.unwrap();
        assert_eq!(context.len(), 2);
        assert_eq!(context[0].content, "Hello");
        assert_eq!(context[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_conversation_history_limit() {
        let store = InMemoryVectorStore::new().with_max_history(3);
        let group_id = Uuid::new_v4();

        for i in 0..5 {
            store.add_conversation_message(
                group_id,
                ContextMessage {
                    role: ContextRole::User,
                    content: format!("Message {}", i),
                },
            );
        }

        let context = store.get_context(group_id, 10).await.unwrap();
        assert_eq!(context.len(), 3);
        // Should have the last 3 messages
        assert_eq!(context[0].content, "Message 2");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((InMemoryVectorStore::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(InMemoryVectorStore::cosine_similarity(&a, &b).abs() < 0.001);

        let a = vec![1.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = InMemoryVectorStore::cosine_similarity(&a, &b);
        assert!(sim > 0.7 && sim < 0.8); // ~0.707
    }

    #[tokio::test]
    async fn test_clear_conversation() {
        let store = InMemoryVectorStore::new();
        let group_id = Uuid::new_v4();

        store.add_conversation_message(
            group_id,
            ContextMessage {
                role: ContextRole::User,
                content: "Hello".to_string(),
            },
        );

        store.clear_conversation(group_id);

        let context = store.get_context(group_id, 10).await.unwrap();
        assert!(context.is_empty());
    }

    #[tokio::test]
    async fn test_retrieve_with_importance() {
        let store = InMemoryVectorStore::new();
        let group_id = Uuid::new_v4();

        // Store memories with different importance
        let m1 = create_test_memory(group_id, "Important programming tip", 0.9);
        let m2 = create_test_memory(group_id, "Less important programming note", 0.3);
        store.store(m1).await.unwrap();
        store.store(m2).await.unwrap();

        let results = store.retrieve("programming", 2).await.unwrap();
        assert_eq!(results.len(), 2);
        // Higher importance should come first
        assert!(results[0].importance > results[1].importance);
    }
}
