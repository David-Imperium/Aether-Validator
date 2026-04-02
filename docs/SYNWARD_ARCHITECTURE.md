# Synward — Technical Architecture

**Version:** 2.0 (Autonomous Design)
**Implementation Language:** Rust
**Related:** [SYNWARD_MASTER_DESIGN.md](./SYNWARD_MASTER_DESIGN.md), [SYNWARD_RUST_IMPLEMENTATION.md](./SYNWARD_RUST_IMPLEMENTATION.md), [ADR_AUTONOMOUS_SYNWARD.md](./ADR_AUTONOMOUS_SYNWARD.md)

---

## Overview

This document details the technical architecture of Synward, including:
- Core engine components (**AI-Free Core**)
- Data flow
- Module interfaces
- Parser abstraction layer
- Validation pipeline
- Git hooks integration

**Key Principles (v2.0):**
- **AI-Free Core**: Nessuna AI esterna richiesta per validazione
- **Memory-Driven**: Configurazione dinamica basata sulla memoria appresa
- **TOML Format**: State file leggibili e modificabili dall'utente

---

## System Architecture

### Standalone Validation Architecture

Based on market research (CodeRabbit "State of AI vs Human Code Generation Report" 2025):
- AI generates **1.7x more issues** overall
- Security issues **2.74x higher** in AI code
- Error handling **2x more problematic**
- Readability issues **3x higher**
- **84%** developers use AI, but only **29%** trust it (down from 40% in 2024)

**Key Insight:** Universal validation for all AI agents and CI/CD pipelines.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    SYNWARD VALIDATION                                          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                      VALIDATORE STANDALONE                              │ │
│  │                                                                         │ │
│  │  • CLI, VS Code Extension, CI/CD                                       │ │
│  │  • Funziona con TUTTI gli agenti AI                                    │ │
│  │  • Manuale o automatico                                                │ │
│  │                                                                         │ │
│  │  UNIVERSAL COMPATIBILITY:                                              │ │
│  │  ✅ Tutti gli agenti AI                                                │ │
│  │  ✅ Editor (VS Code, Cursor)                                           │ │
│  │  ✅ CI/CD pipelines                                                    │ │
│  │  ✅ Local models (Ollama)                                              │ │
│  │  ✅ GitHub Copilot                                                     │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │              MEMORY-DRIVEN CORE (Apprendimento)                         │ │
│  │                                                                         │ │
│  │   • LearnedConfig configura layers DINAMICAMENTE                       │ │
│  │   • Thresholds, rules, whitelist uniche per progetto                   │ │
│  │   • Migliora nel tempo: meno falsi positivi, più valore                │ │
│  │   • Vedi: MEMORY_DRIVEN_CORE.md                                        │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Core Runtime Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            SYNWARD RUNTIME                                    │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                         ORCHESTRATOR (Rust)                           │  │
│  │                                                                        │  │
│  │   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐              │  │
│  │   │   Session    │   │   Pipeline   │   │    State     │              │  │
│  │   │   Manager    │──▶│   Builder    │──▶│   Tracker    │              │  │
│  │   └──────────────┘   └──────────────┘   └──────────────┘              │  │
│  │          │                  │                  │                       │  │
│  │          └──────────────────┴──────────────────┘                       │  │
│  │                             │                                          │  │
│  │                             ▼                                          │  │
│  │                    ┌──────────────┐                                    │  │
│  │                    │   Context    │                                    │  │
│  │                    │   Registry   │                                    │  │
│  │                    └──────────────┘                                    │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                     PARSER LAYER │                                    │   │
│  │                                   ▼                                    │   │
│  │   ┌──────────────────────────────────────────────────────────────┐    │   │
│  │   │                    PARSER REGISTRY                           │    │   │
│  │   │                                                              │    │   │
│  │   │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │    │   │
│  │   │  │ C++     │ │  Rust   │ │  Lex    │ │ Python  │ │ Custom │ │    │   │
│  │   │  │ Parser  │ │ Parser  │ │ Parser  │ │ Parser  │ │ Parser │ │    │   │
│  │   │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └───┬────┘ │    │   │
│  │   │       │           │           │           │          │      │    │   │
│  │   │  ┌────┴───────────┴───────────┴───────────┴──────────┴────┐ │    │   │
│  │   │  │              UNIFIED AST INTERFACE                      │ │    │   │
│  │   │  └─────────────────────────────────────────────────────────┘ │    │   │
│  │   └──────────────────────────────────────────────────────────────┘    │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                   VALIDATION LAYER │                                  │   │
│  │                                     ▼                                 │   │
│  │   ┌──────────────────────────────────────────────────────────────┐    │   │
│  │   │                  VALIDATION PIPELINE                         │    │   │
│  │   │                                                              │    │   │
│  │   │  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐       │    │   │
│  │   │  │Contract │──▶│ Syntax  │──▶│Semantic │──▶│  Logic  │       │    │   │
│  │   │  │ Layer   │   │ Check   │   │ Check   │   │ Check   │       │    │   │
│  │   │  └─────────┘   └─────────┘   └─────────┘   └────┬────┘       │    │   │
│  │   │                                                   │            │    │   │
│  │   │                                              ┌────┴────┐       │    │   │
│  │   │                                              │  Arch.  │       │    │   │
│  │   │                                              │  Check  │       │    │   │
│  │   │                                              └────┬────┘       │    │   │
│  │   │                                                   │            │    │   │
│  │   │                                              ┌────┴────┐       │    │   │
│  │   │                                              │  Style  │       │    │   │
│  │   │                                              │  Check  │       │    │   │
│  │   │                                              └─────────┘       │    │   │
│  │   └──────────────────────────────────────────────────────────────┘    │   │
│  │                                                                        │   │
│  │   ┌──────────────────────────────────────────────────────────────┐    │   │
│  │   │                    CONTRACT ENGINE                           │    │   │
│  │   │  ┌────────────┐  ┌────────────┐  ┌────────────────────┐      │    │   │
│  │   │  │  Contract  │  │   Rule     │  │  Violation         │      │    │   │
│  │   │  │  Registry  │  │  Evaluator │  │  Reporter          │      │    │   │
│  │   │  └────────────┘  └────────────┘  └────────────────────┘      │    │   │
│  │   └──────────────────────────────────────────────────────────────┘    │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                    OUTPUT LAYER │                                      │   │
│  │                                  ▼                                     │   │
│  │   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐              │   │
│  │   │    Report    │   │  Certificate │   │   Feedback   │              │   │
│  │   │   Generator  │   │   Generator  │   │   Generator  │              │   │
│  │   └──────────────┘   └──────────────┘   └──────────────┘              │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Orchestrator

The orchestrator manages the entire validation session lifecycle.

```rust
// src/core/orchestrator.rs
use std::collections::HashMap;
use uuid::Uuid;

pub struct Orchestrator {
    sessions: SessionManager,
    pipeline_builder: PipelineBuilder,
    state_tracker: StateTracker,
    context_registry: ContextRegistry,
}

impl Orchestrator {
    /// Create a new validation session
    pub fn create_session(&mut self, config: SessionConfig) -> Result<SessionId, SynwardError> {
        self.sessions.create(config)
    }

    /// Destroy a session
    pub fn destroy_session(&mut self, id: SessionId) -> Result<(), SynwardError> {
        self.sessions.destroy(id)
    }

    /// Execute validation pipeline
    pub fn validate(&mut self, id: SessionId, request: ValidationRequest) -> ValidationResult {
        let session = self.sessions.get(id)?;
        let pipeline = self.pipeline_builder.build(&session.config)?;
        pipeline.execute(request)
    }

    /// Get iteration count
    pub fn iteration_count(&self, id: SessionId) -> Result<usize, SynwardError> {
        self.state_tracker.iteration_count(id)
    }

    /// Check if retry is allowed
    pub fn can_retry(&self, id: SessionId) -> Result<bool, SynwardError> {
        let session = self.sessions.get(id)?;
        Ok(self.state_tracker.iteration_count(id)? < session.config.max_iterations)
    }
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub language: String,
    pub contract_paths: Vec<PathBuf>,
    pub max_iterations: usize,
    pub learning_enabled: bool,
    pub cert_level: CertificationLevel,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            language: "rust".to_string(),
            contract_paths: vec![PathBuf::from("./contracts")],
            max_iterations: 3,
            learning_enabled: true,
            cert_level: CertificationLevel::Full,
        }
    }
}

#[derive(Debug)]
pub struct ValidationRequest {
    pub source: String,
    pub file_path: Option<PathBuf>,
    pub prompt_context: Option<PromptContext>,
}
```
```

### 2. Parser Abstraction Layer

Language-specific parsers implement a unified interface, producing a common AST representation.

```rust
// src/parsers/mod.rs
use std::collections::HashMap;

/// Unified AST node type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Module,
    Function,
    Struct,
    Enum,
    Impl,
    Trait,
    Variable,
    Expression,
    Statement,
    Type,
    Comment,
    Import,
    Export,
    // ... language-specific extensions
}

/// Unified AST representation
#[derive(Debug, Clone)]
pub struct ASTNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub name: String,
    pub location: SourceLocation,
    pub children: Vec<ASTNode>,
    pub attributes: HashMap<String, String>,
}

/// Parser trait - language-specific implementations
pub trait Parser: Send + Sync {
    fn parse(&self, source: &str) -> ParseResult;
    fn language(&self) -> &str;
    fn supported_extensions(&self) -> &[&str];
}

/// Parse result
#[derive(Debug)]
pub struct ParseResult {
    pub success: bool,
    pub ast: Option<ASTNode>,
    pub errors: Vec<SyntaxError>,
    pub tokens: Vec<Token>,
}

/// Parser registry
pub struct ParserRegistry {
    parsers: HashMap<String, Box<dyn Parser>>,
    extension_map: HashMap<String, String>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            extension_map: HashMap::new(),
        }
    }

    pub fn register<P: Parser + 'static>(&mut self, parser: P) {
        let lang = parser.language().to_string();
        for ext in parser.supported_extensions() {
            self.extension_map.insert(ext.to_string(), lang.clone());
        }
        self.parsers.insert(lang, Box::new(parser));
    }

    pub fn get(&self, language: &str) -> Option<&dyn Parser> {
        self.parsers.get(language).map(|p| p.as_ref())
    }

    pub fn get_for_extension(&self, ext: &str) -> Option<&dyn Parser> {
        self.extension_map.get(ext)
            .and_then(|lang| self.parsers.get(lang))
            .map(|p| p.as_ref())
    }
}
```
```

### 3. Tree-Sitter Integration

Tree-sitter provides robust parsing for most languages.

```rust
// src/parsers/tree_sitter.rs
use tree_sitter::{Parser, Node, Tree};

pub struct TreeSitterParser {
    parser: Parser,
    lang_name: String,
}

impl TreeSitterParser {
    pub fn new(language: tree_sitter::Language, lang_name: &str) -> Result<Self, SynwardError> {
        let mut parser = Parser::new();
        parser.set_language(language)?;
        Ok(Self {
            parser,
            lang_name: lang_name.to_string(),
        })
    }

    pub fn parse(&mut self, source: &str) -> ParseResult {
        let tree = self.parser.parse(source, None);

        match tree {
            Some(tree) => {
                let root = tree.root_node();
                let ast = self.convert_tree(root, source);
                ParseResult {
                    success: true,
                    ast: Some(ast),
                    errors: vec![],
                    tokens: vec![],
                }
            }
            None => ParseResult {
                success: false,
                ast: None,
                errors: vec![SyntaxError::ParseError("Failed to parse".into())],
                tokens: vec![],
            },
        }
    }

    fn convert_tree(&self, node: Node, source: &str) -> ASTNode {
        // Convert tree-sitter node to unified AST
        // ...
    }
}

/// Tree-sitter language loader
pub struct TreeSitterLoader;

impl TreeSitterLoader {
    pub fn load_rust() -> Result<TreeSitterParser, SynwardError> {
        TreeSitterParser::new(tree_sitter_rust::language(), "rust")
    }

    pub fn load_cpp() -> Result<TreeSitterParser, SynwardError> {
        TreeSitterParser::new(tree_sitter_cpp::language(), "cpp")
    }

    pub fn load_python() -> Result<TreeSitterParser, SynwardError> {
        TreeSitterParser::new(tree_sitter_python::language(), "python")
    }
}
```
```

### 4. Validation Pipeline

The pipeline chains validation layers in sequence.

```rust
// src/validation/pipeline.rs
use std::sync::Arc;

pub struct ValidationPipeline {
    layers: Vec<Box<dyn ValidationLayer>>,
}

impl ValidationPipeline {
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn add_layer<L: ValidationLayer + 'static>(&mut self, layer: L) {
        self.layers.push(Box::new(layer));
    }

    pub fn execute(&self, ast: &ASTNode, ctx: &ValidationContext) -> PipelineResult {
        let mut result = PipelineResult::new();

        for layer in &self.layers {
            let layer_result = layer.validate(ast, ctx);
            result.merge_layer(layer.name(), layer_result);

            if !layer.can_continue(&result) {
                result.should_stop = true;
                break;
            }
        }

        result
    }
}

/// Layer trait
pub trait ValidationLayer: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult;
    fn can_continue(&self, result: &LayerResult) -> bool;
}

#[derive(Debug)]
pub struct LayerResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub infos: Vec<Info>,
    pub should_stop: bool,
}

#[derive(Debug)]
pub struct ValidationContext {
    pub language: String,
    pub contracts: Arc<ContractRegistry>,
    pub patterns: Arc<PatternLibrary>,
    pub project: Option<ProjectContext>,
    pub custom_data: HashMap<String, Box<dyn Any>>,
}
```
```

### 5. Validation Layers

#### Contract Layer

The ContractLayer loads human-authored YAML contracts and evaluates them against source code using regex patterns. This is the first layer in the pipeline, enabling project-specific validation rules.

```rust
// src/validation/layers/contract.rs
use serde::{Deserialize, Serialize};
use regex::Regex;

/// YAML contract definition
#[derive(Debug, Clone, Deserialize)]
pub struct ContractDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: String,  // error, warning, info
    pub pattern: String,   // regex pattern
    pub suggestion: Option<String>,
}

/// Contract layer - loads and evaluates YAML contracts
pub struct ContractLayer {
    contracts_path: PathBuf,
    cache: HashMap<String, Vec<ContractDef>>,
}

impl ContractLayer {
    pub fn with_path(path: &Path) -> Self {
        Self {
            contracts_path: path.to_path_buf(),
            cache: HashMap::new(),
        }
    }

    fn load_contracts(&mut self, language: &str) -> Vec<ContractDef> {
        // Load from ~/.synward/contracts/{language}/*.yaml
        // Cache for subsequent calls
    }

    fn evaluate_pattern(&self, pattern: &str, source: &str) -> Vec<Match> {
        // All patterns treated as regex
        // Supports multiline matching with [\s\S]*?
    }
}

impl ValidationLayer for ContractLayer {
    fn name(&self) -> &str { "contract" }

    fn validate(&self, source: &str, language: &str) -> LayerResult {
        let contracts = self.load_contracts(language);
        let mut violations = Vec::new();

        for contract in contracts {
            for m in self.evaluate_pattern(&contract.pattern, source) {
                violations.push(Violation {
                    id: contract.id.clone(),
                    message: contract.description.clone(),
                    severity: parse_severity(&contract.severity),
                    suggestion: contract.suggestion.clone(),
                    location: m.location,
                });
            }
        }

        LayerResult { violations, .. }
    }
}
```

**Key Features:**
- Loads YAML contracts from `~/.synward/contracts/{language}/*.yaml`
- Regex patterns support multiline matching via `[\s\S]*?`
- Severity mapping: `error`, `warning`, `info`
- Caches loaded contracts per language

**Contract File Example:**
```yaml
# ~/.synward/contracts/python/error-handling.yaml
contracts:
  - id: PYERR001
    name: silent-exception-caught
    description: "Silent exception caught - errors are hidden"
    severity: error
    pattern: "except[^:]*:[\\s\\S]*?pass"
    suggestion: "Log or handle the exception properly"
```

#### Syntax Layer

```rust
// src/validation/layers/syntax.rs
use super::{ValidationLayer, LayerResult, ValidationContext};
use crate::parsers::ASTNode;

pub struct SyntaxLayer;

impl ValidationLayer for SyntaxLayer {
    fn name(&self) -> &str { "syntax" }

    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult {
        let mut result = LayerResult::passed();

        // Check for unparsed nodes
        self.check_for_parse_errors(ast, &mut result);

        // Check for invalid tokens
        self.check_for_invalid_tokens(ast, &mut result);

        // Language-specific syntax rules
        self.apply_syntax_rules(ast, &ctx.language, &mut result);

        result
    }

    fn can_continue(&self, result: &LayerResult) -> bool {
        // Syntax errors block further validation
        result.passed
    }
}

impl SyntaxLayer {
    fn check_for_parse_errors(&self, node: &ASTNode, result: &mut LayerResult) {
        // Implementation...
    }

    fn check_for_invalid_tokens(&self, node: &ASTNode, result: &mut LayerResult) {
        // Implementation...
    }

    fn apply_syntax_rules(&self, node: &ASTNode, language: &str, result: &mut LayerResult) {
        // Language-specific rules...
    }
}
```

#### Semantic Layer

```rust
// src/validation/layers/semantic.rs
pub struct SemanticLayer;

impl ValidationLayer for SemanticLayer {
    fn name(&self) -> &str { "semantic" }

    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult {
        let mut result = LayerResult::passed();

        // Build symbol table
        let symbols = self.build_symbol_table(ast);

        // Check undefined references
        self.check_undefined_references(ast, &symbols, &mut result);

        // Check type consistency
        self.check_type_consistency(ast, &symbols, &mut result);

        // Check scope rules
        self.check_scope_rules(ast, &mut result);

        result
    }
}
```

#### Logic Layer

```rust
// src/validation/layers/logic.rs
pub struct LogicLayer;

impl ValidationLayer for LogicLayer {
    fn name(&self) -> &str { "logic" }

    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult {
        let mut result = LayerResult::passed();

        // Apply domain contracts
        for contract in ctx.contracts.get_contracts_for_domain("logic") {
            let violations = contract.evaluate(ast, ctx);
            result.violations.extend(violations);
        }

        // Detect logical issues
        self.detect_unreachable_code(ast, &mut result);
        self.detect_unused_variables(ast, &mut result);
        self.detect_dead_code(ast, &mut result);

        result.passed = result.violations.iter()
            .all(|v| v.severity != Severity::Error);

        result
    }
}
```

#### Architecture Layer

```rust
// src/validation/layers/architecture.rs
pub struct ArchitectureLayer;

impl ValidationLayer for ArchitectureLayer {
    fn name(&self) -> &str { "architecture" }

    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult {
        let mut result = LayerResult::passed();

        // Build dependency graph
        let deps = self.build_dependency_graph(ast);

        // Detect circular dependencies
        self.detect_circular_dependencies(&deps, &mut result);

        // Check layer boundaries (if project has defined layers)
        if let Some(project) = &ctx.project {
            if project.has_architecture_layers() {
                self.check_layer_boundaries(ast, project.layers(), &mut result);
            }
        }

        // Apply architecture contracts
        for contract in ctx.contracts.get_contracts_for_domain("architecture") {
            let violations = contract.evaluate(ast, ctx);
            result.violations.extend(violations);
        }

        result
    }
}
```

#### Style Layer

```rust
// src/validation/layers/style.rs
pub struct StyleLayer;

impl ValidationLayer for StyleLayer {
    fn name(&self) -> &str { "style" }

    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult {
        let mut result = LayerResult::passed();

        // Check naming conventions
        self.check_naming_conventions(ast, &ctx.language, &mut result);

        // Check formatting
        self.check_formatting(ast, &mut result);

        // Check comment coverage
        self.check_comment_coverage(ast, &mut result);

        // Apply style contracts
        for contract in ctx.contracts.get_contracts_for_domain("style") {
            let violations = contract.evaluate(ast, ctx);
            result.violations.extend(violations);
        }

        result
    }
}
```

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           INPUT                                         │
│  ┌─────────────┐                                                        │
│  │   Source    │  Source code (string) + optional file path            │
│  │   Code      │  + optional prompt context                            │
│  └──────┬──────┘                                                        │
└─────────┼───────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        PARSING                                          │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                    │
│  │   Detect    │──▶│    Parse    │──▶│   Build     │                    │
│  │  Language   │   │   Source    │   │    AST      │                    │
│  └─────────────┘   └─────────────┘   └─────────────┘                    │
│                                                │                         │
│                                                ▼                         │
│                                        ┌─────────────┐                   │
│                                        │  Unified    │                   │
│                                        │    AST      │                   │
│                                        └─────────────┘                   │
└──────────────────────────────────────────┬──────────────────────────────┘
                                           │
                                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      VALIDATION PIPELINE                                │
│                                                                         │
│   ┌──────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│   │ Contract │─▶│ Syntax  │─▶│Semantic │─▶│  Logic  │─▶│  Arch.  │    │
│   │  Layer   │  │ Check   │  │ Check   │  │ Check   │  │ Check   │    │
│   └────┬─────┘  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘    │
│        │             │            │            │             │          │
│        └─────────────┴────────────┴────────────┴─────────────┘         │
│                                              │                         │
│                                         ┌────┴────┐                     │
│                                         │  Style  │                     │
│                                         │  Check  │                     │
│                                         └────┬────┘                     │
│                                              │                          │
│                                              ▼                          │
│                              ┌───────────────┐                           │
│                              │ Violations    │                           │
│                              │ Collected     │                           │
│                              └───────────────┘                           │
└──────────────────────────────┬─────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          OUTPUT                                         │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────┐       │
│   │                    Validation Result                        │       │
│   │                                                             │       │
│   │  passed: bool                                               │       │
│   │  violations: Violation[]                                    │       │
│   │  metrics: { errors, warnings, infos, score }                │       │
│   │  certificate?: Certificate (if passed)                      │       │
│   │  feedback?: Feedback (if failed, for AI retry)              │       │
│   │                                                             │       │
│   └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Violation Structure

```rust
// src/violation.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    // Identification
    pub id: String,              // "RUST-MEM-001"
    pub contract_name: String,    // "no-unwrap-without-context"

    // Classification
    pub severity: Severity,
    pub category: String,        // "memory-safety"
    pub domain: String,           // "rust"

    // Location
    pub location: SourceLocation,
    pub range: SourceRange,

    // Message
    pub message: String,          // "unwrap() called without context"
    pub suggestion: String,       // "Use expect() with context message"
    pub example_fix: String,      // Actual code example

    // AI Context
    pub ai_hint: String,           // Context for AI to understand why
    pub related_patterns: Vec<String>,

    // Learning
    pub is_learned: bool,          // From pattern library
    pub confidence: f32,           // 0.0 - 1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,      // Must fix
    Warning,    // Should fix
    Info,       // FYI
    Hint,       // Suggestion
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}
```
```

---

## Configuration

```yaml
# .synward/config.yaml
version: "1.0"

# Language settings
languages:
  cpp:
    standard: "c++20"
    parser: "tree-sitter"  # or "clang"
  rust:
    edition: "2021"
    parser: "syn"

# Contract paths
contracts:
  paths:
    - "./contracts/"
    - "./.synward/contracts/"
  
# Validation settings
validation:
  max_iterations: 3
  stop_on_syntax_error: true
  parallel_layers: false
  
# Severity thresholds
thresholds:
  error_limit: 0      # Max errors allowed
  warning_limit: 10   # Max warnings before fail
  score_minimum: 80   # Minimum quality score

# Learning
learning:
  enabled: true
  pattern_library: "./.synward/patterns/"
  
# Certification
certification:
  enabled: true
  signing_key: "./.synward/keys/private.key"
  level: "full"  # basic, standard, full

# Output
output:
  format: "json"  # json, yaml, text
  include_source_snippets: true
  include_fix_examples: true
```

---

## Performance Considerations

### Parsing

- **Tree-sitter** is incremental — only re-parse changed portions
- **AST caching** — Cache parsed ASTs for unchanged files
- **Lazy parsing** — Only parse what's needed for validation

### Validation

- **Layer independence** — Some layers can run in parallel
- **Early termination** — Stop pipeline on critical errors
- **Contract indexing** — Pre-index contracts for fast lookup

### Memory

- **AST pooling** — Reuse AST nodes across validations
- **String interning** — Deduplicate strings in AST
- **Violation pooling** — Pre-allocate violation objects

### Benchmarks (Target)

| Operation | Target |
|-----------|--------|
| Parse 1000-line C++ file | < 50ms |
| Full validation (5 layers) | < 100ms |
| Contract evaluation (100 contracts) | < 20ms |
| Certificate generation | < 10ms |
| Total validation request | < 200ms |

---

## Thread Safety

Synward is designed for concurrent use:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

// Thread-safe session isolation
let orchestrator = Arc::new(RwLock::new(Orchestrator::new()));

// Create sessions
let session1 = orchestrator.write().await.create_session(config1)?;
let session2 = orchestrator.write().await.create_session(config2)?;

// Sessions can be validated concurrently
let o1 = orchestrator.clone();
let o2 = orchestrator.clone();

let r1 = tokio::spawn(async move {
    o1.read().await.validate(session1, request1).await
});

let r2 = tokio::spawn(async move {
    o2.read().await.validate(session2, request2).await
});

let (result1, result2) = tokio::try_join!(r1, r2)?;
```

**Thread-safe components:**
- `Orchestrator` — Sessions are isolated via `Arc<RwLock<...>>`
- `ParserRegistry` — Read-only after initialization, uses `Arc`
- `ContractRegistry` — Read-only after loading, uses `Arc`
- `PatternLibrary` — Concurrent reads via `Arc<RwLock<...>>`

**Session-local state:**
- `ValidationContext` — Per-session, no sharing
- `StateTracker` — Per-session, no sharing
- Violation accumulation — Per-session, no sharing

---

## Error Handling

```rust
// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynwardError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Certification error: {0}")]
    CertificationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

impl SynwardError {
    pub fn recoverable(&self) -> bool {
        matches!(self,
            SynwardError::ParseError(_) |
            SynwardError::ContractError(_)
        )
    }
}
```

Error recovery strategies:
- **Parse errors** — Return partial AST, continue with available nodes
- **Contract errors** — Skip malformed contracts, log warning
- **Configuration errors** — Use defaults, log warning
- **Certification errors** — Return validation result without certificate

---

## Extension Points

### Custom Parsers

```rust
use crate::parsers::{Parser, ParseResult};

struct MyCustomParser;

impl Parser for MyCustomParser {
    fn parse(&self, source: &str) -> ParseResult {
        // Implementation...
    }

    fn language(&self) -> &str {
        "my-custom-lang"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["mcl", "mylang"]
    }
}

// Register
let mut registry = ParserRegistry::new();
registry.register(MyCustomParser);
```

### Custom Validation Layers

```rust
use crate::validation::{ValidationLayer, LayerResult, ValidationContext};
use crate::parsers::ASTNode;

struct MyCustomLayer;

impl ValidationLayer for MyCustomLayer {
    fn name(&self) -> &str { "my-custom" }

    fn validate(&self, ast: &ASTNode, ctx: &ValidationContext) -> LayerResult {
        // Implementation...
    }

    fn can_continue(&self, result: &LayerResult) -> bool {
        result.passed
    }
}

// Add to pipeline
let mut pipeline = ValidationPipeline::new();
pipeline.add_layer(MyCustomLayer);
```

### Custom Contracts

```yaml
# my-contracts/custom.contracts.yaml
domain: my-dsl
contracts:
  - id: MYDSL-001
    name: custom-rule
    # ...
```

---

## Dependencies

| Crate | Purpose | License |
|-------|---------|---------|
| **tree-sitter** | Incremental parsing | MIT |
| **syn** | Rust AST parsing | MIT |
| **nom** | Parser combinators | MIT |
| **serde** | Serialization | MIT |
| **toml** | TOML handling (human-readable state) | MIT |
| **serde_yaml** | YAML config | MIT |
| **ed25519-dalek** | Certificate signing | Apache-2.0 |
| **tokio** | Async runtime | MIT |
| **tracing** | Logging | MIT |
| **clap** | CLI interface | MIT |
| **thiserror** | Error handling | MIT |

---

## Build System

```toml
# Cargo.toml
[package]
name = "synward"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "AI Code Validation & Certification System"

[workspace]
members = [
    "crates/synward-core",
    "crates/synward-parsers",
    "crates/synward-validation",
    "crates/synward-contracts",
    "crates/synward-certification",
    "crates/synward-cli",
]

[dependencies]
# Parser
tree-sitter = "0.24"
tree-sitter-rust = "0.21"
tree-sitter-cpp = "0.22"
syn = { version = "2.0", features = ["full", "parsing"] }
nom = "7.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
serde_yaml = "0.9"

# Crypto
ed25519-dalek = { version = "2.1", features = ["rand_core"] }
rand = "0.8"

# Async
tokio = { version = "1.0", features = ["full"] }

# CLI
clap = { version = "4.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
tempfile = "3.10"

[[bench]]
name = "validation_benchmark"
harness = false
```

### Project Structure

```
synward/
├── crates/
│   ├── synward-core/         # Core types and orchestrator
│   ├── synward-parsers/      # Parser implementations
│   ├── synward-validation/   # Validation layers
│   ├── synward-contracts/     # Contract engine
│   ├── synward-certification/ # Certificate generation
│   └── synward-cli/          # CLI interface
├── contracts/               # Default contracts
├── benches/                # Benchmarks
├── tests/                   # Integration tests
└── Cargo.toml
```

---

---

## Commercial Components

### Memory-Driven Core Architecture

> **Architettura completa:** Vedi [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md)

Il Memory-Driven Core non si limita a memorizzare — **configura dinamicamente** i validation layers basandosi sulla knowledge appresa dal progetto.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                      MEMORY-DRIVEN CORE                                      │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                         STORAGE BACKENDS                               │  │
│  │                                                                        │  │
│  │   ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────┐  │  │
│  │   │    SQLite    │   │   Qdrant     │   │      PostgreSQL          │  │  │
│  │   │   (Solo/Pro) │   │   (Team)     │   │      (Enterprise)        │  │  │
│  │   │              │   │              │   │                          │  │  │
│  │   │ • Local file │   │ • Vector DB  │   │ • pgvector extension    │  │  │
│  │   │ • Keyword    │   │ • Hybrid     │   │ • Hybrid search         │  │  │
│  │   │   only       │   │   search     │   │ • Audit & backup        │  │  │
│  │   └──────────────┘   └──────────────┘   └──────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                     LEARNED CONFIG (Dynamic) │                         │   │
│  │                                   ▼                                    │   │
│  │   LearnedConfig {                                                      │   │
│  │     thresholds: HashMap<String, f64>,    // complexity, line_length   │   │
│  │     custom_rules: Vec<DiscoveredRule>,   // Generated from patterns   │   │
│  │     security_whitelist: Vec<WhitelistEntry>, // Accepted violations   │   │
│  │     conventions: StyleConventions,       // Learned from codebase     │   │
│  │   }                                                                    │   │
│  │                                                                        │   │
│  │   → Applicato ai layers PRIMA di ogni validazione                      │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                        DATA MODEL │                                    │   │
│  │                                   ▼                                    │   │
│  │   ProjectMemory {                                                      │   │
│  │     project_id: Uuid,                                                  │   │
│  │     decisions: Vec<ArchitecturalDecision>,  // "Why we chose X"       │   │
│  │     violations: Vec<ViolationHistory>,      // Fixed/ignored/FPs      │   │
│  │     patterns: Vec<LearnedPattern>,          // Naming, idioms         │   │
│  │     annotations: Vec<UserAnnotation>,       // User notes             │   │
│  │   }                                                                    │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                        QUERY INTERFACE │                               │   │
│  │                                   ▼                                    │   │
│  │   trait MemoryStore {                                                  │   │
│  │     async fn load_config(&self, project) -> LearnedConfig;            │   │
│  │     async fn store_decision(&self, decision) -> Result<Uuid>;         │   │
│  │     async fn record_feedback(&self, validation, user_action);         │   │
│  │     async fn hybrid_search(&self, query, project) -> SearchResult;    │   │
│  │   }                                                                    │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Cosa la memoria CONFIGURA:**
| Layer | Configurazione Dinamica |
|-------|------------------------|
| Syntax | `max_complexity`, `max_line_length`, `max_params` |
| Security | `whitelist` (accepted violations with reason) |
| Logic | `custom_rules` (discovered from patterns) |
| Style | `conventions` (naming, formatting from codebase) |

**Cosa la memoria NON tocca:**
- Parser/AST, sintassi base, security hard limits, pipeline execution

### Billing System Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          BILLING SYSTEM                                      │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                        ACCOUNT MANAGEMENT                              │  │
│  │                                                                        │  │
│  │   Account {                                                            │  │
│  │     id: Uuid,                                                          │  │
│  │     email: String,                                                     │  │
│  │     tier: SubscriptionTier,  // Solo/Pro/Team/Enterprise              │  │
│  │     subscription: Option<Subscription>,                                │  │
│  │     usage: UsageStats,                                                 │  │
│  │     credits_balance: u32,                                              │  │
│  │   }                                                                    │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                        RATE LIMITING │                                 │   │
│  │                                   ▼                                    │   │
│  │   ┌───────────────────┐   ┌───────────────────────────────────────┐   │   │
│  │   │      Redis        │   │         Usage Limits by Tier          │   │   │
│  │   │   (Rate Counter)  │   │                                       │   │   │
│  │   │                   │   │   Solo:   100 scans/day              │   │   │
│  │   │ • scans:{id}:{day}│   │   Pro:    1,000 scans/day            │   │   │
│  │   │ • TTL: 24h        │   │   Team:   Unlimited                   │   │   │
│  │   └───────────────────┘   │   Enterprise: Unlimited               │   │   │
│  │                           └───────────────────────────────────────┘   │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│  ┌─────────────────────────────────┼─────────────────────────────────────┐   │
│  │                      STRIPE INTEGRATION │                              │   │
│  │                                   ▼                                    │   │
│  │   • create_subscription(account, tier, payment_method)                │   │
│  │   • handle_webhook(event)  // invoice.paid, subscription.deleted      │   │
│  │   • purchase_credits(account, pack)                                   │   │
│  │                                                                        │   │
│  │   Credit Packs:                                                        │   │
│  │   ┌─────────────┬─────────┬────────────┐                              │   │
│  │   │ Pack        │ Price   │ Scans      │                              │   │
│  │   ├─────────────┼─────────┼────────────┤                              │   │
│  │   │ Starter     │ $9      │ 500        │                              │   │
│  │   │ Boost       │ $29     │ 2,000      │                              │   │
│  │   │ Power       │ $99     │ 10,000     │                              │   │
│  │   │ Enterprise  │ $500    │ 100,000    │                              │   │
│  │   └─────────────┴─────────┴────────────┘                              │   │
│  └───────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Enterprise Features

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        ENTERPRISE FEATURES                                   │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                           SSO / SAML                                   │  │
│  │                                                                        │  │
│  │   • Okta, Azure AD, Google Workspace integration                      │  │
│  │   • Role-based access control                                          │  │
│  │   • Organization-level billing                                         │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                      COMPLIANCE REPORTS                                │  │
│  │                                                                        │  │
│  │   • SOC 2 Type II compliance reports                                   │  │
│  │   • ISO 27001 control mapping                                          │  │
│  │   • Custom audit trails                                                │  │
│  │   • Violation trends over time                                         │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                     ON-PREMISE DEPLOYMENT                              │  │
│  │                                                                        │  │
│  │   docker-compose:                                                      │  │
│  │   ├── synward-api      (validation service)                            │  │
│  │   ├── synward-mcp      (MCP server)                                    │  │
│  │   ├── postgres        (database)                                      │  │
│  │   ├── redis           (rate limiting)                                 │  │
│  │   └── qdrant          (vector storage)                                │  │
│  │                                                                        │  │
│  │   Features:                                                            │  │
│  │   • Air-gapped operation                                               │  │
│  │   • Custom contract loading                                           │  │
│  │   • License key activation                                            │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.
4. **Implement CLI** — Basic validate command with clap
5. **Add Tests** — Unit tests for each component
6. **Setup CI/CD** — GitHub Actions for build + test + release
7. **Commercial Launch** — Billing, RAG, tier enforcement (Phase 10)

For Rust-specific implementation details, see [SYNWARD_RUST_IMPLEMENTATION.md](./SYNWARD_RUST_IMPLEMENTATION.md).
For pricing strategy, see [PRICING_STRATEGY.md](./PRICING_STRATEGY.md).
