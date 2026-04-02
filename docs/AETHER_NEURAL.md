# Synward Neural Native — Design Document

**Versione**: 1.0
**Data**: 2026-03-30
**Stato**: Phase 1-2 Implementate (Architetture + Forward Pass ✅)
**ADR**: [ADR-002-neural-native.md](ADR/ADR-002-neural-native.md)

---

## 1. Visione

Synward evolve da validatore rule-based a **sistema neuro-symbolic nativo**. Le regole e i contratti rimangono il layer simbolico, ma una rete neurale custom fornisce reasoning, pattern discovery, predizione drift e suggerimenti fix — tutto senza dipendere da LLM esterni (locali o cloud).

**Principio**: Il layer simbolico garantisce correttezza deterministica. Il layer neurale fornisce intelligenza adattiva. Insieme, sono più forti di entrambi singolarmente.

---

## 2. Architettura

```
┌──────────────────────────────────────────────────────────────┐
│                    SYNWARD NEURAL NATIVE                       │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              LAYER 5: NEURAL ORCHESTRATOR                │ │
│  │  Coordina le reti, fusiona segnali, decide azioni        │ │
│  │  • Ensemble voting tra reti                              │ │
│  │  • Confidence routing (neurale vs simbolico)             │ │
│  │  • Auto-training trigger                                 │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐ │
│  │   RETE A:    │  │   RETE B:    │  │     RETE C:        │ │
│  │  CODE REASON │  │  PATTERN MEM │  │   DRIFT PREDICT    │ │
│  │              │  │              │  │                    │ │
│  │  GNN su CPG  │  │  TreeFFN +   │  │  Temporal GNN +    │ │
│  │  (GGNN/GAT)  │  │  Hopfield    │  │  Sequence Model    │ │
│  │              │  │              │  │                    │ │
│  │  Input: CPG  │  │  Input: Cod. │  │  Input: Snapshots  │ │
│  │  Out: Class. │  │  Out: Embed  │  │  Out: Predizione   │ │
│  │  + Spiegaz.  │  │  + Similarità│  │  + Gravità         │ │
│  │  ~8M params  │  │  ~5M params  │  │  ~5M params       │ │
│  └──────────────┘  └──────────────┘  └────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              LAYER 2: FEATURE PIPELINE                    │ │
│  │  Codice → Tree-sitter → AST → CPG → Feature Vectors      │ │
│  │  Riusa CodeGraph esistente + estensione CPG               │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              LAYER 1: SYMBOLIC CORE (esistente)           │ │
│  │  Contratti • Regole • Compliance Engine • Certification   │ │
│  │  Validation Layers 1-3 (Syntax, Semantic, Logic)          │ │
│  │  CodeGraph • DecisionLog • PatternLearner                 │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│                    TRAINING PIPELINE                           │
│  NexusTrain (Python/PyTorch) esteso per architetture custom   │
│  Training → Export ONNX → Inference Burn (Rust)              │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. Le Tre Reti

### 3.1 Rete A — Code Reasoner (GNN su CPG)

**Scopo**: Capisce la struttura del codice e classifica problemi, genera spiegazioni, suggerisce fix.

| Aspetto | Dettaglio |
|---|---|
| **Architettura** | GGNN (Gated Graph Neural Network) o GAT (Graph Attention Network) |
| **Input** | Code Property Graph: AST + Control Flow Graph + Data Flow Graph |
| **Output** | Classificazione (tipo errore), confidence score, feature vector per spiegazione |
| **Parametri** | ~8M |
| **Training** | Supervisionato su risultati validazione Synward + dataset esterni |

**Pipeline**:
```
Codice sorgente
    → Tree-sitter (parsing multi-linguaggio, già in Synward)
    → AST
    → CPG Builder (nuovo: combina AST + CFG + DFG)
    → Node features (tipo nodo, tipo dato, scope depth, complessità)
    → Edge features (control flow, data flow, containment)
    → GNN (T message-passing steps)
    → Graph-level readout
    → Task heads: classificazione, spiegazione, fix suggestion
```

**Head di output**:
- `ClassificationHead`: Multi-label (tipo di problema) — softmax su N categorie
- `ExplanationHead`: Genera embedding di spiegazione → confrontato con template per produrre testo
- `FixHead`: Genera feature vector per la regione problematica → confrontato con fix noti

### 3.2 Rete B — Pattern Memory (TreeFFN + Hopfield)

**Scopo**: Memorizza pattern ricorrenti, riconosce similarità semantica tra frammenti di codice, recupera esperienze passate.

| Aspetto | Dettaglio |
|---|---|
| **Architettura** | TreeFFN (encoder-decoder attention-free) + Modern Hopfield Network |
| **Input** | Feature vector da Rete A + pattern storici |
| **Output** | Embedding di similarità, pattern match score, retrieved experiences |
| **Parametri** | ~5M (3M TreeFFN + 2M Hopfield) |
| **Training** | Self-supervised + contrastive learning su dataset di validazioni |

**TreeFFN** (ispirato a TreeGPT, 3.16M params):
- Encoder: propagazione L→R tra nodi adiacenti
- Decoder: propagazione R→L
- Attention-free: efficiente, lineare nella sequenza
- Perfetto per reasoning strutturato su AST

**Hopfield Network** (associative memory):
- Memorizza pattern di codice come attractor states
- Retrieval per similarità (query = codice attuale, result = esperienze simili)
- Capacità di storage esponenziale (Modern Hopfield)
- Update incrementale: ogni validazione aggiorna la memoria

### 3.3 Rete C — Drift Predictor (Temporal GNN + Sequence Model)

**Scopo**: Prevede come il codice evolverà, rileva drift architetturale prima che accada, segnala trend negativi.

| Aspetto | Dettaglio |
|---|---|
| **Architettura** | Temporal GNN + piccolo sequence model (Mamba-style SSM o LSTM) |
| **Input** | Sequenza temporale di CPG snapshots (da DriftSnapshots esistente) |
| **Output** | Predizione drift, gravità, timeframe, azione suggerita |
| **Parametri** | ~5M |
| **Training** | Supervisionato su git history con etichette di drift |

**Pipeline**:
```
Git history (commit sequence)
    → Per commit: CPG extraction
    → Temporal alignment
    → Temporal GNN (evoluzione della struttura nel tempo)
    → Sequence model (trend prediction)
    → Output: drift probability, severity, affected components
```

---

## 4. Feature Pipeline

### 4.1 Code Property Graph (CPG)

Estensione del CodeGraph esistente. Oltre all'AST, combina:

```
CPG = AST + CFG + DFG

AST  → Struttura sintattica (già in Synward via tree-sitter)
CFG  → Control flow (branch, loop, try/catch)
DFG  → Data flow (def-use chains, variabili, dipendenze)
```

**Node features** (per ogni nodo del CPG):
- `node_type`: tipo AST (function, if, assignment, call, ...)
- `data_type`: tipo dato (int, string, Option, Result, ...)
- `scope_depth`: profondità di nesting
- `complexity`: cyclomatic complexity locale
- `violations_count`: violazioni Synward storiche in questa posizione
- `is_entry_point`: se è un punto di ingresso (main, public API, handler)

**Edge features**:
- `AST_CHILD`: relazione genitore-figlio
- `CFG_FLOW`: flusso di controllo (sequential, branch, merge)
- `DFG_FLOW`: flusso dati (def→use)
- `CALL`: chiamata di funzione

### 4.2 Generazione Training Data

Synward genera automaticamente training data ad ogni validazione:

```toml
# .synward/training_data/val_20260330_001.toml
[source]
file = "src/renderer.rs"
commit = "abc1234"

[cpg_snapshot]
# Feature vector serializzato
nodes = 342
edges = 891
max_depth = 7

[validation_result]
rule_id = "RUST_001"
severity = "error"
category = "error_handling"
passed = false

[human_feedback]
accepted = true
reason = "Vero positivo, il Result non è gestito"

[neural_features]
# Output della pipeline feature (per training supervisionato)
classification_label = "unhandled_error"
explanation_embedding = [0.12, -0.34, ...]  # Da matching template
fix_suggestion_embedding = [0.56, 0.78, ...]
```

---

## 5. Training Pipeline

### 5.1 NexusTrain Extended

NexusTrain diventa il training hub per tutte le reti di Synward.

**Estensioni necessarie**:

```
nexustrain/
├── src/nexustrain/
│   ├── backends/              # Esistente (Blackwell, Triton, PyTorch)
│   ├── core/
│   │   ├── trainer.py         # Esistente — estendere per custom architectures
│   │   ├── memory_controller  # Esistente — riusare
│   │   ├── device_manager     # Esistente — riusare
│   │   ├── architectures/     # NUOVO
│   │   │   ├── __init__.py
│   │   │   ├── gnn.py         #    GGNN/GAT per Code Reasoner
│   │   │   ├── treeffn.py     #    TreeFFN per Pattern Memory
│   │   │   ├── hopfield.py    #    Modern Hopfield Network
│   │   │   ├── temporal_gnn.py #   Temporal GNN per Drift Predictor
│   │   │   └── ensemble.py    #    Orchestratore ensemble
│   │   └── curriculum.py      # Esistente — riusare per curriculum learning
│   ├── data/                  # NUOVO
│   │   ├── cpg_dataset.py     #    Dataset loader per CPG data
│   │   ├── synward_loader.py   #    Loader da .synward/training_data/
│   │   └── code_corpus.py     #    Loader per corpus esterni (CodeSearchNet, etc.)
│   ├── export/
│   │   ├── gguf_exporter.py   # Esistente
│   │   └── onnx_exporter.py   # NUOVO — export per Burn inference
│   └── config/
│       └── generate_template  # Esistente — estendere per config reti custom
├── configs/
│   ├── synward_gnn.yaml        # NUOVO — config Code Reasoner
│   ├── synward_pattern.yaml    # NUOVO — config Pattern Memory
│   └── synward_drift.yaml      # NUOVO — config Drift Predictor
```

### 5.2 Config di Esempio (Code Reasoner)

```yaml
# configs/synward_gnn.yaml
model:
  type: "gnn"                    # Nuovo: tipo architettura
  architecture: "gat"            # GGNN o GAT
  hidden_dim: 256
  num_layers: 6
  num_heads: 4                   # Per GAT
  dropout: 0.1
  params_estimate: "~8M"

training:
  task: "multitask_classification"
  epochs: 50
  batch_size: 32
  learning_rate: 1.0e-3
  optimizer: "adamw"
  weight_decay: 1.0e-4
  scheduler: "cosine"
  gradient_clip: 1.0

  # Curriculum learning (supporto già in NexusTrain)
  curriculum:
    enabled: true
    strategy: "difficulty"       # Easy → Hard
    difficulty_metric: "ast_depth"

data:
  sources:
    - type: "synward_validation"  # Da .synward/training_data/
      path: ".synward/training_data/"
    - type: "code_corpus"        # Dataset esterni
      name: "CodeSearchNet"
      languages: ["rust", "python", "typescript"]

  preprocessing:
    cpg: true                    # Genera Code Property Graph
    max_nodes: 500               # Truncate grafi grandi
    max_edges: 2000

export:
  format: "onnx"                 # Per Burn inference
  optimize: true
  quantize: "fp16"               # Quantizzazione per inference veloce

memory:
  enable_cpu_offload: true       # NexusTrain memory controller
  vram_threshold: 0.85
  gradient_checkpointing: true
```

### 5.3 Auto-Training Continuo

Il sistema impara continuamente dalle validazioni:

```
Validazione Synward
    → Risultato + feedback utente
    → Scritto in .synward/training_data/
    → Quando accumula N nuovi esempi (configurable)
    → Trigger auto-training incrementale
    → Nuovo modello validato su test set
    → Se migliore → deploy (Burn inference update)
    → Se peggiore → rollback (Temporal Memory gestisce versioni)
```

**Temporal Memory** (già esistente in Synward) traccia le versioni dei modelli:
- Ogni modello ha version, metrics, timestamp
- Supersession mechanism per modelli deprecati
- Rollback automatico se il nuovo modello peggiora su regression tests

---

## 6. Runtime (Burn in Synward)

### 6.1 Crate Structure

```
crates/synward-neural/          # NUOVO crate
├── Cargo.toml
├── src/
│   ├── lib.rs                  # API pubblica
│   ├── inference.rs            # Burn inference engine
│   ├── models/
│   │   ├── mod.rs
│   │   ├── code_reasoner.rs    # GNN inference
│   │   ├── pattern_memory.rs   # TreeFFN + Hopfield inference
│   │   └── drift_predictor.rs  # Temporal GNN inference
│   ├── features/
│   │   ├── mod.rs
│   │   ├── cpg.rs              # CPG builder (da AST esistente)
│   │   └── vectorizer.rs       # Feature extraction
│   ├── orchestrator.rs         # Ensemble + routing
│   └── model_registry.rs       # Versioning modelli
```

### 6.2 API Pubblica

```rust
/// Synward Neural — Layer neurale per Synward
pub struct SynwardNeural {
    code_reasoner: CodeReasoner,
    pattern_memory: PatternMemory,
    drift_predictor: DriftPredictor,
    orchestrator: NeuralOrchestrator,
    registry: ModelRegistry,
}

impl SynwardNeural {
    /// Carica il sistema neurale da directory modelli
    pub fn load(models_dir: &Path) -> Result<Self>;

    /// Analisi completa di un file sorgente
    pub fn analyze(&self, source: &str, language: &str) -> NeuralResult {
        // 1. CPG extraction
        // 2. Rete A: classificazione + spiegazione
        // 3. Rete B: pattern matching + esperienze simili
        // 4. Rete C: drift prediction (se dati temporali disponibili)
        // 5. Orchestrator: fusiona segnali, produce risultato
    }

    /// Query diretta al pattern memory
    pub fn recall_similar(&self, source: &str, k: usize) -> Vec<PatternMatch>;

    /// Predizione drift per un progetto
    pub fn predict_drift(&self, snapshots: &[DriftSnapshot]) -> DriftPrediction;

    /// Aggiorna il pattern memory con nuova esperienza
    pub fn store_experience(&mut self, experience: ValidationExperience);

    /// Stato del sistema neurale
    pub fn status(&self) -> NeuralStatus;
}

/// Risultato dell'analisi neurale
pub struct NeuralResult {
    pub classifications: Vec<Classification>,    // Da Rete A
    pub explanation: Option<String>,              // Da Rete A + B
    pub similar_patterns: Vec<PatternMatch>,      // Da Rete B
    pub fix_suggestions: Vec<FixSuggestion>,      // Da Rete A + B
    pub drift_warning: Option<DriftPrediction>,   // Da Rete C
    pub confidence: f32,                          // Orchestrator
    pub should_defer_to_symbolic: bool,           // Se neural è incerto
}
```

### 6.3 Integration con Synward esistente

```rust
// In synward-intelligence/src/lib.rs — estensione

impl SynwardIntelligence {
    /// Nuovo: accesso al layer neurale
    pub fn neural(&self) -> Option<&SynwardNeural>;

    /// Estensione: validazione ibrida (simbolico + neurale)
    pub fn validate_hybrid(
        &mut self,
        source: &str,
        language: &str,
    ) -> HybridValidationResult {
        // 1. Validazione simbolica (esistente, deterministica)
        let symbolic = self.validate(source, language);

        // 2. Analisi neurale (se disponibile)
        let neural = self.neural().map(|n| n.analyze(source, language));

        // 3. Fusion
        match neural {
            Some(n) => HybridValidationResult::merge(symbolic, n),
            None => HybridValidationResult::symbolic_only(symbolic),
        }
    }
}
```

### 6.4 Confidence Routing

Il sistema decide quando usare il neurale e quando affidarsi al simbolico:

```
Confidence > 0.9  →  Risultato neurale, alta fiducia
Confidence 0.7-0.9 → Risultato neurale + verifica simbolica
Confidence 0.5-0.7 → Risultato simbolico + suggerimento neurale
Confidence < 0.5  → Solo simbolico, neurale incerto (Dubbioso Mode)
```

---

## 7. Fasi di Implementazione

### Fase 1 — Fondamenta (Settimana 1-3)

**Obiettivo**: GNN su CPG funzionante, prima classificazione.

| Task | Dettaglio |
|---|---|
| Estendere CodeGraph a CPG | Aggiungere CFG + DFG al code graph esistente |
| Feature extraction | Node features + edge features da CPG |
| Dataset iniziale | Generare training data dalle validazioni Synward esistenti + corpus esterno |
| GNN in NexusTrain | Implementare GGNN/GAT in `architectures/gnn.py` |
| Training iniziale | Addestrare su classification task (violation detection) |
| ONNX export | Implementare `onnx_exporter.py` in NexusTrain |
| Burn inference base | `synward-neural` crate con caricamento modello ONNX |
| Test end-to-end | Codice → CPG → GNN → classificazione → risultato |

**Deliverable**: Synward rileva problemi con il GNN, oltre alle regole.

### Fase 2 — Pattern Memory (Settimana 4-7)

**Obiettivo**: Il sistema ricorda e riconosce pattern.

| Task | Dettaglio |
|---|---|
| TreeFFN in NexusTrain | Implementare `architectures/treeffn.py` |
| Hopfield Network | Implementare `architectures/hopfield.py` |
| Pattern dataset | Dataset contrastivo per similarità semantica |
| Training | Self-supervised + contrastive |
| Pattern memory in Burn | Inference per retrieval |
| Store/recall API | `store_experience()`, `recall_similar()` |
| Integration con DecisionLog | Esperienze neurali collegate a decisioni simboliche |

**Deliverable**: Synward riconosce pattern ricorrenti e recupera esperienze simili.

### Fase 3 — Drift Prediction (Settimana 8-10)

**Obiettivo**: Prevede drift prima che accada.

| Task | Dettaglio |
|---|---|
| Temporal GNN in NexusTrain | Implementare `architectures/temporal_gnn.py` |
| Git history dataset | Estrare sequenze temporali di CPG |
| Training | Supervisionato su drift etichettato |
| DriftPredictor in Burn | Inference temporale |
| Integration con DriftSnapshots | Utilizzare dati esistenti |
| Alert system | Notifiche drift con gravità e timeframe |

**Deliverable**: Synward prevede drift e segnala trend negativi.

### Fase 4 — Orchestrator + Spiegazioni (Settimana 11-14)

**Obiettivo**: Sistema unificato che fusiona segnali e produce spiegazioni.

| Task | Dettaglio |
|---|---|
| NeuralOrchestrator | Ensemble voting + confidence routing |
| ExplanationHead | Generazione spiegazioni da embedding |
| FixHead | Suggerimenti fix da feature matching |
| Auto-training pipeline | Training incrementale automatico |
| Model registry | Versioning, rollback, A/B testing |
| Dubbioso Mode neurale | Integrazione con Dubbioso Mode esistente |

**Deliverable**: Synward fornisce spiegazioni, suggerisce fix, migliora autonomamente.

### Fase 5 — Neuro-Symbolic Unification (Settimana 15-20)

**Obiettivo**: I layer simbolico e neurale diventano un sistema unico.

| Task | Dettaglio |
|---|---|
| Contratti neuro-symbolic | I contratti Synward informano direttamente le reti |
| Neural-guided validation | La rete guida quali regole attivare |
| Symbolic-explained neural | Le regole spiegano le decisioni neurali |
| Unified API | Singola interfaccia per validazione ibrida |
| Performance optimization | Quantizzazione, caching, batch inference |
| Documentazione completa | Aggiornamento di tutti i documenti Synward |

**Deliverable**: Synward è un sistema neuro-symbolic nativo, integrato e autonomo.

---

## 8. Dipendenze

### Rust (Synward)

```toml
# crates/synward-neural/Cargo.toml
[dependencies]
burn = { version = "0.17", features = ["wgpu", "onnx"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tracing = "0.1"

# Riutilizzo da Synward esistente
tree-sitter = { version = "0.24", optional = true }
```

### Python (NexusTrain)

```python
# Dipendenze nuove per architetture custom
torch_geometric >= 2.6    # GNN support
torch_sparse              # Sparse tensor operations
hopfield-networks         # Modern Hopfield
onnx >= 1.16              # Export ONNX
onnxruntime               # Validazione export
```

### Dataset Esterni

| Dataset | Uso | Dimensione |
|---|---|---|
| CodeSearchNet | Pre-training code representation | 2M snippet |
| Big-Vul | Vulnerability detection patterns | 188K funzioni |
| ManySStuBs4J | Simple bug patterns | 18K bug-fix |
| Synward DecisionLog | Domain-specific validation data | Cresce con l'uso |

---

## 9. Metriche di Successo

### Per Rete

| Rete | Metrica | Target |
|---|---|---|
| Code Reasoner (A) | Accuracy classificazione | > 85% |
| Code Reasoner (A) | Precision per categoria | > 80% |
| Pattern Memory (B) | Recall @ 10 | > 70% |
| Pattern Memory (B) | Similarità corretta | > 75% |
| Drift Predictor (C) | AUC-ROC | > 0.80 |
| Drift Predictor (C) | Predizione entro 5 commit | > 60% |

### Sistema Overall

| Metrica | Target |
|---|---|
| Inference latency per file | < 50ms |
| Memory overhead | < 200MB |
| Miglioramento vs solo simbolico | > 20% su casi ambigui |
| Auto-training miglioramento | > 5% dopo 100 validazioni |
| False positive rate | < 10% |

---

## 10. Rischi e Mitigazioni

| Rischio | Probabilità | Impatto | Mitigazione |
|---|---|---|---|
| Training data insufficiente | Media | Alto | Dataset esterni + synthetic augmentation |
| Overfitting su progetti specifici | Media | Medio | Regularization + cross-project validation |
| Performance inference lenta | Bassa | Medio | Quantizzazione + caching + batch |
| Burn immaturo per GNN | Media | Alto | Fallback ONNX Runtime; Burn evolve rapidamente |
| NexusTrain estensione complessa | Bassa | Medio | Architetture come moduli indipendenti |

---

## 11. Relazione con Documenti Esistenti

| Documento | Relazione |
|---|---|
| [SYNWARD_INTELLIGENCE.md](SYNWARD_INTELLIGENCE.md) | Layer 3→5 diventano neuro-symbolic |
| [MEMORY_DRIVEN_CORE.md](MEMORY_DRIVEN_CORE.md) | Memory estesa con neural pattern memory |
| [PATTERN_LEARNER.md](PATTERN_LEARNER.md) | PatternLearner diventa feature extractor per reti |
| [DUBBIOSO_MODE.md](DUBBIOSO_MODE.md) | Dubbioso Mode usa confidence neurale |
| [FEATURE_ROADMAP.md](FEATURE_ROADMAP.md) | Neural aggiunto come fase majeure |
