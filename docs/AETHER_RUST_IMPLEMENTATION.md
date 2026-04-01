# Aether — Rust Implementation

**Version:** 0.2.0
**Last Updated:** 2026-03-16
**Related:** [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md), [AETHER_ARCHITECTURE.md](./AETHER_ARCHITECTURE.md)

---

## Why Rust?

Aether is implemented in **Rust** for commercial and technical reasons:

### Commercial Advantages

| Factor | Rust Advantage |
|--------|----------------|
| **Trust** | "Built in Rust" signals reliability and security |
| **Memory Safety** | No buffer overflows, use-after-free, data races by design |
| **Modern** | Appeals to forward-thinking teams |
| **Growing Adoption** | Increasing enterprise adoption (AWS, Google, Microsoft, Cloudflare) |
| **No GC** | Predictable performance, suitable for real-time systems |

### Technical Advantages

| Factor | Rust Advantage |
|--------|----------------|
| **Performance** | Zero-cost abstractions, comparable to C/C++ |
| **Parser Ecosystem** | Excellent libraries: syn, nom, tree-sitter bindings |
| **Type System** | Strong guarantees, catches errors at compile time |
| **Error Handling** | Result<T, E> pattern, explicit error propagation |
| **Concurrency** | Fearless concurrency with ownership model |
| **Tooling** | Cargo, clippy, rustfmt, all built-in |

### Why Not Prism?

Prism remains an internal tool for:
- **Aegis** — Security system where "unknown language = inviolable" is a feature
- **Internal tools** — Parser, codegen, stdlib development
- **Future open source** — When mature, Prism may be released

For Aether (commercial product), using an unknown language would be a liability:
- Customers would ask: "Who maintains this language?"
- Security teams would flag it as "unknown risk"
- No ecosystem, no hiring pool, no community support

---

## Project Structure

```
aether/
├── Cargo.toml                  # Workspace configuration
├── Cargo.lock
├── README.md
│
├── crates/
│   ├── aether-core/           # Core types and orchestrator
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs
│   │       ├── session.rs
│   │       └── pipeline.rs
│   │
│   ├── aether-parsers/        # Parser implementations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs      # Parser trait
│   │       ├── registry.rs    # ParserRegistry
│   │       ├── rust.rs        # Rust parser (syn + tree-sitter)
│   │       ├── cpp.rs         # C++ parser (tree-sitter)
│   │       └── lex.rs         # Lex parser (custom)
│   │
│   ├── aether-validation/     # Validation layers
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pipeline.rs
│   │       ├── layer.rs       # ValidationLayer trait
│   │       └── layers/
│   │           ├── mod.rs
│   │           ├── syntax.rs
│   │           ├── semantic.rs
│   │           ├── logic.rs
│   │           ├── architecture.rs
│   │           └── style.rs
│   │
│   ├── aether-contracts/       # Contract engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── contract.rs     # Contract trait
│   │       ├── registry.rs     # ContractRegistry
│   │       ├── loader.rs       # YAML loader
│   │       └── evaluator.rs    # Rule evaluator
│   │
│   ├── aether-certification/   # Certificate generation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── certificate.rs  # Certificate struct
│   │       ├── signer.rs       # Ed25519 signing
│   │       ├── audit.rs        # Audit logging
│   │       └── storage.rs     # Certificate storage
│   │
│   └── aether-cli/             # CLI interface
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands/
│               ├── mod.rs
│               ├── validate.rs
│               ├── certify.rs
│               └── analyze.rs
│
├── contracts/                  # Default contracts
│   ├── rust/
│   │   ├── memory-safety.yaml
│   │   ├── error-handling.yaml
│   │   └── performance.yaml
│   ├── cpp/
│   └── lex/
│
├── interfaces/                 # External interfaces
│   ├── http/                  # HTTP API (axum)
│   ├── lsp/                   # LSP server (tower-lsp)
│   └── mcp/                   # MCP server
│
├── sdk/                        # Agent SDKs
│   ├── python/                # Python SDK (pyo3)
│   └── typescript/            # TypeScript SDK
│
├── tests/                      # Integration tests
│   ├── integration/
│   └── e2e/
│
└── benches/                    # Benchmarks
    └── validation.rs
```

---

## Dependencies

### Core Dependencies

```toml
# Cargo.toml (workspace)
[workspace]
members = [
    "crates/aether-core",
    "crates/aether-parsers",
    "crates/aether-validation",
    "crates/aether-contracts",
    "crates/aether-certification",
    "crates/aether-cli",
]

[workspace.dependencies]
# Parser
syn = { version = "2.0", features = ["full", "parsing", "visit"] }
tree-sitter = "0.24"
tree-sitter-rust = "0.21"
tree-sitter-cpp = "0.22"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
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

# HTTP (for interfaces)
axum = "0.7"
tower-lsp = "0.20"
```

### Crate-Specific Dependencies

```toml
# crates/aether-parsers/Cargo.toml
[dependencies]
syn = { workspace = true }
tree-sitter = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-cpp = { workspace = true }
thiserror = { workspace = true }

# crates/aether-certification/Cargo.toml
[dependencies]
ed25519-dalek = { workspace = true }
rand = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

# crates/aether-cli/Cargo.toml
[dependencies]
aether-core = { path = "../aether-core" }
aether-parsers = { path = "../aether-parsers" }
aether-validation = { path = "../aether-validation" }
aether-contracts = { path = "../aether-contracts" }
aether-certification = { path = "../aether-certification" }
clap = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
```

---

## Key Types

### Orchestrator

```rust
// crates/aether-core/src/orchestrator.rs
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Orchestrator {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    pipeline: Pipeline,
    config: Config,
}

impl Orchestrator {
    pub fn new(config: Config) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pipeline: Pipeline::new(&config),
            config,
        }
    }

    pub async fn validate(&self, request: ValidationRequest) -> ValidationResult {
        let session = self.create_session(&request).await;
        self.pipeline.execute(&session, request.source).await
    }

    pub async fn certify(&self, request: CertificationRequest) -> CertificationResult {
        let validation = self.validate(request.validation).await;
        if !validation.passed {
            return CertificationResult::Failed(validation);
        }
        self.certifier.sign(validation).await
    }
}
```

### Parser Trait

```rust
// crates/aether-parsers/src/parser.rs
use async_trait::async_trait;

#[async_trait]
pub trait Parser: Send + Sync {
    async fn parse(&self, source: &str) -> Result<AST, ParseError>;
    fn language(&self) -> &str;
    fn extensions(&self) -> &[&str];
}

pub struct AST {
    pub root: ASTNode,
    pub tokens: Vec<Token>,
    pub errors: Vec<ParseError>,
}
```

### Validation Layer

```rust
// crates/aether-validation/src/layer.rs
use async_trait::async_trait;

#[async_trait]
pub trait ValidationLayer: Send + Sync {
    fn name(&self) -> &str;
    async fn validate(&self, ast: &AST, ctx: &ValidationContext) -> LayerResult;
    fn can_continue(&self, result: &LayerResult) -> bool { true }
}

#[derive(Debug)]
pub struct LayerResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub infos: Vec<Info>,
}
```

### Contract

```rust
// crates/aether-contracts/src/contract.rs
use async_trait::async_trait;

#[async_trait]
pub trait Contract: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn domain(&self) -> &str;
    fn severity(&self) -> Severity;
    async fn evaluate(&self, ast: &AST, ctx: &ValidationContext) -> Vec<Violation>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}
```

### Certificate

```rust
// crates/aether-certification/src/certificate.rs
use ed25519_dalek::{Signature, Signer, VerifyingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub file_hash: String,
    pub validation_result: ValidationResult,
    pub timestamp: u64,
    pub agent: AgentInfo,
    pub signature: String,
}

impl Certificate {
    pub fn sign(&self, keypair: &Keypair) -> Result<String, Error> {
        let message = self.canonical_form();
        let signature = keypair.sign(message.as_bytes());
        Ok(base64::encode(signature.to_bytes()))
    }

    pub fn verify(&self, public_key: &VerifyingKey) -> Result<bool, Error> {
        let message = self.canonical_form();
        let signature = Signature::from_bytes(&base64::decode(&self.signature)?)?;
        Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
    }
}
```

---

## Rust-Specific Contracts

### Memory Safety Contracts

| ID | Name | Severity | Description |
|----|------|----------|-------------|
| RUST001 | no-unwrap-without-context | warning | unwrap() without context message |
| RUST002 | prefer-result-for-errors | error | Use Result instead of panic |
| RUST003 | no-clone-unnecessarily | warning | Unnecessary clone() |
| RUST004 | use-unwrap-or-default | info | Prefer unwrap_or_default() |
| RUST005 | prefer-borrow | info | Prefer &T over T when possible |

### Error Handling Contracts

| ID | Name | Severity | Description |
|----|------|----------|-------------|
| RUST006 | no-expect-without-context | warning | expect() without context |
| RUST007 | prefer-anyhow | info | Use anyhow for error propagation |
| RUST008 | no-panic-in-lib | error | panic!() in library code |

### Performance Contracts

| ID | Name | Severity | Description |
|----|------|----------|-------------|
| RUST009 | avoid-allocations-in-loops | warning | Allocation inside loop |
| RUST010 | use-iter-for-collection | info | Prefer .iter() over indexing |
| RUST011 | prefer-slice-patterns | info | Use slice patterns for matching |

### Idiomatic Rust Contracts

| ID | Name | Severity | Description |
|----|------|----------|-------------|
| RUST012 | prefer-newtype | warning | Use newtype pattern for domain types |
| RUST013 | use-derive-where-possible | info | Prefer derive macros |
| RUST014 | avoid-deep-nesting | info | Refactor deeply nested code |

---

## Build & Test

### Build

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build -p aether-core --release

# Build with all features
cargo build --all-features
```

### Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test -p aether-validation

# Run tests with coverage
cargo tarpaulin --out Html
```

### Benchmark

```bash
# Run benchmarks
cargo bench

# Compare with baseline
cargo bench -- baseline.json
```

### Lint

```bash
# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt --check
```

---

## CI/CD

### GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check

  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo bench --no-run

  release:
    needs: [test]
    if: startsWith(github.ref, 'refs/tags/')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/aether
```

---

## Performance Targets

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Parse 1000-line Rust file | < 30ms | syn + tree-sitter-rust |
| Full validation (5 layers) | < 100ms | Parallel layer execution |
| Contract evaluation (100 contracts) | < 20ms | Indexed lookup |
| Certificate generation | < 10ms | ed25519-dalek |
| Total validation request | < 200ms | End-to-end |

---

## Security Considerations

### Memory Safety

Rust guarantees memory safety at compile time:
- No buffer overflows
- No use-after-free
- No data races (in safe Rust)
- No null pointer dereferences

### Supply Chain

- All dependencies are from crates.io with verified publishers
- `cargo audit` runs in CI to check for vulnerabilities
- Minimal dependency tree to reduce attack surface

### Certificate Security

- Ed25519 signatures for certificates
- Private keys never leave the signing machine
- Certificates include file hash for tamper detection

---

## Future Considerations

### WASM Target

Aether can compile to WASM for:
- Browser-based validation
- CloudFlare Workers deployment
- Deno/Node.js integration

### Plugin System

Rust's trait system enables a plugin architecture:
- Custom validation layers
- Custom contracts
- Custom output formats

### Language Adapters

Priority order for language support:
1. **Rust** — Primary target
2. **C++** — Game development
3. **Lex** — Imperium integration
4. **Python** — AI/ML code
5. **TypeScript/JavaScript** — Web development

---

## Summary

Aether is implemented in Rust because:

1. **Commercial** — "Built in Rust" is a trust marker
2. **Technical** — Memory safety, performance, modern tooling
3. **Ecosystem** — Excellent parser libraries
4. **Future-proof** — Growing adoption, active development

Prism remains internal for Aegis and internal tools where "unknown language = security by obscurity" is a feature.
