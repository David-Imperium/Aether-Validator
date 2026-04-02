# Synward Language Scaling Strategy

**Versione:** v1.0
**Data:** 2026-03-10

---

## Il Problema

Supportare 5-7 linguaggi è gestibile manualmente. Ma cosa succede se vogliamo supportare 20+, 50+, 100+ linguaggi?

---

## Strategia a 3 Tier

### Tier 1: Core Languages (5-7 linguaggi)

**Criteri:**
- Alta domanda nel mercato
- Ecosistema maturo
- Team con expertise

**Cosa Fornisce:**
- Parsing AST completo (tree-sitter o parser nativo)
- Contratti custom scritti manualmente
- Test suite completa
- Integrazione SDK/CLI completa

**Linguaggi Correnti:**
- Rust ✅
- C++ ✅
- Python (in corso)
- JavaScript (in corso)
- TypeScript (in corso)

**Costo per linguaggio:** 3-5 giorni di sviluppo

---

### Tier 2: Extended Languages (23 linguaggi pubblici)

**Criteri:**
- Domanda moderata
- Grammatica tree-sitter disponibile
- Regole principalmente generiche

**Cosa Fornisce:**
- Parsing AST via tree-sitter
- Contratti generici + specifici AI-generati
- Test suite base
- Integrazione SDK/CLI standard

**Linguaggi Target:**
- Go
- Java
- Kotlin
- Swift
- Ruby
- PHP
- C#
- Scala
- R
- Lua

**Costo per linguaggio:** 1-2 giorni di sviluppo

---

### Tier 3: Community Languages (illimitato)

**Criteri:**
- Domanda bassa o di nicchia
- Grammatica tree-sitter disponibile
- Supporto community-driven

**Cosa Fornisce:**
- Parsing AST via tree-sitter (auto-detect)
- Contratti generici + AI-generated
- Integrazione SDK/CLI base
- Qualità "best effort"

**Linguaggi Target:**
- Haskell, Erlang, Elixir
- Clojure, F#, OCaml
- Zig, Nim, Crystal
- Shell (bash, zsh, fish)
- SQL variants
- Config formats (YAML, TOML, JSON)
- DSLs (GraphQL, protobuf, etc.)

**Costo per linguaggio:** Ore, non giorni

---

## Contratti Gerarchici

### Struttura

```
contracts/
├── generic/                    # Applicati a TUTTI i linguaggi
│   ├── security.yaml           # password, api_key, secret
│   ├── formatting.yaml         # line length, tabs/spaces
│   └── comments.yaml           # TODO, FIXME, XXX
│
├── family/                     # Applicati a FAMIGLIE di linguaggi
│   ├── c-family.yaml           # C, C++, C#, Java, JavaScript, TypeScript
│   ├── functional.yaml         # Haskell, OCaml, F#, Scala
│   ├── scripting.yaml          # Python, Ruby, PHP, Lua
│   └── systems.yaml            # Rust, Go, Zig, C
│
└── {language}/                 # Specifici per linguaggio
    ├── rust/
    │   ├── memory-safety.yaml
    │   ├── ownership.yaml
    │   └── ...
    ├── python/
    │   ├── security.yaml
    │   └── ...
    └── ...
```

### Regole Generic (Esempio)

```yaml
# contracts/generic/security.yaml
contracts:
  - id: GEN_SEC001
    name: "No hardcoded credentials"
    domain: security
    severity: error
    patterns:
      - 'password\s*=\s*"'
      - 'api_key\s*=\s*"'
      - 'secret\s*=\s*"'
      - 'token\s*=\s*"'
      - 'private_key\s*=\s*"'
    applies_to: all
    
  - id: GEN_SEC002
    name: "No SQL injection patterns"
    domain: security
    severity: error
    patterns:
      - 'SELECT .* FROM .* WHERE .* \+'
      - 'INSERT .* VALUES .* \+'
    applies_to: all
    
  - id: GEN_SEC003
    name: "No command injection"
    domain: security
    severity: error
    patterns:
      - 'exec\('
      - 'system\('
      - 'eval\('
      - 'shell\('
    applies_to: all
```

### Regole Family (Esempio)

```yaml
# contracts/family/c-family.yaml
contracts:
  - id: CFAM_001
    name: "Braces on new line"
    domain: style
    severity: info
    patterns:
      - '\)\s*\{'  # function() { should be function()\n{
    applies_to: [c, cpp, csharp, java, javascript, typescript]
    
  - id: CFAM_002
    name: "Semicolon required"
    domain: syntax
    severity: error
    check: "ends_with_semicolon"
    applies_to: [c, cpp, csharp, java, javascript, typescript, go]
```

---

## Integrazione Linter Esterni

### Wrapper System

```rust
// synward-validation/src/external/mod.rs
pub struct ExternalLinterWrapper {
    linter: ExternalLinter,
    severity_map: HashMap<String, Severity>,
}

pub enum ExternalLinter {
    // Python
    Pylint,
    Flake8,
    Bandit,      // Security-focused
    MyPy,        // Type checking
    
    // JavaScript/TypeScript
    ESLint,
    TSLint,
    Prettier,    // Formatting
    
    // Go
    GolangCI,
    StaticCheck,
    
    // Java
    Checkstyle,
    SpotBugs,
    PMD,
    
    // C/C++
    ClangTidy,
    CppCheck,
    
    // Ruby
    RuboCop,
    Brakeman,    // Security
    
    // PHP
    PHPStan,
    Psalm,
    
    // C#
    RoslynAnalyzer,
    StyleCop,
    
    // Rust
    Clippy,      // Già usato
}

impl ExternalLinterWrapper {
    pub fn run(&self, file: &Path, config: &Config) -> Vec<Violation> {
        // 1. Esegue linter esterno
        // 2. Parsa output
        // 3. Converte in Violation
        // 4. Mappa severity
    }
}
```

### Configurazione

```yaml
# synward.yaml
validation:
  layers:
    - syntax
    - semantic
    - logic
    - external:
        enabled: true
        linters:
          python: [pylint, bandit]
          javascript: [eslint]
          go: [golangci]
          java: [checkstyle, spotbugs]
        severity_mapping:
          error: error
          warning: warning
          info: info
          convention: info
```

### Vantaggi

1. **Migliaia di regole già pronte** — Non reinventiamo la ruota
2. **Manutenzione community** — Aggiornamenti automatici
3. **Ecosistema esistente** — Config files standard (.eslintrc, pylintrc, etc.)
4. **Coverage immediata** — Nuovo linguaggio = abilitare linter esistente

---

## Generazione AI di Contratti

### Pipeline

```rust
// synward-contracts/src/generator.rs
pub struct ContractGenerator {
    model: AIModel,
}

impl ContractGenerator {
    pub fn generate_for_language(&self, language: &str) -> Vec<Contract> {
        // 1. Analizza best practices ufficiali
        let docs = self.fetch_official_docs(language);
        
        // 2. Estrai pattern comuni
        let patterns = self.extract_patterns(&docs);
        
        // 3. Genera contratti
        let contracts = self.generate_contracts(&patterns);
        
        // 4. Valida con sample code
        self.validate_contracts(&contracts, language);
        
        contracts
    }
}
```

### Esempio Output (per Zig)

```yaml
# contracts/zig/generated.yaml
contracts:
  - id: ZIG_001
    name: "Use try for error handling"
    domain: error-handling
    severity: warning
    patterns:
      - 'return error\.'  # Prefer try
    suggestion: "Use try instead of explicit error return"
    
  - id: ZIG_002
    name: "Prefer defer for cleanup"
    domain: memory-safety
    severity: info
    patterns:
      - 'defer\s+'
    suggestion: "Use defer for resource cleanup"
```

---

## Auto-Detection Linguaggio

```rust
// synward-parsers/src/detector.rs
pub struct LanguageDetector;

impl LanguageDetector {
    pub fn detect_from_extension(path: &Path) -> Option<Language> {
        match path.extension()?.to_str()? {
            "rs" => Some(Language::Rust),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
            "py" => Some(Language::Python),
            "js" => Some(Language::JavaScript),
            "ts" => Some(Language::TypeScript),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            // ... 100+ estensioni
            _ => self.detect_from_content(path),  // Fallback
        }
    }
    
    pub fn detect_from_content(path: &Path) -> Option<Language> {
        // Analizza shebang, syntax patterns, etc.
    }
}
```

---

## Costi Stimati per Linguaggio

| Tier | Setup Parser | Contratti | Test | Totale |
|------|-------------|-----------|------|--------|
| Tier 1 | 2 giorni | 2 giorni | 1 giorno | **5 giorni** |
| Tier 2 | 0.5 giorni | 0.5 giorni | 0.5 giorni | **1.5 giorni** |
| Tier 3 | Ore | Ore | Ore | **Ore** |

---

## Esempio: Aggiungere Kotlin (Tier 2)

```bash
# 1. Abilita grammatica tree-sitter
# synward-parsers/Cargo.toml
tree-sitter-kotlin = "0.3"

# 2. Registra parser
# synward-parsers/src/registry.rs
pub fn kotlin() -> Self {
    TreeSitterParser::new(tree_sitter_kotlin::language())
}

# 3. Genera contratti (AI-assisted)
$ synward generate-contracts --language kotlin --from-docs

# 4. Test base
$ synward test --language kotlin --sample test_samples/kotlin/
```

**Tempo totale: ~1 giorno**

---

## Roadmap per 50+ Linguaggi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.

### Strategia a Tier

| Tier | Anno | Linguaggi Target |
|------|------|------------------|
| Tier 1 | Anno 1 | Rust, C++, Python, JavaScript, TypeScript |
| Tier 2 | Anno 1-2 | Go, Java, Kotlin, Swift, Ruby, PHP, C#, Scala |
| Tier 3 | Anno 2+ | Tutti gli altri via linter/AI |

---

## Conclusione

La strategia gerarchica permette di:
1. **Fornire qualità alta** dove serve (Tier 1)
2. **Scalare velocemente** per linguaggi intermedi (Tier 2)
3. **Coprire tutto** con automazione (Tier 3)

La combinazione di:
- Contratti generici cross-language
- Integrazione linter esistenti
- Generazione AI di contratti

...rende possibile supportare **qualsiasi linguaggio** con costi contenuti.
