# Aether Private Layers — Architettura per Codice Proprietario

**Versione:** v1.0
**Data:** 2026-03-10
**Classificazione:** Interno

---

## Obiettivo

Permettere layer di validazione **privati e cifrati** per codice proprietario (es. Prism, engine interno, NDAs).

---

## Architettura

### Layer Pubblici vs Privati

```
aether-validation/
├── src/
│   ├── layers/
│   │   ├── mod.rs           # Esporta solo layer pubblici
│   │   ├── syntax.rs        # Pubblico
│   │   ├── semantic.rs      # Pubblico
│   │   ├── logic.rs         # Pubblico
│   │   └── ast.rs           # Pubblico
│   │
│   └── private/             # ← MODULO PRIVATO
│       ├── mod.rs           # Non esportato
│       ├── encrypted.rs     # Decifra in memoria
│       └── prism/           # Layer Prism
│           ├── mod.rs
│           ├── shader.rs    # Validazione shader DSL
│           ├── memory.rs    # Validazione memory management
│           └── neural.rs    # Validazione neural inference
```

### Cargo.toml Separato

```toml
# aether-validation/Cargo.toml
[package]
name = "aether-validation"
version = "0.1.0"

[features]
default = ["public-layers"]

# Layer pubblici (default)
public-layers = ["syntax", "semantic", "logic", "ast"]

# Layer privati (solo build interno)
private-layers = ["prism"]

[dependencies]
# Dipendenze pubbliche
aether-core = { path = "../aether-core" }

# Dipendenze private (feature-gated)
[dependencies.aether-private-prism]
path = "../../private/aether-prism-layer"
optional = true
```

---

## Implementazione

### 1. Struttura Directory

```
C:\lex-exploratory\
├── Aether/                      # Pubblico (GitHub)
│   └── crates/
│       └── aether-validation/
│           └── src/layers/      # Solo layer pubblici
│
├── private/                     # Privato (non in repo)
│   └── aether-prism-layer/      # Layer Prism privato
│       ├── Cargo.toml
│       ├── encrypted/           # Regole cifrate
│       │   └── prism_rules.enc
│       └── src/
│           ├── lib.rs
│           ├── shader.rs
│           ├── memory.rs
│           └── neural.rs
│
└── .gitignore                   # Ignora private/
```

### 2. Layer Prism Privato

```rust
// private/aether-prism-layer/src/lib.rs
//! Prism Private Validation Layer
//! 
//! Questo modulo è CIFRATO e non in repository pubblico.

use aether_validation::{ValidationLayer, Violation, ValidationContext};
use aether_core::Result;

/// Prism shader DSL validation
pub struct PrismShaderLayer;

impl ValidationLayer for PrismShaderLayer {
    fn name(&self) -> &str { "prism-shader" }
    
    async fn validate(&self, ctx: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Regole specifiche Prism shader
        // Queste regole sono proprietarie e cifrate
        
        violations
    }
}

/// Prism memory management validation
pub struct PrismMemoryLayer;

impl ValidationLayer for PrismMemoryLayer {
    fn name(&self) -> &str { "prism-memory" }
    
    async fn validate(&self, ctx: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Regole memory Prism
        // Niente borrow checker obbligatorio, ma safety checks
        
        violations
    }
}

/// Prism neural inference validation
pub struct PrismNeuralLayer;

impl ValidationLayer for PrismNeuralLayer {
    fn name(&self) -> &str { "prism-neural" }
    
    async fn validate(&self, ctx: &ValidationContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        
        // Regole neural inference
        // Validazione tensor shapes, memory layout
        
        violations
    }
}
```

### 3. Build Conditional

```rust
// aether-validation/src/layers/mod.rs
pub mod syntax;
pub mod semantic;
pub mod logic;
pub mod ast;

// Layer privati (feature-gated)
#[cfg(feature = "private-layers")]
pub mod private;

// Re-exports pubblici
pub use syntax::SyntaxLayer;
pub use semantic::SemanticLayer;
pub use logic::LogicLayer;
pub use ast::ASTLayer;

// Re-exports privati (solo se feature abilitata)
#[cfg(feature = "private-layers")]
pub use private::{
    PrismShaderLayer,
    PrismMemoryLayer,
    PrismNeuralLayer,
};
```

### 4. Registrazione Condizionale

```rust
// aether-cli/src/main.rs
fn build_pipeline(features: &Features) -> ValidationPipeline {
    let mut pipeline = ValidationPipeline::new()
        .add_layer(SyntaxLayer::new())
        .add_layer(SemanticLayer::new())
        .add_layer(LogicLayer::new());
    
    // Aggiungi layer privati se disponibili
    #[cfg(feature = "private-layers")]
    {
        pipeline = pipeline
            .add_layer(PrismShaderLayer::new())
            .add_layer(PrismMemoryLayer::new())
            .add_layer(PrismNeuralLayer::new());
    }
    
    pipeline
}
```

---

## Contratti Prism (Esempio)

### Shader DSL

```yaml
# private/aether-prism-layer/contracts/shader.yaml
contracts:
  - id: PRISM_SHADER_001
    name: "Shader entry point required"
    domain: shader
    severity: error
    patterns:
      - "@vertex"
      - "@fragment"
      - "@compute"
    message: "Shader must have entry point (@vertex, @fragment, @compute)"
    
  - id: PRISM_SHADER_002
    name: "No dynamic allocation in shader"
    domain: shader
    severity: error
    patterns:
      - "new "
      - "malloc"
      - "alloc"
    message: "Dynamic allocation not allowed in shaders"
    
  - id: PRISM_SHADER_003
    name: "Uniform buffer alignment"
    domain: shader
    severity: warning
    check: "uniform_buffer_alignment"
    message: "Uniform buffers must be 16-byte aligned"
```

### Memory Management

```yaml
# private/aether-prism-layer/contracts/memory.yaml
contracts:
  - id: PRISM_MEM_001
    name: "No use-after-free"
    domain: memory
    severity: error
    check: "use_after_free"
    
  - id: PRISM_MEM_002
    name: "Optional borrow check"
    domain: memory
    severity: warning
    check: "borrow_check_optional"
    message: "Consider enabling borrow check for safety"
```

### Neural Inference

```yaml
# private/aether-prism-layer/contracts/neural.yaml
contracts:
  - id: PRISM_NEURAL_001
    name: "Tensor shape consistency"
    domain: neural
    severity: error
    check: "tensor_shape_consistency"
    
  - id: PRISM_NEURAL_002
    name: "Memory layout for GPU"
    domain: neural
    severity: warning
    check: "gpu_memory_layout"
    message: "Consider row-major layout for GPU efficiency"
```

---

## Cifrazione Regole

### Schema

```
private/aether-prism-layer/
├── encrypted/
│   ├── prism_rules.enc      # Regole cifrate
│   └── key.der              # Chiave derivata (non in repo)
└── src/
    └── decrypt.rs           # Decifra in memoria
```

### Decifratura Runtime

```rust
// private/aether-prism-layer/src/decrypt.rs
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub fn decrypt_rules(encrypted: &[u8], key: &[u8]) -> Vec<u8> {
    // Decifra in memoria, mai su disco
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(&encrypted[..12]);
    let ciphertext = &encrypted[12..];
    
    cipher.decrypt(nonce, ciphertext)
        .expect("Decryption failed")
}

pub fn load_rules() -> Vec<Contract> {
    // Carica regole cifrate
    let encrypted = include_bytes!("../encrypted/prism_rules.enc");
    let key = std::env::var("PRISM_KEY")
        .expect("PRISM_KEY not set");
    
    // Decifra in memoria
    let decrypted = decrypt_rules(encrypted, key.as_bytes());
    
    // Parsa regole
    let contracts: Vec<Contract> = serde_yaml::from_slice(&decrypted)
        .expect("Invalid rules format");
    
    // Zero-out memoria dopo uso
    zero_memory(&mut decrypted);
    
    contracts
}

fn zero_memory(data: &mut [u8]) {
    use std::ptr;
    unsafe {
        ptr::write_bytes(data.as_mut_ptr(), 0, data.len());
    }
}
```

---

## Build Script

```bash
# scripts/build-private.sh
#!/bin/bash

# Build con layer privati
cargo build --features private-layers

# Senza layer privati (per distribuzione pubblica)
cargo build

# Build cifrato
python3 scripts/encrypt-rules.py \
    --input private/aether-prism-layer/contracts/*.yaml \
    --output private/aether-prism-layer/encrypted/prism_rules.enc \
    --key $PRISM_KEY
```

---

## Distribuzione

### Binari Pubblici

```bash
# GitHub release (solo layer pubblici)
cargo build --release
# Non contiene Prism layer
```

### Binari Privati

```bash
# Build interno (con Prism layer)
PRISM_KEY=$(cat ~/.config/prism/key) cargo build --release --features private-layers
# Contiene Prism layer decifrato in memoria
```

---

## .gitignore

```gitignore
# .gitignore

# Directory privata
private/

# Chiavi
*.key
*.pem
*.der

# Regole cifrate (generate, non versionate)
*.enc

# Variabili ambiente
.env
PRISM_KEY=*
```

---

## Integrazione con Aether CLI

```bash
# Validazione pubblica (default)
$ aether validate --input main.rs

# Validazione con Prism layer (richiede build privato)
$ aether validate --input shader.prism --language prism

# Output:
# Validating: shader.prism (prism)
#   [syntax] OK
#   [semantic] OK
#   [prism-shader] 2 warnings
#     - PRISM_SHADER_002: Dynamic allocation not allowed
#     - PRISM_SHADER_003: Uniform buffer not aligned
```

---

## Security Checklist

- [ ] Regole cifrate con AES-256-GCM
- [ ] Chiave mai in repository
- [ ] Decifrazione solo in memoria
- [ ] Zero-out memoria dopo uso
- [ ] Feature-gated in Cargo.toml
- [ ] .gitignore per private/
- [ ] Build script separato
- [ ] CI/CD non accede a private/

---

## Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.
