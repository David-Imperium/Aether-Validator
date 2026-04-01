# Aether Memory-Driven Core

**Version:** 3.0
**Last Updated:** 2026-03-20
**Author:** Droid + David
**See Also:** [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md)

---

## Executive Summary

Aether non è un validatore statico con regole fisse. È un **sistema autonomo che evolve** con il tuo codebase attraverso la memoria. Ogni istanza di Aether diventa unica perché impara dal contesto in cui opera.

**Key Principles (v3.0):**
- **AI-Free Core**: Funziona senza AI esterna. L'AI è opzionale, solo come "dizionario"
- **Graph RAG Autonomo**: Attraversa progetti, capisce dipendenze, impara pattern
- **Dubbioso Mode**: Confidence-based validation, chiede quando incerto
- **TOML Format**: Memoria leggibile e modificabile dall'utente
- **Temporal Memory** (v3.0): Decisioni versionate, fact supersession, multi-signal scoring
- **Memory Hierarchy** (v3.0): Core/Archival layers con decay automatico

**Il differentiator:** Regole dinamiche che si adattano al progetto, non imposte dall'alto.

---

## Il Problema dei Validatori Tradizionali

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    VALIDATORI TRADIZIONALI                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Regole Fisse (Same per tutti)                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Complessità massima: 15                                                  │
│  • Linea massima: 120 caratteri                                             │
│  • Naming: snake_case                                                       │
│  • Error handling: sempre esplicito                                         │
│                                                                             │
│  Problemi:                                                                  │
│  ─────────────────────────────────────────────────────────────────────────  │
│  ❌ Falsi positivi costanti                                                 │
│  ❌ Non rispetta convenzioni del team                                       │
│  ❌ Stesso output per progetti diversi                                      │
│  ❌ Valore costante nel tempo (non migliora)                                │
│                                                                             │
│  Risultato: L'utente IGNORA le warning                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## La Soluzione: Memory-Driven Core

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AETHER MEMORY-DRIVEN CORE (v3.0)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  MEMORIA HIERARCHICAL          CORE (DINAMICO)                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
│  ┌─────────────────┐        ┌─────────────────────────────────────────────┐ │
│  │ CORE LAYER      │        │ LAYER CONFIGURATION                          │ │
│  │ (Active Session)│───────▶│                                              │ │
│  │ • Decisioni     │        │ • Soglie: Memory → Syntax                    │ │
│  │ • Violations    │        │ • Whitelist: Memory → Security               │ │
│  │ • Patterns      │        │ • Custom rules: Memory → Logic               │ │
│  └─────────────────┘        │ • Conventions: Memory → Style                │ │
│                             │                                              │ │
│  ┌─────────────────┐        │ Ogni progetto ha il SUO set di regole        │ │
│  │ ARCHIVAL LAYER  │        └─────────────────────────────────────────────┘ │
│  │ (History)       │                         │                              │
│  │ • Superseded    │                         ▼                              │
│  │ • Decay score   │        ┌─────────────────────────────────────────────┐ │
│  │ • Consolidated  │        │ VALIDAZIONE PERSONALIZZATA                   │ │
│  └─────────────────┘        │                                              │ │
│                             │ • Warning che rispecchiano il progetto       │ │
│  MULTI-SIGNAL SCORING       │ • Falsi positivi RIDOTTI nel tempo           │ │
│  ─────────────────────────  │ • Valore che CRESCE con l'uso                │ │
│  score = α*relevance +      │                                              │ │
│         β*recency +         └─────────────────────────────────────────────┘ │
│         γ*importance                                                         │
│                                                                             │
│  FEEDBACK LOOP: Ogni validazione → Memory impara → Core si adatta           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## ADR: Temporal Memory Upgrade (v3.0)

**Status:** Proposed  
**Date:** 2026-03-20  
**Context:** State-of-the-art agent memory research (Mem0, Zep, Letta/MemGPT, LangMem)

### Context

La ricerca SOTA 2026 su agent memory ha identificato pattern che Aether attualmente non implementa:

| Feature | Aether v2.0 | SOTA 2026 | Impact |
|---------|-------------|-----------|--------|
| **Temporal Fact Supersession** | ❌ | ✅ Zep | Decisioni obsolete non gestite |
| **Multi-Signal Scoring** | ❌ Solo confidence | ✅ Mem0/Zep | Scoring monodimensionale |
| **Memory Hierarchy** | ❌ Flat storage | ✅ Letta | Core vs Archival separation |
| **Vector Similarity** | ❌ Jaccard tokens | ✅ Standard | Semantic recall impreciso |
| **Fact Deduplication** | ❌ | ✅ Mem0/Zep | Dati duplicati |
| **Decay Functions** | ❌ | ✅ LangMem | Stale data persiste |

### Decision

#### DEC-001: Temporal Decision Log

Implementare versioning delle decisioni con `superseded_by` per gestire obsolescenza.

```rust
/// Decision node con supporto temporale (v3.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNodeV3 {
    pub id: MemoryId,
    pub decision_type: DecisionType,
    pub content: String,
    pub location: CodeLocation,
    
    // NUOVO: Temporal fields
    pub created_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub superseded_by: Option<MemoryId>,  // Link a decisione più recente
    pub supersedes: Vec<MemoryId>,         // Decisioni che questa rimpiazza
    
    // NUOVO: Multi-signal score
    pub relevance: f32,   // Quanto è rilevante al contesto corrente
    pub recency: f32,     // Decay basato su tempo (1.0 = oggi, 0.0 = mai)
    pub importance: f32,  // Impatto se ignorata (0.0-1.0)
    pub score: f32,       // Composto: α*relevance + β*recency + γ*importance
}

impl DecisionNodeV3 {
    /// Calcola score composito
    pub fn calculate_score(&mut self, config: &ScoringConfig) {
        self.recency = self.calculate_recency();
        self.score = config.alpha * self.relevance 
                   + config.beta * self.recency 
                   + config.gamma * self.importance;
    }
    
    /// Decay esponenziale basato su tempo
    fn calculate_recency(&self) -> f32 {
        let age_days = (Utc::now() - self.created_at).num_days() as f32;
        (-age_days / RECENCY_HALF_LIFE_DAYS).exp()
    }
}
```

**Configurazione scoring:**

```toml
# .aether/config.toml
[memory.scoring]
alpha = 0.4   # Relevance weight
beta = 0.3    # Recency weight  
gamma = 0.3   # Importance weight
recency_half_life_days = 30.0  # Decay rate
```

#### DEC-002: Memory Hierarchy (Core/Archival)

Implementare layer gerarchici ispirati a Letta/MemGPT.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MEMORY HIERARCHY                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ CORE LAYER (Hot)                                                     │   │
│  │ ─────────────────────────────────────────────────────────────────    │   │
│  │ • Ultimi 30 giorni di validazione                                    │   │
│  │ • Violazioni attive (non risolte)                                    │   │
│  │ • Decisioni correnti (non superseded)                                │   │
│  │ • Pattern ad alta frequenza                                          │   │
│  │ • Accesso: O(1) lookup, sempre caricato                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼ Consolidation (automatica)             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ ARCHIVAL LAYER (Cold)                                                │   │
│  │ ─────────────────────────────────────────────────────────────────    │   │
│  │ • Decisioni superseded (con link a versione corrente)                │   │
│  │ • Violazioni risolte > 30 giorni fa                                  │   │
│  │ • Pattern consolidati (sintesi di occorrenze multiple)               │   │
│  │ • Statistiche storiche per trend analysis                            │   │
│  │ • Accesso: Lazy load, compressato                                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  CONSOLIDATION PIPELINE                                                     │
│  ─────────────────────────────────────────────────────────────────────────  │
│  1. Ogni 7 giorni: scan Core entries                                       │
│  2. Se entry > 30 giorni AND score < 0.3 → candidate archival              │
│  3. Se pattern con 10+ occorrenze → consolida in singola entry             │
│  4. Superseded decisions → move to archival                                │
│  5. Mantieni link bidirezionali per audit trail                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Implementazione:**

```rust
/// Memory store gerarchico (v3.0)
pub struct HierarchicalMemoryStore {
    /// Core layer - sempre in memoria
    core: HashMap<MemoryId, MemoryEntry>,
    
    /// Archival layer - lazy load da disco
    archival_path: PathBuf,
    archival_index: HashMap<MemoryId, ArchivalMetadata>,
    
    /// Configurazione
    config: HierarchyConfig,
}

impl HierarchicalMemoryStore {
    /// Retrieve con fallback gerarchico
    pub fn get(&self, id: &MemoryId) -> Option<MemoryEntry> {
        // 1. Check core layer (hot path)
        if let Some(entry) = self.core.get(id) {
            return Some(entry.clone());
        }
        
        // 2. Check archival index (cold path)
        if let Some(meta) = self.archival_index.get(id) {
            return self.load_from_archival(id);
        }
        
        None
    }
    
    /// Promuovi entry da archival a core
    pub fn promote(&mut self, id: &MemoryId) -> Result<()> {
        if let Some(entry) = self.load_from_archival(id) {
            self.core.insert(id.clone(), entry);
            tracing::info!("Promoted {} to core layer", id);
        }
        Ok(())
    }
    
    /// Consolidation pipeline (chiamato periodicamente)
    pub fn consolidate(&mut self) -> Result<ConsolidationReport> {
        let mut report = ConsolidationReport::default();
        
        let now = Utc::now();
        let archive_threshold = now - chrono::Duration::days(self.config.archive_after_days);
        
        // Find candidates for archival
        let to_archive: Vec<_> = self.core.iter()
            .filter(|(_, entry)| {
                entry.created_at < archive_threshold && entry.score < self.config.archive_score_threshold
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in to_archive {
            if let Some(entry) = self.core.remove(&id) {
                self.save_to_archival(&id, &entry)?;
                report.archived += 1;
            }
        }
        
        // Consolidate patterns
        report.patterns_consolidated = self.consolidate_patterns()?;
        
        tracing::info!("Consolidation complete: {:?}", report);
        Ok(report)
    }
}
```

#### DEC-003: Vector-Backed Semantic Recall

Integrare con david-rag per semantic search invece di Jaccard.

```rust
/// Semantic retriever usando vector embeddings (v3.0)
pub struct SemanticRetriever {
    /// Client david-rag (MCP o diretto)
    rag_client: RagClient,
    
    /// Cache locale per lookup frequenti
    embedding_cache: LruCache<String, Vec<f32>>,
    
    /// Config
    config: SemanticConfig,
}

impl SemanticRetriever {
    /// Cerca entries semanticamente simili
    pub async fn semantic_search(
        &self, 
        query: &str, 
        limit: usize,
        search_type: SearchType,
    ) -> Result<Vec<ScoredEntry>> {
        match search_type {
            SearchType::Semantic => {
                self.rag_client.search(query, limit, "semantic").await
            }
            SearchType::Hybrid => {
                // Combina semantic + BM25 (implementato in david-rag)
                self.rag_client.search(query, limit, "hybrid_reranked").await
            }
            SearchType::Keyword => {
                self.rag_client.search(query, limit, "keyword").await
            }
        }
    }
}
```

**Integrazione con MemoryStore:**

```rust
impl MemoryStore {
    /// Recall con semantic search (v3.0)
    pub async fn recall_semantic(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>> {
        let retriever = SemanticRetriever::new(&self.config.rag_endpoint)?;
        
        let scored = retriever.semantic_search(query, limit, SearchType::Hybrid).await?;
        
        // Converti ScoredEntry → MemoryEntry
        let entries: Vec<_> = scored.into_iter()
            .filter_map(|s| self.get(&s.id))
            .collect();
        
        Ok(entries)
    }
}
```

#### DEC-004: Fact Deduplication

Evitare duplicazione di decisioni simili.

```rust
impl DecisionLog {
    /// Aggiungi decisione con deduplication (v3.0)
    pub fn add_decision(&mut self, mut decision: DecisionNodeV3) -> Result<MemoryId> {
        // 1. Cerca decisioni simili esistenti
        let similar = self.find_similar(&decision.content, 0.85)?;
        
        if let Some(existing) = similar.first() {
            // 2. Se molto simile, aggiorna invece di duplicare
            if existing.similarity > 0.95 {
                decision.supersedes.push(existing.id.clone());
                decision.related.push(existing.id.clone());
            } else {
                // 3. Se solo correlata, aggiungi come related
                decision.related.push(existing.id.clone());
            }
        }
        
        // 4. Calcola score
        decision.calculate_score(&self.scoring_config);
        
        // 5. Salva
        let id = decision.id.clone();
        self.decisions.insert(id.clone(), decision);
        
        Ok(id)
    }
}
```

### Consequences

**Positivo:**
- Scoring più preciso con multi-signal
- Memoria più efficiente con hierarchy
- Query semantiche più accurate
- Audit trail completo con temporal links

**Negativo:**
- Complessità implementativa aumentata
- Dipendenza opzionale da david-rag per semantic search
- Overhead di consolidation

**Mitigazione:**
- david-rag è opzionale: fallback a Jaccard se non disponibile
- Consolidation è async e schedulato
- Core layer rimane semplice per uso quotidiano

---

## Autonomia AI-Free

Aether funziona completamente senza AI esterna. L'apprendimento avviene tramite Graph RAG + Memory.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AETHER AUTONOMOUS LEARNING                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Valida file A                                                           │
│       ↓                                                                     │
│  2. Trova violazione in funzione foo()                                      │
│       ↓                                                                     │
│  3. Graph RAG: chi chiama foo()? Dove è definita?                           │
│       ↓                                                                     │
│  4. Attraversa: import → file B → definizione foo()                         │
│       ↓                                                                     │
│  5. Controlla B → capisce contesto completo                                 │
│       ↓                                                                     │
│  6. Se ancora dubbioso → chiede via MCP                                     │
│       ↓                                                                     │
│  7. Agente/Developer risponde                                               │
│       ↓                                                                     │
│  8. Memory salva: pattern + esito → confidence update                       │
│       ↓                                                                     │
│  9. Prossima volta: confidence più alto                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Componenti Autonomi vs AI-Opzionale

| Componente | Autonomo | Richiede AI |
|------------|----------|-------------|
| Syntax validation | ✅ tree-sitter | ❌ |
| Semantic validation | ✅ AST analysis | ❌ |
| Graph RAG traversal | ✅ import parsing | ❌ |
| Memory/Feedback | ✅ TOML storage | ❌ |
| Pattern discovery | ✅ statistics | ❌ |
| Confidence scoring | ✅ formula matematica | ❌ |
| Git integration | ✅ hooks | ❌ |
| suggest_fixes | ⚠️ Regole statiche | ✅ opzionale |
| Semantic search | ⚠️ Jaccard (v2.0) | ✅ david-rag (v3.0) |

### AI come "Dizionario" Opzionale

L'AI esterna può essere usata SOLO per:
- Traduzione messaggi in altre lingue
- Spiegazioni dettagliate di termini tecnici
- Generazione documentazione (opzionale)

**Mai** per validazione core o decisioni.

---

## Dubbioso Mode (Confidence-Based)

Aether non dice "questo è sbagliato", dice "questo è problematico con X% confidence".

### Confidence Thresholds

| Confidence | Comportamento | Output |
|------------|---------------|--------|
| **> 0.8** | Riporta come fatto | `[ERROR] UNWRAP001: ...` |
| **0.5 - 0.8** | Zona grigia, chiede | `[?] UNWRAP001 (65%): Potenziale panic. Confermi?` |
| **0.25 - 0.5** | Warning forte | `[WARNING?] STYLE001 (35%): Non sono sicuro, ma...` |
| **< 0.25** | Ignorato (filtered) | Nessun output — filtrato da Phase 3 |

### Phase 3: Confidence Filtering ✅ IMPLEMENTED (2026-03-25)

Le violazioni con confidence inferiore al 25% vengono automaticamente filtrate prima del report.

**Implementazione:** `executor.rs` — Soglia configurabile nel codice.

### Phase 4: Test File Filtering ✅ IMPLEMENTED (2026-03-25)

In file di test, le regole LOGIC001 (panic!) e LOGIC002 (todo!) vengono automaticamente ignorate.

**Pattern rilevati:** `/tests/`, `\tests\`, `/test_`, `\test_`, `_test.rs`, `_tests.rs`

**Implementazione:** `executor.rs` — Fase 4 del pipeline di filtering.

### Multi-Signal Scoring (v3.0)

Il confidence score ora è composto:

```rust
/// Scoring multi-dimensionale (v3.0)
pub struct MultiSignalScore {
    /// Rilevanza al contesto corrente
    pub relevance: f32,
    
    /// Freshness (decay temporale)
    pub recency: f32,
    
    /// Impatto se ignorata
    pub importance: f32,
    
    /// Score composito
    pub composite: f32,
}

impl MultiSignalScore {
    pub fn new(relevance: f32, age_days: f32, importance: f32) -> Self {
        let recency = (-age_days / 30.0).exp();  // Half-life 30 giorni
        let composite = 0.4 * relevance + 0.3 * recency + 0.3 * importance;
        
        Self { relevance, recency, importance, composite }
    }
    
    pub fn confidence_category(&self) -> ConfidenceCategory {
        match self.composite {
            x if x > 0.8 => ConfidenceCategory::High,
            x if x > 0.5 => ConfidenceCategory::Medium,
            _ => ConfidenceCategory::Low,
        }
    }
}
```

### Esempio Output

```
⚠️ UNWRAP001 (confidence: 0.65)
   File: src/parser.rs:142
   Message: Potenziale panic. In 12 progetti simili, questo 
            pattern ha causato crash 4 volte (33%).
   Signals: relevance=0.70, recency=0.85, importance=0.45
   
   [?] Vuoi ignorare questa violazione? (y/n/a=always)
```

### Configurazione

```toml
# aether.toml
[doubt]
ask_threshold = 0.5    # Sotto questo, chiede conferma
warn_threshold = 0.3   # Sotto questo, warning forte
auto_ignore = 0.1      # Sotto questo, ignora automaticamente (se abilitato)

# v3.0: Multi-signal weights
[doubt.scoring]
alpha = 0.4   # Relevance weight
beta = 0.3    # Recency weight
gamma = 0.3   # Importance weight
recency_half_life_days = 30.0
```

---

## Memoria Formato TOML

La memoria è salvata in TOML, leggibile e modificabile dall'utente.

### Struttura Directory (v3.0)

```
~/.aether/                    # Memoria globale utente
├── core/                     # Core layer (hot, sempre caricato)
│   ├── decisions.toml        # Decisioni attive
│   ├── patterns.toml         # Pattern ad alta frequenza
│   └── violations.toml       # Violazioni non risolte
├── archival/                 # Archival layer (cold, lazy load)
│   ├── 2026-02/              # Organizzato per mese
│   │   ├── decisions.toml.gz
│   │   └── violations.toml.gz
│   └── index.toml            # Indice per lookup rapido
├── learned_patterns.toml     # Pattern imparati cross-project
├── global_whitelist.toml     # Whitelist globale
├── stats.toml                # Statistiche aggregate
└── presets/                  # Bundled presets (sola lettura)
    ├── rust-security.toml
    ├── python-style.toml
    └── ...

project/.aether/              # Memoria locale (override)
├── core/
│   ├── learned_config.toml   # Config imparata per progetto
│   ├── validation_state.toml # Stato validazione file
│   └── decisions.toml        # Decisioni progetto-specifiche
├── archival/                 # Archivio progetto
└── cache/                    # Cache AST (binario, nascosto)
```

### Esempio decision.toml (v3.0)

```toml
# Decisioni attive con temporal support
[[decisions]]
id = "dec-001-abc"
decision_type = "AcceptViolation"
content = "UNWRAP001 accettato: unwrap in test helper è ok"
location = { file = "src/test_utils.rs", line = 42 }

# Temporal fields
created_at = 2026-03-15T10:30:00Z
valid_from = 2026-03-15T10:30:00Z
valid_until = null  # Mai scade
superseded_by = null  # Non ancora obsoleto
supersedes = []  # Non rimpiazza nulla

# Multi-signal score
relevance = 0.85
recency = 0.92  # Calcolato da created_at
importance = 0.3  # Bassa, è un test
score = 0.72  # Composto

tags = ["test", "helper", "accepted"]
author = "user"

[[decisions]]
id = "dec-002-def"
decision_type = "AcceptViolation"
content = "UNWRAP001 accettato: questo unwrap è garantito dal contratto"
location = { file = "src/parser.rs", line = 142 }

# Questa decisione SUPPLANTA una precedente
created_at = 2026-03-18T14:00:00Z
supersedes = ["dec-001-xyz"]  # Rimpiazza decisione vecchia

# La decisione vecchia in archival avrà:
# superseded_by = "dec-002-def"

relevance = 0.70
recency = 0.88
importance = 0.5
score = 0.68
```

### Esempio archival/index.toml

```toml
# Indice archival per lookup rapido
# Non caricare tutto, solo metadata

[entries."dec-001-xyz"]
file = "archival/2026-02/decisions.toml.gz"
superseded_by = "dec-002-def"  # Link a versione corrente
archive_date = 2026-03-01T00:00:00Z

[entries."pattern-005"]
file = "archival/2026-01/patterns.toml.gz"
consolidated_from = ["pat-001", "pat-002", "pat-003"]
occurrence_count = 47
```

---

## Git Integration (Feedback Implicito)

Aether si integra con Git per imparare dal workflow del developer.

### Hooks Configurabili

| Hook | Funzione | Default |
|------|----------|---------|
| `pre-commit` | Valida staged files | ON |
| `post-commit` | Analizza, aggiorna memoria | ON |
| `pre-push` | Validazione completa | OFF |

### Configurazione

```toml
# aether.toml
[git]
pre_commit = true
block_on = ["error"]  # ["error", "warning", "style"]
post_commit = true
pre_push = false
```

### Flusso Feedback Implicito

```
Developer committa codice
    ↓
pre-commit: Aether valida
    ↓
Se errori critici (configurabile) → blocca commit
    ↓
Se proceede senza fixare certe violazioni →
    ↓
post-commit: Aether nota pattern "accettato implicitamente"
    ↓
Dopo N volte → suggerisce whitelist automatica
    ↓
Developer conferma → Memory salva
```

---

## Architettura Tecnica

### Flusso di Validazione Memory-Driven (v3.0)

```rust
// Prima: Validazione statica
fn validate_static(source: &str) -> Vec<Violation> {
    let rules = BUILTIN_RULES;  // Fisse
    apply_rules(source, rules)  // Stesso risultato per tutti
}

// Dopo: Validazione memory-driven v3.0
async fn validate_memory_driven_v3(
    source: &str,
    project: &ProjectContext,
    memory: &HierarchicalMemoryStore,
) -> Vec<Violation> {
    
    // 1. Carica configurazione appresa (core layer)
    let learned = memory.load_core_config(project).await?;
    
    // 2. Calcola multi-signal scores per entries correnti
    memory.update_scores(&project).await?;
    
    // 3. Configura layers dinamicamente
    let mut layers = ValidationPipeline::new();
    
    layers.syntax.set_thresholds(&learned.thresholds.syntax);
    layers.semantic.set_custom_checks(&learned.custom_checks);
    layers.logic.set_learned_patterns(&learned.patterns);
    layers.security.set_whitelist(&learned.security_whitelist);
    layers.style.set_conventions(&learned.conventions);
    
    // 4. Esegui validazione
    let result = layers.validate(source).await;
    
    // 5. Registra per apprendimento futuro
    memory.record_validation(project, &result).await?;
    
    // 6. Trigger consolidation se necessario
    if memory.should_consolidate() {
        memory.consolidate().await?;
    }
    
    result.violations
}
```

### Cosa la Memoria Può Modificare

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MEMORY-DRIVEN CONFIGURATION                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ✅ SOGLIE E PARAMETRI                                                      │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Complessità ciclomatica: default 15 → team usa 25 → Memory aggiorna      │
│  • Lunghezza riga: default 120 → team usa 100 → Memory aggiorna             │
│  • Numero parametri: default 5 → team usa 7 → Memory aggiorna               │
│  • Nested depth: default 4 → team usa 6 → Memory aggiorna                   │
│                                                                             │
│  ✅ REGOLE CUSTOM                                                           │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Pattern ricorrente identificato N volte                                  │
│  • Memory propone: "CUSTOM001: Description"                                 │
│  • Utente conferma → Diventa regola permanente per il progetto              │
│                                                                             │
│  ✅ WHITELIST DINAMICHE                                                     │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Violazione accettata con giustificazione                                 │
│  • Memory crea whitelist contestuale                                        │
│  • Esempio: "UNWRAP001 accettabile in **/test/**"                           │
│                                                                             │
│  ✅ CONVENZIONI STILE                                                       │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Team usa camelCase per public, snake_case per private                    │
│  • Memory impara dal codice esistente                                       │
│  • Applica automaticamente                                                  │
│                                                                             │
│  ✅ ANTI-PATTERN IMPARATI                                                   │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • "Nel nostro codice, X spesso precede bug Y"                              │
│  • Correlazione statistica dal codebase                                     │
│  • Memory genera warning custom                                             │
│                                                                             │
│  🆕 DECISIONI TEMPORALI (v3.0)                                              │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Versioning automatico con superseded_by                                  │
│  • Audit trail completo per compliance                                      │
│  • Scoring con decay temporale                                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Cosa la Memoria NON Può Toccare

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CORE PROTETTO (INVOLABILE)                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ❌ PARSER & AST                                                            │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Tree-sitter grammars                                                     │
│  • Tokenization                                                             │
│  • AST building                                                             │
│  • Motivo: Fondamenta, non negoziabili                                      │
│                                                                             │
│  ❌ VALIDAZIONE BASE DI SINTASSI                                            │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Parentesi non chiuse                                                     │
│  • Keyword malformate                                                       │
│  • Indentazione rotta (se significativa)                                    │
│  • Motivo: Errori oggettivi, non opinabili                                 │
│                                                                             │
│  ❌ SECURITY HARD LIMITS                                                    │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • SQL injection                                                            │
│  • Hardcoded credentials                                                    │
│  • Unsafe deserialization                                                   │
│  • Command injection                                                        │
│  • Motivo: Questi NON possono essere whitelistati                           │
│                                                                             │
│  ❌ PIPELINE EXECUTION                                                      │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Layer orchestration                                                      │
│  • Error reporting                                                          │
│  • Output formatting                                                        │
│  • Motivo: Infrastruttura, non business logic                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Feedback Loop

### Flusso Completo

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FEEDBACK LOOP (v3.0)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. VALIDAZIONE                                                             │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │  Memory.load_core_config(project)                               │    │
│     │           ↓                                                       │    │
│     │  Memory.update_scores()  // Multi-signal scoring                 │    │
│     │           ↓                                                       │    │
│     │  Pipeline.apply_learned_config(config)                           │    │
│     │           ↓                                                       │    │
│     │  Pipeline.validate(source) → Result                              │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  2. INTERAZIONE UTENTE                                                      │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │  Utente vede violations (con multi-signal confidence)           │    │
│     │           ↓                                                       │    │
│     │  Per ogni violation:                                             │    │
│     │    • FIX → Corregge codice                                       │    │
│     │    • ACCEPT → "È ok perché..."                                   │    │
│     │    • IGNORE → Niente                                             │    │
│     │    • SUPERSede → "Questa decisione rimpiazza una vecchia"        │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  3. APPRENDIMENTO                                                           │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │  FIX → Memory.record_correction(violation, fix)                  │    │
│     │  ACCEPT → Memory.record_accepted(violation, reason)              │    │
│     │  SUPERSEDE → Memory.supersede_decision(old_id, new_decision)     │    │
│     │           ↓                                                       │    │
│     │  Pattern Analysis:                                               │    │
│     │    • Stesso tipo accettato 5 volte → Whitelist candidate         │    │
│     │    • Pattern ricorrente in fix → Custom rule candidate           │    │
│     │    • Stile coerente → Convention candidate                       │    │
│     │    • Decisione vecchia + nuova → Supersession link              │    │
│     │           ↓                                                       │    │
│     │  Config aggiornata per prossima validazione                      │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  4. CONSOLIDATION (v3.0)                                                    │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │  Periodico (ogni 7 giorni):                                      │    │
│     │    • Scan Core layer per entries con score < 0.3                 │    │
│     │    • Move to Archival con link preservation                      │    │
│     │    • Consolidate pattern occorrenze (10+ → sintesi)              │    │
│     │    • Update archival index                                       │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  5. EVOLUZIONE                                                              │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │  Con l'uso, Aether diventa:                                      │    │
│     │    • Meno falsi positivi                                         │    │
│     │    • Più rilevante per il progetto                               │    │
│     │    • Allineato con convenzioni del team                          │    │
│     │    • Capace di rilevare pattern specifici del dominio            │    │
│     │    • Audit trail completo per compliance                         │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Valore per Enterprise

### Perché Ogni Istanza Diventa Unica

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ENTERPRISE VALUE PROPOSITION (v3.0)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PROJECT A (Game Engine)                                                    │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Convenzioni: camelCase pubblico, snake_case privato                      │
│  • Complessità tollerata: 25 (sistemi complessi)                            │
│  • Whitelist: unwrap in hot paths, unsafe in SIMD                           │
│  • Custom rules: "Check alignment in SIMD structs"                          │
│  • Anti-pattern appresi: "Missing padding in struct → crash"                │
│  • Decisioni temporali: 47 superseded, 23 attive                            │
│                                                                             │
│  PROJECT B (Web API)                                                        │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Convenzioni: snake_case ovunque                                          │
│  • Complessità tollerata: 15 (mantenibilità)                                │
│  • Whitelist: nessuna, strict mode                                          │
│  • Custom rules: "Check pagination params in endpoints"                     │
│  • Anti-pattern appresi: "N+1 query in /users endpoint"                     │
│  • Decisioni temporali: 12 superseded, 8 attive                             │
│                                                                             │
│  PROJECT C (Embedded)                                                       │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Convenzioni: UPPERCASE per macro                                         │
│  • Complessità tollerata: 10 (risorse limitate)                             │
│  • Whitelist: unsafe in drivers                                             │
│  • Custom rules: "Check memory allocation in ISR"                           │
│  • Anti-pattern appresi: "Dynamic allocation in interrupt handler"          │
│  • Decisioni temporali: 5 superseded, 15 attive                             │
│                                                                             │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Conclusione: Non puoi copiare l'Aether di un altro progetto.               │
│               La memoria è legata al codebase, il valore è unico.           │
│               + Audit trail temporale per compliance enterprise.            │
│  ─────────────────────────────────────────────────────────────────────────  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Retention e Valore Crescente

| Tempo | Falsi Positivi | Regole Custom | Decisioni Attive | Valore |
|-------|----------------|---------------|-------------------|--------|
| Giorno 1 | 30% | 0 | 0 | Base |
| Mese 1 | 20% | 3 | 5-10 | Medio |
| Mese 3 | 12% | 8 | 15-25 | Alto |
| Mese 6 | 5% | 15 | 30-50 | Molto Alto |
| Anno 1 | 2% | 25 | 50-100 | Inestimabile |

**Il costo di switch diventa proibitivo:** Perdere la memoria = ricominciare da zero.

---

## Implementazione v3.0

### Phase 1: Temporal Decision Log (1 settimana)

- [ ] `DecisionNodeV3` struct con temporal fields
- [ ] `superseded_by` / `supersedes` link handling
- [ ] Migration da v2.0 decision format
- [ ] Test: supersession chain, audit trail

### Phase 2: Multi-Signal Scoring (1 settimana)

- [ ] `MultiSignalScore` struct
- [ ] Decay function per recency
- [ ] Config per alpha/beta/gamma weights
- [ ] Update Dubbioso Mode per usare composite score

### Phase 3: Memory Hierarchy (2 settimane)

- [ ] `HierarchicalMemoryStore` struct
- [ ] Core/Archival layer separation
- [ ] Consolidation pipeline
- [ ] Archival index for fast lookup
- [ ] Lazy loading from archival

### Phase 4: Vector-Backed Semantic (1 settimana)

- [ ] Integrazione opzionale con david-rag
- [ ] `SemanticRetriever` client
- [ ] Fallback a Jaccard se RAG non disponibile
- [ ] Cache per embeddings frequenti

### Phase 5: Deduplication (1 settimana)

- [ ] Similarity check prima di aggiungere decisioni
- [ ] Auto-merge per decisioni identiche
- [ ] Link related per decisioni simili

---

## CLI Commands (v3.0)

```bash
# Mostra configurazione appresa
aether memory config show --project myproject

# Statistiche di apprendimento
aether memory stats --project myproject

# v3.0: Mostra memory hierarchy stats
aether memory hierarchy --project myproject

# v3.0: Forza consolidation
aether memory consolidate --project myproject

# v3.0: Mostra decisioni temporali
aether memory decisions --project myproject --show-superseded

# Proposte di regole pendenti
aether memory rules pending

# Approva una regola proposta
aether memory rules approve CUSTOM001

# Rifiuta una regola proposta
aether memory rules reject CUSTOM002

# v3.0: Supersede una decisione
aether memory supersede DEC-001 "Nuova motivazione..." --project myproject

# Reset configurazione (ma non la memoria)
aether memory config reset --project myproject

# Esporta configurazione per backup (TOML)
aether memory config export --project myproject -o aether-config.toml

# Importa configurazione
aether memory config import aether-config.toml --project myproject

# Dubbioso mode: configura soglie
aether doubt set --ask 0.5 --warn 0.3

# v3.0: Configura multi-signal weights
aether doubt weights --alpha 0.4 --beta 0.3 --gamma 0.3

# Git hooks: installa
aether git hooks install

# Git hooks: configura blocking
aether git set --block-on error,warning

# Presets: lista disponibili
aether presets list

# Presets: applica un preset
aether presets apply rust-security --project myproject
```

---

## Summary

| Aspetto | Tradizionale | v2.0 | v3.0 (Proposed) |
|---------|--------------|------|-----------------|
| Regole | Fisse, uguali per tutti | Dinamiche | Dinamiche + temporali |
| Falsi positivi | Costanti | Diminuiscono | Diminuiscono + dedup |
| Valore | Costante | Crescente | Crescente + audit |
| Switch cost | Basso | Altissimo | Proibitivo |
| AI required | Spesso | Mai | Mai (RAG opzionale) |
| Confidence | Assente | Singolo score | Multi-signal |
| Formato memoria | N/A | TOML | TOML + hierarchy |
| Temporal support | N/A | ❌ | ✅ Supersession |
| Git integration | No | Hooks | Hooks |
| Enterprise appeal | Basso | Molto alto | Enterprise-ready |

**Questo è il vero differentiator di Aether.**
