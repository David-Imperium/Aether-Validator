# Synward Knowledge Base

Directory contenente la knowledge base per Synward Intelligence.

## Struttura

```
knowledge/
├── stubs/              # Type stub files (.pyi, .d.ts, etc.)
│   ├── python/         # Python type stubs (da typeshed)
│   ├── typescript/     # TypeScript .d.ts files (da DefinitelyTyped)
│   └── rust/           # Rust metadata (da rust-analyzer)
│
└── signatures/         # API signatures in YAML format
    ├── python.yaml     # Python stdlib + popular packages
    ├── typescript.yaml # JavaScript/TypeScript built-ins + React
    └── rust.yaml       # Rust std + common crates
```

## Fonti Type Stubs

| Linguaggio | Repository | Note |
|------------|------------|------|
| Python | [typeshed](https://github.com/python/typeshed) | Stub ufficiali Python |
| TypeScript | [DefinitelyTyped](https://github.com/DefinitelyTyped/DefinitelyTyped) | @types/* packages |
| Rust | rust-analyzer metadata | Da crates.io index |

## Formato YAML Signatures

```yaml
python:
  module:
    function:
      params:
        - name: param_name
          type: str
          optional: false
          position: 0
      return: ReturnType
      raises: [ExceptionType]
      common_errors:
        - error: "Description"
          correct: "right_usage()"
          wrong: "wrong_usage()"
```

## Aggiornamento

Le signatures vengono:
1. **Generate** dai type stub ufficiali
2. **Estese** manualmente per API critiche
3. **Apprese** dal LLM per API sconosciute (cached)

## Usage

```rust
use synward_intelligence::knowledge::{TypeStubLoader, LlmApiResolver};

// Load type stubs
let loader = TypeStubLoader::new();
loader.load_python_stubs("knowledge/stubs/python")?;

// Check API call
let result = loader.check_api_call("requests", "get", &args)?;

// Fallback to LLM if no stub
if result.is_no_signature() {
    let sig = llm_resolver.resolve("requests", "get").await?;
}
```
