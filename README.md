# Aether — Universal AI Validation Layer

**Version:** 0.1.0
**Status:** MCP-Driven Architecture
**Implementation Language:** Rust

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

```
Aether/
├── crates/
│   ├── aether-core/           # Core types and orchestrator
│   ├── aether-parsers/        # Parser implementations
│   ├── aether-validation/     # Validation layers
│   ├── aether-contracts/      # Contract engine
│   ├── aether-certification/  # Certificate generation
│   ├── aether-api/            # HTTP API (axum)
│   ├── aether-sdk/            # Agent SDKs
│   ├── aether-cli/            # CLI interface + RAG
│   └── aether-desktop/        # Tauri desktop app
│
├── contracts/                  # Default contracts
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

| Tool | Description |
|------|-------------|
| `aether_validate` | Validate code against contracts and rules |
| `aether_certify` | Generate cryptographic certificate for validated code |
| `aether_analyze` | Analyze code structure and extract AST statistics |

### Supported Languages

Rust, C++, Lex, Python, JavaScript, TypeScript, Go, Java, Lua

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

### Manual MCP Configuration

Add to your MCP configuration file:

**Factory CLI (`~/.factory/mcp.json`):**
```json
{
  "mcpServers": {
    "aether": {
      "type": "stdio",
      "command": "/path/to/aether-mcp-server",
      "args": ["--contracts", "/path/to/contracts"],
      "disabled": false
    }
  }
}
```

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

Proprietary — Part of the Lex/Aegis ecosystem.
