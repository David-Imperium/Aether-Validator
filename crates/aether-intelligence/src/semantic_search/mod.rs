//! Phase 4: Vector-Backed Semantic Search
//!
//! Provides semantic search capabilities for code and documentation.
//!
//! ## Architecture
//!
//! - **SearchEngine trait** - Common interface for all search implementations
//! - **TfidfSearch** - Keyword-based matching (always available, no deps)
//! - **VectorSearch** - Semantic embeddings with candle-transformers (feature-gated)
//!
//! ## Usage
//!
//! ```ignore
//! use aether_intelligence::semantic_search::{SearchEngine, TfidfSearch};
//!
//! // TF-IDF (Free tier, always available)
//! let mut engine = TfidfSearch::new();
//! engine.index("doc1", "authentication with JWT tokens");
//! engine.index("doc2", "user login and session management");
//!
//! let results = engine.search("auth tokens", 5);
//!
//! // Vector Search (Pro tier, requires semantic-search feature)
//! #[cfg(feature = "semantic-search")]
//! {
//!     let engine = VectorSearch::new().await?;
//!     engine.index("doc1", "authentication with JWT tokens").await?;
//!     let results = engine.search("how to handle user auth", 5).await?;
//! }
//! ```

mod tfidf;

#[cfg(feature = "semantic-search")]
mod vector;

#[cfg(feature = "semantic-search")]
mod embeddings;

pub use tfidf::{TfidfSearch, TfidfConfig};

use serde::{Deserialize, Serialize};

/// Common trait for all search engines
pub trait SearchEngine: Send + Sync {
    /// Index a document with the given ID and content
    fn index(&mut self, id: &str, content: &str) -> crate::Result<()>;
    
    /// Search for documents matching the query
    fn search(&self, query: &str, limit: usize) -> Vec<SearchResult>;
    
    /// Remove a document from the index
    fn remove(&mut self, id: &str) -> crate::Result<()>;
    
    /// Clear the entire index
    fn clear(&mut self);
    
    /// Get the number of indexed documents
    fn len(&self) -> usize;
    
    /// Check if the index is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get the name of this search engine
    fn engine_name(&self) -> &'static str;
}

/// Search result with score and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    
    /// Similarity/relevance score (0.0 to 1.0)
    pub score: f32,
    
    /// Original document content (optional)
    pub content: Option<String>,
    
    /// Additional metadata (optional)
    pub metadata: Option<serde_json::Value>,
}

impl SearchResult {
    pub fn new(id: impl Into<String>, score: f32) -> Self {
        Self {
            id: id.into(),
            score,
            content: None,
            metadata: None,
        }
    }
    
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
    
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(feature = "semantic-search")]
pub use vector::{VectorSearch, VectorConfig};

#[cfg(feature = "semantic-search")]
pub use embeddings::{EmbeddingModel, EmbeddingConfig};

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub embedding_dim: usize,
    pub max_seq_length: usize,
}

/// Configuration for hybrid search (TF-IDF + Vector)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    /// Weight for TF-IDF results (0.0 to 1.0)
    pub tfidf_weight: f32,
    
    /// Weight for vector search results (0.0 to 1.0)
    pub vector_weight: f32,
    
    /// Minimum score threshold for results
    pub min_score: f32,
    
    /// Maximum results to return
    pub max_results: usize,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            tfidf_weight: 0.3,
            vector_weight: 0.7,
            min_score: 0.1,
            max_results: 10,
        }
    }
}

#[cfg(feature = "semantic-search")]
pub use hybrid::HybridSearch;

#[cfg(feature = "semantic-search")]
mod hybrid {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    /// Hybrid search combining TF-IDF and vector search
    /// 
    /// Uses weighted scoring to merge results from both engines.
    /// This provides the best of both worlds:
    /// - TF-IDF for exact keyword matching
    /// - Vector search for semantic similarity
    pub struct HybridSearch {
        tfidf: Arc<RwLock<super::TfidfSearch>>,
        vector: Arc<RwLock<super::VectorSearch>>,
        config: HybridConfig,
    }
    
    impl HybridSearch {
        pub fn new(
            tfidf: super::TfidfSearch,
            vector: super::VectorSearch,
            config: HybridConfig,
        ) -> Self {
            Self {
                tfidf: Arc::new(RwLock::new(tfidf)),
                vector: Arc::new(RwLock::new(vector)),
                config,
            }
        }
        
        /// Index a document in both engines
        pub async fn index(&self, id: &str, content: &str) -> crate::Result<()> {
            // Index in TF-IDF
            self.tfidf.write().await.index(id, content)?;
            
            // Index in vector search
            self.vector.write().await.index(id, content).await?;
            
            Ok(())
        }
        
        /// Hybrid search combining both engines
        pub async fn search(&self, query: &str, limit: usize) -> crate::Result<Vec<SearchResult>> {
            let limit = limit.min(self.config.max_results);
            
            // Run both searches in parallel
            let tfidf = self.tfidf.read().await;
            let tfidf_results = tfidf.search(query, limit * 2);
            drop(tfidf);
            
            let vector = self.vector.read().await;
            let vector_results = vector.search(query, limit * 2).await;
            drop(vector);
            
            // Merge results with weighted scoring
            let mut merged: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
            
            for result in tfidf_results {
                let score = result.score * self.config.tfidf_weight;
                *merged.entry(result.id).or_default() += score;
            }
            
            for result in vector_results {
                let score = result.score * self.config.vector_weight;
                *merged.entry(result.id).or_default() += score;
            }
            
            // Sort and filter
            let mut results: Vec<SearchResult> = merged
                .into_iter()
                .filter(|(_, score)| *score >= self.config.min_score)
                .map(|(id, score)| SearchResult::new(id, score))
                .collect();
            
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(limit);
            
            Ok(results)
        }
        
        /// Remove a document from both indexes
        pub async fn remove(&self, id: &str) -> crate::Result<()> {
            self.tfidf.write().await.remove(id)?;
            self.vector.write().await.remove(id).await?;
            Ok(())
        }
        
        /// Clear both indexes
        pub async fn clear(&self) {
            self.tfidf.write().await.clear();
            self.vector.write().await.clear().await;
        }
        
        pub fn config(&self) -> &HybridConfig {
            &self.config
        }
    }
}
