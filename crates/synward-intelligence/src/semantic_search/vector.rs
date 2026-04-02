//! Vector Search Engine
//!
//! Semantic search using candle-transformers with all-MiniLM-L6-v2 model.
//! Provides true semantic understanding of code and documentation.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{SearchResult, embeddings::EmbeddingModel};
use crate::Result;

/// Vector search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    /// Model name (default: all-MiniLM-L6-v2)
    pub model_name: String,
    
    /// Embedding dimension (384 for all-MiniLM-L6-v2)
    pub embedding_dim: usize,
    
    /// Maximum sequence length (256 for all-MiniLM-L6-v2)
    pub max_seq_length: usize,
    
    /// Cache embeddings in memory
    pub cache_embeddings: bool,
    
    /// Use GPU if available
    pub use_gpu: bool,
    
    /// Batch size for encoding
    pub batch_size: usize,
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            embedding_dim: 384,
            max_seq_length: 256,
            cache_embeddings: true,
            use_gpu: false,
            batch_size: 32,
        }
    }
}

/// Vector search engine using candle-transformers
pub struct VectorSearch {
    /// Configuration
    config: VectorConfig,
    
    /// Document storage: id -> content
    documents: Arc<RwLock<HashMap<String, String>>>,
    
    /// Embedding cache: id -> vector
    embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    
    /// Embedding model
    model: Arc<RwLock<Option<EmbeddingModel>>>,
    
    /// Index for efficient similarity search (simplified, no external deps)
    /// In production, you'd use HNSW or similar
    index: Arc<RwLock<Vec<String>>>,
}

impl VectorSearch {
    /// Create new vector search engine
    pub async fn new() -> Result<Self> {
        Self::with_config(VectorConfig::default()).await
    }
    
    /// Create with custom configuration
    pub async fn with_config(config: VectorConfig) -> Result<Self> {
        let model = EmbeddingModel::load(&config.model_name, config.use_gpu).await?;
        
        Ok(Self {
            config,
            documents: Arc::new(RwLock::new(HashMap::new())),
            embeddings: Arc::new(RwLock::new(HashMap::new())),
            model: Arc::new(RwLock::new(Some(model))),
            index: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Index a document and compute its embedding
    pub async fn index(&self, id: &str, content: &str) -> Result<()> {
        // Remove old entry if exists
        if self.documents.read().await.contains_key(id) {
            self.remove(id).await?;
        }
        
        // Compute embedding
        let embedding = self.encode(content).await?;
        
        // Store
        self.documents.write().await.insert(id.to_string(), content.to_string());
        self.embeddings.write().await.insert(id.to_string(), embedding);
        self.index.write().await.push(id.to_string());
        
        Ok(())
    }
    
    /// Encode text to embedding vector
    async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let model_guard = self.model.read().await;
        
        if let Some(model) = model_guard.as_ref() {
            model.encode(text).await
        } else {
            // Fallback to zero vector if model not loaded
            tracing::warn!("Embedding model not loaded, returning zero vector");
            Ok(vec![0.0f32; self.config.embedding_dim])
        }
    }
    
    /// Search for similar documents
    pub async fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_embedding = self.encode(query).await.ok().unwrap_or_else(|| vec![0.0f32; self.config.embedding_dim]);
        
        let embeddings = self.embeddings.read().await;
        let documents = self.documents.read().await;
        
        let mut results: Vec<SearchResult> = embeddings
            .iter()
            .map(|(id, embedding)| {
                let score = cosine_similarity(&query_embedding, embedding);
                SearchResult::new(id.clone(), score)
                    .with_content(documents.get(id).cloned().unwrap_or_default())
            })
            .filter(|r| r.score > 0.0)
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        results
    }
    
    /// Remove a document from the index
    pub async fn remove(&self, id: &str) -> Result<()> {
        self.documents.write().await.remove(id);
        self.embeddings.write().await.remove(id);
        self.index.write().await.retain(|i| i != id);
        Ok(())
    }
    
    /// Clear the entire index
    pub async fn clear(&self) {
        self.documents.write().await.clear();
        self.embeddings.write().await.clear();
        self.index.write().await.clear();
    }
    
    /// Get the number of indexed documents
    pub async fn len(&self) -> usize {
        self.documents.read().await.len()
    }
    
    /// Check if empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
    
    /// Get model info
    pub fn model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.config.model_name.clone(),
            embedding_dim: self.config.embedding_dim,
            max_seq_length: self.config.max_seq_length,
        }
    }
    
    /// Batch index documents
    pub async fn batch_index(&self, docs: Vec<(String, String)>) -> Result<()> {
        for (id, content) in docs {
            self.index(&id, &content).await?;
        }
        Ok(())
    }
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

/// Model information
use super::ModelInfo;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_vector_search() {
        let search = VectorSearch::new().await.unwrap();
        
        search.index("doc1", "authentication with JWT tokens").await.unwrap();
        search.index("doc2", "user login and session management").await.unwrap();
        
        let results = search.search("auth tokens", 5).await;
        
        assert!(!results.is_empty());
    }
}
