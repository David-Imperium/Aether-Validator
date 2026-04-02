# Synward Custom Language Support

**Version:** 1.0
**Last Updated:** 2026-03-16
**Author:** Droid + David

---

## Executive Summary

Synward supporta **23 linguaggi pubblici** out-of-the-box (+ Prism privato). Per aziende con linguaggi proprietari (DSL, internal tools, legacy code), offre 3 livelli di integrazione a complessità crescente.

---

## Linguaggi Supportati (Built-in)

| Linguaggio | Parser | Regole | Security Checks |
|------------|--------|--------|-----------------|
| Rust | tree-sitter | 45 | 12 |
| Python | tree-sitter | 52 | 15 |
| JavaScript | tree-sitter | 38 | 10 |
| TypeScript | tree-sitter | 42 | 11 |
| C | tree-sitter | 35 | 14 |
| C++ | tree-sitter | 40 | 14 |
| Go | tree-sitter | 32 | 8 |
| Java | tree-sitter | 36 | 9 |
| Lua | tree-sitter | 24 | 6 |
| Bash | tree-sitter | 20 | 5 |
| Lex | Custom | 28 | 5 |
| GLSL | tree-sitter | 18 | 4 |
| CSS | tree-sitter | 12 | 2 |
| HTML | tree-sitter | 8 | 3 |
| JSON | tree-sitter | 6 | 1 |
| YAML | tree-sitter | 10 | 2 |
| TOML | tree-sitter-toml-ng | 8 | 2 |
| CMake | tree-sitter-cmake | 15 | 3 |
| CUDA | tree-sitter-cuda | 35 | 10 |
| SQL | tree-sitter | 22 | 6 |
| GraphQL | tree-sitter | 14 | 3 |
| Markdown | tree-sitter | 10 | 2 |
| Notebook | tree-sitter | 8 | 2 |

**Totale:** 23 linguaggi pubblici (+ Prism privato), ~500 regole, ~130 security checks

---

## 4 Livelli di Supporto Custom

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CUSTOM LANGUAGE SUPPORT LEVELS                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  LEVEL 0: AUTOMATIC FALLBACK (Security)             Costo: ZERO              │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Funziona per QUALSIASI linguaggio                                        │
│  • Zero configurazione richiesta                                            │
│  • Regex-based security checks automatici                                   │
│  • Setup: immediato (built-in)                                              │
│                                                                             │
│  LEVEL 1: PATTERN-BASED (YAML)                     Costo: LOW               │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Definisci pattern in YAML                                                │
│  • Nessun codice richiesto                                                  │
│  • Regex + semplici regole semantiche                                       │
│  • Setup: 1-2 ore per linguaggio semplice                                   │
│                                                                             │
│  LEVEL 2: TREE-SITTER GRAMMAR                       Costo: MEDIUM            │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Scrivi grammar tree-sitter                                               │
│  • Parsing completo con AST                                                 │
│  • Regole semantiche custom                                                 │
│  • Setup: 1-2 settimane per linguaggio complesso                            │
│                                                                             │
│  LEVEL 3: FULL PARSER PLUGIN                        Costo: HIGH              │
│  ─────────────────────────────────────────────────────────────────────────  │
│  • Plugin Rust completo                                                     │
│  • Controllo totale su parsing e validation                                 │
│  • Integrazione con toolchain esistente                                     │
│  • Setup: 1-2 mesi per linguaggio enterprise                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Level 0: Automatic Fallback (Security)

### Quando Si Attiva

Synward ha un **FallbackSecurityLayer** automatico che si attiva per **qualsiasi linguaggio non supportato**. Questo significa che anche senza configurazione, ottieni security checks di base.

### Architettura

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    LEVEL 0: AUTOMATIC FALLBACK                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Input: any-file.xyz (linguaggio sconosciuto)                               │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    FallbackSecurityLayer                             │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │   │
│  │  │ Hardcoded    │  │ SQL          │  │ Command      │  │ Path      │ │   │
│  │  │ Secrets      │  │ Injection    │  │ Injection    │  │ Traversal │ │   │
│  │  │ SEC001       │  │ SEC002       │  │ SEC003       │  │ SEC004    │ │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └───────────┘ │   │
│  │                                                                      │   │
│  │  Output: Violations[] (se trovate)                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Nessuna configurazione richiesta. Zero setup.                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Pattern Supportati

| ID | Nome | Pattern | Severity |
|----|------|---------|----------|
| SEC001 | Hardcoded Secrets | `(password|api_key|secret|token|credential)\s*=\s*['"][^'"]+['"]` | Critical |
| SEC002 | SQL Injection | `(SELECT|INSERT|UPDATE|DELETE).*\+.*['"]` | Critical |
| SEC003 | Command Injection | `(exec|system|eval|shell)\s*\([^)]*\+` | Critical |
| SEC004 | Path Traversal | `\.\./|\.\.\\` | High |
| SEC005 | Debug Code Left | `(TODO|FIXME|HACK|XXX|BUG)` | Warning |
| SEC006 | Weak Crypto | `(MD5|SHA1|DES)\s*\(` | High |

### Esempio: File Kotlin

```kotlin
// UserService.kt - linguaggio NON supportato nativamente

class UserService {
    // SEC001: Hardcoded secret detected
    val apiKey = "sk-live-abc123def456"

    fun queryUser(id: String): User {
        // SEC002: Potential SQL injection
        val query = "SELECT * FROM users WHERE id = " + id
        return db.execute(query)
    }

    fun runCommand(cmd: String): String {
        // SEC003: Command injection risk
        return Runtime.getRuntime().exec(cmd).text
    }

    // SEC004: Path traversal vulnerability
    fun readFile(path: String): String {
        return File("/data/" + path).readText()
    }
}
```

**Output Synward:**

```bash
$ synward validate UserService.kt

[SEC001] CRITICAL: Hardcoded secret detected
  → UserService.kt:5:15
  → Pattern: apiKey = "sk-live-abc123def456"
  → Fix: Use environment variables or secrets manager

[SEC002] CRITICAL: Potential SQL injection
  → UserService.kt:9:20
  → Pattern: "SELECT * FROM users WHERE id = " + id
  → Fix: Use parameterized queries

[SEC003] CRITICAL: Command injection risk
  → UserService.kt:14:35
  → Pattern: Runtime.getRuntime().exec(cmd)
  → Fix: Validate and sanitize input, use allowlist

[SEC004] HIGH: Path traversal vulnerability
  → UserService.kt:19:25
  → Pattern: "/data/" + path
  → Fix: Validate path, use basename() or realpath()

4 violations found (3 critical, 1 high)
```

### CLI Usage

```bash
# Automatico per qualsiasi file
synward validate mystery-file.xyz

# Verbose mode per vedere quale layer è attivo
synward validate mystery-file.xyz --verbose

# Output:
# [INFO] Language not recognized, using FallbackSecurityLayer (Level 0)
# [INFO] Running 6 security patterns...
```

### Limiti del Level 0

- **Solo security checks** — niente syntax validation, semantic analysis
- **Regex-based** — può avere false positives/negatives
- **Nessun context** — non capisce la semantica del linguaggio
- **Nessun fix automatico** — solo detection

Per validazione completa, passa a Level 1+.

---

## Level 1: Pattern-Based (YAML)

### Quando Usarlo

- DSL semplici (config files, templates)
- Linguaggi di markup
- Prototipi veloci
- Team non tecnici

### Struttura File

```yaml
# .synward/languages/custom-lang.yaml

metadata:
  name: "CustomLang"
  version: "1.0.0"
  extensions: [".cl", ".custom"]
  description: "Internal DSL for configuration"

# Pattern di sintassi
syntax:
  # Commenti
  comments:
    single_line: "#"
    multi_line_start: "/*"
    multi_line_end: "*/"
  
  # Stringhe
  strings:
    - delimiter: '"'
      escape: "\\"
      allow_multiline: true
    - delimiter: "'"
      escape: "\\"
      allow_multiline: false

  # Keywords
  keywords:
    - "def"
    - "if"
    - "else"
    - "for"
    - "return"
    - "import"

  # Built-in functions (per security checks)
  builtins:
    dangerous:
      - name: "eval"
        severity: "critical"
        message: "eval() can execute arbitrary code"
      - name: "exec"
        severity: "high"
        message: "exec() can run system commands"
    safe:
      - "print"
      - "len"
      - "str"

# Regole di validazione
rules:
  # Syntax rules
  - id: "SYNTAX001"
    name: "Missing semicolon"
    pattern: "^\\s*[a-zA-Z_][a-zA-Z0-9_]*\\s*=[^;]*$"
    severity: "error"
    message: "Statement missing semicolon"
  
  # Security rules
  - id: "SECURITY001"
    name: "Dangerous function call"
    pattern: "\\b(eval|exec)\\s*\\("
    severity: "critical"
    message: "Dangerous function usage"
    fix_suggestion: "Use safer alternatives: parse() or safe_eval()"
  
  # Style rules
  - id: "STYLE001"
    name: "Line too long"
    pattern: "^.{121,}"
    severity: "warning"
    message: "Line exceeds 120 characters"

# Anti-pattern detection
anti_patterns:
  - id: "ANTI001"
    name: "Empty block"
    pattern: "\\b(if|for|def)\\s*\\([^)]*\\)\\s*\\{\\s*\\}"
    severity: "warning"
    message: "Empty control block"
  
  - id: "ANTI002"
    name: "Hardcoded credentials"
    pattern: "(password|api_key|secret)\\s*=\\s*['\"][^'\"]+['\"]"
    severity: "critical"
    message: "Hardcoded credential detected"

# Custom severity levels
severity_levels:
  critical:
    color: "red"
    exit_code: 2
  error:
    color: "yellow"
    exit_code: 1
  warning:
    color: "blue"
    exit_code: 0
  info:
    color: "gray"
    exit_code: 0
```

### CLI Usage

```bash
# Validate con linguaggio custom
synward validate config.cl --lang custom-lang

# List available custom languages
synward languages list --custom

# Test custom language definition
synward languages test custom-lang --file test.cl
```

### Esempio Pratico: Shader DSL

```yaml
# .synward/languages/shader-dsl.yaml

metadata:
  name: "ShaderDSL"
  extensions: [".sdr"]
  description: "Internal shader DSL"

syntax:
  comments:
    single_line: "//"
    multi_line_start: "/*"
    multi_line_end: "*/"
  
  keywords:
    - "vertex"
    - "fragment"
    - "uniform"
    - "varying"
    - "sampler2D"

rules:
  - id: "SHADER001"
    name: "Missing precision qualifier"
    pattern: "\\b(float|int|vec2|vec3|vec4)\\s+[a-z]"
    severity: "warning"
    message: "Missing precision qualifier (lowp/mediump/highp)"
    fix_suggestion: "Add precision: mediump float x;"
  
  - id: "SHADER002"
    name: "Texture lookup without bias"
    pattern: "texture2D\\s*\\([^,]+,\\s*[^)]+\\)\\s*;"
    severity: "info"
    message: "Consider adding LOD bias for better quality"

anti_patterns:
  - id: "SHADER_ANTI001"
    name: "Branch in fragment shader"
    pattern: "\\bif\\s*\\([^)]*\\)\\s*\\{"
    severity: "warning"
    message: "Branching in fragment shader may hurt performance"
```

---

## Level 2: Tree-sitter Grammar

### Quando Usarlo

- Linguaggi con sintassi complessa
- Necessità di AST completo
- Semantic analysis avanzata
- IDE integration (highlighting, navigation)

### Struttura Progetto

```
custom-lang/
├── grammar.js           # Tree-sitter grammar
├── package.json         # npm config
├── src/
│   ├── parser.c         # Generated parser
│   └── tree_sitter/
│       └── parser.h
├── test/
│   └── corpus/
│       └── test.txt     # Test cases
└── synward-rules/
    ├── semantic.rs      # Semantic rules
    └── security.rs      # Security checks
```

### Grammar Example

```javascript
// grammar.js

module.exports = grammar({
  name: 'custom_lang',
  
  rules: {
    // Entry point
    source_file: $ => repeat($._definition),
    
    // Definitions
    _definition: $ => choice(
      $.function_def,
      $.variable_def,
      $.import_statement,
    ),
    
    function_def: $ => seq(
      'def',
      field('name', $.identifier),
      field('parameters', $.parameter_list),
      field('return_type', optional($.type)),
      field('body', $.block),
    ),
    
    variable_def: $ => seq(
      optional('const'),
      field('name', $.identifier),
      optional(seq(':', field('type', $.type))),
      '=',
      field('value', $._expression),
      ';',
    ),
    
    // Expressions
    _expression: $ => choice(
      $.number,
      $.string,
      $.identifier,
      $.binary_expression,
      $.call_expression,
      $.member_expression,
    ),
    
    binary_expression: $ => prec.left(2, seq(
      field('left', $._expression),
      field('operator', choice('+', '-', '*', '/', '==', '!=')),
      field('right', $._expression),
    )),
    
    call_expression: $ => seq(
      field('function', $._expression),
      field('arguments', $.argument_list),
    ),
    
    // Primitives
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    number: $ => /\d+(\.\d+)?/,
    string: $ => choice(
      seq('"', repeat(choice(/[^"\\]+, $.escape_sequence)), '"'),
      seq("'", repeat(choice(/[^'\\]+, $.escape_sequence)), "'"),
    ),
    
    // ... more rules
  }
});
```

### Synward Integration (Rust)

```rust
// synward-rules/semantic.rs

use synward_parser::{Parser, Node, Tree};

/// Semantic validation for CustomLang
pub struct CustomLangSemantic {
    /// Symbol table
    symbols: HashMap<String, Symbol>,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub scope: Scope,
    pub declared_at: usize,
}

#[derive(Clone, Debug)]
pub enum SymbolType {
    Function { params: Vec<Type>, returns: Type },
    Variable { mutable: bool, typ: Type },
    Constant(Type),
}

impl CustomLangSemantic {
    /// Validate semantic rules
    pub fn validate(&mut self, tree: &Tree, source: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Walk AST
        self.walk_tree(tree.root_node(), source, &mut violations);
        
        violations
    }
    
    fn walk_tree(&mut self, node: Node, source: &str, violations: &mut Vec<Violation>) {
        match node.kind() {
            "function_def" => {
                self.check_function_def(node, source, violations);
            }
            "call_expression" => {
                self.check_call_expression(node, source, violations);
            }
            "variable_def" => {
                self.check_variable_def(node, source, violations);
            }
            _ => {}
        }
        
        // Recurse
        for i in 0..node.child_count() {
            self.walk_tree(node.child(i).unwrap(), source, violations);
        }
    }
    
    fn check_function_def(&mut self, node: Node, source: &str, violations: &mut Vec<Violation>) {
        let name_node = node.child_by_field_name("name").unwrap();
        let name = name_node.utf8_text(source.as_bytes()).unwrap();
        
        // Check duplicate definition
        if self.symbols.contains_key(name) {
            violations.push(Violation {
                rule_id: "SEMANTIC001".to_string(),
                message: format!("Duplicate function definition: {}", name),
                severity: Severity::Error,
                line: name_node.start_position().row + 1,
            });
        } else {
            self.symbols.insert(name.to_string(), Symbol {
                name: name.to_string(),
                symbol_type: SymbolType::Function { 
                    params: vec![], // Extract from AST
                    returns: Type::Unknown,
                },
                scope: Scope::Global,
                declared_at: node.start_position().row,
            });
        }
    }
    
    fn check_call_expression(&self, node: Node, source: &str, violations: &mut Vec<Violation>) {
        let func_node = node.child_by_field_name("function").unwrap();
        let func_name = func_node.utf8_text(source.as_bytes()).unwrap();
        
        // Check undefined function
        if !self.symbols.contains_key(func_name) {
            violations.push(Violation {
                rule_id: "SEMANTIC002".to_string(),
                message: format!("Undefined function: {}", func_name),
                severity: Severity::Error,
                line: func_node.start_position().row + 1,
            });
        }
    }
}
```

### Build and Install

```bash
# Generate parser
cd custom-lang
tree-sitter generate

# Build for Synward
cargo build --release

# Install in Synward
cp target/release/libcustom_lang_parser.so ~/.synward/parsers/

# Register language
synward languages register custom-lang --parser libcustom_lang_parser.so
```

---

## Level 3: Full Parser Plugin

### Quando Usarlo

- Linguaggi enterprise complessi
- Integrazione con toolchain esistente
- Performance critiche
- Requirement di compliance

### Architettura Plugin

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SYNWARD PLUGIN ARCHITECTURE                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                      synward-core                                       │ │
│  │                                                                        │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │ │
│  │  │ LanguageRegistry│  │ PluginLoader    │  │ ValidationPipeline     │ │ │
│  │  └────────┬────────┘  └────────┬────────┘  └───────────┬─────────────┘ │ │
│  └───────────┼────────────────────┼───────────────────────┼───────────────┘ │
│              │                    │                       │                 │
│              └────────────────────┼───────────────────────┘                 │
│                                   │                                         │
│  ┌────────────────────────────────▼──────────────────────────────────────┐ │
│  │                      Plugin Interface (trait)                          │ │
│  │                                                                        │ │
│  │  trait LanguagePlugin {                                                │ │
│  │      fn name(&self) -> &str;                                           │ │
│  │      fn extensions(&self) -> Vec<&str>;                                │ │
│  │      fn parse(&self, source: &str) -> Result<ParseTree>;               │ │
│  │      fn validate(&self, tree: &ParseTree) -> Vec<Violation>;           │ │
│  │      fn semantic_analysis(&self, tree: &ParseTree) -> Vec<Violation>;  │ │
│  │  }                                                                     │ │
│  │                                                                        │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                   │                                         │
│              ┌────────────────────┼────────────────────┐                    │
│              │                    │                    │                    │
│  ┌───────────▼──────────┐ ┌───────▼────────┐ ┌────────▼─────────┐           │
│  │   Built-in Plugins   │ │ Tree-sitter   │ │   Custom Plugin  │           │
│  │   (Rust, Python...)  │ │   Plugins     │ │   (Your DSL)     │           │
│  └──────────────────────┘ └────────────────┘ └──────────────────┘           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Plugin Trait

```rust
// synward-plugin/src/lib.rs

use synward_core::{ParseTree, Violation, Severity};

/// Plugin interface for custom languages
pub trait LanguagePlugin: Send + Sync {
    /// Language name
    fn name(&self) -> &str;
    
    /// Supported file extensions
    fn extensions(&self) -> Vec<&str>;
    
    /// Parse source code
    fn parse(&self, source: &str) -> Result<ParseTree, ParseError>;
    
    /// Syntax validation (no semantic analysis)
    fn validate_syntax(&self, tree: &ParseTree) -> Vec<Violation>;
    
    /// Semantic analysis (types, scopes, etc.)
    fn semantic_analysis(&self, tree: &ParseTree, source: &str) -> Vec<Violation>;
    
    /// Security checks
    fn security_checks(&self, tree: &ParseTree, source: &str) -> Vec<Violation>;
    
    /// Custom rules (user-defined)
    fn custom_rules(&self, tree: &ParseTree, rules: &[CustomRule]) -> Vec<Violation>;
    
    /// Optional: LSP features
    fn lsp_capabilities(&self) -> Option<LspCapabilities> {
        None
    }
}

/// Plugin metadata
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

/// Declare plugin
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_plugin() -> *mut dyn LanguagePlugin {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }
        
        #[no_mangle]
        pub extern "C" fn plugin_metadata() -> PluginMetadata {
            <$plugin_type>::metadata()
        }
    };
}
```

### Custom Plugin Example

```rust
// my-lang-plugin/src/lib.rs

use synward_plugin::*;
use synward_core::{ParseTree, Violation, Severity, ParseError};

pub struct MyLangPlugin {
    parser: MyLangParser,
    semantic: MyLangSemantic,
    security: MyLangSecurity,
}

impl MyLangPlugin {
    pub fn new() -> Self {
        Self {
            parser: MyLangParser::new(),
            semantic: MyLangSemantic::new(),
            security: MyLangSecurity::new(),
        }
    }
    
    pub fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "my-lang".to_string(),
            version: "1.0.0".to_string(),
            author: "Company Inc.".to_string(),
            description: "Internal DSL for XYZ".to_string(),
            homepage: None,
            repository: None,
        }
    }
}

impl LanguagePlugin for MyLangPlugin {
    fn name(&self) -> &str {
        "my-lang"
    }
    
    fn extensions(&self) -> Vec<&str> {
        vec![".myl", ".mylang"]
    }
    
    fn parse(&self, source: &str) -> Result<ParseTree, ParseError> {
        self.parser.parse(source)
    }
    
    fn validate_syntax(&self, tree: &ParseTree) -> Vec<Violation> {
        self.parser.validate(tree)
    }
    
    fn semantic_analysis(&self, tree: &ParseTree, source: &str) -> Vec<Violation> {
        self.semantic.analyze(tree, source)
    }
    
    fn security_checks(&self, tree: &ParseTree, source: &str) -> Vec<Violation> {
        self.security.check(tree, source)
    }
    
    fn custom_rules(&self, tree: &ParseTree, rules: &[CustomRule]) -> Vec<Violation> {
        // Apply user-defined rules
        rules.iter()
            .flat_map(|r| self.apply_rule(tree, r))
            .collect()
    }
    
    fn lsp_capabilities(&self) -> Option<LspCapabilities> {
        Some(LspCapabilities {
            completion: true,
            goto_definition: true,
            find_references: true,
            hover: true,
            diagnostics: true,
        })
    }
}

// Declare plugin
declare_plugin!(MyLangPlugin);
```

### Build and Deploy

```bash
# Build as dynamic library
cargo build --release --crate-type cdylib

# Output: target/release/my_lang_plugin.dll (Windows)
#         target/release/libmy_lang_plugin.so (Linux)
#         target/release/libmy_lang_plugin.dylib (macOS)

# Deploy
cp target/release/my_lang_plugin.dll ~/.synward/plugins/

# Register
synward plugins register ~/.synward/plugins/my_lang_plugin.dll

# Verify
synward plugins list
```

---

## Enterprise Support (Tiers)

| Tier | Level 0 (Fallback) | Level 1 (YAML) | Level 2 (Tree-sitter) | Level 3 (Full Plugin) |
|------|-------------------|----------------|----------------------|----------------------|
| **Solo (Free)** | ✅ Automatico | ✅ Self-service | ✅ Self-service | ❌ |
| **Team** | ✅ Automatico | ✅ + Templates | ✅ + Support | ❌ |
| **Enterprise** | ✅ Automatico | ✅ + Consulting | ✅ + Integration | ✅ Full support |

---

## Supporto Enterprise (Onboarding)

Per clienti Enterprise, offriamo:

1. **Consulting iniziale** (2-4 ore): Analisi linguaggio, scelta livello
2. **Development assistito**: Templates, best practices
3. **Integration**: CI/CD, IDE plugins
4. **Training**: Workshop per team

**Contatti:** enterprise@synward.dev

---

## Quick Reference

### Level Comparison

| Aspetto | Level 0 (Fallback) | Level 1 (YAML) | Level 2 (Tree-sitter) | Level 3 (Plugin) |
|---------|-------------------|----------------|----------------------|------------------|
| Setup time | 0 (automatico) | 1-2 ore | 1-2 settimane | 1-2 mesi |
| Parsing | Nessuno (raw text) | Regex | AST completo | Custom |
| Security checks | ✅ 6 pattern base | ✅ Custom | ✅ Custom | ✅ Completo |
| Semantic analysis | ❌ | Limitato | Base | Completo |
| IDE support | ❌ | ❌ | Parziale | Completo |
| Performance | Veloce | Media | Buona | Ottimizzata |
| Expertise richiesta | Nessuna | Base | Intermedia | Avanzata |
| Costo | Gratis | Gratis | Gratis | Enterprise |

### CLI Commands

```bash
# List languages
synward languages list

# Validate with custom lang
synward validate file.xyz --lang my-lang

# Test custom language
synward languages test my-lang --file samples/

# Generate language template
synward languages create my-lang --level 1

# Register plugin
synward plugins register /path/to/plugin.dll
```
