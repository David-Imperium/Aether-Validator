# Synward — Universal Autonomous Validation Layer

**Version:** 3.0 (Core First Philosophy)
**Status:** 85% Complete (Phase 15 in progress)
**Last Updated:** 2026-03-19
**Implementation Language:** Rust
**See Also:** [ADR_AUTONOMOUS_SYNWARD.md](./ADR_AUTONOMOUS_SYNWARD.md)

---

## Executive Summary

Synward is a universal **autonomous** validation framework that ensures AI-generated code meets rigorous quality standards. It acts as a **trust layer** between AI agents and production code, providing:

- **AI-Free Core** — No external AI required for validation (optional dictionary role)
- **Multi-Language Validation** — Syntax, semantic, logic, and style checks
- **Contract Engine** — Formal verification against defined rules
- **Code Certification** — Cryptographic proof of validated code
- **Memory-Driven Learning** — Dynamic layer configuration from learned patterns

**Key Principles (v3.0):**
- **AI-Free Core**: Nessuna AI esterna richiesta per validazione
- **Graph RAG Autonomo**: Attraversa progetti, capisce dipendenze, impara pattern
- **Dubbioso Mode**: Confidence-based validation, chiede quando incerto via MCP
- **TOML Format**: Memoria leggibile e modificabile dall'utente

**Mission:** Make AI-generated code trustworthy through autonomous validation, not hope.

**Key Differentiator — Core First Philosophy:**
All core features (Memory-Driven Core, Hyper-Context, Dubbioso, MCP, Custom Contracts) are available to everyone.

> See [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md) for full architecture.

---

## The Problem

The market doesn't trust AI agents with code because:

1. **No accountability** — Who's responsible when AI breaks production?
2. **Inconsistent quality** — Same prompt, different results
3. **Context blindness** — AI doesn't know project conventions
4. **No verification** — Output is taken as-is without validation
5. **Hidden bugs** — Logic errors that pass syntax checks

**Synward's solution:** Don't trust AI — verify AI.

---

## High-Level Architecture

### Standalone Validation Architecture (2026)

Based on market research (CodeRabbit Report 2025):
- AI generates **1.7x more issues** overall
- Security issues **2.74x higher** in AI code
- **84%** developers use AI, but only **29%** trust it

**Key Insight:** Universal validation for all AI agents and CI/CD pipelines.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SYNWARD VALIDATION                                         │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      VALIDATORE STANDALONE                           │   │
│  │                                                                       │   │
│  │  • CLI, VS Code Extension, CI/CD                                     │   │
│  │  • UNIVERSALE: tutti gli agenti AI                                   │   │
│  │                                                                       │   │
│  │  Compatibilita':                                                     │   │
│  │  [x] Droid, Claude Code, Cursor                                      │   │
│  │  [x] Ollama, Copilot                                                 │   │
│  │  [x] CI/CD, manual validation                                        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │              MEMORY-DRIVEN CORE (Apprendimento)                      │   │
│  │   • LearnedConfig → Layers dinamici                                 │   │
│  │   • Thresholds, rules, conventions → Unici per progetto            │   │
│  │   • Pattern, errori, correzioni → Migliora nel tempo               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Core Engine Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SYNWARD LAYER                                   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        ORCHESTRATION ENGINE                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │   │
│  │  │   Prompt    │  │  Contract   │  │  Iteration  │  │   Report   │  │   │
│  │  │  Analyzer   │  │   Engine    │  │    Loop     │  │  Generator │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        VALIDATION ENGINE                             │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐    │   │
│  │  │ Syntax  │ │Semantic │ │  Logic  │ │  Arch.  │ │    Style    │    │   │
│  │  │ Layer   │ │  Layer  │ │  Layer  │ │  Layer  │ │    Layer    │    │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────────┘    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        PARSER ABSTRACTION                            │   │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │   │
│  │  │Tree-   │ │ Clang  │ │  Syn   │ │ Python │ │ Custom │ │  JSON  │  │   │
│  │  │Sitter  │ │  Lib   │ │ (Rust) │ │  AST   │ │ Parsers│ │ /YAML  │  │   │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        LANGUAGE ADAPTERS                             │   │
│  │  C++ | Rust | Python | JavaScript | TypeScript | Go | Java | Lex    │   │
│  │  SQL | JSON | YAML | TOML | Markdown | Custom DSLs                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        INTERFACE LAYER                               │   │
│  │  CLI | HTTP API | LSP | MCP Server | FFI (C/C++/Rust/Python)        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        CERTIFICATION ENGINE                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│  │  │  Signature  │  │  Receipt    │  │  Audit Log  │                  │   │
│  │  │  Generator  │  │  Generator  │  │  Storage    │                  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AI AGENTS                                      │
│  Claude | GPT-4/5 | Cursor | Copilot | Factory Droid | Custom Agents       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Prompt Analyzer

**Purpose:** Understand what the user wants before code is generated.

**Capabilities:**
- **Intent Classification** — CREATE / MODIFY / FIX / REFACTOR / EXPLAIN / DELETE
- **Scope Extraction** — Single file? Module? Full project?
- **Domain Mapping** — Gameplay / UI / Performance / Security / Data / Infrastructure
- **Ambiguity Detection** — Vague requests trigger clarification questions
- **Context Binding** — Link request to relevant project files/patterns

**Output:** Structured prompt context that guides code generation.

See: [SYNWARD_INTELLIGENCE.md](./SYNWARD_INTELLIGENCE.md) (consolidato)

---

### 2. Validation Engine

**Purpose:** Verify code against multiple quality dimensions.

**Five Validation Layers:**

| Layer | Checks | Example |
|-------|--------|---------|
| **Syntax** | Parse-ability, valid tokens | `int x = ;` is invalid |
| **Semantic** | Type safety, references | `undefined_var` used |
| **Logic** | Domain rules, constraints | `health > 0`, `damage <= max` |
| **Architecture** | Patterns, dependencies | Circular imports |
| **Style** | Naming, formatting | snake_case vs camelCase |

**Extensibility:** Each layer supports custom rules via contracts.

See: [SYNWARD_ARCHITECTURE.md](./SYNWARD_ARCHITECTURE.md)

---

### 3. Contract Engine

**Purpose:** Define and enforce formal code requirements.

**Contract Definition Language (CDL):**

```yaml
# contracts/cpp/memory.contracts.yaml
domain: cpp
category: memory-safety

contracts:
  - id: CPP-MEM-001
    name: no-raw-pointers-owning
    description: "Owning raw pointers are forbidden"
    severity: error
    pattern: "Type* var = new Type"
    suggestion: "Use std::unique_ptr<Type> or std::shared_ptr<Type>"
    ai_hint: "Smart pointers automatically manage memory and prevent leaks"
    
  - id: CPP-MEM-002
    name: no-malloc-free
    description: "Use RAII, not manual memory management"
    severity: warning
    pattern: "malloc|free|realloc"
    suggestion: "Use std::vector, std::string, or smart pointers"
```

**Features:**
- Declarative rule definitions
- Severity levels (error, warning, info, hint)
- Auto-fix suggestions
- AI-friendly hints for context

See: [SYNWARD_CONTRACTS.md](./SYNWARD_CONTRACTS.md)

---

### 4. Iteration Loop

**Purpose:** Automatically guide AI to correct code.

```
┌─────────────────────────────────────────────────────────────┐
│                    ITERATION LOOP                           │
│                                                             │
│  ┌─────────┐      ┌─────────┐      ┌─────────────────┐     │
│  │  AI     │ ───▶ │ Synward  │ ───▶ │     PASS        │     │
│  │ Proposes│      │ Validates│     │  Certificate    │     │
│  └─────────┘      └────┬────┘      └─────────────────┘     │
│                        │                                    │
│                        │ FAIL                               │
│                        ▼                                    │
│                   ┌─────────────┐                           │
│                   │   Error     │                           │
│                   │   Report    │                           │
│                   │ + Suggest   │                           │
│                   │ + Example   │                           │
│                   └──────┬──────┘                           │
│                          │                                  │
│                          ▼                                  │
│                   ┌─────────────┐                           │
│                   │  Increment  │                           │
│                   │  Counter    │                           │
│                   └──────┬──────┘                           │
│                          │                                  │
│              ┌───────────┴───────────┐                      │
│              │                       │                      │
│              ▼                       ▼                      │
│      counter < MAX            counter >= MAX               │
│              │                       │                      │
│              ▼                       ▼                      │
│     ┌─────────────────┐    ┌─────────────────┐             │
│     │   AI Retries    │    │ Human Escalation│             │
│     │ with context    │    │ + Full Summary  │             │
│     └─────────────────┘    └─────────────────┘             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Configuration:**
- `MAX_ITERATIONS` — Default: 3 (configurable)
- `ESCALATION_THRESHOLD` — When to involve human
- `LEARNING_MODE` — Store successful corrections for future

---

### 5. Certification Engine

**Purpose:** Provide cryptographic proof that code was validated.

**Synward Certificate:**

```json
{
  "certificate_id": "SYNWARD-2026-03-08-ABC123",
  "version": "1.0",
  "timestamp": "2026-03-08T23:30:00Z",
  "hash": {
    "algorithm": "SHA-256",
    "value": "e3b0c44298fc1c149afbf4c8996fb924..."
  },
  "validation": {
    "layers": ["syntax", "semantic", "logic", "architecture", "style"],
    "contracts_applied": ["CPP-MEM-001", "CPP-SEC-003", "LEX-VAL-002"],
    "all_passed": true
  },
  "agent": {
    "type": "claude-3-opus",
    "session_id": "session-xyz"
  },
  "signature": {
    "algorithm": "Ed25519",
    "value": "304402200a3b4c..."
  }
}
```

**Use Cases:**
- **CI/CD Integration** — Only certified code can merge
- **Audit Trail** — Prove what was validated, when, by whom
- **Commercial Trust** — "Certified by Synward" as quality mark

See: [SYNWARD_CERTIFICATION.md](./SYNWARD_CERTIFICATION.md)

---

### 6. Pattern Library

**Purpose:** Learn from existing code to enforce project consistency.

**Features:**
- Extracts patterns from codebase (naming, structure, idioms)
- Builds project-specific vocabulary
- Flags deviations from established patterns
- Auto-updates as codebase evolves

**Example:**

```
Project Pattern: entities always use "faction: Player | Enemy"
AI proposes: "team: GoodGuys"
Synward: "Inconsistent with pattern. Use 'faction' not 'team'"
```

---

## Language Adapters

Currently supporting **23 public languages** + Prism (private):

| Category | Languages |
|----------|-----------|
| **System** | Rust, C, C++, CUDA, Go, Java |
| **Scripting** | Python, JavaScript, TypeScript, Lua, Bash |
| **DSL** | Lex, GLSL, SQL, GraphQL, CSS, HTML, Markdown |
| **Config** | JSON, YAML, TOML, CMake, Notebook (.ipynb) |
| **Private** | Prism (David only, not in public releases) |

All languages use tree-sitter parsers except Lex (custom parser).

---

## Why Rust?

Synward is implemented in **Rust** for commercial and technical reasons:

| Factor | Rust Advantage |
|--------|----------------|
| **Marketing** | "Built in Rust" = trust marker for enterprises |
| **Memory Safety** | No buffer overflows, use-after-free, data races |
| **Performance** | Zero-cost abstractions, comparable to C++ |
| **Ecosystem** | Excellent parser libraries (syn, nom, tree-sitter bindings) |
| **Talent Pool** | Growing Rust developer community |
| **Modern Tooling** | Cargo, clippy, rustfmt built-in |

---

## Interfaces

### CLI

```bash
# Validate a file
synward validate src/main.cpp --contracts ./contracts/

# Validate with auto-fix hints
synward validate src/main.cpp --fix-hints

# Generate certificate
synward certify src/main.cpp --output cert.toml

# Analyze a prompt
synward analyze-prompt "Add a patrol enemy" --context ./project/
```

### HTTP API

```
POST /api/v1/validate
Content-Type: application/json

{
  "language": "cpp",
  "source": "...",
  "contracts": ["CPP-MEM-001", "CPP-SEC-003"]
}

Response:
{
  "status": "fail",
  "violations": [...],
  "certificate": null
}
```

### MCP Server

For AI agents using Model Context Protocol:

```json
{
  "tool": "synward_validate",
  "params": {
    "source": "...",
    "language": "rust"
  }
}
```

---

## Integration with AI Agents

### The Synward Flow

```
1. USER → Prompt
2. Synward analyzes prompt → Structured context
3. Synward sends context to AI Agent
4. AI Agent generates code with full understanding
5. Synward validates code
6. ┌─ PASS → Certificate issued → Code accepted
   └─ FAIL → Structured feedback → AI retries
7. Repeat until pass or human escalation
```

### Agent SDK

```python
from synward import SynwardClient

synward = SynwardClient(api_key="...")

# Analyze prompt
context = synward.analyze_prompt(
    "Add an enemy that patrols the dungeon",
    project_context="./mygame/"
)

# Generate code with context
code = ai_agent.generate(context)

# Validate and iterate
result = synward.validate(code, language="cpp")
while not result.passed and result.iterations < 3:
    code = ai_agent.fix(code, result.violations)
    result = synward.validate(code)

if result.passed:
    cert = synward.certify(code)
    print(f"Certified: {cert.id}")
else:
    escalate_to_human(result)
```

---

## File Structure

```
Synward/
├── docs/
│   ├── SYNWARD_MASTER_DESIGN.md      # This file
│   ├── SYNWARD_ARCHITECTURE.md       # Technical architecture
│   ├── SYNWARD_CONTRACTS.md          # Contract system
│   ├── SYNWARD_INTELLIGENCE.md       # AI Intelligence (consolidato)
│   ├── SYNWARD_CERTIFICATION.md      # Certification system
│   ├── SYNWARD_RUST_IMPLEMENTATION.md# Rust-specific details
│   ├── MEMORY_DRIVEN_CORE.md        # Memory-Driven Core architecture
│
├── src/                             # Core engine (Rust)
│   ├── core/
│   │   ├── mod.rs
│   │   ├── orchestrator.rs
│   │   ├── session.rs
│   │   └── pipeline.rs
│   ├── validation/
│   │   ├── mod.rs
│   │   ├── layers.rs
│   │   ├── syntax.rs
│   │   ├── semantic.rs
│   │   ├── logic.rs
│   │   ├── architecture.rs
│   │   └── style.rs
│   ├── contracts/
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   ├── evaluator.rs
│   │   └── registry.rs
│   ├── certification/
│   │   ├── mod.rs
│   │   ├── certificate.rs
│   │   ├── signer.rs
│   │   └── audit.rs
│   └── cli/
│       ├── mod.rs
│       └── main.rs
│
├── adapters/                        # Language adapters
│   ├── rust/
│   ├── cpp/
│   └── lex/
│
├── interfaces/                      # External interfaces
│   ├── http/
│   ├── lsp/
│   └── mcp/
│
├── contracts/                       # Default contracts
│   ├── rust/
│   ├── cpp/
│   └── lex/
│
├── sdk/                             # Agent SDKs
│   ├── python/
│   └── typescript/
│
├── Cargo.toml
└── Cargo.lock
```

---

## Roadmap

### Phase 1 — Foundation (v0.1)
- [ ] Core orchestration engine
- [ ] C++ adapter with tree-sitter
- [ ] Contract definition language
- [ ] Basic CLI

### Phase 2 — Validation (v0.2)
- [ ] Rust adapter
- [ ] Lex adapter (integrate from Aegis)
- [ ] All 5 validation layers
- [ ] HTTP API

### Phase 3 — Certification (v0.3)
- [ ] Certificate generation
- [ ] Ed25519 signing
- [ ] Audit log storage

### Phase 4 — Prompt Analysis (v0.4) [OPZIONALE]
- [ ] Intent classifier (richiede LLM opzionale)
- [ ] Ambiguity detector
- [ ] Context binding

> **Nota**: Il Prompt Analysis richiede LLM esterno. Il core autonomo funziona senza.

### Phase 5 — Iteration (v0.5)
- [ ] Automated retry loop
- [ ] Human escalation
- [ ] Learning from corrections

### Phase 6 — Integration (v1.0)
- [ ] MCP server
- [ ] Python SDK
- [ ] Full agent integration
- [ ] Documentation complete

### Phase 16 — Memory-Driven Core (v0.2)
- [x] Architecture document (MEMORY_DRIVEN_CORE.md)
- [ ] LearnedConfig implementation
- [ ] Dynamic layer configuration
- [ ] Memory-driven validation flow
- [ ] CLI memory commands

> Full roadmap: [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)

---

## Relation to Other Projects

| Project | Relationship |
|---------|--------------|
| **Aegis Validation** | Predecessor. Synward is the universal evolution. Lex-specific logic lives in Synward's `lex` adapter. |
| **Aegis (Security)** | Synward certificates integrate with Aegis security framework |
| **Lex Compiler** | Synward validates `.lex` files via the Lex adapter |
| **Prism Engine** | Synward validates engine C++ code |
| **Archivista** | Synward provides context for Archivista (original Synward purpose preserved) |

---

## Conclusion

Synward transforms AI-generated code from "probably works" to "proven correct."

By combining:
- Deep prompt understanding
- Rigorous multi-layer validation
- Automated iteration
- Cryptographic certification

...we create a system where **trust is earned through verification, not assumed.**

The market can finally trust AI with code — because Synward guarantees it.
