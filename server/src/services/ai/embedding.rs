//! Embedding service for generating text embeddings

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

use super::AIAgentError;

/// Embedding service trait
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AIAgentError>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AIAgentError>;
}

/// OpenAI embedding service
pub struct OpenAIEmbedding {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout_secs: u64,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

impl OpenAIEmbedding {
    pub fn new(api_key: String, api_base: Option<String>, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_base: api_base.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: model.unwrap_or_else(|| "text-embedding-3-small".to_string()),
            timeout_secs: 30,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AIAgentError> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AIAgentError::EmbeddingError("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AIAgentError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        let response = self
            .client
            .post(format!("{}/embeddings", self.api_base))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::EmbeddingError(format!("Timeout after {}s", self.timeout_secs))
                } else {
                    AIAgentError::EmbeddingError(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::EmbeddingError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AIAgentError::EmbeddingError(format!("Failed to parse response: {}", e)))?;

        // Sort by index to maintain order
        let mut data = embedding_response.data;
        data.sort_by_key(|d| d.index);

        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

/// SiliconFlow embedding service (硅基流动)
/// Uses OpenAI-compatible API format
/// Supports models like BAAI/bge-large-zh-v1.5, BAAI/bge-m3, etc.
pub struct SiliconFlowEmbedding {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
    timeout_secs: u64,
}

impl SiliconFlowEmbedding {
    /// Create a new SiliconFlow embedding service
    /// Default model: BAAI/bge-large-zh-v1.5 (optimized for Chinese)
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_base: "https://api.siliconflow.cn/v1".to_string(),
            model: model.unwrap_or_else(|| "BAAI/bge-large-zh-v1.5".to_string()),
            timeout_secs: 30,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[async_trait]
impl EmbeddingService for SiliconFlowEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AIAgentError> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| AIAgentError::EmbeddingError("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AIAgentError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        let response = self
            .client
            .post(format!("{}/embeddings", self.api_base))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(self.timeout_secs))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIAgentError::EmbeddingError(format!("Timeout after {}s", self.timeout_secs))
                } else {
                    AIAgentError::EmbeddingError(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIAgentError::EmbeddingError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AIAgentError::EmbeddingError(format!("Failed to parse response: {}", e)))?;

        // Sort by index to maintain order
        let mut data = embedding_response.data;
        data.sort_by_key(|d| d.index);

        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

/// Mock embedding service for testing
pub struct MockEmbeddingService {
    dimension: usize,
}

impl MockEmbeddingService {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Generate a deterministic embedding based on text hash
    fn hash_to_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // Generate deterministic pseudo-random values
        let mut embedding = Vec::with_capacity(self.dimension);
        let mut seed = hash;
        for _ in 0..self.dimension {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = ((seed >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
            embedding.push(value);
        }

        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut embedding {
                *v /= norm;
            }
        }

        embedding
    }
}

impl Default for MockEmbeddingService {
    fn default() -> Self {
        Self::new(1536) // Default OpenAI dimension
    }
}

#[async_trait]
impl EmbeddingService for MockEmbeddingService {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AIAgentError> {
        Ok(self.hash_to_embedding(text))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AIAgentError> {
        Ok(texts.iter().map(|t| self.hash_to_embedding(t)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_deterministic() {
        let service = MockEmbeddingService::new(1536);
        
        let embedding1 = service.embed("hello world").await.unwrap();
        let embedding2 = service.embed("hello world").await.unwrap();
        
        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 1536);
    }

    #[tokio::test]
    async fn test_mock_embedding_different_texts() {
        let service = MockEmbeddingService::new(1536);
        
        let embedding1 = service.embed("hello").await.unwrap();
        let embedding2 = service.embed("world").await.unwrap();
        
        assert_ne!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_mock_embedding_normalized() {
        let service = MockEmbeddingService::new(1536);
        
        let embedding = service.embed("test").await.unwrap();
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        // Should be approximately 1.0 (unit vector)
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_mock_embedding_batch() {
        let service = MockEmbeddingService::new(1536);
        
        let texts = vec!["hello".to_string(), "world".to_string()];
        let embeddings = service.embed_batch(&texts).await.unwrap();
        
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 1536);
        assert_eq!(embeddings[1].len(), 1536);
    }

    #[tokio::test]
    async fn test_mock_embedding_empty_batch() {
        let service = MockEmbeddingService::new(1536);
        
        let embeddings = service.embed_batch(&[]).await.unwrap();
        assert!(embeddings.is_empty());
    }

    #[test]
    fn test_openai_embedding_creation() {
        let service = OpenAIEmbedding::new(
            "test-key".to_string(),
            Some("https://api.example.com".to_string()),
            Some("text-embedding-ada-002".to_string()),
        );

        assert_eq!(service.api_key, "test-key");
        assert_eq!(service.api_base, "https://api.example.com");
        assert_eq!(service.model, "text-embedding-ada-002");
    }

    #[test]
    fn test_siliconflow_embedding_creation() {
        let service = SiliconFlowEmbedding::new("test-key".to_string(), None);

        assert_eq!(service.api_key, "test-key");
        assert_eq!(service.api_base, "https://api.siliconflow.cn/v1");
        assert_eq!(service.model, "BAAI/bge-large-zh-v1.5");
    }

    #[test]
    fn test_siliconflow_embedding_custom_model() {
        let service =
            SiliconFlowEmbedding::new("test-key".to_string(), Some("BAAI/bge-m3".to_string()));

        assert_eq!(service.model, "BAAI/bge-m3");
    }

    #[test]
    fn test_siliconflow_embedding_with_timeout() {
        let service = SiliconFlowEmbedding::new("test-key".to_string(), None).with_timeout(60);

        assert_eq!(service.timeout_secs, 60);
    }
}
