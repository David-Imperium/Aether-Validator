# Aether RAG — Retrieval Augmented Generation

**Version:** 0.1.0
**Status:** Planned
**Related:** [AETHER_ARCHITECTURE.md](./AETHER_ARCHITECTURE.md), [AETHER_PRE_GUIDANCE.md](./AETHER_PRE_GUIDANCE.md)

---

## Overview

Aether RAG (Retrieval Augmented Generation) è un sistema ibrido per la ricerca intelligente nella documentazione del progetto. Permette ad Aether di:

1. **Cercare documentazione pertinente** prima che l'agente scriva codice
2. **Recuperare pattern imparati** dalle interazioni passate
3. **Fornire contesto** per ridurre gli errori

---

## Architettura Ibrida

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AETHER RAG ARCHITECTURE                           │
│                                                                              │
│  ┌──────────────┐                                                           │
│  │   Query      │                                                           │
│  │   Input      │                                                           │
│  └──────┬───────┘                                                           │
│         │                                                                    │
│         ▼                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        QUERY PROCESSOR                                │  │
│  │  • Normalize query                                                    │  │
│  │  • Extract keywords                                                   │  │
│  │  • Detect query type (keyword/semantic)                              │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│         ┌─────────────────────────┼─────────────────────────┐              │
│         │                         │                          │              │
│         ▼                         ▼                          ▼              │
│  ┌─────────────┐         ┌─────────────────┐        ┌───────────────┐     │
│  │   KEYWORD   │         │    SEMANTIC     │        │    PATTERN    │     │
│  │   INDEX     │         │    SEARCH       │        │    LIBRARY    │     │
│  │             │         │                 │        │               │     │
│  │ TF-IDF      │         │ Embeddings     │        │ Learned       │     │
│  │ BM25        │         │ Cosine Sim.    │        │ Patterns      │     │
│  │ Fast        │         │ Cached         │        │ User Stats    │     │
│  │ <10ms       │         │ ~50ms          │        │ Instant       │     │
│  └──────┬──────┘         └────────┬────────┘        └───────┬───────┘     │
│         │                         │                         │              │
│         └─────────────────────────┼─────────────────────────┘              │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        RESULT MERGER                                 │  │
│  │  • Deduplicate results                                                │  │
│  │  • Rank by relevance                                                  │  │
│  │  • Limit to top N                                                     │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        CONTEXT GENERATOR                             │  │
│  │  • Format results for agent                                          │  │
│  │  • Include source references                                         │  │
│  │  • Add relevance scores                                              │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Componenti

### 1. Keyword Index

Ricerca veloce basata su parole chiave.

```rust
// crates/aether-rag/src/keyword.rs
use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;

/// Documento indicizzato.
#[derive(Debug, Clone)]
pub struct IndexedDocument {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub tfidf_scores: HashMap<String, f32>,
}

/// Indice keyword basato su TF-IDF e BM25.
pub struct KeywordIndex {
    /// Documenti indicizzati.
    documents: HashMap<String, IndexedDocument>,
    /// Term frequency per documento.
    term_freq: HashMap<String, HashMap<String, usize>>,
    /// Inverse document frequency.
    idf: HashMap<String, f32>,
    /// Document frequency.
    doc_freq: HashMap<String, usize>,
    /// Totale documenti.
    total_docs: usize,
}

impl KeywordIndex {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            term_freq: HashMap::new(),
            idf: HashMap::new(),
            doc_freq: HashMap::new(),
            total_docs: 0,
        }
    }
    
    /// Indicizza un documento.
    pub fn index(&mut self, doc: IndexedDocument) {
        // Calcola TF-IDF
        // Aggiorna statistiche
    }
    
    /// Cerca con query keyword.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        // BM25 ranking
        // Restituisci top N risultati
    }
    
    /// Tempo medio di ricerca: < 10ms
}
```

### 2. Semantic Search

Ricerca semantica con embeddings cached.

```rust
// crates/aether-rag/src/semantic.rs
use std::collections::HashMap;
use std::path::PathBuf;

/// Embedding cache per evitare ricalcoli.
pub struct EmbeddingCache {
    /// Embeddings precached.
    embeddings: HashMap<String, Vec<f32>>,
    /// Dimensione embeddings.
    dimension: usize,
    /// Cache hit rate.
    hit_rate: f32,
}

/// Motore di ricerca semantica.
pub struct SemanticSearch {
    /// Cache degli embeddings.
    cache: EmbeddingCache,
    /// Modello di embedding (locale, no API).
    model: LocalEmbeddingModel,
}

impl SemanticSearch {
    /// Cerca per similarità semantica.
    pub fn search(&mut self, query: &str, limit: usize) -> Vec<SearchResult> {
        // Genera embedding query
        let query_embedding = self.model.embed(query);
        
        // Cerca nella cache per similarità coseno
        let mut results: Vec<_> = self.cache.embeddings.iter()
            .map(|(id, emb)| {
                let similarity = cosine_similarity(&query_embedding, emb);
                (id.clone(), similarity)
            })
            .collect();
        
        // Ordina per similarità
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Restituisci top N
        results.into_iter()
            .take(limit)
            .map(|(id, score)| SearchResult { id, score, source: "semantic" })
            .collect()
    }
    
    /// Tempo medio: ~50ms (con cache)
}
```

### 3. Pattern Library

Pattern imparati dall'utente.

```rust
// crates/aether-rag/src/pattern.rs
use std::collections::HashMap;

/// Pattern imparato dall'utente.
#[derive(Debug, Clone)]
pub struct LearnedPattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub content: String,
    pub frequency: usize,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub success_rate: f32,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    /// Codice preferito dall'utente.
    PreferredCode,
    /// Errore comune da evitare.
    CommonError,
    /// Regola violata spesso.
    ViolatedRule,
    /// Stile di codice.
    CodeStyle,
}

/// Libreria di pattern imparati.
pub struct PatternLibrary {
    patterns: HashMap<String, LearnedPattern>,
    user_preferences: HashMap<String, String>,
}

impl PatternLibrary {
    /// Cerca pattern rilevanti per dominio/intent.
    pub fn find_relevant(&self, domain: &str, intent: &str) -> Vec<&LearnedPattern> {
        // Trova pattern per dominio e intent
    }
    
    /// Registra un nuovo pattern.
    pub fn learn(&mut self, pattern: LearnedPattern) {
        // Aggiungi o aggiorna pattern
    }
}
```

---

## Flusso di Ricerca

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SEARCH FLOW                                        │
│                                                                              │
│  Input: "Add enemy patrol behavior"                                         │
│                                                                              │
│  Step 1: Query Processing                                                   │
│  ├── Keywords: ["enemy", "patrol", "behavior"]                            │
│  ├── Domain: "gameplay"                                                     │
│  └── Intent: "CREATE"                                                       │
│                                                                              │
│  Step 2: Parallel Search                                                    │
│  ├── Keyword Index → 15 results (10ms)                                     │
│  ├── Semantic Search → 8 results (50ms)                                    │
│  └── Pattern Library → 3 results (instant)                                 │
│                                                                              │
│  Step 3: Merge & Rank                                                       │
│  ├── Deduplicate → 20 unique results                                       │
│  ├── Rank by relevance → top 5                                             │
│  └── Add confidence scores                                                  │
│                                                                              │
│  Step 4: Context Generation                                                 │
│  ├── Format for agent                                                      │
│  ├── Include source references                                              │
│  └── Return to Pre-Guidance                                                 │
│                                                                              │
│  Output:                                                                    │
│  [                                                                                          │
│    { source: "docs/AI.md", snippet: "...", score: 0.95 },                   │
│    { source: "docs/Enemy.md", snippet: "...", score: 0.89 },                │
│    { source: "pattern:preferred_enemy_impl", score: 0.85 },                │
│    ...                                                                                     │
│  ]                                                                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Indicizzazione

### Documenti Supportati

| Tipo | Estensioni | Priorità |
|------|------------|----------|
| Markdown | `.md` | Alta |
| Rust | `.rs` | Alta |
| C++ | `.cpp`, `.h`, `.hpp` | Alta |
| Config | `.yaml`, `.toml`, `.json` | Media |
| Text | `.txt` | Bassa |

### Struttura Indice

```
.aether/
├── index/
│   ├── keyword.idx      # Indice keyword (TF-IDF)
│   ├── semantic.idx     # Embeddings cache
│   ├── patterns.idx     # Pattern imparati
│   └── metadata.json    # Metadati indice
└── config/
    └── rag.yaml         # Configurazione RAG
```

### Configurazione

```yaml
# .aether/config/rag.yaml
version: "1.0"

# Indicizzazione
indexing:
  include_patterns:
    - "docs/**/*.md"
    - "src/**/*.rs"
    - "src/**/*.cpp"
    - "*.md"
  exclude_patterns:
    - "target/**"
    - "node_modules/**"
    - ".git/**"
  max_file_size: 1MB
  
# Ricerca
search:
  keyword_weight: 0.6
  semantic_weight: 0.3
  pattern_weight: 0.1
  max_results: 10
  
# Performance
performance:
  cache_embeddings: true
  cache_size_mb: 100
  keyword_index_in_memory: true
  
# Aggiornamento
update:
  watch_files: true
  reindex_on_change: true
  debounce_ms: 500
```

---

## API

### Rust API

```rust
use aether_rag::{RagEngine, SearchResult, IndexedDocument};

// Inizializza RAG
let mut rag = RagEngine::new("./project")?;

// Indicizza documenti
rag.index_document(IndexedDocument::from_file("docs/AI.md")?)?;

// Cerca
let results = rag.search("enemy patrol behavior", 5)?;

for result in results {
    println!("{} (score: {:.2})", result.source, result.score);
    println!("  {}", result.snippet);
}
```

### MCP Tool

```json
{
  "name": "aether_rag_search",
  "description": "Search project documentation for relevant context",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Search query"
      },
      "limit": {
        "type": "integer",
        "description": "Maximum results",
        "default": 5
      },
      "sources": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Filter by source types (docs, code, patterns)"
      }
    },
    "required": ["query"]
  }
}
```

---

## Performance

| Operazione | Target | Tipico |
|------------|--------|--------|
| Keyword search | < 10ms | 5ms |
| Semantic search (cached) | < 50ms | 35ms |
| Pattern lookup | < 1ms | instant |
| Index document | < 100ms | 50ms |
| Full reindex | < 30s | 15s |

---

## Integrazione con Pre-Guidance

Il RAG alimenta il sistema di Pre-Guidance:

```rust
// crates/aether-wrapper/src/pre_guidance.rs
impl PreGuidance {
    pub fn generate_context(&self, prompt: &str) -> GuidanceContext {
        // 1. Analizza prompt
        let analysis = self.analyzer.analyze(prompt);
        
        // 2. Cerca documentazione pertinente
        let docs = self.rag.search(&analysis.keywords.join(" "), 5)?;
        
        // 3. Recupera pattern imparati
        let patterns = self.learner.get_relevant_patterns(&analysis);
        
        // 4. Genera contesto per l'agente
        GuidanceContext {
            prompt_analysis: analysis,
            documentation: docs,
            patterns,
            warnings: self.generate_warnings(&analysis),
        }
    }
}
```

---

## Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.

---

## Note Commerciali

| Feature | Community | Commercial |
|---------|-----------|------------|
| Keyword Search | ✅ | ✅ |
| Semantic Search (cached) | ❌ | ✅ |
| Pattern Library | ❌ | ✅ |
| Cloud Sync | ❌ | ✅ (opzionale) |
| Custom Embeddings | ❌ | ✅ |
