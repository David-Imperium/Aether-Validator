//! TF-IDF Search Engine
//!
//! Lightweight keyword-based matching using Term Frequency-Inverse Document Frequency.
//! No external dependencies, always available.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{SearchEngine, SearchResult};
use crate::Result;

/// TF-IDF search engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfidfConfig {
    /// Minimum term frequency to include
    pub min_term_freq: usize,
    
    /// Maximum document frequency ratio (terms in > this ratio of docs are ignored)
    pub max_df_ratio: f32,
    
    /// Enable stemming (basic suffix stripping)
    pub enable_stemming: bool,
    
    /// N-gram size for phrases
    pub ngram_size: usize,
}

impl Default for TfidfConfig {
    fn default() -> Self {
        Self {
            min_term_freq: 1,
            max_df_ratio: 0.85,
            enable_stemming: true,
            ngram_size: 2,
        }
    }
}

/// TF-IDF search engine
pub struct TfidfSearch {
    /// Document storage: id -> content
    documents: HashMap<String, String>,
    
    /// Term frequencies per document: id -> term -> count
    term_freqs: HashMap<String, HashMap<String, f32>>,
    
    /// Document frequencies: term -> number of documents containing term
    doc_freqs: HashMap<String, usize>,
    
    /// TF-IDF vectors per document: id -> term -> weight
    tfidf_vectors: HashMap<String, HashMap<String, f32>>,
    
    /// IDF cache: term -> idf value
    idf_cache: HashMap<String, f32>,
    
    /// Configuration
    config: TfidfConfig,
    
    /// Document norms for cosine similarity
    norms: HashMap<String, f32>,
}

impl TfidfSearch {
    pub fn new() -> Self {
        Self::with_config(TfidfConfig::default())
    }
    
    pub fn with_config(config: TfidfConfig) -> Self {
        Self {
            documents: HashMap::new(),
            term_freqs: HashMap::new(),
            doc_freqs: HashMap::new(),
            tfidf_vectors: HashMap::new(),
            idf_cache: HashMap::new(),
            norms: HashMap::new(),
            config,
        }
    }
    
    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut terms = Vec::new();
        
        // Split on non-alphanumeric, convert to lowercase
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            let word = word.to_lowercase();
            if word.len() >= 2 {
                let term = if self.config.enable_stemming {
                    self.stem(&word)
                } else {
                    word
                };
                if !self.is_stopword(&term) {
                    terms.push(term);
                }
            }
        }
        
        // Add n-grams
        if self.config.ngram_size > 1 && terms.len() >= self.config.ngram_size {
            for i in 0..=terms.len() - self.config.ngram_size {
                let ngram: String = terms[i..i + self.config.ngram_size].join("_");
                terms.push(ngram);
            }
        }
        
        terms
    }
    
    /// Basic stemmer (Porter-like suffix stripping)
    fn stem(&self, word: &str) -> String {
        let word = word.trim();
        
        // Common suffixes to strip
        let suffixes = [
            ("ization", "ize"),
            ("ational", "ate"),
            ("fulness", "ful"),
            ("ousness", "ous"),
            ("iveness", "ive"),
            ("ational", "ate"),
            ("tional", "t"),
            ("ences", "ence"),
            ("ances", "ance"),
            ("ments", "ment"),
            ("ities", "ity"),
            ("ings", "ing"),
            ("ives", "ive"),
            ("ized", "ize"),
            ("ful", ""),
            ("ous", ""),
            ("ive", ""),
            ("ing", ""),
            ("ed", ""),
            ("es", ""),
            ("ly", ""),
            ("er", ""),
            ("s", ""),
        ];
        
        for (suffix, replacement) in suffixes {
            if word.ends_with(suffix) && word.len() > suffix.len() + 2 {
                return format!("{}{}", &word[..word.len() - suffix.len()], replacement);
            }
        }
        
        word.to_string()
    }
    
    /// Common stopwords to ignore
    fn is_stopword(&self, word: &str) -> bool {
        const STOPWORDS: &[&str] = &[
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "be", "have", "has", "had", "do", "does", "did", "will", "would",
            "could", "should", "may", "might", "must", "shall", "can", "this",
            "that", "these", "those", "it", "its", "not", "no", "yes", "all",
            "any", "each", "every", "both", "few", "more", "most", "other",
            "some", "such", "than", "too", "very", "just", "also", "now",
            "here", "there", "where", "when", "which", "who", "what", "how",
            "why", "if", "then", "else", "so", "up", "out", "into", "over",
            "after", "before", "between", "through", "during", "about", "new",
        ];
        
        STOPWORDS.contains(&word)
    }
    
    /// Compute term frequencies for a document
    fn compute_term_freqs(&self, text: &str) -> HashMap<String, f32> {
        let terms = self.tokenize(text);
        let mut freqs = HashMap::new();
        let total = terms.len() as f32;
        
        for term in terms {
            *freqs.entry(term).or_default() += 1.0;
        }
        
        // Normalize by document length
        for count in freqs.values_mut() {
            *count /= total;
        }
        
        freqs
    }
    
    /// Update document frequencies after indexing
    fn update_doc_freqs(&mut self, terms: &HashMap<String, f32>) {
        for term in terms.keys() {
            *self.doc_freqs.entry(term.clone()).or_default() += 1;
        }
    }
    
    /// Compute IDF for a term
    fn compute_idf(&mut self, term: &str) -> f32 {
        if let Some(&idf) = self.idf_cache.get(term) {
            return idf;
        }
        
        let n_docs = self.documents.len() as f32;
        let df = *self.doc_freqs.get(term).unwrap_or(&1) as f32;
        
        // Standard IDF formula: log(N / df)
        let idf = (n_docs / df).ln();
        self.idf_cache.insert(term.to_string(), idf);
        
        idf
    }
    
    /// Compute TF-IDF vector for a document
    fn compute_tfidf(&mut self, id: &str) {
        // Clone term frequencies to avoid borrow conflicts
        let tf = match self.term_freqs.get(id).cloned() {
            Some(tf) => tf,
            None => return,
        };
        
        let mut tfidf = HashMap::new();
        let mut norm = 0.0f32;
        
        for (term, tf_val) in &tf {
            let idf = self.compute_idf(term);
            let weight = tf_val * idf;
            tfidf.insert(term.clone(), weight);
            norm += weight * weight;
        }
        
        self.tfidf_vectors.insert(id.to_string(), tfidf);
        self.norms.insert(id.to_string(), norm.sqrt());
    }
    
    /// Rebuild all IDF values (call after batch indexing)
    pub fn rebuild_idf(&mut self) {
        self.idf_cache.clear();
        
        // Clone IDs to avoid borrow conflicts
        let ids: Vec<String> = self.documents.keys().cloned().collect();
        for id in &ids {
            self.compute_tfidf(id);
        }
    }
    
    /// Compute cosine similarity between query and document
    fn cosine_similarity(&self, query_terms: &HashMap<String, f32>, doc_id: &str) -> f32 {
        if let (Some(doc_vec), Some(&doc_norm)) = (
            self.tfidf_vectors.get(doc_id),
            self.norms.get(doc_id),
        ) {
            let mut dot_product = 0.0f32;
            let mut query_norm = 0.0f32;
            
            for (term, &query_weight) in query_terms {
                if let Some(&doc_weight) = doc_vec.get(term) {
                    dot_product += query_weight * doc_weight;
                }
                query_norm += query_weight * query_weight;
            }
            
            let query_norm = query_norm.sqrt();
            let denom = query_norm * doc_norm;
            
            if denom > 0.0 {
                return dot_product / denom;
            }
        }
        
        0.0
    }
}

impl Default for TfidfSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine for TfidfSearch {
    fn index(&mut self, id: &str, content: &str) -> Result<()> {
        // Remove old entry if exists
        if self.documents.contains_key(id) {
            self.remove(id)?;
        }
        
        // Store document
        self.documents.insert(id.to_string(), content.to_string());
        
        // Compute term frequencies
        let tf = self.compute_term_freqs(content);
        
        // Update document frequencies
        self.update_doc_freqs(&tf);
        
        // Store term frequencies
        self.term_freqs.insert(id.to_string(), tf);
        
        // Compute TF-IDF vector
        self.compute_tfidf(id);
        
        Ok(())
    }
    
    fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_terms = self.compute_term_freqs(query);
        
        let mut results: Vec<SearchResult> = self
            .documents
            .keys()
            .map(|id| {
                let score = self.cosine_similarity(&query_terms, id);
                SearchResult::new(id.clone(), score).with_content(self.documents.get(id).cloned().unwrap_or_default())
            })
            .filter(|r| r.score > 0.0)
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        results
    }
    
    fn remove(&mut self, id: &str) -> Result<()> {
        if let Some(tf) = self.term_freqs.remove(id) {
            // Update document frequencies
            for term in tf.keys() {
                if let Some(df) = self.doc_freqs.get_mut(term) {
                    *df = df.saturating_sub(1);
                    if *df == 0 {
                        self.doc_freqs.remove(term);
                        self.idf_cache.remove(term);
                    }
                }
            }
        }
        
        self.documents.remove(id);
        self.tfidf_vectors.remove(id);
        self.norms.remove(id);
        
        Ok(())
    }
    
    fn clear(&mut self) {
        self.documents.clear();
        self.term_freqs.clear();
        self.doc_freqs.clear();
        self.tfidf_vectors.clear();
        self.idf_cache.clear();
        self.norms.clear();
    }
    
    fn len(&self) -> usize {
        self.documents.len()
    }
    
    fn engine_name(&self) -> &'static str {
        "TF-IDF"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenize() {
        let engine = TfidfSearch::new();
        let tokens = engine.tokenize("Hello World! This is a TEST.");
        
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }
    
    #[test]
    fn test_index_and_search() {
        let mut engine = TfidfSearch::new();
        
        engine.index("doc1", "authentication with JWT tokens").unwrap();
        engine.index("doc2", "user login and session management").unwrap();
        engine.index("doc3", "database connection pooling").unwrap();
        
        // Rebuild IDF after batch indexing
        engine.rebuild_idf();
        
        let results = engine.search("jwt authentication", 5);
        
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "doc1");
    }
    
    #[test]
    fn test_semantic_similarity() {
        let mut engine = TfidfSearch::new();
        
        engine.index("doc1", "function to calculate user age").unwrap();
        engine.index("doc2", "compute age of person").unwrap();
        engine.index("doc3", "database query optimization").unwrap();
        
        // Rebuild IDF after batch indexing
        engine.rebuild_idf();
        
        let results = engine.search("calculate age", 3);
        
        // Should find doc1 and doc2 as semantically similar
        assert!(results.iter().any(|r| r.id == "doc1"));
        assert!(results.iter().any(|r| r.id == "doc2"));
    }
    
    #[test]
    fn test_remove() {
        let mut engine = TfidfSearch::new();
        
        engine.index("doc1", "hello world").unwrap();
        assert_eq!(engine.len(), 1);
        
        engine.remove("doc1").unwrap();
        assert_eq!(engine.len(), 0);
    }
}
