# Synward Dubbioso Mode

**Version:** 1.0
**Last Updated:** 2026-03-19
**Author:** David + Droid
**Status:** 📋 Design Phase
**See Also:** [ADR_AUTONOMOUS_SYNWARD.md](./ADR_AUTONOMOUS_SYNWARD.md), [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md)

---

## Executive Summary

**Dubbioso Mode** è il cuore dell'intelligenza di Synward: non è un semplice validatore che dice "sì/no", ma un sistema che **sa quando non è sicuro** e chiede conferma.

Il dubbio è il motore dell'apprendimento: ogni volta che Synward chiede e riceve risposta, impara. La memoria cresce, il confidence aumenta, le domande diminuiscono.

---

## Il Problema dei Validatori Tradizionali

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    VALIDATORI TRADIZIONALI                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Output: "Error: unwrap() used in production code"                         │
│                                                                             │
│  Problemi:                                                                  │
│  ─────────────────────────────────────────────────────────────────────────  │
│  ❌ Non sa se è un problema reale o un'eccezione valida                     │
│  ❌ Non capisce il contesto (magari è gestito altrove)                      │
│  ❌ L'utente ignora perché sa che è falso positivo                          │
│  ❌ Nessun apprendimento: stesso errore, stessa warning sempre              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## La Soluzione: Dubbioso Mode

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SYNWARD DUBBIOSO MODE                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Output:                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Violazione: "unwrap() in production code"                           │   │
│  │ ├── Confidence: 73%                                                 │   │
│  │ ├── Contesto: handle_request() → process() → questo file            │   │
│  │ ├── Perché dubbia: Nessuna gestione errore visibile, ma...          │   │
│  │ │   ...process() ha fallback in caller                              │   │
│  │ └── Domanda: "La funzione process() può fallire? Il fallback        │   │
│  │              in handle_request() è sufficiente? (y/n/a=always)"     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Risposta utente → Memoria impara → Confidence aumenta → Non chiede più    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Architettura: Hyper-Context Engine

Dubbioso Mode è alimentato da 3 layer che lavorano insieme:

### Layer 1: Graph RAG Profondo

**Funzione:** Capisce il CONTESTO

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Graph RAG Traversal                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  File A ──┬──▶ imports File B ──┬──▶ imports File D                        │
│           │                     │                                           │
│           └──▶ calls func_X()   └──▶ defines func_X()                       │
│                                                                             │
│  Domande che sa rispondere:                                                 │
│  • Chi chiama questa funzione?                                              │
│  • Chi è chiamato da questa funzione?                                       │
│  • Dove è definito questo tipo?                                             │
│  • Quali file dipendono da questo?                                          │
│                                                                             │
│  Multi-livello: non solo imports diretti, ma intera catena                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Layer 2: Semantic Analysis

**Funzione:** Capisce l'INTENTO

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Tree-Sitter Queries                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Esempio: Rileva pattern di error handling                                  │
│                                                                             │
│  (try_statement                                                             │
│    (catch_clause                                                            │
│      (identifier) @exception_type                                           │
│      (block) @handler))                                                     │
│                                                                             │
│  Capisce:                                                                   │
│  • Questo codice gestisce errori? Come?                                     │
│  • È un pattern noto? (MVC, Repository, etc.)                               │
│  • È un anti-pattern? (God Object, Spaghetti)                               │
│  • Qual è l'intento dello sviluppatore?                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Layer 3: Context Scoring

**Funzione:** Sa QUANDO dubitare

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Confidence Calculation                                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Score = f(                                                                 │
│    history_score,      // Quante volte validato prima                       │
│    memory_score,       // Quanto la memoria conosce questo pattern          │
│    graph_score,        // Quanto è connesso nel progetto                    │
│    semantic_score,     // Quanto è chiaro l'intento                         │
│    feedback_score      // Feedback storico su file simili                   │
│  )                                                                          │
│                                                                             │
│  Soglie configurabili:                                                      │
│  ├── ask_threshold: 60%  // Sotto, chiede conferma                         │
│  ├── warn_threshold: 80% // Sotto, warning ma continua                     │
│  └── auto_accept: 95%   // Sopra, accetta automaticamente                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Fusione dei Layer

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CONFIDENCE INTELLIGENCE                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   Graph RAG Profundo ──────┐                                                │
│                            │                                                │
│   Semantic Analysis ───────┼──▶ Fusion Engine ──▶ Confidence + Contesto    │
│                            │                                                │
│   Context Scoring ─────────┘                                                │
│                                                                             │
│  Output:                                                                    │
│  {                                                                          │
│    "violation": "unwrap() in production",                                   │
│    "confidence": 0.73,                                                      │
│    "context": {                                                             │
│      "callers": ["handle_request", "process"],                              │
│      "error_handling": "partial",                                           │
│      "intent": "data_extraction"                                            │
│    },                                                                       │
│    "should_ask": true,                                                      │
│    "question": "La funzione process() può fallire?"                         │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Interazione via MCP

### Modalità di Interrogazione

Quando confidence < ask_threshold, Synward chiede via MCP:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  MCP Question Flow                                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Synward rileva violazione con confidence 65%                            │
│  2. MCP invia: {                                                            │
│       "type": "ask",                                                        │
│       "message": "Questa unwrap() può causare panic. Gestito altrove?",     │
│       "options": ["y", "n", "a=always", "s=skip", "e=explain"]              │
│     }                                                                       │
│  3. Utente risponde: "a" (always accept this pattern)                       │
│  4. Memoria salva: {                                                        │
│       "pattern": "unwrap_in_process_functions",                             │
│       "decision": "accept",                                                 │
│       "confidence_boost": +10%                                              │
│     }                                                                       │
│  5. Prossima volta: confidence 75% → auto-accept                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Opzioni Risposta

| Opzione | Significato | Impatto Memoria |
|---------|-------------|-----------------|
| `y` | Sì, è ok questa volta | +5% per questo caso |
| `n` | No, è un errore | -5% per questo caso |
| `a` | Always, accetta sempre | +20% per pattern, whitelist |
| `s` | Skip, ignora | Nessun cambiamento |
| `e` | Explain, spiega perché | Mostra contesto Graph RAG |

---

## Feedback Loop: Il Dubbio Insegna

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    LEARNING CYCLE                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. Validazione file                                                        │
│       ↓                                                                     │
│  2. Dubbioso Mode: confidence 65%                                          │
│       ↓                                                                     │
│  3. Chiede via MCP                                                          │
│       ↓                                                                     │
│  4. Utente risponde "a" (always)                                            │
│       ↓                                                                     │
│  5. Memoria salva pattern + decisione                                       │
│       ↓                                                                     │
│  6. Core si adatta: whitelist quel pattern                                  │
│       ↓                                                                     │
│  7. Prossima validazione: confidence 85% (auto-accept)                      │
│       ↓                                                                     │
│  8. Dopo N accettazioni: pattern diventa regola permanente                  │
│                                                                             │
│  Risultato: Synward impara le convenzioni del team                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Configurazione

### Thresholds (`.synward.toml`)

```toml
[dubbioso]
# Sotto questa soglia, chiede conferma
ask_threshold = 0.60

# Sotto questa soglia, warning ma continua
warn_threshold = 0.80

# Sopra questa soglia, accetta automaticamente
auto_accept_threshold = 0.95

# Dopo N accettazioni, rendi permanente
permanent_after = 5

# Abilita questioning via MCP
enable_mcp_questions = true

# Abilita CLI interactive mode (fallback)
enable_cli_questions = false
```

### Memoria Format

```toml
# ~/.synward/learned_patterns.toml

[[patterns]]
name = "unwrap_in_test_functions"
rule = "unwrap() in test/** is acceptable"
confidence = 0.95
accepted_count = 12
permanent = true

[[patterns]]
name = "public_camel_case"
rule = "public functions use camelCase in this project"
confidence = 0.88
accepted_count = 8
permanent = false
```

---

## Implementazione

### Stato Attuale (2026-03-25)

| Componente | Stato | Note |
|------------|-------|------|
| Confidence scoring | ✅ Implementato | Nei risultati validazione |
| Confidence filtering (Phase 3) | ✅ Implementato | 25% threshold, filtra violazioni a bassa confidence |
| Test file filtering (Phase 4) | ✅ Implementato | Esclude LOGIC001/LOGIC002 in file di test |
| Graph RAG | ✅ Base | Attraversamento 1 livello |
| Semantic Analysis | ❌ | Non implementato |
| Context Scoring | ❌ | Non implementato |
| MCP Questioning | ❌ | Non implementato |
| Thresholds configurabili | ❌ | Non implementato |
| Memory → Core Loop | ❌ | Non implementato |

### Task da Completare

- [ ] Graph RAG multi-livello (2+ livelli di traversal)
- [ ] Tree-sitter queries per semantic analysis
- [ ] Context scoring algorithm
- [ ] MCP question protocol
- [ ] Threshold configuration in `.synward.toml`
- [ ] Memory pattern persistence
- [ ] Feedback loop integration

---

## Phase 3: Confidence Filtering ✅ COMPLETE (2026-03-25)

Implementato in `executor.rs` con soglia al 25%:

```rust
// Phase 3: Confidence filtering - filtra violazioni a bassa confidence
let confidence_threshold = 0.25;
all_violations = all_violations.into_iter()
    .enumerate()
    .filter_map(|(idx, mut violation)| {
        if violation.confidence >= confidence_threshold {
            Some(violation)
        } else {
            None
        }
    })
    .collect();
```

**Soglia attuale:** 25% (0.25) — Le violazioni con confidence inferiore vengono automaticamente scartate.

### Risultato
- False positives ridotti significativamente
- Solo violazioni con confidence >= 25% vengono riportate
- Il valore è configurabile nel codice (futuro: configurabile via TOML)

---

## Phase 4: Test File Filtering ✅ COMPLETE (2026-03-25)

Implementato in `executor.rs` — Esclude regole specifiche nei file di test:

```rust
// Phase 4: Test file filtering - esclude LOGIC001/LOGIC002 in test files
const TEST_ONLY_RULES: &[&str] = &["LOGIC001", "LOGIC002"];
let is_test_file = path_str.contains("/tests/") || path_str.contains("\\tests\\") ||
                  path_str.contains("/test_") || path_str.contains("\\test_") ||
                  path_str.ends_with("_test.rs") || path_str.ends_with("_tests.rs");
```

**Regole filtrate nei test:**
- `LOGIC001` (panic!) — Spesso accettabile in test
- `LOGIC002` (todo!) — Spesso accettabile in test

**Pattern rilevati:**
- `/tests/`, `\tests\`
- `/test_`, `\test_`
- `_test.rs`, `_tests.rs`

### Risultato
- False positives ridotti in test code
- Nessuna segnalazione di panic/todo non gestiti nei test
- 172 test passano senza warnings spurii

---

## Esempio Completo

### Input
```rust
// src/handler.rs
pub fn handle_request(req: Request) -> Response {
    let data = req.body.unwrap();  // Linea problematica
    process(data)
}
```

### Output Dubbioso Mode

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  SYNWARD VALIDATION                                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ⚠️  src/handler.rs:3 - unwrap() in production code                        │
│                                                                             │
│  Confidence: 73% (Moderate)                                                 │
│                                                                             │
│  Context:                                                                   │
│  ├── Called by: server.rs::run() (main entry point)                        │
│  ├── Caller chain: run → handle_request → unwrap                           │
│  ├── Error handling: None visible in chain                                 │
│  └── Intent: Extract request body for processing                           │
│                                                                             │
│  Why uncertain:                                                             │
│  • unwrap() can panic if body is None                                      │
│  • No try/catch or Result handling in callers                              │
│  • BUT: server.rs has graceful shutdown on panic                           │
│                                                                             │
│  Question via MCP:                                                          │
│  "Can Request.body be None? Is graceful shutdown acceptable here?          │
│   (y/n/a=always/s=skip/e=explain)"                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Vedi Anche

- [ADR_AUTONOMOUS_SYNWARD.md](./ADR_AUTONOMOUS_SYNWARD.md) — Decisioni architetturali
- [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md) — Architettura memoria
- [ROADMAP_INDEX.md](./ROADMAP_INDEX.md) — Stato implementazione
