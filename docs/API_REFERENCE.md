# Synward Intelligence API Reference

**Version:** 0.2.0
**Status:** Layers 1-5 Complete
**Last Updated:** 2026-03-16

---

## Feature Flags

```toml
[dependencies.synward-intelligence]
version = "0.2"
default-features = false
features = [
    "memory",      # Layer 2: Code Graph, Decision Log, Validation State
    "patterns",    # Layer 3: Feature Extraction, Anomaly Detection
    "intent-api",  # Layer 4: LLM Intent Inference
    "drift",       # Layer 5: Git-based Drift Detection
]
```

| Flag | Description | Dependencies |
|------|-------------|--------------|
| `memory` (default) | Semantic memory system | uuid, chrono, serde |
| `patterns` | Rule-based pattern discovery | No extra deps |
| `intent-api` | External LLM API integration | reqwest, tokio |
| `drift` | Git-based drift detection | git2 (optional) |

---

## Quick Start

```rust
use synward_intelligence::{SynwardIntelligence, Config, MemoryQuery};

// Initialize with config
let config = Config::default();
let ai = SynwardIntelligence::new(config)?;

// Query memory (Layer 2)
let result = ai.recall(MemoryQuery::WhyExists {
    file: "src/main.rs".into(),
    line: 42,
})?;

// Index project for Code Graph
ai.index_project(&PathBuf::from("./src"))?;
```

---

## Layer 2: Memory System

### Layer 2A: Code Graph

**Purpose:** AST-based structural queries (who calls what, dependencies).

```rust
use synward_intelligence::memory::{CodeGraph, CodeNode, CodeNodeType, ImpactResult};

// Create and populate
let mut graph = CodeGraph::new();

// Parse files (auto-detect language from extension)
graph.parse_file(&rust_code, "src/main.rs", "rust");
graph.parse_file(&python_code, "lib/utils.py", "python");

// Build reverse indices (callers, dependencies)
graph.build_callers();

// Query: Who calls this function?
let callers = graph.who_calls("process_data", "src/main.rs");
for node in callers {
    println!("Called by: {} at {}:{}", node.name, node.file, node.line);
}

// Query: Impact analysis
let impact = graph.impact_analysis("process_data", "src/main.rs");
println!("Direct callers: {}", impact.direct_callers);
println!("Files affected: {:?}", impact.affected_files);
```

**Key Types:**

| Type | Description |
|------|-------------|
| `CodeGraph` | Main graph structure |
| `CodeNode` | Function, class, method, module |
| `CodeNodeType` | `Function`, `Method`, `Class`, `Module`, `File` |
| `ImpactResult` | Impact analysis result |

### Layer 2B: Decision Log

**Purpose:** Knowledge graph for "why" questions.

```rust
use synward_intelligence::memory::{
    DecisionLog, DecisionNode, DecisionType, DecisionAuthor, DecisionStatus
};

let mut log = DecisionLog::new(None)?;

// Record a decision
let decision = DecisionNode {
    id: MemoryId::default(),
    decision_type: DecisionType::AcceptedViolation,
    content: "This unwrap is safe: config file always exists in production".into(),
    author: DecisionAuthor::User("David".into()),
    location: CodeLocation { file: "src/config.rs".into(), line: 42 },
    timestamp: Utc::now(),
    status: DecisionStatus::Active,
    related: vec![],
};
log.record(decision)?;

// Query: Why does this code exist?
let reasons = log.why_exists("src/config.rs", 42);

// Query: Is this violation accepted?
let accepted = log.is_accepted("LOGIC042");
```

**Decision Types:**

| Type | Use Case |
|------|----------|
| `IntentDeclaration` | "Why this code exists" |
| `AcceptedViolation` | "This violation is ok because..." |
| `PatternApproval` | "This pattern is project style" |
| `RefactorReason` | "Refactored because..." |
| `DoNotTouch` | "Don't modify this code" |
| `TechnicalDebt` | "Will be removed in future" |

### Layer 2C: Validation State

**Purpose:** File-based persistence for validation state.

```rust
use synward_intelligence::memory::{
    ValidationState, ProjectState, FileState, ViolationRecord, AcceptedViolation
};

let mut state = ValidationState::new(None)?;

// Get or create project state
let project = state.get_project(&PathBuf::from("."));

// Mark violation as accepted
project.accept_violation("LOGIC042", "Safe in this context".into(), "David");

// Save state
state.save(project)?;

// Check if accepted
if project.is_accepted("LOGIC042", "src/config.rs") {
    println!("Violation is accepted");
}

// Compute delta from previous validation
let delta = project.compute_file_delta("src/main.rs", &new_hash);
```

### Unified Memory API

```rust
use synward_intelligence::MemoryQuery;

// Who calls this function?
let result = ai.recall(MemoryQuery::WhoCalls {
    function: "process_data".into(),
    file: "src/main.rs".into(),
})?;

// Why does this code exist?
let result = ai.recall(MemoryQuery::WhyExists {
    file: "src/config.rs".into(),
    line: 42,
})?;

// Is this violation accepted?
let result = ai.recall(MemoryQuery::IsAccepted {
    violation_id: "LOGIC042".into(),
    file: "src/config.rs".into(),
})?;

// Semantic search in memory
let result = ai.recall(MemoryQuery::SemanticRecall {
    query: "unwrap without error handling".into(),
    limit: 5,
})?;

// Drift trend analysis
let result = ai.recall(MemoryQuery::DriftTrend {
    file: Some("src/main.rs".into()),
    days: 30,
})?;

// Impact analysis
let result = ai.recall(MemoryQuery::ImpactAnalysis {
    file: "src/main.rs".into(),
    function: "process_data".into(),
})?;
```

---

## Layer 3: Pattern Discovery

### Feature Extraction

```rust
use synward_intelligence::patterns::{CodeFeatures, FeatureExtractor};

let extractor = FeatureExtractor::new();

// Auto-detect language
let features = extractor.extract(&code, "auto");

// Or specify language
let features = extractor.extract(&code, "rust");

// Available features
println!("Lines: {}", features.line_count);
println!("Functions: {}", features.function_count);
println!("Cyclomatic complexity: {}", features.cyclomatic_complexity);
println!("unwrap() count: {}", features.unwrap_count);
println!("TODO count: {}", features.todo_count);
println!("panic! count: {}", features.panic_count);
```

**CodeFeatures Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `line_count` | usize | Total lines |
| `function_count` | usize | Number of functions |
| `cyclomatic_complexity` | usize | McCabe complexity |
| `unwrap_count` | usize | `unwrap()` calls |
| `expect_count` | usize | `expect()` calls |
| `panic_count` | usize | `panic!` macros |
| `todo_count` | usize | `TODO` comments |
| `unsafe_count` | usize | `unsafe` blocks |
| `error_type` | Option<String> | Error type used |

### Anomaly Detection

```rust
use synward_intelligence::patterns::{AnomalyDetector, Anomaly, AnomalyType};

let detector = AnomalyDetector::new();
let anomalies = detector.detect(&features);

for anomaly in anomalies {
    println!("{:?}: {} (severity: {:?})", 
        anomaly.anomaly_type, 
        anomaly.message,
        anomaly.severity
    );
}
```

**Anomaly Types:**

| Type | Description |
|------|-------------|
| `HighComplexity` | Cyclomatic complexity > 20 |
| `ExcessiveUnwrap` | Too many `unwrap()` calls |
| `MissingErrorHandling` | No error handling found |
| `UnsafeBlock` | `unsafe` code detected |
| `MissingDocs` | Public items without docs |

### Rule Generation

```rust
use synward_intelligence::patterns::{RuleGenerator, CandidateRule};

let generator = RuleGenerator::new();
let rules = generator.generate(&features, &anomalies);

for rule in rules {
    println!("Rule: {} (confidence: {})", rule.id, rule.confidence);
    println!("Description: {}", rule.description);
    println!("Pattern: {}", rule.pattern);
}
```

---

## Layer 4: Intent Inference

**Feature flag:** `intent-api`

```rust
use synward_intelligence::{Intent, IntentInferrer};

// Create with endpoint
let inferrer = IntentInferrer::new(Some("http://localhost:8080/api".into()));

// Check if configured
if inferrer.is_configured() {
    // Infer intent (async)
    let intent = inferrer.infer(&code).await?;
    
    println!("Summary: {}", intent.summary);
    println!("Purpose: {}", intent.purpose);
    println!("Invariants: {:?}", intent.invariants);
    println!("Side effects: {:?}", intent.side_effects);
    println!("Dependencies: {:?}", intent.dependencies);
    println!("Confidence: {}", intent.confidence);
}

// With context
let intent = inferrer.infer_with_context(&code, &context).await?;
```

**Intent Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `summary` | String | One-line description |
| `purpose` | String | What the code achieves |
| `invariants` | Vec<String> | Conditions maintained |
| `side_effects` | Vec<String> | External effects |
| `dependencies` | Vec<String> | What code relies on |
| `confidence` | f32 | 0.0 - 1.0 |

---

## Layer 5: Drift Detection

**Feature flag:** `drift`

```rust
use synward_intelligence::drift::{DriftDetector, DriftMetrics, DriftReport, Trend};

// Create detector
let detector = DriftDetector::new(Some(PathBuf::from(".")))?;

// Analyze single file
let report = detector.analyze_file("src/main.rs")?;
println!("Drift score: {}", report.drift_score);
println!("Trends: {:?}", report.trends);

// Analyze entire project
let report = detector.analyze_project()?;
for (file, drift) in &report.files {
    println!("{}: drift={}", file, drift.drift_score);
}

// Load from git history (requires git2 feature)
let snapshots = detector.load_from_git(30)?;
println!("Loaded {} snapshots from last 30 commits", snapshots.len());
```

**DriftMetrics Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `score` | f32 | Overall drift score (0.0 - 1.0) |
| `rate` | f32 | Rate of change |
| `confidence` | f32 | Detection confidence |

**Trend Types:**

| Trend | Description |
|-------|-------------|
| `Declining` | Quality degrading |
| `Increasing` | Quality improving |
| `Stable` | No significant change |

---

## Knowledge System

### Type Stub Loading

```rust
use synward_intelligence::knowledge::{TypeStubLoader, ApiSignature};

let mut loader = TypeStubLoader::new();

// Load Python type stubs
loader.load_python_stubs(&PathBuf::from("typeshed/stdlib"))?;

// Check API call
let result = loader.check_api_call("requests", "get", &args)?;
match result {
    ApiCheckResult::Valid => println!("API call is valid"),
    ApiCheckResult::UnknownParam(name) => println!("Unknown param: {}", name),
    ApiCheckResult::NoSignature(key) => println!("No signature for: {}", key),
}
```

---

## CLI Commands

### Recall

```bash
# Who calls this function?
synward recall who-calls --function process_data --file src/main.rs

# Why does this code exist?
synward recall why-exists --file src/config.rs --line 42

# Is violation accepted?
synward recall is-accepted --violation LOGIC042 --file src/config.rs

# Semantic search
synward recall search "unwrap without error handling"

# Drift trend
synward recall drift --file src/main.rs --days 30

# Impact analysis
synward recall impact --file src/main.rs --function process_data
```

### Drift

```bash
# Analyze drift for project
synward drift analyze

# Analyze specific file
synward drift analyze --file src/main.rs

# Load from git history
synward drift load-git --commits 30
```

---

## Error Handling

```rust
use synward_intelligence::{Error, Result};

match ai.recall(query) {
    Ok(result) => { /* use result */ },
    Err(Error::Io(e)) => eprintln!("IO error: {}", e),
    Err(Error::Json(e)) => eprintln!("JSON error: {}", e),
    Err(Error::Config(msg)) => eprintln!("Config error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Feature Flag Matrix

| Feature | CLI Command | Methods Available |
|---------|-------------|-------------------|
| `memory` | `recall` | `who_calls`, `why_exists`, `is_accepted`, `semantic_recall`, `impact_analysis` |
| `patterns` | (internal) | `FeatureExtractor`, `AnomalyDetector`, `RuleGenerator` |
| `intent-api` | (internal) | `IntentInferrer::infer()` |
| `drift` | `drift` | `analyze_file`, `analyze_project`, `load_from_git` |
