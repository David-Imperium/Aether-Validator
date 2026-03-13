# Aether Multi-Language Support — Piano di Implementazione

**Versione:** v1.0
**Data:** 2026-03-10
**Stato:** Planning

---

## Executive Summary

Aether attualmente supporta Rust con parsing AST completo e C++ con validazione pattern-based. Questo documento descrive il piano per espandere il supporto a **Python, JavaScript, TypeScript, Go, Java, e migliorare C++** con parsing AST.

---

## Stato Attuale

### Linguaggi Supportati

| Linguaggio | Parser | AST | Contratti | Priorità |
|------------|--------|-----|-----------|----------|
| **Rust** | syn | ✅ Completo | 7 file (45 regole) | ✅ Done |
| **C++** | pattern | ⚠️ Parziale | 6 file (48 regole) | Alta |
| **Python** | ❌ | ❌ | ❌ | Alta |
| **JavaScript** | ❌ | ❌ | ❌ | Media |
| **TypeScript** | ❌ | ❌ | ❌ | Media |
| **Go** | ❌ | ❌ | ❌ | Bassa |
| **Java** | ❌ | ❌ | ❌ | Bassa |

### Architettura Attuale

```
aether-parsers/
├── parser.rs          # Trait Parser
├── registry.rs        # ParserRegistry per dispatch
├── ast.rs             # AST generico
├── ast_matcher.rs      # Pattern matching su AST
├── rust.rs            # Parser Rust (syn)
└── (cpp.rs)           # TODO: Parser C++
```

---

## Opzioni Tecniche

### Opzione 1: Tree-sitter (Raccomandata)

**Vantaggi:**
- Grammatiche mature per tutti i linguaggi target
- Parsing incrementale (veloce per editor)
- AST uniforme tra linguaggi
- Mantenuto da Neovim/Emacs community
- Bindings Rust via `tree-sitter` crate

**Svantaggi:**
- Dipendenza esterna (grammatiche WASM/compiled)
- Overhead di build per le grammatiche
- AST meno dettagliato di syn (Rust)

**Implementazione:**
```rust
// aether-parsers/src/python.rs
use tree_sitter::{Parser, Tree};

pub struct PythonParser {
    parser: Parser,
}

impl Parser for PythonParser {
    async fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = self.parser.parse(source, None)?;
        Self::tree_to_ast(&tree.root_node())
    }
}
```

### Opzione 2: Parser Specializzati

**Vantaggi:**
- Controllo completo
- AST più dettagliato
- Integrazione con tool esistenti (pylint, eslint)

**Svantaggi:**
- Manutenzione alta
- Dipendenze multiple
- Duplicazione logica

### Opzione 3: Pattern-Based Only

**Vantaggi:**
- Semplice da implementare
- Veloce
- Nessuna dipendenza esterna

**Svantaggi:**
- Meno preciso (false positives/negatives)
- No analisi semantica profonda
- Limitato per regole complesse

---

## Piano di Implementazione

### Fase 1: Infrastruttura Tree-sitter (2-3 giorni)

**Obiettivo:** Setup tree-sitter per multi-linguaggio

**Task:**
1. [ ] Aggiungere `tree-sitter` crate come dipendenza
2. [ ] Creare `tree-sitter-loader` per caricare grammatiche
3. [ ] Implementare conversione tree-sitter → AST generico
4. [ ] Aggiungere test per conversione AST

**Output:**
```rust
// aether-parsers/src/tree_sitter/mod.rs
pub mod converter;
pub mod loader;

pub struct TreeSitterParser {
    language: tree_sitter::Language,
}

impl TreeSitterParser {
    pub fn python() -> Self { /* ... */ }
    pub fn javascript() -> Self { /* ... */ }
    pub fn typescript() -> Self { /* ... */ }
    pub fn go() -> Self { /* ... */ }
    pub fn java() -> Self { /* ... */ }
}
```

### Fase 2: Python Support (3-4 giorni)

**Obiettivo:** Python con AST completo

**Task:**
1. [ ] `aether-parsers/src/python.rs` — TreeSitterParser per Python
2. [ ] `contracts/python/` — Contratti Python
   - `memory-safety.yaml` (GC, ma unsafe operations)
   - `security.yaml` (eval, exec, input validation)
   - `idioms.yaml` (list comprehension, context managers)
   - `performance.yaml` (comprehensions vs loops)
   - `error-handling.yaml` (exceptions)
3. [ ] Test con file Python reali
4. [ ] Aggiornare SDK e CLI

**Contratti Python (esempio):**
```yaml
# contracts/python/security.yaml
contracts:
  - id: PY_SEC001
    name: "No eval on user input"
    domain: security
    severity: error
    rules:
      - pattern: "eval("
        message: "eval() is dangerous with untrusted input"
        suggestion: "Use ast.literal_eval() for safe evaluation"
  
  - id: PY_SEC002
    name: "No pickle on untrusted data"
    domain: security
    severity: error
    rules:
      - pattern: "pickle.loads("
        message: "pickle can execute arbitrary code"
        suggestion: "Use json or msgpack for untrusted data"
```

### Fase 3: JavaScript/TypeScript Support (4-5 giorni)

**Obiettivo:** JS/TS con AST

**Task:**
1. [ ] `aether-parsers/src/javascript.rs`
2. [ ] `aether-parsers/src/typescript.rs`
3. [ ] `contracts/javascript/`
   - `security.yaml` (XSS, eval, prototype pollution)
   - `idioms.yaml` (const vs let, destructuring)
   - `performance.yaml` (spread in loops)
   - `error-handling.yaml` (try-catch, promises)
4. [ ] `contracts/typescript/`
   - `type-safety.yaml` (any, unknown, type assertions)
   - `security.yaml` (Same as JS + type narrowing)

**Contratti TypeScript (esempio):**
```yaml
# contracts/typescript/type-safety.yaml
contracts:
  - id: TS_TYPE001
    name: "Avoid any type"
    domain: type-safety
    severity: warning
    rules:
      - pattern: ": any"
        message: "any defeats TypeScript type checking"
        suggestion: "Use unknown or specific type"
  
  - id: TS_TYPE002
    name: "Prefer unknown over any"
    domain: type-safety
    severity: info
    rules:
      - pattern: ": any"
        message: "Consider unknown for type-safe handling"
        suggestion: "unknown forces type narrowing before use"
```

### Fase 4: C++ AST Parser (3-4 giorni)

**Obiettivo:** Migliorare C++ con parsing AST

**Task:**
1. [ ] Implementare parser C++ via tree-sitter
2. [ ] Mappare costrutti C++ → NodeKind (Class, Template, Namespace)
3. [ ] Migliorare contratti esistenti con pattern AST
4. [ ] Aggiornare validation layer per usare AST

**Gap attuale:**
```rust
// Attualmente C++ usa solo pattern text-based
// Con AST potremmo avere:
pub enum NodeKind {
    // Esistenti
    Function,
    Struct,
    Enum,
    
    // Nuovi per C++
    Class,
    Template,
    Namespace,
    Macro,
}
```

### Fase 5: Go Support (2-3 giorni)

**Obiettivo:** Go con AST

**Task:**
1. [ ] `aether-parsers/src/go.rs`
2. [ ] `contracts/go/`
   - `idioms.yaml` (error handling, defer, goroutines)
   - `concurrency.yaml` (race conditions, channel patterns)
   - `performance.yaml` (slice preallocation)

**Contratti Go (esempio):**
```yaml
# contracts/go/idioms.yaml
contracts:
  - id: GO_IDM001
    name: "Check errors immediately"
    domain: idioms
    severity: error
    rules:
      - pattern: "if err != nil { return err }"
        message: "Go idiom: check errors immediately"
        suggestion: "Handle or propagate errors right away"
```

### Fase 6: Java Support (2-3 giorni)

**Obiettivo:** Java con AST

**Task:**
1. [ ] `aether-parsers/src/java.rs`
2. [ ] `contracts/java/`
   - `security.yaml` (SQL injection, XSS)
   - `performance.yaml` (String concatenation, collections)
   - `idioms.yaml` (try-with-resources, Optional)

---

## Architettura Target

```
aether-parsers/
├── parser.rs              # Trait Parser
├── registry.rs            # ParserRegistry
├── ast.rs                 # AST generico (esteso per tutti i linguaggi)
├── ast_matcher.rs         # Pattern matching
├── tree_sitter/           # Tree-sitter integration
│   ├── mod.rs
│   ├── loader.rs          # Carica grammatiche
│   └── converter.rs       # tree-sitter → AST
├── rust.rs                # Parser Rust (syn)
├── cpp.rs                 # Parser C++ (tree-sitter)
├── python.rs              # Parser Python (tree-sitter)
├── javascript.rs          # Parser JavaScript (tree-sitter)
├── typescript.rs          # Parser TypeScript (tree-sitter)
├── go.rs                  # Parser Go (tree-sitter)
└── java.rs                # Parser Java (tree-sitter)
```

---

## Dipendenze

```toml
# Cargo.toml - aether-parsers
[dependencies]
syn = { version = "2.0", features = ["full", "parsing", "visit"] }
async-trait = "0.1"
tree-sitter = "0.24"  # Core
tree-sitter-python = "0.23"  # Python grammar
tree-sitter-javascript = "0.23"  # JS grammar
tree-sitter-typescript = "0.23"  # TS grammar
tree-sitter-cpp = "0.23"  # C++ grammar
tree-sitter-go = "0.23"  # Go grammar
tree-sitter-java = "0.23"  # Java grammar
```

---

## Contratti per Linguaggio

### Python (45 regole pianificate)

| Dominio | Regole | Esempi |
|---------|--------|--------|
| Security | 10 | eval, exec, pickle, subprocess |
| Memory | 5 | GC cycles, large allocations |
| Idioms | 10 | comprehension, context managers |
| Performance | 10 | list vs generator, early return |
| Error Handling | 10 | exception types, re-raise |

### JavaScript (40 regole)

| Dominio | Regole | Esempi |
|---------|--------|--------|
| Security | 15 | XSS, eval, prototype pollution |
| Idioms | 10 | const/let, destructuring |
| Performance | 10 | spread in loops, async patterns |
| Error Handling | 5 | try-catch, promise rejection |

### TypeScript (30 regole aggiuntive)

| Dominio | Regole | Esempi |
|---------|--------|--------|
| Type Safety | 15 | any, unknown, type assertions |
| Idioms | 10 | generics, type guards |
| Security | 5 | type narrowing for validation |

### Go (35 regole)

| Dominio | Regole | Esempi |
|---------|--------|--------|
| Concurrency | 10 | race conditions, channel patterns |
| Idioms | 15 | error handling, defer |
| Performance | 10 | slice preallocation, map sizing |

### Java (30 regole)

| Dominio | Regole | Esempi |
|---------|--------|--------|
| Security | 10 | SQL injection, XSS |
| Idioms | 10 | try-with-resources, Optional |
| Performance | 10 | String builder, collections |

---

## Testing Strategy

### Test Files per Linguaggio

```
test_samples/
├── rust/
│   ├── clean.rs
│   ├── memory_violations.rs
│   └── security_violations.rs
├── python/
│   ├── clean.py
│   ├── security_violations.py
│   └── performance_violations.py
├── javascript/
│   ├── clean.js
│   └── security_violations.js
├── typescript/
│   ├── clean.ts
│   └── type_violations.ts
├── go/
│   ├── clean.go
│   └── concurrency_violations.go
└── java/
    ├── clean.java
    └── security_violations.java
```

### Test自动化

```rust
// tests/integration_test.rs
#[test]
fn test_python_security_violations() {
    let client = AetherClient::new();
    let result = client.validate_file("test_samples/python/security_violations.py", "python");
    assert!(!result.passed);
    assert!(result.violations.iter().any(|v| v.id == "PY_SEC001"));
}
```

---

## Timeline

| Fase | Durata | Dipendenze |
|------|--------|------------|
| 1. Infrastruttura Tree-sitter | 2-3 giorni | Nessuna |
| 2. Python Support | 3-4 giorni | Fase 1 |
| 3. JavaScript/TypeScript | 4-5 giorni | Fase 1 |
| 4. C++ AST Parser | 3-4 giorni | Fase 1 |
| 5. Go Support | 2-3 giorni | Fase 1 |
| 6. Java Support | 2-3 giorni | Fase 1 |

**Totale stimato:** 16-22 giorni di lavoro

---

## Rischi e Mitigazioni

| Rischio | Probabilità | Impatto | Mitigazione |
|---------|-------------|---------|-------------|
| Grammatiche tree-sitter non aggiornate | Media | Bassa | Pin versione, fork se necessario |
| Performance parsing su file grandi | Bassa | Media | Parsing incrementale, cache |
| Contratti troppo specifici | Media | Bassa | Review con esperti linguaggio |
| False positives | Alta | Media | Tuning contratti, whitelist |

---

## Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.

---

## Note Tecniche

### Tree-sitter vs Syn (Rust)

```rust
// Syn (Rust attuale)
let ast = syn::parse_file(source)?;

// Tree-sitter (nuovi linguaggi)
let parser = tree_sitter::Parser::new();
parser.set_language(&tree_sitter_python::language()).unwrap();
let tree = parser.parse(source, None)?;
```

### Conversione AST

```rust
// tree-sitter produce un albero generico
// Noi lo convertiamo nel nostro AST tipizzato

pub fn tree_to_ast(node: tree_sitter::Node) -> AST {
    match node.kind() {
        "function_definition" => ASTNode::Function { ... },
        "class_definition" => ASTNode::Class { ... },
        // ... mapping per ogni linguaggio
    }
}
```

---

## Conclusione

Il piano prevede l'aggiunta di **5 nuovi linguaggi** con **~200 nuove regole** in circa **3 settimane** di lavoro. L'approccio tree-sitter garantisce uniformità e manutenibilità a lungo termine.

**Raccomandazione:** Procedere con Fase 1 (tree-sitter infrastructure) come prossimo step.
