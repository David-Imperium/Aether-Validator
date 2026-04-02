# Synward Contracts Registry — Architettura

**Versione:** v1.0
**Data:** 2026-03-11
**Status:** Design Phase

---

## 1. Panoramica

Synward ha bisogno di un sistema per gestire contratti di validazione per linguaggi multipli, con:

1. **Installer interattivo** — L'utente sceglie linguaggi, piattaforma, livello
2. **Aggiornamenti automatici** — I contratti si aggiornano da repository remoti
3. **Multi-piattaforma** — Supporto per Claude Code, VS Code, Cursor, Neovim, Zed

---

## 2. Architettura

```
┌─────────────────────────────────────────────────────────────────┐
│                    SYNWARD CONTRACTS REGISTRY                      │
│                    (GitHub: David-Imperium/contracts)                   │
│                                                                   │
│  index.json                                                       │
│  ├── rust/                                                        │
│  │   ├── v1.0.0.yaml                                              │
│  │   ├── v1.1.0.yaml                                              │
│  │   └── latest.yaml                                              │
│  ├── cpp/                                                         │
│  │   └── ...                                                      │
│  ├── prism/                                                       │
│  │   ├── v0.5.0.yaml                                              │
│  │   └── latest.yaml                                              │
│  └── lua/                                                         │
│      └── ...                                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/Git
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SYNWARD INSTALLER                               │
│                                                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │    TUI      │  │    CLI      │  │  Config     │              │
│  │  (interatt.)│  │  (batch)   │  │  (YAML)      │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│                              │                                    │
│                              ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    GENERATOR                              │    │
│  │                                                          │    │
│  │  • Scarica contratti dal registry                        │    │
│  │  • Genera config per piattaforma scelta                  │    │
│  │  • Installa hooks/scripts                                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                    │
└──────────────────────────────│─────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    PIATTAFORME SUPPORTATE                         │
│                                                                   │
│  Claude Code          VS Code            Cursor                  │
│  ├── .factory/        ├── .vscode/       ├── .cursor/           │
│  │   ├── settings.json│   ├── settings   │   ├── settings      │
│  │   ├── contracts/   │   └── extensions │   └── rules/        │
│  │   └── scripts/     │                  │                       │
│  │       └── validate │                  │                       │
│  │                     │                  │                       │
│  Neovim               Zed                 JetBrains              │
│  ├── lua/synward/      ├── extensions/    ├── .idea/             │
│  │   ├── init.lua      │   └── synward/    │   └── synward.xml    │
│  │   └── contracts/    │                  │                       │
│                                                                   │
│  Gemini CLI           Antigravity                               │
│  ├── .gemini/         ├── .antigravity/                          │
│  │   ├── settings.json│   ├── config.yaml                       │
│  │   ├── contracts/   │   ├── contracts/                         │
│  │   └── scripts/     │   ├── rules/                             │
│  │       └── validate │   └── hooks/                             │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Contratti

### 3.1 Formato YAML

```yaml
# contracts/rust/v1.2.0.yaml
meta:
  language: rust
  version: "1.2.0"
  released: "2026-03-11"
  min_synward: "0.1.0"
  author: "David-Imperium"
  source: "https://github.com/David-Imperium/contracts/blob/main/rust/v1.2.0.yaml"

contracts:
  # Pattern matching — No code required
  - id: RUST_001
    name: "No unwrap in production"
    severity: error
    patterns:
      - ".unwrap()"
      - ".unwrap_or_else"
    message: "Use ? operator or match instead of unwrap"
    exceptions:
      - "test_*.rs"
      - "tests/**/*.rs"

  - id: RUST_002
    name: "No panic macros"
    severity: error
    patterns:
      - "panic!"
      - "unimplemented!"
      - "todo!"
    message: "Avoid panic macros in production code"

  - id: RUST_003
    name: "No unsafe blocks"
    severity: warning
    patterns:
      - "unsafe {"
      - "unsafe{"
    message: "Unsafe blocks require review"

  # Semantic analysis — Requires code
  - id: RUST_010
    name: "Use-after-free detection"
    severity: error
    check: use_after_free
    # Calls Rust function in synward-validation/src/semantic/rust.rs

  - id: RUST_011
    name: "Null pointer dereference"
    severity: error
    check: null_deref
```

### 3.2 Contratti Prism

```yaml
# contracts/prism/v0.5.0.yaml
meta:
  language: prism
  version: "0.5.0"
  released: "2026-03-11"
  min_synward: "0.1.0"
  author: "David-Imperium"
  source: "https://github.com/David-Imperium/contracts/blob/main/prism/v0.5.0.yaml"

contracts:
  # Shader DSL
  - id: PRISM_SHADER_001
    name: "Shader entry point required"
    severity: error
    patterns:
      - "@vertex"
      - "@fragment"
      - "@compute"
    require_one: true
    message: "Shader must have entry point (@vertex, @fragment, @compute)"
    scope: "shader"

  - id: PRISM_SHADER_002
    name: "No dynamic allocation in shader"
    severity: error
    patterns:
      - "new "
      - "alloc"
      - "malloc"
    message: "Dynamic allocation not allowed in shaders"
    scope: "shader"

  - id: PRISM_SHADER_003
    name: "Uniform buffer alignment"
    severity: warning
    check: uniform_alignment
    message: "Uniform buffers must be 16-byte aligned"
    scope: "shader"

  - id: PRISM_SHADER_004
    name: "No recursion in shaders"
    severity: error
    check: no_recursion
    message: "Recursion not allowed in shader code"
    scope: "shader"

  # Memory
  - id: PRISM_MEM_001
    name: "Use-after-free detection"
    severity: error
    check: use_after_free

  - id: PRISM_MEM_002
    name: "Optional borrow check"
    severity: warning
    check: borrow_check_optional
    message: "Consider enabling borrow check for safety"
```

### 3.3 Indice del Registry

```json
// index.json
{
  "version": "1.0",
  "updated": "2026-03-11T12:00:00Z",
  "languages": {
    "rust": {
      "latest": "1.2.0",
      "versions": ["1.2.0", "1.1.0", "1.0.0"],
      "min_synward": "0.1.0"
    },
    "cpp": {
      "latest": "2.0.0",
      "versions": ["2.0.0", "1.5.0", "1.0.0"],
      "min_synward": "0.1.0"
    },
    "prism": {
      "latest": "0.5.0",
      "versions": ["0.5.0", "0.4.0"],
      "min_synward": "0.1.0"
    },
    "lua": {
      "latest": "1.0.0",
      "versions": ["1.0.0"],
      "min_synward": "0.1.0"
    }
  }
}
```

---

## 4. Installer

### 4.1 TUI Interattivo

```
╔═══════════════════════════════════════════════════════════════╗
║                     SYNWARD SETUP v0.1                          ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  Step 1/3: Seleziona linguaggi                                ║
║  ──────────────────────────────────────────────────────────── ║
║                                                               ║
║   [x] Rust        Modern systems language                    ║
║   [ ] C++         Systems programming                         ║
║   [ ] Python      Scripting & ML                              ║
║   [x] Prism       Game development language                   ║
║   [ ] Lua         Embedded scripting                          ║
║   [ ] JavaScript  Web development                             ║
║   [ ] TypeScript  Typed JavaScript                            ║
║   [ ] Go         Cloud-native                                 ║
║   [ ] Java       Enterprise                                   ║
║                                                               ║
║                    [ Avanti → ]                                ║
╚═══════════════════════════════════════════════════════════════╝

╔═══════════════════════════════════════════════════════════════╗
║                     SYNWARD SETUP v0.1                          ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  Step 2/3: Seleziona piattaforma                              ║
║  ──────────────────────────────────────────────────────────── ║
║                                                               ║
║   [x] Claude Code / Droid      AI assistant integration      ║
║   [ ] VS Code                   Microsoft IDE                 ║
║   [ ] Cursor                    AI-powered IDE                ║
║   [ ] Neovim                    Modern Vim                    ║
║   [ ] Zed                       High-performance editor       ║
║   [ ] JetBrains                 IntelliJ platform             ║
║   [ ] Gemini CLI                Google AI assistant           ║
║   [ ] Antigravity               Custom IDE integration        ║
║                                                               ║
║                    [ ← Indietro ]  [ Avanti → ]               ║
╚═══════════════════════════════════════════════════════════════╝

╔═══════════════════════════════════════════════════════════════╗
║                     SYNWARD SETUP v0.1                          ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  Step 3/3: Seleziona livello                                  ║
║  ──────────────────────────────────────────────────────────── ║
║                                                               ║
║   [ ] Basic      Pattern matching only                        ║
║                  • Velocità massima                           ║
║                  • Nessuna analisi semantica                   ║
║                  • Ideale per CI veloce                        ║
║                                                               ║
║   [x] Standard   Pattern matching + Analisi semantica        ║
║                  • Bilanciato                                  ║
║                  • Controlli use-after-free, null safety       ║
║                  • Raccomandato                                ║
║                                                               ║
║   [ ] Strict     Tutte le analisi disponibili                 ║
║                  • Massima sicurezza                           ║
║                  • Controlli aggressivi                        ║
║                  • Più lento                                   ║
║                                                               ║
║                    [ ← Indietro ]  [ Installa ]               ║
╚═══════════════════════════════════════════════════════════════╝

╔═══════════════════════════════════════════════════════════════╗
║                     INSTALLAZIONE                              ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║  Scaricando contratti...                                       ║
║  ├── rust/v1.2.0.yaml     ████████████████████ 100%           ║
║  └── prism/v0.5.0.yaml    ████████████████████ 100%           ║
║                                                               ║
║  Generando configurazione...                                   ║
║  ├── .factory/settings.json    ✓                               ║
║  ├── .factory/contracts/rust/  ✓                               ║
║  ├── .factory/contracts/prism/ ✓                               ║
║  └── .factory/scripts/validate.ps1 ✓                          ║
║                                                               ║
║  ✓ Installazione completata!                                  ║
║                                                               ║
║  Per aggiornare: synward contracts update                      ║
║  Per verificare: synward contracts check                        ║
║                                                               ║
║                    [ Chiudi ]                                  ║
╚═══════════════════════════════════════════════════════════════╝
```

### 4.2 CLI Batch Mode

```bash
# Installazione completa
synward init

# Modalità non-interattiva
synward init --lang rust,prism --platform claude --level standard

# Da file di configurazione
synward init --config synward.yaml
```

### 4.3 File di Configurazione

```yaml
# synward.yaml
version: "1.0"

languages:
  - rust
  - prism

platform: claude

level: standard

# Override contratti specifici
overrides:
  rust:
    RUST_003:
      severity: info  # unsafe = info invece di warning
  prism:
    PRISM_MEM_002:
      enabled: false  # Disabilita borrow check opzionale

# Contratti custom
custom:
  - id: CUSTOM_001
    name: "No TODO comments"
    patterns:
      - "TODO"
      - "FIXME"
      - "XXX"
    severity: info
```

---

## 5. Aggiornamenti Automatici

### 5.1 Controllo Versioni

```bash
# Controlla se ci sono aggiornamenti
synward contracts check

# Output:
# Checking for updates...
# ├── rust: v1.2.0 (installed) → v1.3.0 (available)
# ├── prism: v0.5.0 (installed) → v0.5.0 (up to date)
# └── lua: not installed
#
# Run 'synward contracts update' to install updates
```

### 5.2 Aggiornamento

```bash
# Aggiorna tutti i contratti
synward contracts update

# Aggiorna solo un linguaggio
synward contracts update rust

# Output:
# Updating contracts...
# ├── rust: v1.2.0 → v1.3.0
# │   ├── New contracts: RUST_015, RUST_016
# │   └── Updated: RUST_003 (warning → error)
# └── prism: v0.5.0 (up to date)
#
# ✓ Update complete
```

### 5.3 Cache Locale

```
~/.cache/synward/
├── index.json              ← Cache dell'indice
├── rust/
│   ├── v1.2.0.yaml
│   └── v1.3.0.yaml         ← Nuova versione
├── prism/
│   └── v0.5.0.yaml
└── lua/
    └── v1.0.0.yaml
```

### 5.4 Offline Mode

```bash
# Se offline, usa cache locale
synward contracts update --offline

# Output:
# Offline mode: using cached contracts
# ├── rust: v1.2.0 (from cache)
# └── prism: v0.5.0 (from cache)
```

---

## 6. Piattaforme

### 6.1 Claude Code / Droid

**Output:**
```
.factory/
├── settings.json           ← Config validazione
├── contracts/
│   ├── rust/
│   │   └── v1.2.0.yaml
│   └── prism/
│       └── v0.5.0.yaml
└── scripts/
    └── validate.ps1        ← Script validazione
```

**settings.json:**
```json
{
  "synward": {
    "enabled": true,
    "languages": ["rust", "prism"],
    "level": "standard",
    "validateOnSave": true
  }
}
```

### 6.2 VS Code

**Output:**
```
.vscode/
├── settings.json           ← Config extension
└── extensions.json         ← Recommended extensions

synward-vscode/
├── package.json
├── src/
│   ├── extension.ts
│   └── validation.ts
└── syntaxes/
    └── synward.json
```

**settings.json:**
```json
{
  "synward.enabled": true,
  "synward.languages": ["rust", "prism"],
  "synward.level": "standard",
  "synward.updateOnSave": true
}
```

### 6.3 Cursor

**Output:**
```
.cursor/
├── settings.json
└── rules/
    ├── rust.md
    └── prism.md
```

### 6.4 Neovim

**Output:**
```
~/.config/nvim/
└── lua/
    └── synward/
        ├── init.lua
        ├── config.lua
        └── contracts/
            ├── rust.lua
            └── prism.lua
```

### 6.5 Zed

**Output:**
```
~/.config/zed/
└── extensions/
    └── synward/
        ├── extension.toml
        └── src/
            └── lib.rs
```

### 6.6 Gemini CLI

**Output:**
```
.gemini/
├── settings.json           ← Config Gemini
├── contracts/
│   ├── rust/
│   └── prism/
└── scripts/
    └── validate.sh         ← Script validazione
```

**settings.json:**
```json
{
  "synward": {
    "enabled": true,
    "languages": ["rust", "prism"],
    "level": "standard",
    "validateOnGenerate": true
  }
}
```

**Integrazione:**
- Validazione prima di ogni generazione codice
- Contratti applicati ai file generati

### 6.7 Antigravity

**Output:**
```
.antigravity/
├── config.yaml              ← Config Antigravity
├── contracts/
│   ├── rust/
│   └── prism/
├── rules/
│   ├── validation.yaml      ← Regole custom
│   └── exceptions.yaml      ← Eccezioni
└── hooks/
    └── pre-commit.sh        ← Git hook
```

**config.yaml:**
```yaml
synward:
  enabled: true
  languages:
    - rust
    - prism
  level: standard
  
  validation:
    on_save: true
    on_commit: true
    on_push: true
    
  auto_update:
    enabled: true
    check_interval: "24h"
    
  output:
    format: "inline"  # inline, popup, panel
    severity_filter:
      - error
      - warning
```

**Integrazione:**
- Validazione real-time durante editing
- Git hooks per pre-commit validation
- Auto-update contratti con intervallo configurabile
- Output inline nel editor

---

## 7. Livelli di Validazione

| Livello | Cosa include | Velocità | Use case |
|---------|-------------|----------|----------|
| **Basic** | Pattern matching | Veloce | CI veloce, pre-commit |
| **Standard** | + Analisi semantica | Medio | Development quotidiano |
| **Strict** | + Controlli aggressivi | Lento | Code review, security audit |

**Esempio Basic (solo pattern):**
```yaml
- id: RUST_001
  patterns: [".unwrap()"]
  # Solo pattern matching, niente AST
```

**Esempio Standard (+ semantica):**
```yaml
- id: RUST_010
  check: use_after_free
  # Richiede analisi del dataflow
```

**Esempio Strict (+ aggressivo):**
```yaml
- id: RUST_020
  check: cyclomatic_complexity
  params:
    max_complexity: 10
  # Complexity analysis
```

---

## 8. Registry API

### 8.1 Endpoint

```
GET https://raw.githubusercontent.com/David-Imperium/contracts/main/index.json
GET https://raw.githubusercontent.com/David-Imperium/contracts/main/rust/v1.2.0.yaml
GET https://raw.githubusercontent.com/David-Imperium/contracts/main/prism/v0.5.0.yaml
```

### 8.2 Versioning

- **Semver** per versioni contratti
- **min_synward** per compatibilità
- **Changelog** per ogni release

### 8.3 Source Alternatives

```yaml
# synward.yaml
registry:
  primary: "https://github.com/David-Imperium/contracts"
  mirrors:
    - "https://gitlab.com/David-Imperium/contracts"
    - "https://cdn.synward.ai/contracts"
  
  # Per contratti privati
  custom:
    - url: "./local-contracts/"
      name: "internal"
    - url: "https://internal.company.com/contracts"
      auth:
        type: "bearer"
        token_env: "COMPANY_TOKEN"
```

---

## 9. Contratti Custom

### 9.1 Contratto Locale

```yaml
# local-contracts/company.yaml
meta:
  language: any
  version: "1.0.0"
  author: "company"

contracts:
  - id: COMPANY_001
    name: "No console.log in production"
    severity: error
    patterns:
      - "console.log"
      - "print("
    message: "Remove debug prints before commit"
    exceptions:
      - "test_*.rs"
      - "**/test/**"
```

### 9.2 Import in Config

```yaml
# synward.yaml
imports:
  - "./local-contracts/company.yaml"
  - "https://internal.company.com/contracts/custom.yaml"
```

---

## 10. Integrazione con Synward Core

### 10.1 Caricamento Contratti

```rust
// synward-validation/src/contracts/mod.rs
pub struct ContractLoader {
    cache_dir: PathBuf,
    registry_url: String,
}

impl ContractLoader {
    pub fn load(&self, language: &str, version: &str) -> Result<Contracts> {
        // 1. Controlla cache locale
        // 2. Scarica se necessario
        // 3. Parsa YAML
        // 4. Valida struttura
    }
    
    pub fn check_updates(&self) -> Result<Vec<UpdateInfo>> {
        // 1. Fetch index.json
        // 2. Confronta con versioni installate
        // 3. Ritorna lista aggiornamenti
    }
}
```

### 10.2 Validazione

```rust
// synward-validation/src/validator.rs
impl Validator {
    pub fn validate(&self, code: &str, language: &str) -> Vec<Violation> {
        let contracts = self.contracts.get(language)?;
        let mut violations = Vec::new();
        
        for contract in &contracts.patterns {
            // Pattern matching
            if self.matches_patterns(code, &contract.patterns) {
                violations.push(Violation::from(contract));
            }
        }
        
        for contract in &contracts.semantic {
            // Semantic analysis
            if let Some(check_fn) = SEMANTIC_CHECKS.get(&contract.check) {
                violations.extend(check_fn(code, &contract.params));
            }
        }
        
        violations
    }
}
```

---

## 11. Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.

---

## 12. Riferimenti

- [Synward Architecture](./ARCHITECTURE.md)
- [Private Layers](./PRIVATE_LAYERS_ARCHITECTURE.md)
- [Validation Pipeline](./VALIDATION_PIPELINE.md)
