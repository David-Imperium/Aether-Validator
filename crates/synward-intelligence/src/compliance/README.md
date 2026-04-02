# Compliance Engine Module

Synward's compliance system provides intelligent contract enforcement with context-aware learning and tiered enforcement levels.

## Architecture

```
                       +------------------------+
                       |   ComplianceEngine     |
                       |    (orchestrator)      |
                       +-----------+------------+
                                   |
              +--------------------+--------------------+
              |                    |                    |
         +----v----+         +-----v-----+        +----v----+
         |Classifier|        | Exemptions|        | Decision|
         | (tier)   |        | (learned) |        | (action)|
         +----------+        +-----------+        +---------+
              |                    |                    |
              +--------------------+--------------------+
                                   |
                       +-----------v------------+
                       |    Dubbioso Mode       |
                       | (low-confidence cases) |
                       +------------------------+

Enforcement Flow:
┌─────────────────────────────────────────────────────────────┐
│  1. Classify → Inviolable? → BLOCK (non-negotiable)         │
│  2. Exemption? → ACCEPT (precedent-based)                   │
│  3. Occurrences >= threshold? → LEARN (auto-pattern)        │
│  4. Low confidence? → ASK (Dubbioso mode)                   │
│  5. Default → WARN                                          │
└─────────────────────────────────────────────────────────────┘
```

## Components

### ComplianceEngine

Main orchestrator for intelligent contract enforcement:

```rust
use synward_intelligence::compliance::{
    ComplianceEngine, ComplianceConfig, ComplianceContext
};

let engine = ComplianceEngine::new()?;

// Evaluate a violation
let ctx = ComplianceContext {
    file_path: "src/main.rs".into(),
    line: 42,
    snippet: Some("unwrap()".into()),
    project_type: Some("cli".into()),
    code_region: Some("main".into()),
    function_context: None,
};

let decision = engine.evaluate("STYLE001", "style", "Line too long", &ctx).await?;

match decision.action {
    ComplianceAction::Block => { /* non-negotiable */ }
    ComplianceAction::Warn => { /* show warning */ }
    ComplianceAction::Ask { question, options } => { /* use Dubbioso */ }
    ComplianceAction::Learn { pattern, confidence } => { /* update patterns */ }
    ComplianceAction::Accept { reason, .. } => { /* proceed */ }
}
```

### ContractClassifier

Determines enforcement tier for rules:

```rust
use synward_intelligence::compliance::{ContractClassifier, ContractTier};

let classifier = ContractClassifier::new();

// Classify by rule ID and domain
let tier = classifier.classify("SEC001", "security");
assert_eq!(tier, ContractTier::Inviolable);

// Get metadata for explanation
let meta = classifier.get_metadata("STYLE001", "style");
println!("Tier: {:?}, Reason: {}", meta.tier, meta.reason);

// Add custom rules
let mut classifier = ContractClassifier::new();
classifier.add_inviolable_rule("CUSTOM_SEC".into());
classifier.add_flexible_rule("CUSTOM_STYLE".into());
```

### ExemptionStore

Manages learned exemptions for violations:

```rust
use synward_intelligence::compliance::{
    ExemptionStore, Exemption, ExemptionScope, ExemptionSource
};

let store = ExemptionStore::with_path(".synward/exemptions.json".into());

// Add user-created exemption
let exemption = Exemption::new(
    "STYLE001".into(),
    ExemptionScope::File { path: "src/main.rs".into() },
    "Project uses 120 char lines".into(),
    ExemptionSource::UserCreated,
);
store.add(exemption);

// Find matching exemption
if let Some(exemption) = store.find("STYLE001", "src/main.rs") {
    println!("Exemption found: {}", exemption.reason);
    store.record_application(&exemption.id);
}

// Get statistics
let stats = store.stats();
println!("Total: {}, Learned: {}", stats.total, stats.learned);
```

## Contract Tiers

### Inviolable

**Never bypassed.** Security, safety, and undefined behavior violations.

| Domain | Examples |
|--------|----------|
| Security | SQL injection (SEC001-SEC004), XSS, command injection |
| Memory Safety | Use-after-free (MEM001), buffer overflow (MEM002) |
| Supply Chain | Malicious dependencies (SUPP001-SUPP005) |
| Safety-Critical | Undefined behavior, data races |

```rust
// Inviolable rules always result in Block
assert!(!ContractTier::Inviolable.is_bypassable());
assert!(!ContractTier::Inviolable.supports_learning());
```

### Strict

**Requires explicit acceptance with documented reason.** Logic errors, resource management.

| Domain | Examples |
|--------|----------|
| Logic | Race conditions, incorrect algorithms |
| Error Handling | Missing error cases, swallowed errors |
| Resource Management | Leaks, unclosed handles |
| Concurrency | Deadlock potential |

```rust
// Strict rules can be bypassed with explicit acceptance
assert!(ContractTier::Strict.is_bypassable());
assert!(!ContractTier::Strict.supports_learning());
```

### Flexible

**Can be learned/auto-accepted based on project patterns.** Style, naming, formatting.

| Domain | Examples |
|--------|----------|
| Style | Line length (STYLE001), indentation |
| Naming | Convention adherence (NAME001-NAME002) |
| Formatting | Import order, brace style |
| Documentation | Comment style, doc format |

```rust
// Flexible rules support learning from patterns
assert!(ContractTier::Flexible.is_bypassable());
assert!(ContractTier::Flexible.supports_learning());
```

## Integration with Dubbioso Mode

When confidence is low (`< 0.60`), the Compliance Engine triggers **Dubbioso Mode**:

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
└─────────────────────────────────────────────────────────────────┘
```

## Configuration

```toml
# .synward/compliance.toml
[compliance]
auto_accept_threshold = 0.90    # Confidence to auto-accept
ask_threshold = 0.60            # Confidence below which to ask
learn_after_occurrences = 3     # Occurrences before learning
use_dubbioso = true             # Enable Dubbioso integration
exemption_store_path = ".synward/exemptions.json"
```

## Learning Behavior

| Occurrences | Action |
|-------------|--------|
| 1-2 | Warn, track occurrence |
| 3+ | Auto-learn, create exemption |
| 5+ | Increase confidence, suggest scope expansion |

**Example:**

```
Day 1:  STYLE002 in test_auth.rs → Warn
Day 2:  STYLE002 in test_user.rs → Warn
Day 3:  STYLE002 in test_api.rs → Learn! Create exemption for *test*
Day 4:  STYLE002 in test_handler.rs → Accept (from learned pattern)
```

## Statistics

```rust
let stats = engine.stats();
println!("Total exemptions: {}", stats.exemptions.total);
println!("Learned patterns: {}", stats.exemptions.learned);
println!("User-created: {}", stats.exemptions.user_created);
```
