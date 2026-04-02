# PatternLearner - Sistema di Learning delle Convenzioni

## Panoramica

Il **PatternLearner** è il sistema di Synward che analizza automaticamente un codebase esistente per estrarre le convenzioni di codice. Questo permette ad Synward di "capire" lo stile del progetto e adattare la validazione di conseguenza.

## Architettura Completa

```
┌─────────────────────────────────────────────────────────────────┐
│                     PATTERN LEARNER SYSTEM                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐        │
│  │   CLI Layer  │───▶│  Core Layer  │───▶│ Output Layer │        │
│  │ synward learn │    │ PatternExtractor    │ learned.toml │        │
│  └──────────────┘    └──────────────┘    └──────────────┘        │
│         │                   │                    │               │
│         ▼                   ▼                    ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐        │
│  │ File Scanner │    │ Tree-Sitter  │    │ Contract     │        │
│  │ (multi-lang) │    │ Parser       │    │ Generator    │        │
│  └──────────────┘    └──────────────┘    └──────────────┘        │
│                             │                    │               │
│                             ▼                    ▼               │
│                      ┌──────────────┐    ┌──────────────┐        │
│                      │ Pattern      │    │ Validation   │        │
│                      │ Categories   │    │ Integration  │        │
│                      └──────────────┘    └──────────────┘        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Categorie di Pattern

### 1. Naming Patterns

Estrae convenzioni di naming dal codice esistente.

| Categoria | Esempi | Utilizzo |
|-----------|--------|----------|
| **Struct suffixes** | `Config`, `Builder`, `Error`, `Context` | Rileva anomalie naming |
| **Enum suffixes** | `Kind`, `Type`, `Mode`, `State` | Verifica consistenza enum |
| **Function prefixes** | `get_`, `is_`, `try_`, `parse_` | Identifica getter/setter |
| **Variable style** | `snake_case`, `camelCase` | Verifica convenzione |
| **Module naming** | `mod`, `use` patterns | Verifica organizzazione |

### 2. Derive Patterns (Rust-specifico)

Analizza l'uso di `#[derive(...)]`:

| Metrica | Descrizione |
|---------|-------------|
| `debug_percentage` | % struct con Debug |
| `clone_percentage` | % struct con Clone |
| `default_percentage` | % struct con Default |
| `common_combinations` | Combinazioni più usate |
| `missing_derives` | Derive spesso dimenticati |

### 3. Documentation Patterns

Analizza la copertura documentativa:

| Metrica | Descrizione |
|---------|-------------|
| `public_doc_percentage` | % items pubblici documentati |
| `comment_style` | `///` vs `//!` vs `/* */` |
| `avg_doc_length` | Media parole per doc |
| `example_coverage` | % docs con esempi codice |
| `module_doc_present` | Se presente `//!` module doc |

### 4. Import Patterns

Analizza dipendenze e import:

| Metrica | Descrizione |
|---------|-------------|
| `wildcard_usage_percentage` | % `use module::*` |
| `common_crates` | Crate più usate |
| `import_grouping` | Se usa gruppi ordinati |
| `extern_crates` | Dipendenze esterne |

### 5. Architecture Patterns (Avanzato)

Analizza pattern architetturali:

| Pattern | Rilevamento |
|---------|-------------|
| **Layer architecture** | Import tra layer |
| **Module cohesion** | Dipendenze interne vs esterne |
| **Error handling style** | `Result<T, E>` vs `Option<T>` vs panic |
| **Testing patterns** | `#[test]`, `#[cfg(test)]` usage |

### 6. Testing Patterns

Analizza convenzioni di test:

| Metrica | Descrizione |
|---------|-------------|
| `test_naming` | `test_` prefix, `should_` pattern |
| `test_organization` | `tests/` dir, `#[cfg(test)]` mod |
| `assertion_style` | `assert!`, `assert_eq!`, `assert_ne!` |
| `mock_usage` | Se usa mock libraries |

---

## Pipeline di Analisi

### Step 1: File Scanning

```rust
// Raccoglie file sorgente ricorsivamente
FileScanner::new(path)
    .extensions(["rs", "py", "ts"])  // Multi-linguaggio
    .exclude(["target", "node_modules", ".git"])
    .max_files(100)  // Limite per performance
    .scan()
```

### Step 2: Tree-Sitter Parsing

Per parsing preciso, usa tree-sitter invece di regex:

```rust
// Parsing AST con tree-sitter
let parser = TreeSitterParser::new(Language::Rust);
let tree = parser.parse(source_code)?;

// Query precise sul AST
let structs = tree.query("(struct_item name: (type_identifier) @name)")?;
let derives = tree.query("(attribute_item (meta_item) @derive)")?;
```

**Vantaggi tree-sitter:**
- Parsing corretto di macro complesse
- Nessun falso positivo in stringhe
- Contesto completo (scope, visibilità)
- Query dichiarative

### Step 3: Pattern Extraction

```rust
let extractor = PatternExtractor::new();
extractor.extract_all(tree, source_code);
```

### Step 4: Confidence Calculation

```rust
// Formula di confidenza
fn calculate_confidence(samples: usize, min_required: usize, files: usize) -> f64 {
    let sample_confidence = (samples as f64 / min_required as f64).min(1.0);
    let file_confidence = (files as f64 / 20.0).min(1.0);
    sample_confidence * file_confidence
}
```

### Step 5: Output Generation

```rust
// Output TOML
let output = patterns.to_toml()?;
fs::write(".synward/learned.toml", output)?;

// Genera contracts YAML (opzionale)
let contracts = ContractGenerator::from_patterns(&patterns);
contracts.save(".synward/contracts/learned.yaml")?;
```

---

## Multi-Linguaggio Support

### Rust (`Language::Rust`)

```toml
# learned.toml per Rust
[derives]
debug_percentage = 81.8
clone_percentage = 68.6

[naming.struct_suffixes]
Config = 15
Builder = 8
```

### Python (`Language::Python`)

```toml
# learned.toml per Python
[naming]
class_suffixes = { "Service" = 12, "Handler" = 8, "Error" = 5 }
function_prefixes = { "get_" = 20, "is_" = 15, "_private" = 30 }

[typing]
type_hints_percentage = 65.0  # % funzioni con type hints
docstring_style = "google"    # google, numpy, sphinx

[imports]
from_imports_percentage = 80.0
```

### TypeScript (`Language::TypeScript`)

```toml
# learned.toml per TypeScript
[naming]
interface_prefixes = { "I" = 20 }  # IUserService, etc
class_suffixes = { "Service" = 15, "Component" = 10 }

[typescript]
strict_mode = true
interface_over_type = 70.0  # % uso interface vs type

[testing]
test_framework = "jest"
```

---

## Integrazione con Validation

### 1. Anomaly Detection

Il learned.toml viene usato come baseline:

```rust
// Durante validazione
let learned = LearnedPatterns::load(".synward/learned.toml")?;

// Rileva anomalie rispetto al learned
if !learned.naming.struct_suffixes.contains_key(&suffix) {
    // Nuova convenzione non vista nel learning
    violations.push(Violation::NamingAnomaly {
        found: suffix,
        expected: learned.naming.common_suffixes(),
    });
}
```

### 2. Derive Checker

```rust
// Verifica derives rispetto al progetto
if learned.derives.debug_percentage > 80.0 {
    // Se >80% delle struct ha Debug, mancanza è anomalia
    if !struct_has_debug {
        violations.push(Violation::MissingDerive {
            derive: "Debug",
            confidence: learned.confidence.derives,
        });
    }
}
```

### 3. Contract Generation

Genera automaticamente contracts YAML:

```yaml
# .synward/contracts/learned-naming.yaml
apiVersion: synward.dev/v1
kind: NamingContract
metadata:
  name: learned-naming
  learned: true
spec:
  rules:
    - pattern: "struct_suffix"
      allowed: ["Config", "Builder", "Error", "Context"]
      confidence: 0.85
      
    - pattern: "function_prefix"
      allowed: ["get_", "is_", "try_", "parse_"]
      confidence: 0.92
```

---

## CLI Usage

### Comando Base

```bash
# Analizza progetto Rust
synward learn ./my-project --lang rust

# Output in directory specifica
synward learn ./my-project --output ./custom/learned.toml
```

### Con Opzioni Avanzate

```bash
# Multi-linguaggio
synward learn ./fullstack-project --lang rust,typescript,python

# Con contract generation
synward learn ./my-project --generate-contracts

# Con tree-sitter (default)
synward learn ./my-project --parser tree-sitter

# Con regex (legacy)
synward learn ./my-project --parser regex

# Verbosità
synward learn ./my-project -v --stats
```

### Output Esempio

```
╔══════════════════════════════════════════════════════════════╗
║ SYNWARD LEARN - Pattern Analysis                              ║
╠══════════════════════════════════════════════════════════════╣
║ Project: clap-test                                           ║
║ Language: rust                                               ║
║ Files analyzed: 50                                           ║
╠══════════════════════════════════════════════════════════════╣
║ NAMING PATTERNS                                              ║
║   Struct suffixes:                                           ║
║     • 4x Context                                             ║
║     • 1x Error                                               ║
║   Function prefixes:                                         ║
║     • 110x get_                                              ║
║     • 62x is_                                                ║
╠══════════════════════════════════════════════════════════════╣
║ DERIVE PATTERNS                                              ║
║   Debug: 81.8% ████████████████████░░░░                      ║
║   Clone: 68.6% ██████████████▓▓▓▓▓▓░░░░                      ║
║   Default: 28.4% ██████░░░░░░░░░░░░░░░░░░                    ║
╠══════════════════════════════════════════════════════════════╣
║ CONFIDENCE SCORES                                            ║
║   Naming: 100% ██████████████████████████                    ║
║   Derives: 100% ██████████████████████████                   ║
║   Documentation: 100% ██████████████████████                 ║
╠══════════════════════════════════════════════════════════════╣
║ OUTPUT                                                        ║
║   → .synward/learned.toml                                     ║
║   → .synward/contracts/learned.yaml (--generate-contracts)    ║
╚══════════════════════════════════════════════════════════════╝
```

---

## Tree-Sitter Integration

### Setup

```tomignore
# Cargo.toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-python = "0.20"
tree-sitter-typescript = "0.20"
```

### Query Examples

```rust
// Query tree-sitter per struct names
let query = Query::new(
    language,
    r#"
    (struct_item
      name: (type_identifier) @struct_name)
    "#
)?;

// Query per derive attributes
let query = Query::new(
    language,
    r#"
    (attribute_item
      (meta_item
        identifier: (identifier) @derive_name
        (#eq? @derive_name "derive")))
    "#
)?;
```

---

## Implementation Roadmap

### Phase 1: Core Infrastructure ✅
- [x] PatternLearner base con regex
- [x] Output TOML
- [x] CLI command `synward learn`
- [x] Confidence scores

### Phase 2: Tree-Sitter Integration
- [ ] Aggiungere tree-sitter-rust dependency
- [ ] Implementare TreeSitterParser
- [ ] Query per estrazione precisa
- [ ] Rimuovere regex-based parsing

### Phase 3: Multi-Language
- [ ] tree-sitter-python
- [ ] tree-sitter-typescript
- [ ] Language-specific extractors
- [ ] Output per-language in learned.toml

### Phase 4: Contract Generation
- [ ] ContractGenerator struct
- [ ] YAML output formato contracts
- [ ] Integration con validation pipeline

### Phase 5: Validation Integration
- [ ] LearnedPatterns loader
- [ ] AnomalyDetector basato su learned
- [ ] Violation types per anomalies
- [ ] Quality Score integration

### Phase 6: Advanced Patterns
- [ ] Architecture pattern detection
- [ ] Error handling style analysis
- [ ] Testing pattern analysis
- [ ] Dependency analysis

---

## API Reference

### PatternLearner

```rust
impl PatternLearner {
    /// Crea learner per progetto
    pub fn new(project: &str) -> Self;
    
    /// Imposta linguaggio target
    pub fn with_language(self, lang: Language) -> Self;
    
    /// Imposta parser type
    pub fn with_parser(self, parser: ParserType) -> Self;
    
    /// Analizza singolo file
    pub fn analyze_file(&mut self, source: &str) -> Result<()>;
    
    /// Analizza directory
    pub fn analyze_dir(&mut self, path: &Path) -> Result<usize>;
    
    /// Finalizza e calcola confidence
    pub fn finalize(self) -> LearnedPatterns;
    
    /// Esporta in TOML
    pub fn to_toml(&self) -> Result<String>;
}
```

### LearnedPatterns

```rust
#[derive(Serialize, Deserialize)]
pub struct LearnedPatterns {
    pub project: String,
    pub language: String,
    pub analyzed_at: DateTime<Utc>,
    pub files_analyzed: usize,
    
    pub naming: NamingPatterns,
    pub derives: DerivePatterns,
    pub documentation: DocPatterns,
    pub imports: ImportPatterns,
    pub confidence: ConfidenceScores,
    
    /// Carica da file TOML
    pub fn load(path: &Path) -> Result<Self>;
    
    /// Salva in TOML
    pub fn save(&self, path: &Path) -> Result<()>;
}
```

### ContractGenerator

```rust
impl ContractGenerator {
    /// Genera contracts da patterns
    pub fn from_patterns(patterns: &LearnedPatterns) -> Self;
    
    /// Aggiungi regole custom
    pub fn add_rule(&mut self, rule: ContractRule) -> &mut Self;
    
    /// Esporta in YAML
    pub fn to_yaml(&self) -> Result<String>;
    
    /// Salva in directory contracts
    pub fn save(&self, dir: &Path) -> Result<()>;
}
```

---

*Versione: 2.0*
*Ultimo aggiornamento: Marzo 2026*
*Autore: Synward Intelligence Team*
