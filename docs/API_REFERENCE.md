# Aether — API Reference

**Versione:** 0.1.0  
**Aggiornato:** 2026-03-11

---

## Indice

1. [Core API](#core-api)
2. [SDK API](#sdk-api)
3. [Validation API](#validation-api)
4. [Certification API](#certification-api)
5. [Parsers API](#parsers-api)
6. [RAG API](#rag-api)
7. [Learner API](#learner-api)
8. [MCP API](#mcp-api)
9. [Python Bindings](#python-bindings)

---

## Core API

### `aether_core`

Orchestrator e session management.

#### `Orchestrator`

```rust
use aether_core::{Orchestrator, Config, Result};

// Crea orchestrator con configurazione
let config = Config::from_file("aether.yaml")?;
let orchestrator = Orchestrator::new(config);

// Esegui validazione
let session = orchestrator.create_session();
let result = session.validate("src/main.rs").await?;

// Ottieni risultati
println!("Passed: {}", result.passed);
for violation in result.violations {
    println!("  - {}: {}", violation.id, violation.message);
}
```

#### `Session`

```rust
use aether_core::{Session, SessionId};

// Crea sessione
let session = orchestrator.create_session();
let session_id = session.id(); // SessionId

// Valida file
let result = session.validate("src/main.rs").await?;

// Valida con opzioni
let result = session.validate_with_options(
    "src/main.rs",
    ValidationOptions {
        contracts: vec!["memory-safety"],
        severity: Severity::Warning,
    }
).await?;

// Chiudi sessione
session.close();
```

#### `Config`

```rust
use aether_core::Config;

// Da file
let config = Config::from_file("aether.yaml")?;

// Da environment
let config = Config::from_env()?;

// Default
let config = Config::default();

// Accesso valori
println!("Language: {}", config.language);
println!("Contracts: {:?}", config.contracts);
```

---

## SDK API

### `aether_sdk`

Client library per integrazione.

#### `AetherClient`

```rust
use aether_sdk::{AetherClient, SdkResult};

// Crea client
let client = AetherClient::new("http://localhost:3000")?;

// Oppure con configurazione
let client = AetherClient::builder()
    .server("http://localhost:3000")
    .timeout(Duration::from_secs(30))
    .api_key("my-api-key")
    .build()?;

// Valida file
let result = client.validate_file("src/main.rs").await?;

// Valida codice
let result = client.validate(ValidateRequest {
    language: "rust".to_string(),
    code: "fn main() {}".to_string(),
    contracts: vec!["memory-safety".to_string()],
}).await?;

// Certifica
let cert = client.certify_file("src/main.rs").await?;

// Analizza AST
let analysis = client.analyze_file("src/main.rs").await?;
```

#### `ValidationOptions`

```rust
use aether_sdk::ValidationOptions;

let options = ValidationOptions {
    contracts: vec!["memory-safety".to_string()],
    severity_filter: Some(Severity::Warning),
    format: OutputFormat::Json,
    output_file: Some("results.json".to_string()),
};
```

#### `ValidationResult`

```rust
use aether_sdk::ValidationResult;

pub struct ValidationResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub warnings: Vec<Violation>,
    pub infos: Vec<Violation>,
    pub stats: ValidationStats,
}

pub struct Violation {
    pub id: String,           // es. "RUST001"
    pub name: String,         // es. "no-unsafe"
    pub severity: Severity,   // Error, Warning, Info
    pub message: String,
    pub location: Location,
    pub suggestion: Option<String>,
}

pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}
```

---

## Validation API

### `aether_validation`

Pipeline di validazione multi-layer.

#### `ValidationPipeline`

```rust
use aether_validation::{
    ValidationPipeline,
    SyntaxLayer, SemanticLayer, LogicLayer,
    SecurityLayer, PrivateLayer, StyleLayer,
};

// Crea pipeline
let mut pipeline = ValidationPipeline::new();

// Aggiungi layer
pipeline.add_layer(SyntaxLayer::new());
pipeline.add_layer(SemanticLayer::new());
pipeline.add_layer(LogicLayer::new());
pipeline.add_layer(SecurityLayer::new());
pipeline.add_layer(PrivateLayer::new());
pipeline.add_layer(StyleLayer::new());

// Valida
let result = pipeline.validate(&ast, &source).await?;

// Risultati per layer
for (layer_name, layer_result) in result.layers {
    println!("{}: {} violations", layer_name, layer_result.violations.len());
}
```

#### `ValidationLayer` Trait

```rust
use aether_validation::{ValidationLayer, LayerResult, ValidationContext};

pub trait ValidationLayer: Send + Sync {
    /// Nome del layer
    fn name(&self) -> &str;
    
    /// Valida AST
    fn validate(&self, context: &mut ValidationContext) -> LayerResult;
    
    /// Priorità (lower = higher priority)
    fn priority(&self) -> u8 {
        100
    }
}
```

#### Layer Built-in

| Layer | Descrizione | Regole |
|-------|-------------|--------|
| `SyntaxLayer` | Errori di sintassi | Parentesi non chiuse, stringhe non terminate |
| `SemanticLayer` | Errori semantici | Variabili non usate, codice irraggiungibile |
| `LogicLayer` | Anti-pattern logici | `panic!`, `.unwrap()`, funzioni lunghe |
| `SecurityLayer` | Vulnerabilità | Secret hardcoded, SQL injection, path traversal |
| `PrivateLayer` | Regole private | `print!` in produzione, limiti lunghezza |
| `StyleLayer` | Stile codice | Naming conventions, formattazione |

#### `Violation`

```rust
use aether_validation::{Violation, Severity};

pub struct Violation {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub description: String,
    pub location: Location,
    pub suggestion: Option<String>,
    pub pattern: Option<String>,
}

pub enum Severity {
    Error,    // Blocca certificazione
    Warning,  // Avviso
    Info,     // Informazione
}
```

---

## Certification API

### `aether_certification`

Firma e verifica certificati.

#### `Keypair`

```rust
use aether_certification::Keypair;

// Genera nuovo keypair
let keypair = Keypair::generate();

// Salva
keypair.save("keypair.json")?;

// Carica
let keypair = Keypair::load("keypair.json")?;

// Accedi alle chiavi
let public_key = keypair.public();
let secret_key = keypair.secret(); // KEEP SECRET!
```

#### `CertificateSigner`

```rust
use aether_certification::{CertificateSigner, Certificate};

// Crea signer
let signer = CertificateSigner::new(keypair);

// Crea certificato
let cert = signer.sign(
    file_hash,           // SHA-256 del file
    validation_result,   // Risultato validazione
    agent_info,          // Info sull'agente
)?;

// Verifica firma
let verifier = CertificateVerifier::new(public_key);
let valid = verifier.verify(&cert)?;
```

#### `Certificate`

```rust
use aether_certification::Certificate;

pub struct Certificate {
    pub version: u32,
    pub algorithm: String,        // "Ed25519"
    pub file_hash: String,        // SHA-256
    pub validation_result: ValidationResult,
    pub agent_info: AgentInfo,
    pub certified_at: DateTime<Utc>,
    pub signature: Vec<u8>,
}

pub struct AgentInfo {
    pub name: String,
    pub version: String,
    pub model: Option<String>,
}
```

#### `CertificateStore`

```rust
use aether_certification::CertificateStore;

// Crea store
let store = CertificateStore::new(".aether/certs")?;

// Salva certificato
store.save(&certificate)?;

// Carica certificato
let cert = store.load("cert_id")?;

// Lista certificati
let certs = store.list()?;

// Verifica catena
let chain = CertificateChain::new(vec![cert]);
let verification = chain.verify(&trust_anchor)?;
```

---

## Parsers API

### `aether_parsers`

Parser multi-linguaggio.

#### `Parser` Trait

```rust
use aether_parsers::{Parser, ParseResult, AST};

#[async_trait]
pub trait Parser: Send + Sync {
    /// Nome del linguaggio
    fn language(&self) -> &str;
    
    /// Estensioni file supportate
    fn extensions(&self) -> &[&str];
    
    /// Verifica se può parsare il file
    async fn can_parse(&self, source: &str) -> bool;
    
    /// Parsa il codice sorgente
    async fn parse(&self, source: &str) -> ParseResult<AST>;
}
```

#### `ParserRegistry`

```rust
use aether_parsers::ParserRegistry;

// Crea registry con tutti i parser
let registry = ParserRegistry::with_defaults();

// Registra parser personalizzato
let mut registry = ParserRegistry::new();
registry.register(RustParser::new());
registry.register(PythonParser::new());

// Ottieni parser per linguaggio
let parser = registry.get("rust").ok_or("Parser not found")?;

// Ottieni parser per file
let parser = registry.get_for_file("src/main.rs")?;

// Linguaggi supportati
let languages = registry.languages(); // ["rust", "python", ...]
```

#### `AST`

```rust
use aether_parsers::{AST, ASTNode, NodeKind};

pub struct AST {
    pub root: ASTNode,
    pub source: String,
    pub language: String,
    pub stats: ASTStats,
}

pub struct ASTNode {
    pub kind: NodeKind,
    pub value: String,
    pub span: Span,
    pub children: Vec<ASTNode>,
}

pub enum NodeKind {
    // Generici
    Module, Function, Struct, Enum, Trait,
    Impl, TypeAlias, Constant, Static,
    
    // Rust specific
    Use, Mod, Macro, Attribute,
    
    // Lex specific
    Resource, Era, LexStructure, Unit, Technology,
    Event, Choice, Property, Condition,
    
    // ... altri
}
```

#### Parsers Implementati

| Parser | Linguaggio | Estensioni |
|--------|------------|------------|
| `RustParser` | Rust | `.rs` |
| `PythonParser` | Python | `.py` |
| `JavaScriptParser` | JavaScript | `.js`, `.mjs` |
| `TypeScriptParser` | TypeScript | `.ts`, `.tsx` |
| `CppParser` | C++ | `.cpp`, `.h`, `.hpp` |
| `GoParser` | Go | `.go` |
| `JavaParser` | Java | `.java` |
| `LuaParser` | Lua | `.lua` |
| `LexParser` | Lex | `.lex` |

---

## RAG API

### `aether_rag`

Semantic search e knowledge retrieval.

#### `RagEngine`

```rust
use aether_rag::{RagEngine, SemanticSearch, KeywordIndex};

// Crea engine
let engine = RagEngine::new("models/bge-small-en")?;

// Indicizza documenti
engine.index("docs/architecture.md").await?;
engine.index("docs/contracts/").await?;

// Cerca
let results = engine.search("memory safety rules", 10).await?;

// Risultati
for result in results {
    println!("{}: {} (score: {})", 
        result.source, result.text, result.score);
}
```

#### `SemanticSearch`

```rust
use aether_rag::SemanticSearch;

let search = SemanticSearch::new("models/bge-small-en")?;

// Embed query
let embedding = search.embed("validation contracts").await?;

// Cerca similarità
let results = search.search_similar(&embedding, &index, 10)?;
```

#### `KeywordIndex`

```rust
use aether_rag::KeywordIndex;

let mut index = KeywordIndex::new();

// Indicizza
index.add("doc1", "memory safety rules for Rust");
index.add("doc2", "error handling best practices");

// Cerca (BM25)
let results = index.search("memory safety", 10);
```

---

## Learner API

### `aether_learner`

User profiling e learning.

#### `UserProfile`

```rust
use aether_learner::UserProfile;

// Crea profilo
let profile = UserProfile::new("user-123");

// Registra azione
profile.record_action(Action {
    kind: ActionKind::Validation,
    target: "src/main.rs".to_string(),
    result: ActionResult::Success,
    timestamp: Utc::now(),
});

// Statistiche
let stats = profile.stats();
println!("Validations: {}", stats.validations);
println!("Success rate: {}%", stats.success_rate * 100.0);

// Salva/Carica
profile.save("profiles/user-123.json")?;
let profile = UserProfile::load("profiles/user-123.json")?;
```

#### `StatsTracker`

```rust
use aether_learner::StatsTracker;

let tracker = StatsTracker::new();

// Registra
tracker.record("validation", true);
tracker.record("validation", false);
tracker.record("certification", true);

// Query
let stats = tracker.get_stats("validation");
println!("Success: {}/{}", stats.success, stats.total);
```

#### `MemoryStore`

```rust
use aether_learner::MemoryStore;

let store = MemoryStore::new(".aether/memory")?;

// Memorizza
store.set("user_preference", "strict_validation")?;

// Recupera
let pref = store.get("user_preference")?;

// Query semantica
let results = store.query_similar("error handling", 5)?;
```

---

## MCP API

### `aether_mcp`

Model Context Protocol server.

#### Tools

```json
{
  "tools": [
    {
      "name": "aether_validate",
      "description": "Validate source code",
      "input_schema": {
        "type": "object",
        "properties": {
          "language": { "type": "string" },
          "code": { "type": "string" },
          "contracts": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["language", "code"]
      }
    },
    {
      "name": "aether_certify",
      "description": "Certify validated code",
      "input_schema": {
        "type": "object",
        "properties": {
          "language": { "type": "string" },
          "code": { "type": "string" },
          "agent_info": { "type": "object" }
        },
        "required": ["language", "code"]
      }
    },
    {
      "name": "aether_analyze",
      "description": "Analyze AST structure",
      "input_schema": {
        "type": "object",
        "properties": {
          "language": { "type": "string" },
          "code": { "type": "string" }
        },
        "required": ["language", "code"]
      }
    }
  ]
}
```

#### Uso da Claude

```
User: Validate this Rust code:
```rust
fn main() {
    let x = unsafe { *ptr };
}
```

Claude: [usa aether_validate]

Result: 1 violation found:
- RUST002: Unsafe block detected
  Severity: ERROR
  Suggestion: Avoid unsafe blocks when possible
```

---

## Python Bindings

### `aether` (Python)

```python
from aether import Client, ValidationResult

# Crea client
client = Client()

# Valida file
result: ValidationResult = client.validate_file("src/main.rs")

# Valida codice
result = client.validate(
    language="rust",
    code="fn main() {}",
    contracts=["memory-safety"]
)

# Risultati
print(f"Passed: {result.passed}")
print(f"Violations: {len(result.violations)}")

for v in result.violations:
    print(f"  {v.id}: {v.message} at {v.location.file}:{v.location.line}")
    if v.suggestion:
        print(f"    Suggestion: {v.suggestion}")

# Certifica
cert = client.certify_file("src/main.rs")
print(f"Certificate ID: {cert.id}")
print(f"Signature: {cert.signature}")

# Analizza AST
ast = client.analyze_file("src/main.rs")
print(f"Functions: {ast.function_count}")
print(f"Structs: {ast.struct_count}")
print(f"Enums: {ast.enum_count}")
```

#### Classi Python

```python
class Client:
    def __init__(self, server: str = "http://localhost:3000"): ...
    
    def validate_file(self, path: str, contracts: List[str] = None) -> ValidationResult: ...
    def validate(self, language: str, code: str, contracts: List[str] = None) -> ValidationResult: ...
    def certify_file(self, path: str) -> Certificate: ...
    def certify(self, language: str, code: str, agent_info: dict = None) -> Certificate: ...
    def analyze_file(self, path: str) -> AnalysisResult: ...
    def analyze(self, language: str, code: str) -> AnalysisResult: ...
    def generate_keypair(self) -> Keypair: ...

class ValidationResult:
    passed: bool
    violations: List[Violation]
    warnings: List[Violation]
    infos: List[Violation]

class Violation:
    id: str
    name: str
    severity: str  # "error", "warning", "info"
    message: str
    location: Location
    suggestion: Optional[str]

class Location:
    file: str
    line: int
    column: int
    end_line: Optional[int]
    end_column: Optional[int]

class Certificate:
    id: str
    version: int
    algorithm: str
    file_hash: str
    validation_result: ValidationResult
    certified_at: datetime
    signature: bytes

class AnalysisResult:
    language: str
    function_count: int
    struct_count: int
    enum_count: int
    trait_count: int
    lines_of_code: int
```

---

## Error Handling

### Error Types

```rust
use aether_core::Error;
use aether_sdk::SdkError;
use aether_validation::ValidationError;
use aether_certification::CertificationError;
use aether_parsers::ParseError;

// Core errors
pub enum Error {
    Config(String),
    Io(io::Error),
    Validation(ValidationError),
    Certification(CertificationError),
    Parse(ParseError),
}

// SDK errors
pub enum SdkError {
    Http(String),
    Connection(String),
    Timeout,
    InvalidResponse(String),
}

// Validation errors
pub enum ValidationError {
    NoParser(String),
    InvalidContract(String),
    LayerError(String),
}

// Certification errors
pub enum CertificationError {
    KeypairNotFound,
    InvalidSignature,
    CertificateExpired,
    CertificateRevoked,
}
```

---

## Examples

### Validazione Completa

```rust
use aether_core::{Orchestrator, Config};
use aether_validation::ValidationPipeline;

#[tokio::main]
async fn main() -> Result<()> {
    // Config
    let config = Config::from_file("aether.yaml")?;
    
    // Orchestrator
    let orchestrator = Orchestrator::new(config);
    
    // Sessione
    let session = orchestrator.create_session();
    
    // Validazione
    let result = session.validate("src/main.rs").await?;
    
    // Report
    if result.passed {
        println!("✅ Validation passed!");
    } else {
        println!("❌ Validation failed with {} violations", result.violations.len());
        for v in result.violations {
            println!("  - {}: {}", v.id, v.message);
        }
    }
    
    Ok(())
}
```

### Certificazione

```rust
use aether_certification::{Keypair, CertificateSigner};

#[tokio::main]
async fn main() -> Result<()> {
    // Carica keypair
    let keypair = Keypair::load(".aether/keypair.json")?;
    
    // Signer
    let signer = CertificateSigner::new(keypair);
    
    // Certifica
    let cert = signer.sign(
        file_hash,
        validation_result,
        AgentInfo {
            name: "my-agent".to_string(),
            version: "1.0.0".to_string(),
            model: None,
        },
    )?;
    
    // Salva
    cert.save("cert.json")?;
    
    println!("Certificate created: {}", cert.id);
    Ok(())
}
```

---

## Risorse

- [User Guide](./USER_GUIDE.md)
- [Architecture](./AETHER_ARCHITECTURE.md)
- [Contracts Registry](./CONTRACTS_REGISTRY.md)
- [GitHub Repository](https://github.com/aether-cloud/aether)
