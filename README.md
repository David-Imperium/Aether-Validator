# Aether — Universal AI Validation Layer

**Version:** 0.1.0
**Status:** MCP-Driven Architecture
**Implementation Language:** Rust
**Created:** 2026-03-10 (first file: `crates/aether-core/src/lib.rs`, 02:08 UTC)
**First commit:** 2026-03-13 (13:45 CET)
**Author:** David (Imperium)

---

## What is Aether?

Aether ensures AI-generated code is trustworthy through formal validation. It acts as a **trust layer** between AI agents and production code.

**Mission:** Don't trust AI — verify AI.

**Approach:** MCP-driven — AI agents call Aether tools directly for validation.

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
│                    AETHER MCP TOOLS                             │
│                                                                 │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│   │ aether_validate │  │ aether_certify  │  │ aether_analyze  │ │
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
Aether/
├── crates/
│   ├── aether-core/           # Core types and orchestrator
│   ├── aether-parsers/        # Parser implementations (24 languages)
│   ├── aether-validation/     # Validation layers (20+ layers)
│   ├── aether-contracts/      # Contract engine (118 YAML contracts)
│   ├── aether-certification/  # Certificate generation (Ed25519)
│   ├── aether-api/            # HTTP API (axum)
│   ├── aether-sdk/            # Agent SDKs
│   ├── aether-cli/            # CLI interface + RAG
│   ├── aether-mcp/            # MCP server (24 tools)
│   ├── aether-mcp-test/       # MCP integration tests
│   ├── aether-intelligence/   # AI-powered analysis (feature-gated)
│   └── aether-contract-importer/ # Import ESLint, Clippy, Pylint rules
│
├── contracts/                  # Default contracts (27 languages)
│   ├── rust/
│   ├── cpp/
│   └── lex/
│
├── templates/                  # CI/CD templates
│   └── gitlab-ci-aether.yml
│
└── test_samples/               # Test code samples
```

---

## MCP Tools

**24 tools** organized by category:

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
| `get_version` | Get Aether version and capabilities |
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

### Supported Languages

**24 parsers implemented:** bash, c, cpp, cuda, glsl, go, java, javascript, typescript, python, rust, sql, graphql, html, css, markdown, json, yaml, toml, cmake, lex, lua, prism, triton, notebook

See [crates/aether-mcp/docs/languages.md](crates/aether-mcp/docs/languages.md) for full details.

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
| **aether-validation** | `aether-intelligence`, `memory`, `patterns`, `intent-api`, `drift`, `intelligence-full` |
| **aether-cli** | `intelligence` (default), `intent-api`, `drift` |
| **aether-intelligence** | `memory` (default), `patterns`, `tree-sitter`, `tree-sitter-multi`, `intent-api`, `drift`, `semantic-search`, `full` |

### MCP Tools Reference

See [crates/aether-mcp/docs/MCP_TOOLS.md](crates/aether-mcp/docs/MCP_TOOLS.md) for complete tool documentation.

### MCP Features

**Sampling (AI Suggestions):** Aether can request AI suggestions from the connected LLM client for fixing validation errors. Use `suggest_fixes` tool to get intelligent, context-aware fix recommendations.

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
2. Validate: Call aether_validate with the generated code
3. Fix: If validation fails, fix the code
4. Iterate: Repeat validation until it passes (max 3)
5. Present: Only show validated code to the user
```

---

## CLI Commands

```bash
# Validate a file
aether validate src/enemy.rs --lang rust

# Analyze AST structure
aether analyze src/enemy.rs

# Generate certificate
aether certify src/enemy.rs --output cert.json

# RAG: Search for similar corrections
aether rag search "unwrap panic" --lang rust

# RAG: Show statistics
aether rag stats
```

---

## Installation

### GitHub Releases (Recommended)

Download the latest binary from [GitHub Releases](https://github.com/aether-ai/aether/releases):

**Free Tier (aether-mcp):**
```bash
# Linux/macOS
curl -sL https://github.com/aether-ai/aether/releases/latest/download/aether-mcp-linux-x86_64 -o aether-mcp
chmod +x aether-mcp

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/aether-ai/aether/releases/latest/download/aether-mcp-windows-x86_64.exe -OutFile aether-mcp.exe
```

**Pro Tier (aether-cli):**
```bash
# Linux/macOS
curl -sL https://github.com/aether-ai/aether/releases/latest/download/aether-cli-linux-x86_64 -o aether
chmod +x aether

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/aether-ai/aether/releases/latest/download/aether-cli-windows-x86_64.exe -OutFile aether.exe
```

### Build from Source

```bash
cd Aether
cargo build --release
```

The MCP server binary is at `target/release/aether-mcp.exe` (Windows) or `target/release/aether-mcp` (Unix).

### VS Code Extension

Aether includes a VS Code extension for real-time validation:

```bash
cd extensions/vscode-aether
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
    "aether": {
      "type": "stdio",
      "command": "/path/to/aether-mcp",
      "args": [],
      "disabled": false
    }
  }
}
```

**Note:** Contracts are loaded from the built-in registry. No `--contracts` argument needed.

### Protocol Version

Aether MCP uses protocol version `2024-11-05` and requires clients to send `clientInfo` in the initialize request (per MCP specification).

---

## Relation to Other Projects

| Project | Relationship |
|---------|--------------|
| **Aegis Validation** | Predecessor. Aether is the universal evolution. |
| **Aegis (Security)** | Built in Prism — "unknown language = inviolable" is a feature |
| **Aether** | Built in Rust — MCP-driven, trusted, memory-safe |
| **Lex Compiler** | Aether validates `.lex` files via Lex adapter |
| **Prism** | Internal language. Aether uses Rust for commercial reasons |

---

## License

**BUSL-1.1** (Business Source License 1.1)

- Non-production use permitted (personal, educational, research, open-source)
- Commercial use blocked without separate license
- Converts to **AGPL-3.0-only** on **2029-04-01**

See [LICENSE](LICENSE) for full terms.
