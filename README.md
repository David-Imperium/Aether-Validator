# Synward — Universal AI Validation Layer

**Version:** 0.1.0
**Status:** MCP-Driven Architecture
**Implementation Language:** Rust
**Created:** 2026-03-08 (project directory), first file: `crates/synward-core/src/lib.rs` on 2026-03-10 (02:08 UTC)
**First commit:** 2026-03-13 (13:45 CET)
**Author:** David (Imperium)

---

## What is Synward?

Synward ensures AI-generated code is trustworthy through formal validation. It acts as a **trust layer** between AI agents and production code.

**Mission:** Don't trust AI — verify AI.

**Approach:** MCP-driven — AI agents call Synward tools directly for validation.

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Multi-Language Validation** | Syntax, semantic, logic, architecture, style |
| **Contract Engine** | Declarative rules in YAML |
| **Code Certification** | Cryptographic proof (Ed25519) |
| **RAG Learning** | Learn from corrections, improve over time |
| **MCP Integration** | Native tool integration with AI agents |
| **MCP Sampling** | AI-powered suggestions via connected LLM client |
| **Progress Reporting** | Real-time progress for long-running operations |
| **Completions** | Autocomplete for prompt arguments |
| **Watch Mode** | Monitor directory for file changes |
| **Dubbioso Mode** | Confidence-based validation with filtering |
| **Test File Filtering** | Excludes LOGIC001/LOGIC002 in test code |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI AGENT                                 │
│              (Factory CLI, Claude Code, etc.)                   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              │ MCP Tool Call
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SYNWARD MCP TOOLS                             │
│                                                                 │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│   │ synward_validate │  │ synward_certify  │  │ synward_analyze  │ │
│   └────────┬────────┘  └────────┬────────┘  └────────┬────────┘ │
│            │                    │                    │          │
│            └────────────────────┼────────────────────┘          │
│                                 │                                │
└─────────────────────────────────┼────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                    VALIDATION ENGINE                            │
│                                                                 │
│   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│   │ Syntax  │ │Semantic │ │  Logic  │ │  Arch.  │ │  Style  │  │
│   └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ VALIDATION      │
                    │ RESULT/CERT     │
                    └─────────────────┘
```

---

## Project Structure

**12 crates** organized in a Cargo workspace:

```
Synward/
├── crates/
│   ├── synward-core/           # Core types and orchestrator
│   ├── synward-parsers/        # Parser implementations (24 languages)
│   ├── synward-validation/     # Validation layers (20+ layers)
│   ├── synward-contracts/      # Contract engine (118 YAML contracts)
│   ├── synward-certification/  # Certificate generation (Ed25519)
│   ├── synward-api/            # HTTP API (axum)
│   ├── synward-sdk/            # Agent SDKs
│   ├── synward-cli/            # CLI interface + RAG
│   ├── synward-mcp/            # MCP server (33 tools)
│   ├── synward-mcp-test/       # MCP integration tests
│   ├── synward-intelligence/   # AI-powered analysis (feature-gated)
│   └── synward-contract-importer/ # Import ESLint, Clippy, Pylint rules
│
├── contracts/                  # Default contracts (27 languages)
│   ├── rust/
│   ├── cpp/
│   └── lex/
│
├── templates/                  # CI/CD templates
│   └── gitlab-ci-synward.yml
│
└── test_samples/               # Test code samples
```

---

## MCP Tools

**33 tools** organized by category:

### Core Validation
| Tool | Description |
|------|-------------|
| `validate_file` | Validate a single file against contracts and rules |
| `batch_validate` | Validate multiple files in one call |
| `analyze_code` | Analyze code structure and extract AST statistics |
| `get_metrics` | Calculate code metrics (complexity, maintainability) |
| `suggest_fixes` | Get AI-powered fix suggestions via MCP sampling |
| `certify_code` | Generate cryptographic certificate for validated code |

### Info
| Tool | Description |
|------|-------------|
| `get_version` | Get Synward version and capabilities |
| `list_languages` | List all supported languages with extensions |
| `list_contracts` | List available validation contracts |
| `get_language_info` | Get supported features for a specific language |

### Watch Mode
| Tool | Description |
|------|-------------|
| `watch_start` | Start watching a directory for file changes |
| `watch_check` | Check for modified/deleted files since last check |
| `watch_stop` | Stop watching a directory |

### CodeGraph
| Tool | Description |
|------|-------------|
| `build_graph` | Build dependency graph from codebase |
| `who_calls` | Find callers of a function/symbol |
| `impact_analysis` | Analyze impact of changes |
| `file_dependencies` | Get files that this file depends on |
| `file_dependents` | Get files that depend on this file |
| `get_context` | Get relevant context for a symbol |
| `find_call_chain` | Find call chain between two symbols |

### Memory
| Tool | Description |
|------|-------------|
| `memory_recall` | Recall learned corrections from RAG |
| `memory_store` | Store a correction for future reference |

### State
| Tool | Description |
|------|-------------|
| `save_state` | Save validation state for later |
| `load_state` | Load previously saved state |

### Learning
| Tool | Description |
|------|-------------|
| `accept_violation` | Accept and learn from a violation |

### Advanced Analysis
| Tool | Description |
|------|-------------|
| `analyze_scope` | Analyze variable scopes and detect shadowing/unused variables |
| `infer_types` | Infer types for variables and expressions |
| `get_confidence` | Get confidence score for code quality |

### Compliance
| Tool | Description |
|------|-------------|
| `get_compliance_status` | Get compliance engine status and statistics |
| `evaluate_violation` | Evaluate a violation through the compliance engine |
| `accept_compliance_violation` | Accept a violation with a documented reason |

### Drift Analysis
| Tool | Description |
|------|-------------|
| `analyze_drift` | Analyze drift for a file or directory over time |
| `get_trend_analysis` | Get trend analysis for a file or project |

### Supported Languages

**24 parsers implemented:** bash, c, cpp, cuda, glsl, go, java, javascript, typescript, python, rust, sql, graphql, html, css, markdown, json, yaml, toml, cmake, lex, lua, prism, triton, notebook

See [crates/synward-mcp/docs/languages.md](crates/synward-mcp/docs/languages.md) for full details.

### Validation Layers

**20+ layers** organized by concern:

| Category | Layers |
|----------|--------|
| **Core** | ContractLayer, RulesLayer, StripperLayer, LoopDetection |
| **Preprocessing** | SyntaxLayer, ASTLayer |
| **Analysis** | SemanticLayer, LogicLayer, ComplexityLayer, ClippyLayer |
| **Security** | SecurityLayer, FallbackSecurityLayer, PrivateLayer, SupplyChainLayer |
| **Architecture** | ArchitectureLayer, StyleLayer |
| **Intelligence** | IntelligenceLayer, ComplianceLayer (feature-gated) |
| **Scope** | ScopeAnalysisLayer |
| **TypeInference** | TypeInferenceLayer |
| **LSP** | LspAnalysisLayer |

### Contracts

**118 YAML contracts** covering **27 languages**, plus **5 imported rule sets** (ESLint, Clippy, Pylint).

### Feature Flags

| Crate | Features |
|-------|----------|
| **synward-validation** | `synward-intelligence`, `memory`, `patterns`, `intent-api`, `drift`, `intelligence-full` |
| **synward-cli** | `intelligence` (default), `intent-api`, `drift` |
| **synward-intelligence** | `memory` (default), `patterns`, `tree-sitter`, `tree-sitter-multi`, `intent-api`, `drift`, `semantic-search`, `full` |

### MCP Tools Reference

See [crates/synward-mcp/docs/MCP_TOOLS.md](crates/synward-mcp/docs/MCP_TOOLS.md) for complete tool documentation.

### MCP Features

**Sampling (AI Suggestions):** Synward can request AI suggestions from the connected LLM client for fixing validation errors. Use `suggest_fixes` tool to get intelligent, context-aware fix recommendations.

**Progress Reporting:** Long-running operations report progress in real-time via MCP notifications. Track batch validation progress with progress tokens.

**Completions:** Autocomplete support for prompt arguments (languages, contracts) in MCP clients that support the completions feature.

**Watch Mode:** Monitor directories for file changes. Useful for IDE integration and continuous validation:
```
1. watch_start(directory) → watch_id
2. watch_check(watch_id) → changed_files, deleted_files
3. watch_stop(watch_id) → cleanup
```

### Usage Example

```
When you generate or write code, follow this workflow:

1. Generate Code: Write the code as requested
2. Validate: Call synward_validate with the generated code
3. Fix: If validation fails, fix the code
4. Iterate: Repeat validation until it passes (max 3)
5. Present: Only show validated code to the user
```

---

## CLI Commands

```bash
# Validate a file
synward validate src/enemy.rs --lang rust

# Analyze AST structure
synward analyze src/enemy.rs

# Generate certificate
synward certify src/enemy.rs --output cert.json

# RAG: Search for similar corrections
synward rag search "unwrap panic" --lang rust

# RAG: Show statistics
synward rag stats
```

---

## Installation

### GitHub Releases (Recommended)

Download the latest binary from [GitHub Releases](https://github.com/David-Imperium/synward/releases):

**Free Tier (synward-mcp):**
```bash
# Linux/macOS
curl -sL https://github.com/David-Imperium/synward/releases/latest/download/synward-mcp-linux-x86_64 -o synward-mcp
chmod +x synward-mcp

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/David-Imperium/synward/releases/latest/download/synward-mcp-windows-x86_64.exe -OutFile synward-mcp.exe
```

**Pro Tier (synward-cli):**
```bash
# Linux/macOS
curl -sL https://github.com/David-Imperium/synward/releases/latest/download/synward-cli-linux-x86_64 -o synward
chmod +x synward

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/David-Imperium/synward/releases/latest/download/synward-cli-windows-x86_64.exe -OutFile synward.exe
```

### Build from Source

```bash
cd Synward
cargo build --release
```

The MCP server binary is at `target/release/synward-mcp.exe` (Windows) or `target/release/synward-mcp` (Unix).

### VS Code Extension

Synward includes a VS Code extension for real-time validation:

```bash
cd extensions/vscode-synward
npm install
npm run compile
```

Then install the `.vsix` file in VS Code or press F5 to launch in development mode.

**Features:**
- Real-time validation on save
- Quality score in status bar
- Quick fixes for violations
- Compliance dashboard
- Drift analysis

### MCP Configuration

Add to your MCP configuration file:

**Factory CLI (`~/.factory/mcp.json`):**
```json
{
  "mcpServers": {
    "synward": {
      "type": "stdio",
      "command": "/path/to/synward-mcp",
      "args": [],
      "disabled": false
    }
  }
}
```

**Note:** Contracts are loaded from the built-in registry. No `--contracts` argument needed.

### Protocol Version

Synward MCP uses protocol version `2024-11-05` and requires clients to send `clientInfo` in the initialize request (per MCP specification).

---

## Relation to Other Projects

| Project | Relationship |
|---------|--------------|
| **Aegis Validation** | Predecessor. Synward is the universal evolution. |
| **Aegis (Security)** | Built in Prism — "unknown language = inviolable" is a feature |
| **Synward** | Built in Rust — MCP-driven, trusted, memory-safe |
| **Lex Compiler** | Synward validates `.lex` files via Lex adapter |
| **Prism** | Internal language. Synward uses Rust for commercial reasons |

---

## License

**BUSL-1.1** (Business Source License 1.1)

- Non-production use permitted (personal, educational, research, open-source)
- Commercial use blocked without separate license
- Converts to **AGPL-3.0-only** on **2029-04-01**

See [LICENSE](LICENSE) for full terms.
