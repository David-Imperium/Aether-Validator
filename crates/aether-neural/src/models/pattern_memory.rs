//! Pattern Memory — semantic similarity and experience retrieval.
//!
//! Implements the Pattern Memory (Rete B) of the Aether neuro-symbolic system.
//! Combines:
//! - **TreeFFN encoder**: CPG → L2-normalized embedding (Burn-compiled model)
//! - **HopfieldStore**: pure-Rust associative memory for cosine similarity search
//!
//! The Hopfield store runs independently of Burn — it stores and retrieves
//! pre-computed embeddings using flat f32 math. When the TreeFFN encoder is
//! compiled via `burn-onnx`, the full pipeline becomes:
//!   CPG → TreeFFN encode → HopfieldStore similarity search → PatternMatch results
//!
//! Persistence: embeddings + metadata are saved to a binary file so the
//! pattern memory survives across Aether sessions.

use crate::error::{Error, Result};
use crate::inference::CpgTensorInput;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Metadata stored alongside each pattern embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceMeta {
    /// Unique ID of the validation experience.
    pub experience_id: String,
    /// Pattern category (e.g. "UnhandledError", "Clean").
    pub category: String,
    /// Human-readable description of the pattern.
    pub description: String,
    /// Source file where the pattern was observed.
    pub source_file: Option<String>,
    /// Timestamp (epoch seconds) when this was stored.
    pub stored_at: i64,
}

/// A pattern match from the Pattern Memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// ID of the matched experience/validation record.
    pub experience_id: String,
    /// Similarity score (0.0 — 1.0).
    pub similarity: f32,
    /// Category of the original pattern.
    pub category: String,
    /// Human-readable description of why this matches.
    pub description: String,
    /// The source file where the pattern was found.
    pub source_file: Option<String>,
}

// ---------------------------------------------------------------------------
// HopfieldStore — pure-Rust associative memory
// ---------------------------------------------------------------------------

/// Binary file header for HopfieldStore persistence.
const MAGIC: &[u8; 4] = b"HPFM";
const VERSION: u8 = 1;

/// Pure-Rust associative memory backed by flat f32 cosine similarity.
///
/// Stores L2-normalized embeddings and retrieves top-k matches by cosine
/// similarity. Designed to be the runtime counterpart of the Python
/// HopfieldMemory used during training.
///
/// Capacity management: FIFO eviction when `max_patterns` is exceeded.
pub struct HopfieldStore {
    /// Dimensionality of stored embeddings.
    embed_dim: usize,
    /// Maximum number of patterns before FIFO eviction.
    max_patterns: usize,
    /// Flat storage: embeddings concatenated row-major [num * embed_dim].
    embeddings: Vec<f32>,
    /// Metadata for each stored pattern (parallel to embedding rows).
    metadata: Vec<ExperienceMeta>,
}

impl HopfieldStore {
    /// Create an empty store.
    pub fn new(embed_dim: usize, max_patterns: usize) -> Self {
        Self {
            embed_dim,
            max_patterns,
            embeddings: Vec::new(),
            metadata: Vec::new(),
        }
    }

    /// Number of stored patterns.
    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    /// Embedding dimensionality.
    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }

    /// Store a single embedding + metadata.
    pub fn store(&mut self, embedding: &[f32], meta: ExperienceMeta) {
        assert_eq!(
            embedding.len(),
            self.embed_dim,
            "embedding dimension mismatch: expected {}, got {}",
            self.embed_dim,
            embedding.len()
        );
        self.embeddings.extend_from_slice(embedding);
        self.metadata.push(meta);
        self.evict_if_needed();
    }

    /// Store a batch of embeddings + metadata.
    pub fn store_batch(&mut self, embeddings: &[f32], metas: Vec<ExperienceMeta>) {
        assert_eq!(
            embeddings.len(),
            metas.len() * self.embed_dim,
            "batch size mismatch: {} embeddings vs {} metadata entries",
            embeddings.len() / self.embed_dim,
            metas.len()
        );
        self.embeddings.extend_from_slice(embeddings);
        self.metadata.extend(metas);
        self.evict_if_needed();
    }

    /// Search for top-k most similar patterns by cosine similarity.
    ///
    /// Returns (metadata, similarity) pairs sorted by similarity descending.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(&ExperienceMeta, f32)> {
        if self.is_empty() || query.len() != self.embed_dim {
            return vec![];
        }

        let n = self.len();
        let effective_k = k.min(n);
        if effective_k == 0 {
            return vec![];
        }

        // Compute cosine similarities
        let mut scored: Vec<(usize, f32)> = Vec::with_capacity(n);
        for i in 0..n {
            let start = i * self.embed_dim;
            let sim = cosine_similarity(query, &self.embeddings[start..start + self.embed_dim]);
            scored.push((i, sim));
        }

        // Partial sort: only need top-k
        if effective_k < n {
            scored.select_nth_unstable_by(effective_k, |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            scored.sort_unstable_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        scored[..effective_k]
            .iter()
            .map(|&(idx, sim)| (&self.metadata[idx], sim))
            .collect()
    }

    /// Remove all stored patterns.
    pub fn clear(&mut self) {
        self.embeddings.clear();
        self.metadata.clear();
    }

    /// Save embeddings + metadata to a binary file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(path)?;
        let mut w = BufWriter::new(file);

        // Header
        w.write_all(MAGIC)?;
        w.write_all(&[VERSION])?;
        w.write_all(&(self.embed_dim as u32).to_le_bytes())?;
        w.write_all(&(self.len() as u32).to_le_bytes())?;

        // Embeddings (flat f32)
        let emb_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                self.embeddings.as_ptr() as *const u8,
                self.embeddings.len() * std::mem::size_of::<f32>(),
            )
        };
        w.write_all(emb_bytes)?;

        // Metadata (JSON lines)
        for meta in &self.metadata {
            let line = serde_json::to_string(meta)
                .map_err(|e| Error::LoadFailed(format!("metadata serialization: {e}")))?;
            let line_bytes = line.as_bytes();
            w.write_all(&(line_bytes.len() as u16).to_le_bytes())?;
            w.write_all(line_bytes)?;
        }

        w.flush()?;
        tracing::info!(
            "HopfieldStore saved: {} patterns ({}D) to {}",
            self.len(),
            self.embed_dim,
            path.display()
        );
        Ok(())
    }

    /// Load embeddings + metadata from a binary file.
    pub fn load(path: &Path, max_patterns: usize) -> Result<Self> {
        let file = fs::File::open(path)
            .map_err(|e| Error::LoadFailed(format!("HopfieldStore open: {e}")))?;
        let mut r = BufReader::new(file);
        let mut buf = [0u8; 4];

        // Header
        r.read_exact(&mut buf)?;
        if &buf != MAGIC {
            return Err(Error::LoadFailed(format!(
                "invalid HopfieldStore file (bad magic): {}",
                path.display()
            )));
        }
        r.read_exact(&mut buf[..1])?;
        if buf[0] != VERSION {
            return Err(Error::LoadFailed(format!(
                "unsupported HopfieldStore version: {}",
                buf[0]
            )));
        }
        r.read_exact(&mut buf)?;
        let embed_dim = u32::from_le_bytes(buf) as usize;
        r.read_exact(&mut buf)?;
        let num_patterns = u32::from_le_bytes(buf) as usize;

        // Embeddings
        let total_floats = num_patterns * embed_dim;
        let total_bytes = total_floats * std::mem::size_of::<f32>();
        let mut emb_bytes = vec![0u8; total_bytes];
        r.read_exact(&mut emb_bytes)?;
        let embeddings: Vec<f32> = unsafe {
            std::slice::from_raw_parts(
                emb_bytes.as_ptr() as *const f32,
                total_floats,
            )
            .to_vec()
        };

        // Metadata
        let mut metadata = Vec::with_capacity(num_patterns);
        let mut len_buf = [0u8; 2];
        for _ in 0..num_patterns {
            r.read_exact(&mut len_buf)?;
            let line_len = u16::from_le_bytes(len_buf) as usize;
            let mut line_buf = vec![0u8; line_len];
            r.read_exact(&mut line_buf)?;
            let line = String::from_utf8(line_buf)
                .map_err(|e| Error::LoadFailed(format!("metadata utf-8: {e}")))?;
            let meta: ExperienceMeta = serde_json::from_str(&line)
                .map_err(|e| Error::LoadFailed(format!("metadata parse: {e}")))?;
            metadata.push(meta);
        }

        tracing::info!(
            "HopfieldStore loaded: {} patterns ({}D) from {}",
            num_patterns,
            embed_dim,
            path.display()
        );
        Ok(Self {
            embed_dim,
            max_patterns,
            embeddings,
            metadata,
        })
    }

    /// FIFO eviction when exceeding max capacity.
    fn evict_if_needed(&mut self) {
        if self.len() <= self.max_patterns {
            return;
        }
        let excess = self.len() - self.max_patterns;
        let evict_floats = excess * self.embed_dim;
        self.embeddings.drain(..evict_floats);
        self.metadata.drain(..excess);
        tracing::debug!(
            "HopfieldStore: evicted {} oldest patterns (FIFO), {} remaining",
            excess,
            self.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Cosine similarity (pure f32)
// ---------------------------------------------------------------------------

/// Cosine similarity between two equal-length f32 slices.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-8 || norm_b < 1e-8 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// PatternMemory — high-level API
// ---------------------------------------------------------------------------

/// Embedding dimension for TreeFFN encoder (matches NexusTrain config).
const EMBED_DIM: usize = 256;

/// The Pattern Memory model.
///
/// Wraps the Hopfield associative memory and (eventually) the TreeFFN
/// encoder for end-to-end CPG → similarity search.
///
/// When the TreeFFN model is compiled via `burn-onnx`, `encode()` will
/// produce real embeddings. For now, the store/recall API accepts
/// pre-computed embeddings (e.g. from a sidecar process or fallback).
pub struct PatternMemory {
    /// Whether the model is loaded.
    loaded: bool,
    /// Model name identifier.
    #[allow(dead_code)]
    model_name: String,
    /// The Hopfield associative memory store.
    store: HopfieldStore,
    /// Path to the memory state file (for persistence).
    state_path: Option<std::path::PathBuf>,
}

impl PatternMemory {
    /// Load the Pattern Memory model from the given directory.
    ///
    /// Looks for `pattern_memory.burnpack` (encoder weights) and
    /// `hopfield_state.bin` (memory state). The encoder is optional
    /// (not yet compiled); the memory state enables pre-populated recall.
    pub fn load(models_dir: &Path) -> Result<Self> {
        let model_path = models_dir.join("pattern_memory.burnpack");
        let state_path = models_dir.join("hopfield_state.bin");

        let loaded = model_path.exists();
        if loaded {
            tracing::info!("Pattern Memory encoder found: {}", model_path.display());
        } else {
            tracing::info!(
                "Pattern Memory encoder not found at {} — recall only",
                model_path.display()
            );
        }

        // Load Hopfield memory state if available
        let store = if state_path.exists() {
            HopfieldStore::load(&state_path, 10_000)?
        } else {
            HopfieldStore::new(EMBED_DIM, 10_000)
        };

        Ok(Self {
            loaded,
            model_name: "pattern_memory".into(),
            store,
            state_path: if state_path.exists() {
                Some(state_path)
            } else {
                None
            },
        })
    }

    /// Create a PatternMemory with custom capacity (for testing only).
    #[cfg(test)]
    pub fn with_capacity(embed_dim: usize, max_patterns: usize) -> Self {
        Self {
            loaded: false,
            model_name: "test".into(),
            store: HopfieldStore::new(embed_dim, max_patterns),
            state_path: None,
        }
    }

    /// Store a pre-computed embedding with experience metadata.
    ///
    /// This is the primary API for populating the pattern memory from
    /// external sources (e.g. validation results, TreeFFN sidecar).
    pub fn store_embedding(&mut self, embedding: &[f32], meta: ExperienceMeta) {
        self.store.store(embedding, meta);
    }

    /// Store a batch of embeddings with metadata.
    pub fn store_batch(&mut self, embeddings: &[f32], metas: Vec<ExperienceMeta>) {
        self.store.store_batch(embeddings, metas);
    }

    /// Number of patterns currently stored.
    pub fn num_stored(&self) -> usize {
        self.store.len()
    }

    /// Find similar patterns to the given CPG input.
    ///
    /// **Note**: this requires the TreeFFN encoder to be compiled.
    /// Until then, use `store_embedding()` + `search()` directly.
    pub fn find_similar(&self, input: &CpgTensorInput) -> Result<Vec<PatternMatch>> {
        self.find_similar_top_k(input, 5)
    }

    /// Find top-k most similar patterns to the given CPG input.
    pub fn find_similar_top_k(
        &self,
        input: &CpgTensorInput,
        _k: usize,
    ) -> Result<Vec<PatternMatch>> {
        if !self.loaded {
            return Err(Error::ModelNotLoaded("Pattern Memory".into()));
        }
        input.validate()?;

        // TODO: When TreeFFN ONNX is compiled via burn-onnx:
        // 1. Build Burn tensors from CpgTensorInput
        // 2. Run TreeFFN encoder → [embed_dim] embedding
        // 3. Query self.store.search(&embedding, k)
        // 4. Convert results to Vec<PatternMatch>
        Ok(vec![])
    }

    /// Search the memory directly with a pre-computed embedding.
    ///
    /// This works even without the encoder — useful for testing
    /// and for when embeddings come from an external source.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<PatternMatch> {
        self.store
            .search(query, k)
            .into_iter()
            .map(|(meta, sim)| PatternMatch {
                experience_id: meta.experience_id.clone(),
                similarity: sim,
                category: meta.category.clone(),
                description: meta.description.clone(),
                source_file: meta.source_file.clone(),
            })
            .collect()
    }

    /// Persist the current memory state to disk.
    ///
    /// Saves to the path where the state was loaded from, or to
    /// `models_dir/hopfield_state.bin` if no path was set.
    pub fn persist(&self, models_dir: &Path) -> Result<()> {
        let fallback = models_dir.join("hopfield_state.bin");
        let path = self.state_path.as_deref().unwrap_or(&fallback);
        self.store.save(path)
    }

    /// Check if the model is loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Get a reference to the underlying Hopfield store (for testing).
    #[cfg(test)]
    pub fn store(&self) -> &HopfieldStore {
        &self.store
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::SQRT_2;

    fn make_meta(id: &str, category: &str) -> ExperienceMeta {
        ExperienceMeta {
            experience_id: id.into(),
            category: category.into(),
            description: format!("pattern {id}"),
            source_file: Some(format!("src/{id}.rs")),
            stored_at: 1_700_000_000,
        }
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![0.5, 0.5, 0.5, 0.5];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_45deg() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 1.0];
        let expected = 1.0 / SQRT_2; // cos(45°)
        assert!((cosine_similarity(&a, &b) - expected).abs() < 1e-6);
    }

    #[test]
    fn test_hopfield_store_store_and_search() {
        let mut store = HopfieldStore::new(4, 100);

        store.store(
            &[1.0, 0.0, 0.0, 0.0],
            make_meta("a", "error"),
        );
        store.store(
            &[0.0, 1.0, 0.0, 0.0],
            make_meta("b", "clean"),
        );
        store.store(
            &[0.9, 0.1, 0.0, 0.0],
            make_meta("c", "error"),
        );

        assert_eq!(store.len(), 3);

        // Search for something similar to [1,0,0,0] → should find "a" and "c"
        let results = store.search(&[1.0, 0.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.experience_id, "a"); // top match
        assert_eq!(results[1].0.experience_id, "c"); // second match
        assert!(results[0].1 > results[1].1); // sorted by similarity
    }

    #[test]
    fn test_hopfield_store_fifo_eviction() {
        let mut store = HopfieldStore::new(2, 2);

        store.store(&[1.0, 0.0], make_meta("first", "x"));
        store.store(&[0.0, 1.0], make_meta("second", "y"));
        store.store(&[0.5, 0.5], make_meta("third", "z")); // evicts "first"

        assert_eq!(store.len(), 2);
        let results = store.search(&[0.0, 1.0], 2);
        assert_eq!(results[0].0.experience_id, "second"); // "first" was evicted
    }

    #[test]
    fn test_hopfield_store_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_state.bin");

        let mut store = HopfieldStore::new(4, 100);
        store.store(&[1.0, 0.0, 0.0, 0.0], make_meta("p1", "error"));
        store.store(&[0.0, 1.0, 0.0, 0.0], make_meta("p2", "clean"));
        store.save(&path).unwrap();

        let loaded = HopfieldStore::load(&path, 100).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.embed_dim(), 4);

        let results = loaded.search(&[1.0, 0.0, 0.0, 0.0], 1);
        assert_eq!(results[0].0.experience_id, "p1");
    }

    #[test]
    fn test_pattern_memory_load_missing_dir() {
        // load() creates the store even without burnpack (graceful degradation).
        // It only fails if the parent directory doesn't exist for state file creation.
        let tmp = tempfile::tempdir().unwrap();
        let result = PatternMemory::load(tmp.path());
        assert!(result.is_ok());
        let pm = result.unwrap();
        assert!(!pm.is_loaded()); // no burnpack → encoder not loaded
        assert_eq!(pm.num_stored(), 0); // empty store
    }

    #[test]
    fn test_pattern_memory_search_with_embedding() {
        let mut pm = PatternMemory {
            loaded: false,
            model_name: "test".into(),
            store: HopfieldStore::new(4, 100),
            state_path: None,
        };

        pm.store_embedding(&[1.0, 0.0, 0.0, 0.0], make_meta("x", "error"));
        pm.store_embedding(&[0.0, 1.0, 0.0, 0.0], make_meta("y", "clean"));

        let results = pm.search(&[0.9, 0.1, 0.0, 0.0], 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].experience_id, "x");
        assert!(results[0].similarity > 0.9);
    }

    #[test]
    fn test_pattern_memory_store_batch() {
        let mut pm = PatternMemory {
            loaded: false,
            model_name: "test".into(),
            store: HopfieldStore::new(2, 100),
            state_path: None,
        };

        let embeddings: Vec<f32> = vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let metas = vec![make_meta("a", "x"), make_meta("b", "y"), make_meta("c", "z")];
        pm.store_batch(&embeddings, metas);

        assert_eq!(pm.num_stored(), 3);
    }

    #[test]
    fn test_pattern_memory_persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let models_dir = dir.path();
        let state_path = models_dir.join("hopfield_state.bin");

        // Use direct construction with 4-dim embeddings for test simplicity
        let mut pm = PatternMemory {
            loaded: false,
            model_name: "test".into(),
            store: HopfieldStore::new(4, 100),
            state_path: None,
        };
        pm.store_embedding(&[1.0, 0.0, 0.0, 0.0], make_meta("p1", "error"));
        pm.store_embedding(&[0.0, 1.0, 0.0, 0.0], make_meta("p2", "clean"));
        pm.persist(models_dir).unwrap();

        // Reload via HopfieldStore::load and verify
        let loaded_store = HopfieldStore::load(&state_path, 100).unwrap();
        assert_eq!(loaded_store.len(), 2);
        let results = loaded_store.search(&[1.0, 0.0, 0.0, 0.0], 1);
        assert_eq!(results[0].0.experience_id, "p1");
        assert!(results[0].1 > 0.99);
    }
}
