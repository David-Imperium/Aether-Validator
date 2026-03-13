# Aether Pre-Guidance — Preventive Context System

**Version:** 0.1.0
**Status:** Planned
**Related:** [AETHER_RAG.md](./AETHER_RAG.md), [AETHER_LEARNER.md](./AETHER_LEARNER.md)

---

## Overview

Aether Pre-Guidance è il sistema che **guida l'agente PRIMA che scriva codice**, riducendo drasticamente le iterazioni necessarie.

### Il Problema

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    APPROCCIO TRADIZIONALE (LENTO)                          │
│                                                                              │
│  User: "Add enemy patrol behavior"                                          │
│         ↓                                                                    │
│  Agent: Scrive codice con .unwrap()                                         │
│         ↓                                                                    │
│  Aether: ERRORE — RUST001: unwrap without context                          │
│         ↓                                                                    │
│  Agent: Corregge con .expect()                                              │
│         ↓                                                                    │
│  Aether: ERRORE — RUST003: clone non necessario                            │
│         ↓                                                                    │
│  Agent: Corregge con borrow                                                 │
│         ↓                                                                    │
│  Aether: ✅ OK                                                              │
│                                                                              │
│  Risultato: 3 iterazioni, tempo sprecato, frustrazione                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### La Soluzione

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    APPROCCIO PRE-GUIDANCE (VELOCE)                          │
│                                                                              │
│  User: "Add enemy patrol behavior"                                          │
│         ↓                                                                    │
│  Aether Pre-Guidance:                                                       │
│    • Analizza prompt → Intent: CREATE, Domain: gameplay                    │
│    • Cerca docs → Trova docs/Enemy.md, docs/AI.md                           │
│    • Controlla learner → Utente viola RUST001 5 volte                      │
│    • Genera contesto per l'agente:                                          │
│      ┌─────────────────────────────────────────────────────────────────┐  │
│      │ CONTEXT FOR AGENT:                                               │  │
│      │                                                                   │  │
│      │ Documentation:                                                   │  │
│      │ - docs/Enemy.md: "Enemies use state machine for behavior..."   │  │
│      │ - docs/AI.md: "Patrol follows waypoints..."                     │  │
│      │                                                                   │  │
│      │ User Preferences:                                                │  │
│      │ - User prefers ? operator over .expect()                        │  │
│      │ - User uses snake_case for functions                            │  │
│      │                                                                   │  │
│      │ Warnings (based on past violations):                            │  │
│      │ - RUST001: You often use .unwrap() — prefer ? operator          │  │
│      │ - RUST003: You often clone unnecessarily — check if & works     │  │
│      │                                                                   │  │
│      │ Relevant Patterns:                                               │  │
│      │ - User's preferred enemy implementation: [pattern from past]    │  │
│      └─────────────────────────────────────────────────────────────────┘  │
│         ↓                                                                    │
│  Agent: Scrive codice GIÀ CORRETTO                                          │
│         ↓                                                                    │
│  Aether: ✅ OK (validazione finale sicura)                                 │
│                                                                              │
│  Risultato: 1 iterazione, zero errori, massima velocità                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Architettura

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     AETHER PRE-GUIDANCE ARCHITECTURE                        │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        INPUT                                          │  │
│  │  • User prompt (string)                                              │  │
│  │  • Current context (files aperti, progetto)                          │  │
│  │  • Agent ID (per learner)                                            │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    STEP 1: PROMPT ANALYSIS                            │  │
│  │                                                                       │  │
│  │   PromptAnalyzer                                                      │  │
│  │   ├── IntentClassifier → CREATE, MODIFY, FIX, etc.                  │  │
│  │   ├── ScopeExtractor → FILE, FUNCTION, CLASS, MODULE               │  │
│  │   ├── DomainMapper → gameplay, ui, graphics, etc.                   │  │
│  │   └── AmbiguityDetector → unclear requirements                      │  │
│  │                                                                       │  │
│  │   Output: PromptAnalysis                                             │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    STEP 2: RAG SEARCH                                 │  │
│  │                                                                       │  │
│  │   RagEngine                                                            │  │
│  │   ├── KeywordIndex → Fast search in docs                             │  │
│  │   ├── SemanticSearch → Deep understanding                           │  │
│  │   └── PatternLibrary → User's learned patterns                      │  │
│  │                                                                       │  │
│  │   Output: Vec<SearchResult>                                          │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    STEP 3: LEARNER LOOKUP                             │  │
│  │                                                                       │  │
│  │   Learner                                                              │  │
│  │   ├── UserProfile → User preferences and style                       │  │
│  │   ├── StatsTracker → Common errors for this user                     │  │
│  │   ├── MemoryStore → Past corrections and lessons                     │  │
│  │   └── PatternExtractor → Relevant learned patterns                   │  │
│  │                                                                       │  │
│  │   Output: Vec<GuidanceHint>, Vec<LearnedPattern>                    │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    STEP 4: CONTEXT ASSEMBLY                           │  │
│  │                                                                       │  │
│  │   ContextBuilder                                                       │  │
│  │   ├── Combine all inputs                                              │  │
│  │   ├── Prioritize by relevance                                         │  │
│  │   ├── Format for agent                                                │  │
│  │   └── Add disclaimers                                                 │  │
│  │                                                                       │  │
│  │   Output: PreGuidanceContext                                         │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        OUTPUT                                         │  │
│  │                                                                       │  │
│  │   PreGuidanceContext {                                                │  │
│  │       prompt_analysis: PromptAnalysis,                               │  │
│  │       documentation: Vec<SearchResult>,                              │  │
│  │       user_hints: Vec<GuidanceHint>,                                 │  │
│  │       learned_patterns: Vec<LearnedPattern>,                         │  │
│  │       warnings: Vec<String>,                                          │  │
│  │       relevant_rules: Vec<RuleSummary>,                              │  │
│  │   }                                                                   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Componenti

### 1. PreGuidance (Core)

```rust
// crates/aether-wrapper/src/pre_guidance.rs
use aether_validation::prompt::{PromptAnalyzer, PromptAnalysis};
use aether_rag::{RagEngine, SearchResult};
use aether_learner::{Learner, GuidanceHint, LearnedPattern};

/// Contesto generato per l'agente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreGuidanceContext {
    /// Analisi del prompt.
    pub prompt_analysis: PromptAnalysis,
    
    /// Documentazione pertinente.
    pub documentation: Vec<SearchResult>,
    
    /// Suggerimenti basati sul profilo utente.
    pub user_hints: Vec<GuidanceHint>,
    
    /// Pattern imparati dall'utente.
    pub learned_patterns: Vec<LearnedPattern>,
    
    /// Warning critici (errori frequenti).
    pub warnings: Vec<String>,
    
    /// Regole rilevanti per il dominio.
    pub relevant_rules: Vec<RuleSummary>,
    
    /// Timestamp generazione.
    pub generated_at: chrono::DateTime<chrono::Utc>,
    
    /// Tempo di generazione (ms).
    pub generation_time_ms: u64,
}

/// Sistema di Pre-Guidance.
pub struct PreGuidance {
    /// Analizzatore di prompt.
    analyzer: PromptAnalyzer,
    
    /// Motore RAG.
    rag: RagEngine,
    
    /// Sistema di apprendimento.
    learner: Learner,
    
    /// Configurazione.
    config: PreGuidanceConfig,
}

impl PreGuidance {
    /// Genera contesto per l'agente.
    pub fn generate_context(&mut self, prompt: &str) -> Result<PreGuidanceContext, PreGuidanceError> {
        let start = std::time::Instant::now();
        
        // Step 1: Analizza prompt
        let analysis = self.analyzer.analyze(prompt)?;
        
        // Step 2: Cerca documentazione
        let query = self.build_search_query(&analysis);
        let documentation = self.rag.search(&query, self.config.max_docs)?;
        
        // Step 3: Ottieni hint dal learner
        let user_hints = self.learner.get_guidance_hints(&analysis.domain.primary)?;
        
        // Step 4: Ottieni pattern rilevanti
        let learned_patterns = self.learner.get_patterns_for_domain(&analysis.domain.primary)?;
        
        // Step 5: Genera warning
        let warnings = self.generate_warnings(&user_hints, &analysis);
        
        // Step 6: Ottieni regole rilevanti
        let relevant_rules = self.get_relevant_rules(&analysis)?;
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(PreGuidanceContext {
            prompt_analysis: analysis,
            documentation,
            user_hints,
            learned_patterns,
            warnings,
            relevant_rules,
            generated_at: chrono::Utc::now(),
            generation_time_ms: elapsed,
        })
    }
    
    /// Formatta il contesto per l'agente.
    pub fn format_for_agent(&self, context: &PreGuidanceContext) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str("# Aether Pre-Guidance Context\n\n");
        
        // Intent e dominio
        output.push_str(&format!(
            "**Intent:** {:?}\n**Domain:** {:?}\n**Scope:** {:?}\n\n",
            context.prompt_analysis.intent,
            context.prompt_analysis.domain.primary,
            context.prompt_analysis.scope
        ));
        
        // Warning (più importanti, prima)
        if !context.warnings.is_empty() {
            output.push_str("## ⚠️ Warnings (based on your past errors)\n\n");
            for warning in &context.warnings {
                output.push_str(&format!("- {}\n", warning));
            }
            output.push_str("\n");
        }
        
        // Documentazione
        if !context.documentation.is_empty() {
            output.push_str("## 📚 Relevant Documentation\n\n");
            for doc in &context.documentation {
                output.push_str(&format!(
                    "**{}** (score: {:.2})\n```\n{}\n```\n\n",
                    doc.source, doc.score, doc.snippet
                ));
            }
        }
        
        // Pattern imparati
        if !context.learned_patterns.is_empty() {
            output.push_str("## 💡 Your Preferred Patterns\n\n");
            for pattern in &context.learned_patterns {
                output.push_str(&format!(
                    "- {} (used {} times)\n",
                    pattern.description, pattern.frequency
                ));
            }
            output.push_str("\n");
        }
        
        // Regole rilevanti
        if !context.relevant_rules.is_empty() {
            output.push_str("## 📋 Relevant Rules\n\n");
            for rule in &context.relevant_rules {
                output.push_str(&format!(
                    "- **{}**: {}\n",
                    rule.id, rule.description
                ));
            }
            output.push_str("\n");
        }
        
        output
    }
    
    fn build_search_query(&self, analysis: &PromptAnalysis) -> String {
        // Combina keyword, entità e dominio
        let mut parts = Vec::new();
        parts.extend(analysis.keywords.clone());
        parts.extend(analysis.entities.iter().map(|e| e.name.clone()));
        parts.push(analysis.domain.primary.clone());
        parts.join(" ")
    }
    
    fn generate_warnings(&self, hints: &[GuidanceHint], analysis: &PromptAnalysis) -> Vec<String> {
        hints.iter()
            .filter(|h| h.severity == Severity::Warning || h.severity == Severity::Error)
            .map(|h| h.message.clone())
            .take(self.config.max_warnings)
            .collect()
    }
}
```

### 2. Configurazione

```rust
// crates/aether-wrapper/src/pre_guidance.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreGuidanceConfig {
    /// Massimo documenti da includere.
    pub max_docs: usize,
    
    /// Massimo warning da mostrare.
    pub max_warnings: usize,
    
    /// Massimo pattern da includere.
    pub max_patterns: usize,
    
    /// Timeout per RAG search (ms).
    pub rag_timeout_ms: u64,
    
    /// Abilita/disabilita Pre-Guidance.
    pub enabled: bool,
    
    /// Mostra warning nel contesto.
    pub show_warnings: bool,
    
    /// Mostra documentazione.
    pub show_documentation: bool,
    
    /// Mostra pattern imparati.
    pub show_patterns: bool,
}

impl Default for PreGuidanceConfig {
    fn default() -> Self {
        Self {
            max_docs: 5,
            max_warnings: 3,
            max_patterns: 3,
            rag_timeout_ms: 100,
            enabled: true,
            show_warnings: true,
            show_documentation: true,
            show_patterns: true,
        }
    }
}
```

---

## Integrazione MCP Hook

### Hook Response

Quando un agente sta per rispondere, l'MCP Hook intercetta e aggiunge il contesto:

```rust
// crates/aether-wrapper/src/mcp_hook.rs
use crate::pre_guidance::PreGuidance;

/// MCP Hook per intercettare richieste agente.
pub struct AetherMcpHook {
    pre_guidance: PreGuidance,
}

impl AetherMcpHook {
    /// Hook chiamato prima che l'agente processi un prompt.
    pub fn pre_process(&mut self, prompt: &str) -> Result<HookResult, HookError> {
        // Genera contesto
        let context = self.pre_guidance.generate_context(prompt)?;
        
        // Formatta per l'agente
        let formatted = self.pre_guidance.format_for_agent(&context);
        
        Ok(HookResult {
            action: HookAction::InjectContext(formatted),
            metadata: json!({
                "intent": context.prompt_analysis.intent,
                "domain": context.prompt_analysis.domain.primary,
                "generation_time_ms": context.generation_time_ms,
            }),
        })
    }
}
```

---

## Integrazione PreToolUse

### Validation Hook

Prima che l'agente scriva un file, PreToolUse valida:

```rust
// crates/aether-wrapper/src/pre_tool_use.rs
use crate::pre_guidance::PreGuidance;
use aether_validation::Validator;

/// Hook PreToolUse per validare prima di scrivere file.
pub struct AetherPreToolUse {
    pre_guidance: PreGuidance,
    validator: Validator,
}

impl AetherPreToolUse {
    /// Valida codice prima che venga scritto su file.
    pub fn validate_before_write(
        &self,
        code: &str,
        file_path: &str,
        prompt: &str,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. Genera contesto Pre-Guidance
        let context = self.pre_guidance.generate_context(prompt)?;
        
        // 2. Esegui validazione
        let mut result = self.validator.validate(code, &Language::from_path(file_path))?;
        
        // 3. Arricchisci con info da Pre-Guidance
        result.pre_guidance_context = Some(context);
        
        // 4. Se ci sono errori, blocca la scrittura
        if result.has_errors() {
            return Err(ValidationError::Blocked {
                result,
                message: "Aether has blocked this code write due to validation errors".into(),
            });
        }
        
        Ok(result)
    }
}
```

---

## Esempio di Output

### Input

```
User prompt: "Add enemy patrol behavior with waypoints"
```

### Output (Pre-Guidance Context)

```markdown
# Aether Pre-Guidance Context

**Intent:** CREATE
**Domain:** gameplay
**Scope:** MODULE

## ⚠️ Warnings (based on your past errors)

- RUST001: You often use .unwrap() without context — prefer ? operator
- RUST003: You often clone unnecessarily — check if & works
- MIL001: You used panic!() 3 times last week — avoid in production

## 📚 Relevant Documentation

**docs/Enemy.md** (score: 0.95)
```
Enemies use a state machine for behavior management. Each enemy has:
- CurrentState (Idle, Patrol, Chase, Attack)
- Waypoints for patrol path
- Detection range for player
```

**docs/AI.md** (score: 0.89)
```
Patrol behavior follows waypoints in order. When player enters
detection range, transition to Chase state. Use `update_patrol()`
for waypoint progression.
```

## 💡 Your Preferred Patterns

- Use `?` operator for error propagation (used 23 times)
- Prefer `if let` over `match` for single variant (used 15 times)
- Use `#[derive(Debug)]` on all structs (used 42 times)

## 📋 Relevant Rules

- **MIL001**: No panic!() in production code
- **MIL010**: No .unwrap() without context
- **RUST003**: Avoid unnecessary .clone()
- **SEC001**: No hardcoded secrets
```

---

## Performance

| Operazione | Target | Tipico |
|------------|--------|--------|
| Prompt Analysis | < 5ms | 2ms |
| RAG Search | < 50ms | 25ms |
| Learner Lookup | < 10ms | 5ms |
| Context Assembly | < 5ms | 2ms |
| **Totale** | **< 70ms** | **35ms** |

### Impatto sulla velocità

| Metrica | Senza Pre-Guidance | Con Pre-Guidance |
|---------|--------------------|------------------|
| Iterazioni medie | 2-3 | 1-1.5 |
| Tempo totale | 100% | 40-60% |
| Errori dopo validazione | 15% | < 5% |

---

## API

### Rust API

```rust
use aether_wrapper::{PreGuidance, PreGuidanceContext};

// Inizializza
let mut pg = PreGuidance::new("./project")?;

// Genera contesto
let context = pg.generate_context("Add enemy patrol behavior")?;

// Formatta per agente
let formatted = pg.format_for_agent(&context);

println!("{}", formatted);
```

### MCP Tool

```json
{
  "name": "aether_pre_guidance",
  "description": "Get pre-guidance context for a prompt before writing code",
  "inputSchema": {
    "type": "object",
    "properties": {
      "prompt": {
        "type": "string",
        "description": "The user prompt to analyze"
      },
      "include_docs": {
        "type": "boolean",
        "description": "Include documentation search",
        "default": true
      },
      "include_patterns": {
        "type": "boolean",
        "description": "Include learned patterns",
        "default": true
      }
    },
    "required": ["prompt"]
  }
}
```

---

## Benefici Chiave

### 1. Riduzione Iterazioni

- **Prima:** Agente scrive → Aether trova errori → Agente corregge → Ripeti
- **Dopo:** Aether guida → Agente scrive giusto al primo colpo

### 2. User Learning

- Aether impara dagli errori dell'utente
- Avvisa PRIMA che l'errore accada
- Riduce errori ricorrenti

### 3. Documentazione Sempre Visibile

- Agente ha sempre accesso alla documentazione pertinente
- Non deve cercare manualmente
- Meno contesto perso

### 4. Velocità Complessiva

- Meno tempo speso in iterazioni
- Meno frustrazione per l'utente
- Output di qualità superiore

---

## Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.

---

## Note Commerciali

| Feature | Community | Commercial |
|---------|-----------|------------|
| Basic Pre-Guidance | ✅ | ✅ |
| Documentation Search | ✅ | ✅ |
| User Learning | ❌ | ✅ |
| Pattern Library | ❌ | ✅ |
| Advanced Warnings | ❌ | ✅ |
| Cloud Sync | ❌ | ✅ |
