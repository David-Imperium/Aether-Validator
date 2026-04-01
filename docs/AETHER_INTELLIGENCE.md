# Aether Intelligence — Autonomous Validation

**Version:** 2.0 (Autonomous Design)
**Status:** Phase 15 In Progress
**Last Updated:** 2026-03-18
**Author:** Droid + David
**See Also:** [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md), [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md)

---

## Executive Summary

Aether Intelligence è un sistema **autonomo** con memoria, apprendimento e capacità di scoprire nuovi pattern. L'obiettivo è creare un "guardiano intelligente" che capisce non solo *cosa* è sbagliato, ma *perché* è sbagliato nel contesto del progetto.

**Key Principles (v2.0):**
- **AI-Free Core**: Nessuna AI esterna richiesta per validazione. L'AI è opzionale, solo come "dizionario"
- **Graph RAG Autonomo**: Attraversa progetti, capisce dipendenze, impara pattern senza AI
- **Dubbioso Mode**: Confidence-based validation, chiede quando incerto via MCP
- **TOML Format**: Memoria leggibile e modificabile dall'utente

**Visione:** Da pattern matcher a sistema autonomo specializzato in validazione codice.

---

## Fondamento: Memory-Driven Core

> **Architettura completa:** Vedi [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md)

Prima di entrare nei dettagli dei layers AI, è fondamentale capire il **differentiator principale** di Aether:

**La memoria non "consulta" il core — lo CONFIGURA dinamicamente.**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MEMORY-DRIVEN CORE                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  TRADIZIONALE (Altri validatori)                                            │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Regole → Fisse → Stesso output per tutti                                   │
│  Problema: Falsi positivi costanti, non rispetta convenzioni team           │
│                                                                             │
│  AETHER (Memory-Driven)                                                     │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Memory → LearnedConfig → Layers dinamici                                   │
│  Vantaggio: Regole uniche per progetto, valore che cresce con l'uso        │
│                                                                             │
│  Cosa la memoria modifica:                                                  │
│  ✅ Soglie (complessità, lunghezza riga, parametri)                         │
│  ✅ Regole custom (generate da Pattern Discovery)                           │
│  ✅ Whitelist (violazioni accettate con motivo)                             │
│  ✅ Convenzioni stile (imparate dal codice esistente)                       │
│                                                                             │
│  Cosa la memoria NON tocca:                                                 │
│  ❌ Parser/AST, sintassi base, security hard limits, pipeline execution     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Flusso Memory-Driven

```rust
// Ogni validazione carica configurazione appresa
async fn validate(source: &str, project: &ProjectContext) -> Vec<Violation> {
    // 1. Carica configurazione dalla memoria
    let config = memory.load_config(project).await?;

    // 2. Configura layers dinamicamente
    layers.syntax.set_thresholds(&config.thresholds);
    layers.security.set_whitelist(&config.security_whitelist);
    layers.style.set_conventions(&config.conventions);
    layers.logic.add_custom_rules(&config.custom_rules);

    // 3. Valida
    let result = layers.validate(source).await;

    // 4. Registra per apprendimento futuro
    memory.record_validation(project, &result).await;

    result.violations
}
```

### Valore Enterprise

| Tempo | Falsi Positivi | Regole Custom | Switch Cost |
|-------|----------------|---------------|-------------|
| Giorno 1 | 30% | 0 | Basso |
| Mese 3 | 12% | 8 | Medio |
| Anno 1 | 2% | 25 | **Altissimo** |

**Retention:** Perdere la memoria = ricominciare da zero. Il valore cresce con l'uso.

---

## Il Problema: Code Drift

### Definizione

**Code Drift** (o Entropy Acceleration) è il fenomeno dove il codice generato da AI si degrada nel lungo periodo, diventando incoerente, fragil e difficile da mantenere.

### Cause Principali

| Causa | Descrizione | Impatto |
|-------|-------------|---------|
| **Context Window Limit** | L'AI vede solo N righe, perde visione d'insieme | Incoerenza architetturale |
| **Inconsistency Cascade** | Piccole scelte diverse ad ogni generazione | Stili misti, pattern conflittuali |
| **Missing Intent** | L'AI non capisce *perché* il codice esiste | Refactoring distruttivo |
| **No Architectural Memory** | Ogni generazione è "tabula rasa" | Decisioni dimenticate |
| **Confirmation Bias** | L'AI tende a generare codice simile a sé stesso | Pattern monotoni, mancanza innovazione |
| **Temporal Blindness** | L'AI non vede l'evoluzione del codice | Ripete errori passati |

### Sintomi nel Codice

```python
# Esempio di Code Drift dopo 10 iterazioni AI

# Iterazione 1: Stile coerente
def get_user(user_id: int) -> User:
    return db.query(User).filter_by(id=user_id).first()

# Iterazione 5: Stile diverso
async def fetchUser(userId):
    return await database.users.find_one({"id": userId})

# Iterazione 10: Caos completo
def getUser(user_id: int = None, userId: str = None, id=None):
    # TODO: fix this later
    if user_id:
        return db.query(User).filter(User.id == user_id).first()
    elif userId:
        return database.users.find_one({"id": int(userId)})
    else:
        return None  # ???
```

### Pattern di Degrado Identificati

1. **Parameter Drift**: Segnature di funzioni che cambiano nel tempo
2. **Type Drift**: Tipi che diventano progressivamente più laschi (`Any`, `Optional`)
3. **Naming Drift**: Convenzioni di naming che cambiano (snake_case → camelCase → mix)
4. **Error Drift**: Gestione errori che degrada (try/catch → silent failures)
5. **Import Drift**: Dipendenze duplicate o obsolete

---

## Architettura Aether Intelligence

### Visione a 5 Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      AETHER INTELLIGENCE ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    LAYER 5: DRIFT DETECTION (Temporal)                 │ │
│  │    Analizza evoluzione codice nel tempo, rileva degrado                │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    LAYER 4: INTENT INFERENCE (LLM-lite)                │ │
│  │    Capisce "perché" il codice esiste, non solo "cosa" fa               │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    LAYER 3: PATTERN DISCOVERY (ML)                     │ │
│  │    Scopre nuovi anti-pattern dal codice che vede                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    LAYER 2: SEMANTIC MEMORY (RAG)                      │ │
│  │    Memoria persistente: errori passati, correzioni, contesto          │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                    LAYER 1: STATIC ANALYSIS (Core)                     │ │
│  │    Validazione rule-based attuale (syntax, semantic, logic, security)  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Layer 1: Static Analysis (Esistente)

**Status:** ✅ Implementato

Il core attuale di Aether con 5 layer di validazione:
- Syntax Layer (parser-based)
- Semantic Layer (type checking, scope)
- Logic Layer (anti-pattern, code smells)
- Security Layer (vulnerabilities)
- Style Layer (conventions)

**Nessun cambiamento richiesto** — questo layer rimane la base.

---

## Layer 2: Memory System (Hybrid Architecture)

**Status:** ⚡ Parzialmente implementato → 🔄 Redesign basato su research

### Il Problema: Context Rot

Prima di progettare la memoria, dobbiamo capire il problema che risolve.

**Ricerca Chroma (2026):** Studio su 18 LLM (GPT-4.1, Claude 4, Gemini 2.5, Qwen3) ha rivelato:
- **Claude Sonnet 4**: 99% → 50% accuracy quando il contesto aumenta
- **ChatGPT**: Effective memory di **~7±2 items** (come umani!) anche con 128K token
- Ogni token aggiunge **n² relazioni** che competono per l'attenzione

**Perché la RAG tradizionale fallisce (Manifold Group):**

| Problema | Descrizione |
|----------|-------------|
| **Snippet ≠ Understanding** | Vector DB memorizza frammenti, non relazioni |
| **Retrieval ≠ Reasoning** | Non c'è feedback loop tra recupero e ragionamento |
| **No Time/Decay** | Tutto ha stesso peso per sempre |
| **Session Amnesia** | Ogni sessione ricomincia da zero |

**Impatto pratico:** Un agente AI che ha visto 100 file "dimentica" le decisioni prese sui primi 10 quando processa gli ultimi. Questo porta a:
- Incoerenze architetturali
- Refactoring distruttivi
- Violazioni di decisioni approvate dall'utente

### La Soluzione: Hybrid Memory a 4 Layers

Invece di un semplice RAG, usiamo un'architettura ibrida che combina approcci provati:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      AETHER MEMORY SYSTEM                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │              LAYER 2A: CODE GRAPH (AST-based)                          │ │
│  │                                                                        │ │
│  │  File → Functions → Calls → Dependencies                              │ │
│  │  "Chi chiama questa funzione?"                                        │ │
│  │  "Cosa dipende da questo modulo?"                                     │ │
│  │  Metodo: AST parsing + Graph DB                                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │              LAYER 2B: DECISION LOG (Knowledge Graph)                  │ │
│  │                                                                        │ │
│  │  "Perché questo codice esiste?" → Intent                              │ │
│  │  "Questa decisione è ok perché..." → User feedback                    │ │
│  │  "Questo pattern è accettato" → Pattern approval                      │ │
│  │  Timestamp, author, reason                                            │ │
│  │  Metodo: Knowledge Graph + RAG fallback                               │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │              LAYER 2C: VALIDATION STATE (File-based)                   │ │
│  │                                                                        │ │
│  │  Ultima validazione: {file, hash, violations, status}                 │ │
│  │  Stato "acceptato/rifiutato" per violations                           │ │
│  │  Delta tracking: cosa è cambiato dall'ultima validazione              │ │
│  │  Metodo: TOML files + Git (semplice, debuggabile, leggibile)          │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    ↓                                        │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │              LAYER 2D: DRIFT SNAPSHOTS (Time-series)                   │ │
│  │                                                                        │ │
│  │  Giorno 1: {metrics} → Giorno 30: {metrics} → Trend                    │ │
│  │  Alert quando metriche peggiorano                                     │ │
│  │  Metodo: Time-series DB + Git history                                 │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Perché questa architettura:**
- **Code Graph**: Rappresenta relazioni strutturali (non solo snippet)
- **Decision Log**: Preserva intent e feedback (Knowledge Graph = 94.87% accuracy in Observational Memory)
- **Validation State**: File-based = semplice, debuggabile, raccomandato da Anthropic
- **Drift Snapshots**: Time awareness per rilevare degrado

### Layer 2A: Code Graph (AST-based)

Mantiene la mappa strutturale del codice per queries tipo "chi chiama cosa".

```rust
pub struct CodeGraph {
    /// Nodi: File, Function, Class, Module
    nodes: HashMap<NodeId, CodeNode>,
    
    /// Edges: Calls, Imports, Extends, Implements
    edges: Vec<CodeEdge>,
    
    /// Indici per query veloci
    call_graph: HashMap<NodeId, Vec<NodeId>>,     // caller → callees
    reverse_call_graph: HashMap<NodeId, Vec<NodeId>>, // callee → callers
    dependency_graph: HashMap<NodeId, Vec<NodeId>>,   // file → dependencies
}

pub struct CodeNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub signature: Option<String>,
    pub hash: String,  // Per detect changes
}

pub enum NodeType {
    File,
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Module,
}

pub struct CodeEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
    pub location: (String, usize),  // (file, line)
}

pub enum EdgeType {
    Calls,
    Imports,
    Extends,
    Implements,
    Contains,
    References,
}

impl CodeGraph {
    /// Query: Chi chiama questa funzione?
    pub fn who_calls(&self, function_id: &NodeId) -> Vec<&CodeNode> {
        self.reverse_call_graph
            .get(function_id)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Query: Cosa dipende da questo file?
    pub fn what_depends_on(&self, file_id: &NodeId) -> Vec<&CodeNode> {
        // Reverse dependency lookup
        self.dependency_graph
            .iter()
            .filter(|(_, deps)| deps.contains(file_id))
            .filter_map(|(id, _)| self.nodes.get(id))
            .collect()
    }
    
    /// Query: Impact analysis per refactor
    pub fn impact_analysis(&self, node_id: &NodeId) -> ImpactReport {
        let direct_callers = self.who_calls(node_id);
        let transitive_callers = self.transitive_callers(node_id);
        let dependents = self.what_depends_on(node_id);
        
        ImpactReport {
            direct_impact: direct_callers.len(),
            transitive_impact: transitive_callers.len(),
            files_affected: dependents.len(),
            risk_level: self.calculate_risk(&direct_callers, &transitive_callers),
        }
    }
}
```

### Layer 2B: Decision Log (Knowledge Graph)

Preserva il "perché" delle decisioni - fondamentale per evitare refactoring distruttivi.

```rust
pub struct DecisionLog {
    /// Knowledge Graph: entità e relazioni
    nodes: HashMap<DecisionId, DecisionNode>,
    edges: Vec<DecisionEdge>,
    
    /// RAG fallback per query semantiche
    rag_store: SemanticStore,
}

pub struct DecisionNode {
    pub id: DecisionId,
    pub decision_type: DecisionType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub author: String,  // "David", "Droid", "System"
    pub context: DecisionContext,
    pub status: DecisionStatus,
}

pub enum DecisionType {
    /// "Perché questo codice esiste"
    IntentDeclaration,
    
    /// "Questa violation è accettabile perché..."
    AcceptedViolation,
    
    /// "Questo pattern è lo stile del progetto"
    PatternApproval,
    
    /// "Questo refactor è stato fatto perché..."
    RefactorReason,
    
    /// "Non toccare questo codice perché..."
    DoNotTouch,
    
    /// "Questo codice sarà rimosso in futuro"
    TechnicalDebt,
}

pub struct DecisionContext {
    pub file_path: Option<String>,
    pub function_name: Option<String>,
    pub violation_id: Option<String>,
    pub code_hash: Option<String>,
    pub related_decisions: Vec<DecisionId>,
}

pub enum DecisionStatus {
    Active,
    Deprecated,
    SupersededBy(DecisionId),
}

pub struct DecisionEdge {
    pub from: DecisionId,
    pub to: DecisionId,
    pub relation: DecisionRelation,
}

pub enum DecisionRelation {
    Explains,      // A spiega B
    Supersedes,    // A sostituisce B
    ConflictsWith, // A è in conflitto con B
    DependsOn,     // A dipende da B
}

impl DecisionLog {
    /// Registra una nuova decisione
    pub async fn record(&mut self, decision: DecisionNode) -> Result<DecisionId> {
        let id = decision.id;
        
        // Store in graph
        self.nodes.insert(id, decision.clone());
        
        // Also store in RAG for semantic search
        self.rag_store.store(StorageRequest {
            content: format!("{:?}: {}", decision.decision_type, decision.content),
            metadata: serde_json::to_value(&decision.context)?,
        }).await?;
        
        Ok(id)
    }
    
    /// Query: Perché questo codice esiste?
    pub async fn why_exists(&self, file: &str, function: Option<&str>) -> Vec<&DecisionNode> {
        self.nodes.values()
            .filter(|d| {
                d.decision_type == DecisionType::IntentDeclaration &&
                d.context.file_path.as_deref() == Some(file) &&
                function.map_or(true, |f| d.context.function_name.as_deref() == Some(f))
            })
            .collect()
    }
    
    /// Query: Questa violation è accettabile?
    pub async fn is_accepted(&self, violation_id: &str) -> Option<&DecisionNode> {
        self.nodes.values()
            .find(|d| {
                matches!(d.decision_type, DecisionType::AcceptedViolation) &&
                d.context.violation_id.as_deref() == Some(violation_id) &&
                d.status == DecisionStatus::Active
            })
    }
    
    /// Query semantica via RAG: "ricorda perché abbiamo deciso X"
    pub async fn recall_semantic(&self, query: &str) -> Result<Vec<DecisionNode>> {
        let results = self.rag_store.search(query, 5).await?;
        
        results.into_iter()
            .filter_map(|r| {
                // Lookup full node from graph
                self.nodes.values()
                    .find(|n| r.content.contains(&n.content))
                    .cloned()
            })
            .map(Ok)
            .collect()
    }
}
```

### Layer 2C: Validation State (File-based)

Approccio semplice e debuggabile raccomandato da Anthropic per agenti long-running.

```rust
pub struct ValidationState {
    /// Root directory per state files
    state_dir: PathBuf,
}

/// Un file TOML per progetto (leggibile e modificabile)
#[derive(Serialize, Deserialize)]
pub struct ProjectState {
    pub project_id: String,
    pub last_validation: DateTime<Utc>,
    pub files: HashMap<String, FileState>,
    pub accepted_violations: HashMap<String, AcceptedViolation>,
    pub statistics: ValidationStats,
}

#[derive(Serialize, Deserialize)]
pub struct FileState {
    pub file_path: String,
    pub hash: String,  // Content hash
    pub last_validated: DateTime<Utc>,
    pub violations: Vec<ViolationRecord>,
    pub status: FileStatus,
}

#[derive(Serialize, Deserialize)]
pub struct ViolationRecord {
    pub violation_id: String,
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub status: ViolationStatus,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub times_seen: usize,
}

#[derive(Serialize, Deserialize)]
pub enum ViolationStatus {
    New,
    Acknowledged,
    Accepted(String),  // Reason
    Fixed,
    FalsePositive,
}

#[derive(Serialize, Deserialize)]
pub struct AcceptedViolation {
    pub violation_id: String,
    pub reason: String,
    pub accepted_by: String,  // "David", "Droid"
    pub accepted_at: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,  // Optional TTL
}

impl ValidationState {
    /// Carica o crea stato per progetto
    pub fn load_or_create(&self, project_id: &str) -> Result<ProjectState> {
        let path = self.state_dir.join(format!("{}.state.toml", project_id));
        
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(ProjectState {
                project_id: project_id.to_string(),
                last_validation: Utc::now(),
                files: HashMap::new(),
                accepted_violations: HashMap::new(),
                statistics: ValidationStats::default(),
            })
        }
    }
    
    /// Salva stato
    pub fn save(&self, state: &ProjectState) -> Result<()> {
        let path = self.state_dir.join(format!("{}.state.toml", state.project_id));
        let content = toml::to_string_pretty(state)?;
        fs::write(&path, content)?;
        Ok(())
    }
    
    /// Check se una violation è già accettata
    pub fn is_accepted(&self, state: &ProjectState, violation_id: &str) -> bool {
        state.accepted_violations
            .get(violation_id)
            .map(|v| v.expires.map_or(true, |exp| Utc::now() < exp))
            .unwrap_or(false)
    }
    
    /// Delta: cosa è cambiato dall'ultima validazione
    pub fn compute_delta(&self, old: &FileState, new_hash: &str) -> FileDelta {
        if old.hash == new_hash {
            FileDelta::Unchanged
        } else {
            FileDelta::Changed {
                old_hash: old.hash.clone(),
                new_hash: new_hash.to_string(),
                violations_resolved: vec![],  // Populated by comparison
                new_violations: vec![],
            }
        }
    }
}
```

### Layer 2D: Drift Snapshots (Time-series)

Traccia l'evoluzione delle metriche nel tempo per rilevare trend negativi.

```rust
pub struct DriftSnapshots {
    /// Time-series storage
    snapshots: Vec<DriftSnapshot>,
    
    /// Git integration per correlare con commit
    git_integration: GitIntegration,
}

#[derive(Serialize, Deserialize)]
pub struct DriftSnapshot {
    pub timestamp: DateTime<Utc>,
    pub project_id: String,
    pub commit_hash: Option<String>,
    pub metrics: DriftMetrics,
    pub alerts: Vec<DriftAlert>,
}

#[derive(Serialize, Deserialize)]
pub struct DriftMetrics {
    // Quality metrics
    pub total_violations: usize,
    pub violations_by_severity: HashMap<Severity, usize>,
    pub violation_density: f32,  // violations per 1000 LOC
    
    // Code health metrics
    pub type_strictness: f32,    // 0.0 = all Any, 1.0 = fully typed
    pub error_handling_quality: f32,
    pub complexity_avg: f32,
    pub dead_code_ratio: f32,
    
    // Consistency metrics
    pub naming_consistency: f32,
    pub style_consistency: f32,
    
    // Trend indicators
    pub delta_from_previous: MetricDelta,
}

#[derive(Serialize, Deserialize)]
pub struct MetricDelta {
    pub violations_delta: isize,
    pub quality_delta: f32,
    pub complexity_delta: f32,
}

#[derive(Serialize, Deserialize)]
pub struct DriftAlert {
    pub alert_type: DriftAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_value: f32,
    pub threshold: f32,
    pub trend: Trend,
}

pub enum DriftAlertType {
    ViolationIncrease,
    QualityDegradation,
    ComplexitySpike,
    TypeErosion,
    ErrorHandlingErosion,
    NamingInconsistency,
}

pub enum Trend {
    Improving,
    Stable,
    Declining,
    RapidlyDeclining,
}

impl DriftSnapshots {
    /// Crea snapshot corrente
    pub async fn create_snapshot(&mut self, project_id: &str, metrics: DriftMetrics) -> Result<()> {
        let commit_hash = self.git_integration.current_commit(project_id)?;
        
        // Compute delta from previous
        let delta = self.compute_delta(project_id, &metrics);
        
        // Generate alerts
        let alerts = self.generate_alerts(&metrics, &delta);
        
        let snapshot = DriftSnapshot {
            timestamp: Utc::now(),
            project_id: project_id.to_string(),
            commit_hash,
            metrics,
            alerts,
        };
        
        self.snapshots.push(snapshot);
        Ok(())
    }
    
    /// Analizza trend
    pub fn analyze_trend(&self, project_id: &str, window: Duration) -> TrendReport {
        let snapshots: Vec<_> = self.snapshots
            .iter()
            .filter(|s| s.project_id == project_id)
            .filter(|s| s.timestamp > Utc::now() - window)
            .collect();
        
        if snapshots.len() < 2 {
            return TrendReport::InsufficientData;
        }
        
        // Linear regression on key metrics
        let quality_trend = self.regress(&snapshots, |s| s.metrics.type_strictness);
        let complexity_trend = self.regress(&snapshots, |s| s.metrics.complexity_avg);
        let violation_trend = self.regress(&snapshots, |s| s.metrics.total_violations as f32);
        
        TrendReport::Complete {
            quality_trend: if quality_trend > 0.01 { Trend::Improving }
                          else if quality_trend > -0.01 { Trend::Stable }
                          else if quality_trend > -0.05 { Trend::Declining }
                          else { Trend::RapidlyDeclining },
            complexity_trend,
            violation_trend,
            snapshots_analyzed: snapshots.len(),
        }
    }
}
```

#### Architectural Drift Analysis (Enhancement)

**Status:** 📋 Planned

L'analisi drift singolo-file è limitata: non capisce le dipendenze tra moduli.
L'Architectural Drift Analysis espande l'analisi a intere strutture di codice.

##### CLI Usage

```bash
# Analisi singolo file (baseline)
aether memory recall drift-trend src/engine/renderer.rs

# Analisi con espansione dipendenze (depth-limited)
aether memory recall drift-trend src/engine/renderer.rs --depth 2
```

##### Output Example

```
Analyzing:
  src/engine/renderer.rs (root file)
  ├─ src/engine/shader.rs (depth 1)
  ├─ src/engine/texture.rs (depth 1)
  └─ src/math/vec3.rs (depth 2)

Drift Score: 0.15 (moderate)
  renderer.rs: 0.08 ✓
  shader.rs: 0.22 ⚠ (type_strictness declining)
  texture.rs: 0.12 ⚠
  vec3.rs: 0.03 ✓

Recommendation: Review shader.rs - type annotations becoming sparse
```

##### Architecture

| Componente | Ruolo |
|------------|-------|
| `CodeGraph` (Layer 2A) | Trova dipendenze: `what_depends_on()`, `who_calls()` |
| `DriftSnapshotStore` (Layer 2D) | Metriche per-file nel tempo |
| `ModuleRegistry` (nuovo) | Configurazione manuale moduli (opzionale) |

##### Dependency Expansion Algorithm

```rust
pub struct ArchitecturalDriftAnalyzer {
    code_graph: CodeGraph,
    drift_store: DriftSnapshotStore,
    max_depth: usize,      // Default: 3
    max_files: usize,      // Default: 50
}

impl ArchitecturalDriftAnalyzer {
    /// Espande dalle dipendenze usando BFS con limiti
    pub fn expand_dependencies(&self, root: &Path, depth: usize) -> Vec<PathBuf> {
        let mut files = vec![root.to_path_buf()];
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((root, 0));

        while let Some((file, current_depth)) = queue.pop_front() {
            if current_depth >= depth || files.len() >= self.max_files {
                break;
            }

            // Trova dipendenze dal CodeGraph
            let deps = self.code_graph.what_depends_on(file);

            for dep in deps {
                if !visited.contains(&dep) {
                    visited.insert(dep.clone());
                    files.push(dep.clone());
                    queue.push_back((&dep, current_depth + 1));
                }
            }
        }

        files
    }

    /// Calcola drift aggregato per struttura
    pub fn analyze_architecture(&self, root: &Path, depth: usize, days: u32) -> DriftReport {
        let files = self.expand_dependencies(root, depth);
        let mut file_reports = Vec::new();

        for file in &files {
            let drift = self.drift_store.analyze_trend_days(file, days);
            file_reports.push(FileDrift {
                path: file.clone(),
                score: drift.score,
                trend: drift.trend,
                alerts: drift.alerts,
            });
        }

        // Calcola score aggregato (media ponderata per vicinanza)
        let agg_score = self.compute_weighted_score(&file_reports, depth);

        DriftReport {
            root: root.to_path_buf(),
            files: file_reports,
            aggregate_score: agg_score,
            recommendation: self.generate_recommendation(&file_reports),
        }
    }
}
```

##### Configurazione Manuale (Opzionale)

Per progetti con struttura complessa, è possibile definire moduli manualmente:

```json
// .aether/modules.json
{
  "modules": {
    "engine.rendering": {
      "files": ["src/engine/renderer.rs", "src/engine/shader.rs", "src/engine/texture.rs"],
      "entry_point": "src/engine/renderer.rs"
    },
    "engine.physics": {
      "files": ["src/engine/physics/*.rs"],
      "entry_point": "src/engine/physics/mod.rs"
    }
  }
}
```

##### Limiti Computazionali

| Limite | Valore | Motivazione |
|--------|--------|-------------|
| `max_depth` | 3 | Oltre depth 3, il grafo diventa troppo vasto |
| `max_files` | 50 | Limite pratico per analisi interattiva |
| `cache_ttl` | 1h | Risultati cached per performance |

##### Integrazione

```
Layer 2A (CodeGraph)     Layer 2D (DriftSnapshots)
         │                         │
         └─────────┬───────────────┘
                   ▼
    ArchitecturalDriftAnalyzer (new)
                   │
                   ▼
         aether memory recall drift-trend --depth N
```

### API Unificata: Aether Recall

Quando l'agente perde contesto, chiama `aether recall`:

```rust
impl AetherMemory {
    /// Query unificata per recuperare contesto
    pub async fn recall(&self, query: MemoryQuery) -> Result<MemoryResponse> {
        match query {
            MemoryQuery::WhoCalls { function } => {
                let callers = self.code_graph.who_calls(&function);
                Ok(MemoryResponse::Callers(callers))
            }
            
            MemoryQuery::WhyExists { file, function } => {
                let decisions = self.decision_log.why_exists(&file, function.as_deref()).await;
                Ok(MemoryResponse::Decisions(decisions))
            }
            
            MemoryQuery::IsAccepted { violation_id } => {
                let accepted = self.decision_log.is_accepted(&violation_id).await
                    .or_else(|| self.validation_state.is_accepted(&self.state, &violation_id));
                Ok(MemoryResponse::Acceptance(accepted))
            }
            
            MemoryQuery::SemanticRecall { query } => {
                let decisions = self.decision_log.recall_semantic(&query).await?;
                Ok(MemoryResponse::Decisions(decisions))
            }
            
            MemoryQuery::DriftTrend { project, window } => {
                let trend = self.drift_snapshots.analyze_trend(&project, window);
                Ok(MemoryResponse::Trend(trend))
            }
            
            MemoryQuery::ImpactAnalysis { node } => {
                let impact = self.code_graph.impact_analysis(&node);
                Ok(MemoryResponse::Impact(impact))
            }
        }
    }
}
```

**Esempio di utilizzo:**

```
# Agente perde contesto, chiede ad Aether:
> aether recall "perché c'è questo unwrap()?"

Response:
┌─────────────────────────────────────────────────────────────────┐
│ DECISION LOG                                                    │
│                                                                 │
│ 2026-03-10 14:32 (David):                                      │
│ "Accepted: questo unwrap è sicuro perché il file di config      │
│  è sempre presente in produzione (vedi deployment.yaml)"        │
│                                                                 │
│ Status: Active                                                  │
│ Related: DEPLOY-42, CONFIG-17                                   │
│                                                                 │
│ CODE GRAPH:                                                     │
│ Called by: main(), load_config(), init_app()                    │
│ Impact if removed: 3 functions affected                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Compliance Engine — Intelligent Contract Enforcement

**Status:** ✅ Implementato

Parte di `aether-intelligence`, il Compliance Engine fornisce applicazione intelligente dei contratti che combina regole rigide con apprendimento contestuale.

### Obiettivo

Non tutte le violazioni sono uguali. Il Compliance Engine classifica le regole e decide **come** gestire le violazioni basandosi su:
- **Tier del contratto** (Inviolable vs Strict vs Flexible)
- **Contesto del progetto** (tipo, regione del codice, storia)
- **Precedenti** (come sono state gestite violazioni simili)

### Contract Tiers

```
┌─────────────────────────────────────────────────────────────────┐
│                    CONTRACT ENFORCEMENT TIERS                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  INVIOLABLE (Mai bypassato)                                  ││
│  │  - Security: SQL injection, XSS, path traversal             ││
│  │  - Memory safety: Use-after-free, buffer overflow           ││
│  │  - Supply chain: Malicious dependencies                     ││
│  │  → Action: BLOCK                                            ││
│  └─────────────────────────────────────────────────────────────┘│
│                              ↓                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  STRICT (Richiede accettazione esplicita)                   ││
│  │  - Logic: Dead code, unreachable branches                   ││
│  │  - Error handling: Missing error handling, silent failures  ││
│  │  - Memory: Leaks, resource management                       ││
│  │  → Action: WARN + requires documented reason                ││
│  └─────────────────────────────────────────────────────────────┘│
│                              ↓                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  FLEXIBLE (Può essere imparato)                              ││
│  │  - Style: Line length, brace style                           ││
│  │  - Naming: Conventions, casing                               ││
│  │  - Formatting: Indentation, spacing                          ││
│  │  → Action: LEARN after N occurrences                         ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Decision Flow

```rust
// Il Compliance Engine valuta ogni violazione
async fn evaluate(
    rule_id: &str,
    domain: &str,
    message: &str,
    ctx: &ComplianceContext,
) -> Result<ComplianceDecision> {
    // 1. Classifica il contratto
    let tier = classifier.classify(rule_id, domain);
    
    // 2. Inviolable = immediate block
    if tier == ContractTier::Inviolable {
        return Ok(ComplianceDecision::block(tier, rule_id, message));
    }
    
    // 3. Check per esenzione esistente
    if let Some(exemption) = exemptions.find(rule_id, &ctx.file_path) {
        return Ok(ComplianceDecision::accept(tier, exemption.reason, ...));
    }
    
    // 4. Check per learning (pattern ricorrente)
    if occurrences >= learn_after_occurrences && tier.supports_learning() {
        return Ok(ComplianceDecision::learn(tier, pattern, confidence));
    }
    
    // 5. Check per precedenti simili
    if let Some(precedent) = find_similar_precedent(rule_id, ctx) {
        return Ok(ComplianceDecision::accept(tier, "Based on precedent", ...));
    }
    
    // 6. Low confidence = Ask via Dubbioso
    if confidence < ask_threshold {
        return Ok(ComplianceDecision::ask(tier, question, options, ...));
    }
    
    // 7. Default = Warn
    Ok(ComplianceDecision::warn(tier, message, explanation))
}
```

### Integrazione con Dubbioso Mode

Quando la confidenza è bassa (`< 0.60`), il Compliance Engine attiva **Dubbioso Mode**:

```
┌─────────────────────────────────────────────────────────────────┐
│  DUBBIOSO MODE ACTIVATED                                        │
│                                                                 │
│  Violation: STYLE002 - Line too long (120 chars)                │
│  File: src/api/handlers.rs:45                                   │
│  Context: production code, no similar precedents                │
│                                                                 │
│  Confidence: 0.45 (low)                                         │
│                                                                 │
│  Question: Is this line length violation acceptable?            │
│                                                                 │
│  [1] Yes, accept for this file                                  │
│  [2] Yes, accept for all similar cases                          │
│  [3] No, this should be fixed                                   │
│                                                                 │
│  Reason needed if accepting: ________________________           │
└─────────────────────────────────────────────────────────────────┘
```

### Learning Behavior

Il Compliance Engine impara dai pattern del progetto:

| Occurrences | Action |
|-------------|--------|
| 1-2 | Warn, track occurrence |
| 3+ | Auto-learn, create exemption |
| 5+ | Increase confidence, suggest scope expansion |

**Esempio:**

```
Day 1:  STYLE002 in test_auth.rs → Warn
Day 2:  STYLE002 in test_user.rs → Warn
Day 3:  STYLE002 in test_api.rs → Learn! Create exemption for *test*
Day 4:  STYLE002 in test_handler.rs → Accept (from learned pattern)
```

### API Usage

```rust
use aether_intelligence::compliance::{
    ComplianceEngine, ComplianceConfig, ComplianceContext,
    ContractTier, ComplianceAction,
};

let mut engine = ComplianceEngine::new()?;

let ctx = ComplianceContext {
    file_path: "src/main.rs".into(),
    line: 42,
    project_type: Some("cli".into()),
    code_region: Some("main".into()),
    ..Default::default()
};

let decision = engine.evaluate("SEC001", "security", "SQL injection risk", &ctx).await?;

match decision.action {
    ComplianceAction::Block => { /* Non-negotiable, must fix */ }
    ComplianceAction::Ask { question, options } => { /* Use Dubbioso */ }
    ComplianceAction::Learn { pattern, confidence } => { /* Pattern learned */ }
    ComplianceAction::Accept { reason, .. } => { /* Accept with reason */ }
    ComplianceAction::Warn => { /* Log warning */ }
}
```

---

## Layer 3: Pattern Discovery (ML)

**Status:** 📋 Da implementare

### Obiettivo

Spostarsi dal **pattern matching** (regole predefinite) alla **pattern discovery** (trovare nuovi pattern automaticamente).

### Approccio

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PATTERN DISCOVERY PIPELINE                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Codice Validato ──→ Feature Extraction ──→ Clustering ──→ Pattern Mining  │
│         │                   │                  │                │          │
│         │                   ↓                  ↓                ↓          │
│         │            [AST features]     [K-means/DBSCAN]   [New Rules]     │
│         │            [Token patterns]   [Anomaly detect]   [Alerts]        │
│         │            [Control flow]                                        │
│         │                                                                    │
│         └───────────────────→ Feedback Loop ←─────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Feature Extraction

```rust
pub struct CodeFeatures {
    // Strutturali
    pub ast_depth: usize,
    pub num_functions: usize,
    pub num_classes: usize,
    pub cyclomatic_complexity: f32,
    
    // Semantici
    pub type_diversity: f32,      // Quanti tipi diversi
    pub parameter_count_avg: f32,
    pub return_type_diversity: f32,
    
    // Stilistici
    pub naming_patterns: HashMap<String, f32>,  // snake_case %, camelCase %
    pub line_length_avg: f32,
    pub comment_ratio: f32,
    
    // Logici
    pub error_handling_patterns: Vec<String>,
    pub null_check_patterns: Vec<String>,
    pub loop_patterns: Vec<String>,
    
    // Temporali (per drift)
    pub similarity_to_previous: f32,
    pub consistency_score: f32,
}
```

### Pattern Mining

```rust
impl PatternDiscovery {
    /// Scopre nuovi pattern dal codice validato
    pub async fn discover_patterns(&self, validations: &[ValidationRecord]) -> Result<Vec<DiscoveredPattern>> {
        // 1. Extract features
        let features: Vec<CodeFeatures> = validations
            .iter()
            .map(|v| self.extract_features(&v.code))
            .collect();
        
        // 2. Cluster similar code
        let clusters = self.cluster_features(&features)?;
        
        // 3. Find anomalies
        let anomalies = self.find_anomalies(&features)?;
        
        // 4. Mine patterns from clusters
        let patterns = self.mine_cluster_patterns(&clusters, &validations)?;
        
        // 5. Generate candidate rules
        let rules = self.generate_rules(&patterns, &anomalies)?;
        
        Ok(rules)
    }
    
    /// Valida un pattern scoperto
    pub async fn validate_pattern(&self, pattern: &DiscoveredPattern) -> Result<PatternValidation> {
        // Check against historical data
        let false_positive_rate = self.check_false_positives(pattern).await?;
        let coverage = self.check_coverage(pattern).await?;
        let severity = self.infer_severity(pattern)?;
        
        Ok(PatternValidation {
            pattern: pattern.clone(),
            false_positive_rate,
            coverage,
            severity,
            recommendation: if false_positive_rate < 0.1 && coverage > 0.05 {
                "Accept as new rule"
            } else {
                "Needs review"
            },
        })
    }
}
```

### Output: Nuove Regole

Quando un pattern viene validato, Aether può proporre nuove regole:

```yaml
# Regola scoperta automaticamente
discovered_rules:
  - id: DISCOVERED_001
    name: "React useEffect Missing Dependency"
    description: "useEffect with state access but missing dependency"
    confidence: 0.92
    discovered_at: "2026-03-15T10:30:00Z"
    examples:
      - code: |
          useEffect(() => {
            console.log(count);
          }, []);  // Missing 'count' in deps
        fix: |
          useEffect(() => {
            console.log(count);
          }, [count]);
    status: pending_review  # Needs human approval
```

---

## Layer 4: Intent Inference (LLM-lite) — **OPZIONALE**

**Status:** 📋 Da implementare
**Nota:** Questo layer è **opzionale**. Il core di Aether funziona completamente senza LLM.
L'LLM viene usato solo come "dizionario" per arricchire l'analisi, non è richiesto.

### Obiettivo

Capire **l'intento** del codice, non solo la struttura. Questo è cruciale per:
- Rilevare refactoring distruttivi
- Suggerire alternative che preservano intent
- Evitare "fix" che rompono la logica

### Approccio

Quando disponibile, usare un modello LLM **lightweight** (es. Phi-3, TinyLlama, o quantizzato) per:
1. Inferire intent dal codice + contesto
2. Verificare che cambiamenti proposti preservino intent
3. Generare documentazione automatica

### Implementazione

```rust
pub struct IntentInference {
    /// Modello LLM leggero (locale)
    model: Box<dyn LocalLLM>,
    
    /// Cache degli intent inferiti
    intent_cache: LruCache<String, CodeIntent>,
}

pub struct CodeIntent {
    /// Intent principale
    pub primary_intent: String,
    
    /// Intent secondari
    pub secondary_intents: Vec<String>,
    
    /// Side effects rilevati
    pub side_effects: Vec<String>,
    
    /// Dipendenze implicite
    pub implicit_dependencies: Vec<String>,
    
    /// Invarianti da preservare
    pub invariants: Vec<String>,
    
    /// Confidenza
    pub confidence: f32,
}

impl IntentInference {
    /// Inferisce l'intento del codice
    pub async fn infer_intent(&self, code: &str, context: &ProjectContext) -> Result<CodeIntent> {
        // Check cache
        if let Some(intent) = self.intent_cache.get(code) {
            return Ok(intent.clone());
        }
        
        // Build prompt
        let prompt = format!(
            r#"Analyze this code and infer its intent.
            
Context: {} project, {} language
Framework: {}
Recent patterns: {}

Code:
```
{}
```

Output JSON:
{{
  "primary_intent": "What this code is meant to do",
  "secondary_intents": ["Other purposes"],
  "side_effects": ["Observable side effects"],
  "implicit_dependencies": ["Hidden dependencies"],
  "invariants": ["Assumptions that must hold"]
}}"#,
            context.project_id,
            context.language,
            context.framework.as_deref().unwrap_or("unknown"),
            context.conventions.join(", "),
            code
        );
        
        // Call local LLM
        let response = self.model.generate(&prompt).await?;
        let intent: CodeIntent = serde_json::from_str(&response)?;
        
        // Cache
        self.intent_cache.put(code.to_string(), intent.clone());
        
        Ok(intent)
    }
    
    /// Verifica che un fix preservi l'intento
    pub async fn verify_fix_preserves_intent(
        &self,
        original: &str,
        fixed: &str,
        original_intent: &CodeIntent,
    ) -> Result<IntentPreservation> {
        let prompt = format!(
            r#"Compare these code versions and check if the fix preserves intent.

Original intent: {}
Original code:
```
{}
```

Fixed code:
```
{}
```

Does the fix preserve the original intent? What might break?"#,
            serde_json::to_string_pretty(original_intent)?,
            original,
            fixed
        );
        
        let response = self.model.generate(&prompt).await?;
        
        Ok(IntentPreservation {
            preserves_intent: response.contains("preserves"),
            potential_issues: self.extract_issues(&response),
            confidence: 0.8, // From model
        })
    }
}
```

### Integrazione con Fix Suggestions

```rust
impl ValidationEngine {
    pub async fn suggest_fix(&self, violation: &Violation, code: &str) -> Result<FixSuggestion> {
        // 1. Get code intent
        let intent = self.intent_inference.infer_intent(code, &self.context).await?;
        
        // 2. Generate possible fixes
        let possible_fixes = self.generate_fixes(violation, code)?;
        
        // 3. Filter by intent preservation
        let mut valid_fixes = vec![];
        for fix in possible_fixes {
            let preservation = self.intent_inference
                .verify_fix_preserves_intent(code, &fix.code, &intent)
                .await?;
            
            if preservation.preserves_intent {
                valid_fixes.push(FixWithIntent {
                    fix,
                    intent_match: preservation.confidence,
                });
            }
        }
        
        // 4. Rank by confidence
        valid_fixes.sort_by(|a, b| b.intent_match.partial_cmp(&a.intent_match).unwrap());
        
        Ok(FixSuggestion {
            recommended: valid_fixes.first().map(|f| f.fix.clone()),
            alternatives: valid_fixes.iter().skip(1).map(|f| f.fix.clone()).collect(),
            original_intent: intent,
        })
    }
}
```

---

## Layer 5: Drift Detection (Temporal)

**Status:** 📋 Da implementare

### Obiettivo

Rilevare degrado del codice nel tempo, analizzando l'evoluzione del codice.

### Metriche di Drift

```rust
pub struct DriftMetrics {
    /// Distanza semantica dal codice originale
    pub semantic_drift: f32,
    
    /// Cambiamento nelle convenzioni
    pub convention_drift: f32,
    
    /// Complessità crescente
    pub complexity_trend: f32,
    
    /// Consistenza dei tipi
    pub type_consistency: f32,
    
    /// Coverage dei test (se disponibile)
    pub test_coverage_trend: f32,
    
    /// Pattern di degrado rilevati
    pub degradation_patterns: Vec<DegradationPattern>,
}

pub enum DegradationPattern {
    /// Parametri che diventano opzionali
    ParameterOptionality,
    
    /// Tipi che diventano Any
    TypeErosion,
    
    /// Naming inconsistente
    NamingInconsistency,
    
    /// Error handling che degrada
    ErrorHandlingErosion,
    
    /// Dipendenze duplicate
    DependencyDuplication,
    
    /// Dead code accumulation
    DeadCodeAccumulation,
}
```

### Analisi Temporale

```rust
pub struct DriftDetector {
    /// Storico validazioni
    validation_history: TimeSeries<ValidationRecord>,
    
    /// Storico codice (snapshots)
    code_snapshots: TimeSeries<CodeSnapshot>,
    
    /// Metriche calcolate
    metrics_cache: LruCache<String, DriftMetrics>,
}

impl DriftDetector {
    /// Analizza drift di un file nel tempo
    pub async fn analyze_drift(&self, file_path: &str, time_window: Duration) -> Result<DriftReport> {
        // 1. Get code snapshots
        let snapshots = self.code_snapshots.get_range(file_path, time_window)?;
        
        if snapshots.len() < 2 {
            return Ok(DriftReport::InsufficientData);
        }
        
        // 2. Calculate metrics for each snapshot
        let metrics: Vec<SnapshotMetrics> = snapshots
            .iter()
            .map(|s| self.calculate_metrics(s))
            .collect();
        
        // 3. Detect trends
        let trends = self.detect_trends(&metrics)?;
        
        // 4. Identify degradation patterns
        let patterns = self.identify_degradation(&snapshots, &trends)?;
        
        // 5. Calculate overall drift score
        let drift_score = self.calculate_drift_score(&trends, &patterns);
        
        Ok(DriftReport {
            file_path: file_path.to_string(),
            time_window,
            drift_score,
            trends,
            patterns,
            recommendations: self.generate_recommendations(&patterns),
        })
    }
    
    /// Calcola metriche per uno snapshot
    fn calculate_metrics(&self, snapshot: &CodeSnapshot) -> SnapshotMetrics {
        SnapshotMetrics {
            timestamp: snapshot.timestamp,
            type_strictness: self.measure_type_strictness(&snapshot.code),
            naming_consistency: self.measure_naming_consistency(&snapshot.code),
            error_handling_quality: self.measure_error_handling(&snapshot.code),
            complexity: self.measure_complexity(&snapshot.code),
            dead_code_ratio: self.measure_dead_code(&snapshot.code),
        }
    }
    
    /// Rileva trend
    fn detect_trends(&self, metrics: &[SnapshotMetrics]) -> Result<Vec<Trend>> {
        let mut trends = vec![];
        
        // Type strictness trend
        let type_trend = self.linear_regression(&metrics.iter().map(|m| m.type_strictness).collect::<Vec<_>>());
        if type_trend.slope < -0.01 {
            trends.push(Trend::Declining {
                metric: "type_strictness",
                rate: type_trend.slope,
                severity: if type_trend.slope < -0.05 { "high" } else { "medium" },
            });
        }
        
        // Complexity trend
        let complexity_trend = self.linear_regression(&metrics.iter().map(|m| m.complexity).collect::<Vec<_>>());
        if complexity_trend.slope > 0.02 {
            trends.push(Trend::Increasing {
                metric: "complexity",
                rate: complexity_trend.slope,
                severity: if complexity_trend.slope > 0.1 { "high" } else { "medium" },
            });
        }
        
        Ok(trends)
    }
}
```

### Integrazione con Git

```rust
impl DriftDetector {
    /// Analizza drift da git history
    pub async fn analyze_git_drift(&self, repo_path: &str, branch: &str) -> Result<GitDriftReport> {
        // 1. Get commits
        let commits = self.git_log(repo_path, branch, Duration::from_days(30))?;
        
        // 2. For each file changed, analyze drift
        let mut file_drifts = HashMap::new();
        for commit in &commits {
            for file in &commit.changed_files {
                let drift = self.analyze_file_drift(repo_path, file, &commits).await?;
                file_drifts.insert(file.clone(), drift);
            }
        }
        
        // 3. Aggregate
        let overall_drift = self.aggregate_drift(&file_drifts);
        
        Ok(GitDriftReport {
            branch: branch.to_string(),
            time_window: Duration::from_days(30),
            commits_analyzed: commits.len(),
            overall_drift_score: overall_drift,
            files_at_risk: file_drifts.into_iter()
                .filter(|(_, d)| d.drift_score > 0.5)
                .map(|(f, d)| (f, d))
                .collect(),
        })
    }
}
```

---

## Integrazione Completa

### Pipeline di Validazione AI

```rust
impl AetherIntelligence {
    pub async fn validate(&self, request: ValidationRequest) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        
        // Layer 1: Static Analysis
        let static_result = self.static_analysis.validate(&request.code)?;
        result.merge(static_result);
        
        // Layer 2: Semantic Memory
        let memories = self.semantic_memory.recall_similar(&request.code, 5).await?;
        result.add_context(memories);
        
        // Layer 3: Pattern Discovery
        let discovered_patterns = self.pattern_discovery.check_patterns(&request.code).await?;
        result.add_discovered_patterns(discovered_patterns);
        
        // Layer 4: Intent Inference
        let intent = self.intent_inference.infer_intent(&request.code, &request.context).await?;
        result.set_intent(intent.clone());
        
        // Layer 5: Drift Detection
        if let Some(history) = &request.code_history {
            let drift = self.drift_detector.analyze_drift(&request.file_path, history).await?;
            result.add_drift_analysis(drift);
        }
        
        // Store in memory
        self.semantic_memory.remember(MemoryContent {
            code: request.code,
            result: result.clone(),
            context: request.context,
        }).await?;
        
        Ok(result)
    }
}
```

### Output Esempio

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                        AETHER INTELLIGENCE REPORT                          ║
╠═══════════════════════════════════════════════════════════════════════════╣
║ File: src/api/handlers.rs                                                  ║
║ Language: Rust                                                             ║
║ Last Modified: 2026-03-15 (3 days ago)                                     ║
╠═══════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║ STATIC ANALYSIS (Layer 1)                                                  ║
║ ────────────────────────                                                   ║
║ ⚠️  LOGIC084: File handle leak - open() without close()                    ║
║     Line 45: let file = open(path)?;                                       ║
║     Suggestion: Use `fs::read_to_string()` or ensure Drop                  ║
║                                                                            ║
║ SEMANTIC MEMORY (Layer 2)                                                  ║
║ ────────────────────────                                                   ║
║ 📝 Similar error seen 3 times in this project                              ║
║    Last: 2026-03-10 in src/utils/io.rs                                     ║
║    Your fix was: "Used BufReader with explicit drop"                       ║
║                                                                            ║
║ PATTERN DISCOVERY (Layer 3)                                                ║
║ ────────────────────────                                                   ║
║ 🔍 New pattern detected: "HTTP handler without timeout"                    ║
║    Confidence: 87%                                                         ║
║    Similar code in: src/api/auth.rs, src/api/users.rs                      ║
║    Recommended: Add .timeout(Duration::from_secs(30))                      ║
║                                                                            ║
║ INTENT INFERENCE (Layer 4)                                                 ║
║ ────────────────────────                                                   ║
║ 💡 Intent: "Read configuration file and parse as JSON"                     ║
║    Invariants:                                                             ║
║    - File must be valid UTF-8                                              ║
║    - JSON must contain "api_key" field                                     ║
║    Side effects: None                                                      ║
║                                                                            ║
║ DRIFT DETECTION (Layer 5)                                                  ║
║ ────────────────────────                                                   ║
║ 📊 Drift Score: 0.23 (Low)                                                 ║
║    Type Strictness: 94% → 92% (slight decline)                             ║
║    Complexity: Stable                                                      ║
║    ⚠️  Trend: Error handling eroding (from Result<T, E> to Result<T, Box<dyn Error>>)  ║
║                                                                            ║
╠═══════════════════════════════════════════════════════════════════════════╣
║ RECOMMENDATIONS                                                            ║
║                                                                            ║
║ 1. [HIGH] Fix file handle leak (matches your previous fix pattern)         ║
║    Apply: Use fs::read_to_string() instead of open()                       ║
║    Confidence: 95% (based on your preference history)                      ║
║                                                                            ║
║ 2. [MEDIUM] Add timeout to HTTP handler                                    ║
║    New pattern detected in your codebase                                   ║
║                                                                            ║
║ 3. [LOW] Consider reverting to specific error types                        ║
║    Drift detected: Error handling becoming generic                         ║
║                                                                            ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

---

## Roadmap Implementativa

### Phase 11: Memory System Foundation (3 settimane) ✅ COMPLETATA

- [x] Layer 2A: Code Graph (AST-based)
  - [x] `CodeNode`, `CodeEdge`, `CodeGraph` structs
  - [x] `who_calls()`, `what_depends_on()`, `impact_analysis()`
  - [x] Integration con parser esistenti
- [x] Layer 2C: Validation State (File-based)
  - [x] `ProjectState`, `FileState`, `ViolationRecord`
  - [x] TOML persistence (human-readable, editable)
  - [x] `is_accepted()`, `compute_delta()`

### Phase 12: Pattern Discovery MVP (3 settimane) ✅ COMPLETATA

- [x] Implementare `CodeFeatures` extraction
- [x] Setup clustering (DBSCAN)
- [x] Implementare anomaly detection
- [x] Creare pipeline di rule generation
- [x] 18/18 test passanti

### Phase 13: Intent Inference (3 settimane) ✅ COMPLETATA

- [x] Integrare modello LLM lightweight
- [x] Implementare prompt engineering per intent
- [x] Creare cache per performance
- [x] Integrare con fix suggestions
- [x] 23/23 test passanti

### Phase 14: Drift Detection (2 settimane) ✅ COMPLETATA

- [x] Implementare metriche temporali
- [x] Git history integration
- [x] CLI command: `aether drift`
- [x] Trend analysis
- [x] Layer 2D: Drift Snapshots structure

### Phase 15: Memory System Enhancement (3 settimane) 🔄 IN CORSO

- [x] Layer 2B: Decision Log (Knowledge Graph)
  - [x] `DecisionNode`, `DecisionEdge`, `DecisionLog` structs
  - [x] `why_exists()`, `is_accepted()`, `recall_semantic()`
  - [x] `aether recall` CLI command
- [x] Layer 2D: Drift Snapshots (Time-series)
  - [x] Time-series storage
  - [x] `SnapshotMetrics`, `CodeSnapshot`, `Trend`, `DriftReport`
  - [x] `analyze_trend_days()` con regression
  - [x] Alert thresholds configurabili
  - [x] `DriftSnapshotStore` integrato in `AetherIntelligence`
- [x] API Unificata
  - [x] `MemoryQuery` enum con `DriftTrend`, `WhyExists`, `IsAccepted`
  - [x] `aether memory recall` CLI command
  - [x] Integration con validation pipeline
- [x] Architectural Drift Analysis (enhancement)
  - [x] Dependency expansion via CodeGraph
  - [x] Multi-file drift correlation
  - [ ] Module grouping (auto-discovery + manual config)
- [ ] Integration tests end-to-end

### Phase 16: Memory-Driven Core (4 settimane) 🔄 IN CORSO

- [x] Documento architettura: MEMORY_DRIVEN_CORE.md
- [ ] LearnedConfig struct
- [ ] MemoryStore::load_config() integration
- [ ] ValidationPipeline::apply_learned_config()
- [ ] Threshold adaptation (Syntax Layer)
- [ ] Dynamic whitelist (Security Layer)
- [ ] Style conventions learning
- [ ] CLI: `aether memory config show/stats/rules`

### Phase 17: Integration & Polish (2 settimane)

- [ ] Unificare tutti i layers
- [ ] Ottimizzare performance
- [ ] Documentazione completa
- [ ] Test end-to-end
- [ ] Release v0.2.0

**Totale:** ~18 settimane (15 completate, 3 rimanenti)

---

## Fonti e Riferimenti

### Context Rot Research

| Fonte | Titolo | Data |
|-------|--------|------|
| Chroma | "Solving the Context Rot Problem for Coding Agents" | 2026 |
| Manifold Group | "The Memory Problem: What Nobody Tells You About AI Agents in Production" | Mar 2026 |
| AAAI | ChatGPT effective memory study | 2025 |
| Anthropic | Context rot research | 2025 |

### Key Findings

1. **Context Window ≠ Memory**: Claude Sonnet 4: 99% → 50% accuracy as context grows
2. **Effective Memory**: ~7±2 items (human-like) despite 128K token windows
3. **n² Attention Problem**: Every token adds pairwise relationships that compete
4. **RAG Fragility**: Long chain of operations, fails silently
5. **Solution**: Hybrid memory (Knowledge Graph + File-based + Time-series)

### Approaches Validated

| Approach | Accuracy | Source |
|----------|----------|--------|
| Observational Memory | 94.87% | LongMemEval benchmark |
| Knowledge Graph | High for relations | Manifold Group |
| File-based Memory | Recommended for coding | Anthropic, Letta |
| Vector Store | Baseline | Standard RAG |

---

## Requisiti Tecnici

### Dependencies

```toml
[dependencies]
# Esistenti
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

# Nuove per AI
candle-core = "0.4"           # ML framework
candle-nn = "0.4"             # Neural networks
candle-transformers = "0.4"   # Pre-trained models
tokenizers = "0.15"           # Tokenization
ndarray = "0.15"              # Numerical computing
linfa = "0.7"                 # Clustering/ML
lru = "0.12"                  # Cache
git2 = "0.18"                 # Git integration
```

### Hardware Requirements

| Layer | CPU | RAM | GPU (optional) |
|-------|-----|-----|----------------|
| Layer 1-2 | Any | 512MB | None |
| Layer 3 | 4+ cores | 2GB | None |
| Layer 4 | 4+ cores | 4GB | 4GB VRAM (for local LLM) |
| Layer 5 | 2+ cores | 1GB | None |

**Note:** Layer 4 (Intent Inference) può usare API esterne invece di modello locale per ridurre requisiti.

---

## Metriche di Successo

| Metrica | Target | Misura |
|---------|--------|--------|
| Falsi positivi | < 5% | User feedback rate |
| Coverage pattern discovery | > 50% new patterns | Discovered vs manual rules |
| Intent preservation | > 95% | Fix correctness |
| Drift detection accuracy | > 80% | Manual verification |
| Performance | < 500ms | Full pipeline (no LLM) |
| Performance (with LLM) | < 2s | Full pipeline |

---

## Knowledge Strategy: Cosa Aether Può Imparare vs Cosa Serve Knowledge Base

### Auto-Discovered (Layer 3 Pattern Discovery)

Questi pattern vengono scoperti automaticamente dal codice che Aether vede:

| Tipo | Esempio | Metodo |
|------|---------|--------|
| **Pattern sintattici** | `unwrap()` senza gestione | Clustering + Anomaly |
| **Code smells** | Funzioni troppo lunghe | Feature extraction |
| **Anti-pattern logici** | Loop in loop O(n²) | Control flow analysis |
| **Inconsistenze stile** | snake_case/camelCase misti | Baseline comparison |
| **Errori ricorrenti** | Stesso errore 5 volte | Semantic Memory |

**Nessuna libreria da aggiungere** - Aether impara da solo.

### Knowledge Base Esterna (Necessaria)

Questi pattern richiedono conoscenza esterna:

| Tipo | Esempio | Perché |
|------|---------|--------|
| **API signatures** | `requests.get(url, timeout)` vs `requests.get(timeout, url)` | Non visibile nel codice |
| **Parametri corretti** | Quale parametro va dove | Context esterno |
| **Side effects nascosti** | Funzione muta stato | Semantica libreria |
| **Best practices** | React hooks rules | Convenzioni |

### Strategia Ibrida: Type Stubs + LLM

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AETHER KNOWLEDGE SOURCES                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                  TYPE STUBS (Conosciute)                             │   │
│  │                                                                      │   │
│  │  Python: typeshed/ (stub ufficiali)                                  │   │
│  │  ├── stdlib/ (os, sys, json, etc.)                                  │   │
│  │  ├── third_party/ (numpy, pandas, requests, flask, etc.)           │   │
│  │  └── Pipfile: types-requests, types-redis, etc.                     │   │
│  │                                                                      │   │
│  │  TypeScript: DefinitelyTyped/                                       │   │
│  │  ├── @types/react, @types/node, etc.                               │   │
│  │  └── .d.ts files nel progetto                                       │   │
│  │                                                                      │   │
│  │  Rust: rust-analyzer metadata                                       │   │
│  │  ├── std, crates.io metadata                                        │   │
│  │  └── rustdoc JSON output                                            │   │
│  │                                                                      │   │
│  │  C++: clangd compilation database                                   │   │
│  │  └── compile_commands.json                                          │   │
│  │                                                                      │   │
│  │  Formato: .pyi, .d.ts, JSON metadata                                │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                  LLM CONTEXT (Sconosciute)                           │   │
│  │                                                                      │   │
│  │  Quando: Type stub non disponibile                                   │   │
│  │  Metodo: Query LLM per API signature                                │   │
│  │  Cache: Salva risposta per riuso                                    │   │
│  │                                                                      │   │
│  │  Esempio:                                                            │   │
│  │  User: some_obscure_lib.process(data, config)                       │   │
│  │  Aether: "Non ho stub per some_obscure_lib"                         │   │
│  │         → Query LLM: "What are the parameters of process()?"        │   │
│  │         → Cache: some_obscure_lib.process(data, config)             │   │
│  │         → Check: param order correct?                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                  API SIGNATURE DATABASE (Locale)                     │   │
│  │                                                                      │   │
│  │  Formato YAML:                                                       │   │
│  │  ```yaml                                                             │   │
│  │  python:                                                             │   │
│  │    requests:                                                         │   │
│  │      get:                                                            │   │
│  │        params: [url, params?, data?, json?, timeout?]               │   │
│  │        return: Response                                              │   │
│  │      post:                                                           │   │
│  │        params: [url, data?, json?, timeout?]                        │   │
│  │        return: Response                                              │   │
│  │  ```                                                                 │   │
│  │                                                                      │   │
│  │  Fonti:                                                              │   │
│  │  - Generated from type stubs                                         │   │
│  │  - Manual curation for critical APIs                                │   │
│  │  - LLM-learned and cached                                            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Implementazione Type Stub Integration

```rust
pub struct TypeStubLoader {
    /// Cache delle signatures caricate
    signatures: HashMap<String, ApiSignature>,
    
    /// Type stub directories
    stub_paths: Vec<PathBuf>,
}

pub struct ApiSignature {
    pub module: String,
    pub function: String,
    pub params: Vec<ParamInfo>,
    pub return_type: String,
    pub raises: Vec<String>,
    pub deprecated: bool,
}

pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
    pub optional: bool,
    pub default: Option<String>,
    pub position: usize,  // 0-based
}

impl TypeStubLoader {
    /// Carica stub Python (.pyi)
    pub fn load_python_stubs(&mut self, path: &Path) -> Result<()> {
        for entry in walkdir::WalkDir::new(path) {
            if let Some(ext) = entry.path().extension() {
                if ext == "pyi" {
                    self.parse_python_stub(entry.path())?;
                }
            }
        }
        Ok(())
    }
    
    /// Parse .pyi file
    fn parse_python_stub(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let module = self.extract_module_name(path);
        
        // Parse function signatures
        for line in content.lines() {
            if let Some(sig) = self.parse_function_line(line, &module)? {
                let key = format!("{}.{}", sig.module, sig.function);
                self.signatures.insert(key, sig);
            }
        }
        Ok(())
    }
    
    /// Verifica chiamata API
    pub fn check_api_call(&self, module: &str, function: &str, args: &[Arg]) -> Result<ApiCheckResult> {
        let key = format!("{}.{}", module, function);
        
        if let Some(sig) = self.signatures.get(&key) {
            // Check parameter order
            for (i, arg) in args.iter().enumerate() {
                if let Some(expected) = sig.params.get(i) {
                    if arg.name.is_none() && expected.optional {
                        // Positional arg for optional param - OK
                    } else if let Some(name) = &arg.name {
                        // Named arg - verify exists
                        if !sig.params.iter().any(|p| &p.name == name) {
                            return Ok(ApiCheckResult::UnknownParam(name.clone()));
                        }
                    }
                }
            }
            Ok(ApiCheckResult::Valid)
        } else {
            Ok(ApiCheckResult::NoSignature(key))
        }
    }
}
```

### LLM Fallback per API Sconosciute

```rust
pub struct LlmApiResolver {
    /// Modello LLM locale o API
    model: Box<dyn LlmProvider>,
    
    /// Cache delle risposte
    cache: LruCache<String, ApiSignature>,
}

impl LlmApiResolver {
    /// Risolve API signature sconosciuta
    pub async fn resolve(&mut self, module: &str, function: &str) -> Result<Option<ApiSignature>> {
        let cache_key = format!("{}.{}", module, function);
        
        // Check cache
        if let Some(sig) = self.cache.get(&cache_key) {
            return Ok(Some(sig.clone()));
        }
        
        // Query LLM
        let prompt = format!(
            r#"What are the function signature and parameter order for {}::{}?

Output JSON:
{{
  "params": [{{"name": "...", "type": "...", "optional": true/false}}],
  "return_type": "...",
  "raises": ["..."]
}}

If unknown, output: {{"unknown": true}}"#,
            module, function
        );
        
        let response = self.model.generate(&prompt).await?;
        
        if response.contains("unknown") {
            return Ok(None);
        }
        
        let sig: ApiSignature = serde_json::from_str(&response)?;
        
        // Cache
        self.cache.put(cache_key.clone(), sig.clone());
        
        Ok(Some(sig))
    }
}
```

### Integrazione nel Validation Pipeline

```rust
impl ValidationEngine {
    pub fn check_api_usage(&self, call: &FunctionCall) -> Result<Vec<Violation>> {
        let mut violations = vec![];
        
        // 1. Check type stubs
        match self.stub_loader.check_api_call(&call.module, &call.function, &call.args)? {
            ApiCheckResult::Valid => {},
            ApiCheckResult::UnknownParam(name) => {
                violations.push(Violation::unknown_param(&call, name));
            }
            ApiCheckResult::NoSignature(key) => {
                // 2. Fallback to LLM
                if let Some(sig) = self.llm_resolver.resolve(&call.module, &call.function).await? {
                    // Re-check with LLM-learned signature
                    violations.extend(self.check_against_signature(call, &sig));
                }
            }
        }
        
        Ok(violations)
    }
}
```

### Fonti Type Stubs

| Linguaggio | Repository | Dimensione |
|------------|------------|------------|
| **Python** | [typeshed](https://github.com/python/typeshed) | ~500 packages |
| **TypeScript** | [DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped) | ~8000 packages |
| **Rust** | crates.io metadata + rustdoc | Tutti i crate |
| **C++** | compile_commands.json | Project-specific |

### Dipendenze Aggiuntive

```toml
[dependencies]
# Type stub parsing
pyo3 = { version = "0.20", optional = true }  # Python stub parsing
swc_ecma_parser = { version = "0.140", optional = true }  # TypeScript .d.ts

# LLM integration (per Layer 4 + API resolution)
candle-core = "0.4"
candle-nn = "0.4"
candle-transformers = "0.4"  # Per modello locale

# Oppure API esterne
reqwest = { version = "0.11", features = ["json"] }  # OpenAI/Anthropic API
```

---

## Conclusione

Aether Intelligence rappresenta l'evoluzione naturale del validatore: da strumento passivo che applica regole a sistema attivo che **impara**, **capisce**, e **previene** problemi.

I 5 layers lavorano insieme:
1. **Layer 1** garantisce baseline di qualità
2. **Layer 2** ricorda il passato per evitare ripetizioni
3. **Layer 3** scopre nuovi problemi automaticamente
4. **Layer 4** capisce l'intento per preservarlo
5. **Layer 5** rileva degrado temporale

Il risultato è un "guardiano" che cresce con il progetto, diventando sempre più efficace nel proteggere la qualità del codice.

---

**Prossimo passo:** Implementare Phase 11 (Semantic Memory Enhancement)
