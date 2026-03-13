# Aether — User Guide

**Versione:** 0.1.0  
**Aggiornato:** 2026-03-11

---

## Indice

1. [Introduzione](#introduzione)
2. [Installazione](#installazione)
3. [Quick Start](#quick-start)
4. [CLI Reference](#cli-reference)
5. [Configurazione](#configurazione)
6. [Linguaggi Supportati](#linguaggi-supportati)
7. [Contratti](#contratti)
8. [Certificazione](#certificazione)
9. [Integrazioni](#integrazioni)
10. [Troubleshooting](#troubleshooting)

---

## Introduzione

Aether è un sistema di validazione e certificazione del codice che utilizza contratti YAML per verificare la qualità, sicurezza e correttezza del codice.

### Caratteristiche Principali

- **Validazione multi-layer**: Syntax, Semantic, Logic, Security, Style, Architecture
- **9 linguaggi supportati**: Rust, Python, JavaScript, TypeScript, C++, Go, Java, Lua, Lex
- **Contratti YAML**: Regole personalizzabili per ogni progetto
- **Certificazione crittografica**: Firma Ed25519 per codice validato
- **Integrazione MCP**: Usabile da LLM agents (Claude, GPT, etc.)
- **Python bindings**: Libreria Python con PyO3

---

## Installazione

### Da Binario (Consigliato)

```bash
# Linux/macOS
curl -sSL https://github.com/aether-cloud/aether/releases/latest/download/aether-linux-x64.tar.gz | tar xz
sudo mv aether /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/aether-cloud/aether/releases/latest/download/aether-windows-x64.zip -OutFile aether.zip
Expand-Archive aether.zip
Move-Item aether.exe C:\Windows\
```

### Da Sorgente

```bash
git clone https://github.com/aether-cloud/aether.git
cd aether
cargo build --release
cargo install --path crates/aether-cli
```

### Python Bindings

```bash
pip install aether
```

---

## Quick Start

### 1. Validazione Base

```bash
# Valida un file
aether validate src/main.rs

# Valida con contratti specifici
aether validate src/main.rs --contracts contracts/rust/memory-safety.yaml

# Valida un intero progetto
aether validate src/ --contracts contracts/rust/
```

### 2. Analisi AST

```bash
# Analizza un file
aether analyze src/main.rs

# Output JSON
aether analyze src/main.rs --format json
```

### 3. Certificazione

```bash
# Genera keypair (solo la prima volta)
aether generate-keypair

# Certifica un file validato
aether certify src/main.rs --output cert.json

# Verifica un certificato
aether verify cert.json
```

---

## CLI Reference

### Comandi Principali

| Comando | Descrizione |
|---------|-------------|
| `validate` | Valida codice sorgente |
| `analyze` | Analizza struttura AST |
| `certify` | Valida e genera certificato |
| `verify` | Verifica certificato |
| `list` | Lista contratti disponibili |
| `generate-keypair` | Genera keypair Ed25519 |

### `validate`

```bash
aether validate <FILE|DIR> [OPTIONS]

Opzioni:
  --contracts <PATH>     Directory o file YAML contratti
  --format <FORMAT>      Output: text, json (default: text)
  --severity <LEVEL>     Livello minimo: error, warning, info
  --output <FILE>        Salva output su file
```

### `analyze`

```bash
aether analyze <FILE> [OPTIONS]

Opzioni:
  --format <FORMAT>      Output: text, json (default: text)
  --output <FILE>        Salva output su file
```

### `certify`

```bash
aether certify <FILE> [OPTIONS]

Opzioni:
  --keypair <PATH>       Path al keypair (default: ~/.aether/keypair.json)
  --output <FILE>        Salva certificato su file
  --format <FORMAT>      Output: text, json (default: json)
```

### `verify`

```bash
aether verify <CERTIFICATE> [OPTIONS]

Opzioni:
  --public-key <PATH>    Path alla chiave pubblica (default: ~/.aether/public.key)
```

---

## Configurazione

### File di Configurazione

Aether cerca la configurazione in:

1. `./aether.yaml` (progetto)
2. `~/.aether/config.yaml` (utente)
3. `/etc/aether/config.yaml` (sistema)

### Esempio `aether.yaml`

```yaml
# Configurazione Aether
version: 1

# Linguaggio primario del progetto
language: rust

# Contratti abilitati
contracts:
  - memory-safety
  - error-handling
  - performance
  - idioms

# Livelli di validazione
layers:
  syntax: true
  semantic: true
  logic: true
  security: true
  style: true
  architecture: false

# Ignora file/pattern
ignore:
  - "target/**"
  - "**/generated/**"
  - "**/*.min.js"

# Output
output:
  format: text
  colors: true
  severity: warning
```

### Variabili d'Ambiente

| Variabile | Descrizione |
|-----------|-------------|
| `AETHER_CONFIG` | Path al file di configurazione |
| `AETHER_KEYPAIR` | Path al keypair |
| `AETHER_CONTRACTS` | Directory contratti aggiuntivi |
| `AETHER_NO_COLOR` | Disabilita colori output |

---

## Linguaggi Supportati

### Rust

```yaml
# contracts/rust/memory-safety.yaml
id: RUST001
name: no-unsafe
severity: error
description: Evita blocchi unsafe
pattern: "unsafe \\{"
```

### Python

```yaml
# contracts/python/security.yaml
id: PY001
name: no-eval
severity: error
description: Evita eval() e exec()
pattern: "eval\\(|exec\\("
```

### JavaScript/TypeScript

```yaml
# contracts/javascript/security.yaml
id: JS001
name: no-innerhtml
severity: warning
description: Evita innerHTML
pattern: "\\.innerHTML\\s*="
```

### C++

```yaml
# contracts/cpp/memory-safety.yaml
id: CPP001
name: no-raw-pointers
severity: warning
description: Usa smart pointers
pattern: "\\*\\s*\\w+\\s*;"
```

### Go

```yaml
# contracts/go/idioms.yaml
id: GO001
name: error-check
severity: error
description: Controlla sempre gli errori
pattern: "=.*\\)\\s*err\\s*(?!\\s*if)"
```

### Java

```yaml
# contracts/java/performance.yaml
id: JAVA001
name: string-concat
severity: info
description: Usa StringBuilder per concatenazioni
pattern: "\\+\\s*\""
```

### Lua

```yaml
# contracts/lua/security.yaml
id: LUA001
name: no-loadstring
severity: error
description: Evita loadstring()
pattern: "loadstring\\("
```

### Lex

```yaml
# contracts/lex/gameplay.yaml
id: LEX001
name: era-reference
severity: error
description: Era deve essere definita
pattern: "era:\\s*\\w+"
```

---

## Contratti

### Struttura Contratto

```yaml
id: UNIQUE_ID        # Identificatore unico (es. RUST001)
name: rule-name       # Nome della regola
severity: error       # error | warning | info
description: Descrizione della regola
pattern: "regex pattern"  # Pattern da cercare
suggestion: "Come risolvere"  # Suggerimento opzionale
```

### Severity Levels

| Livello | Descrizione |
|---------|-------------|
| `error` | Violazione critica, blocca certificazione |
| `warning` | Potenziale problema, non blocca |
| `info` | Suggerimento informativo |

### Contratti Predefiniti

Aether include contratti predefiniti per ogni linguaggio:

```
contracts/
├── rust/
│   ├── memory-safety.yaml    # RUST001-RUST010
│   ├── error-handling.yaml   # RUST011-RUST020
│   ├── performance.yaml      # RUST021-RUST030
│   └── idioms.yaml           # RUST031-RUST040
├── python/
│   └── security.yaml         # PY001-PY015
├── javascript/
│   └── security.yaml         # JS001-JS010
├── cpp/
│   └── memory-safety.yaml    # CPP001-CPP010
├── go/
│   └── idioms.yaml           # GO001-GO010
├── java/
│   └── performance.yaml      # JAVA001-JAVA010
├── lua/
│   └── security.yaml         # LUA001-LUA005
└── lex/
    ├── gameplay.yaml         # LEX001-LEX010
    └── semantic.yaml         # LEX101-LEX108
```

### Contratti Personalizzati

Crea contratti personalizzati nella directory `contracts/`:

```yaml
# contracts/custom/my-rules.yaml
id: CUSTOM001
name: no-todo
severity: warning
description: Nessun TODO nel codice
pattern: "TODO|FIXME|XXX"
suggestion: "Completa l'implementazione o rimuovi il commento"
```

---

## Certificazione

### Generazione Keypair

```bash
aether generate-keypair
# Output:
# Keypair generated successfully
# Public key: ~/.aether/public.key
# Keypair: ~/.aether/keypair.json (KEEP SECRET!)
```

### Certificato

Un certificato Aether contiene:

```json
{
  "version": 1,
  "algorithm": "Ed25519",
  "file_hash": "sha256:abc123...",
  "validation_result": {
    "passed": true,
    "violations": []
  },
  "certified_at": "2026-03-11T12:00:00Z",
  "signature": "base64-encoded-signature"
}
```

### Verifica

```bash
# Verifica un certificato
aether verify cert.json

# Verifica con chiave pubblica specifica
aether verify cert.json --public-key /path/to/public.key
```

---

## Integrazioni

### MCP Server

Aether può essere usato come MCP server per LLM agents:

```json
// Claude Desktop config
{
  "mcpServers": {
    "aether": {
      "command": "aether",
      "args": ["mcp"]
    }
  }
}
```

Tools disponibili:
- `aether_validate` — Valida codice
- `aether_certify` — Certifica codice
- `aether_analyze` — Analizza AST

### Python API

```python
from aether import Client

client = Client()

# Valida
result = client.validate_file("src/main.rs")
print(f"Passed: {result.passed}")
print(f"Violations: {result.violations}")

# Certifica
cert = client.certify_file("src/main.rs")
print(f"Certificate: {cert.signature}")

# Analizza
ast = client.analyze_file("src/main.rs")
print(f"Functions: {ast.function_count}")
print(f"Structs: {ast.struct_count}")
```

### HTTP API

```bash
# Start server
aether serve --port 8080

# Validate endpoint
curl -X POST http://localhost:8080/validate \
  -H "Content-Type: application/json" \
  -d '{"language": "rust", "code": "..."}'

# Certify endpoint
curl -X POST http://localhost:8080/certify \
  -H "Content-Type: application/json" \
  -d '{"language": "rust", "code": "..."}'
```

---

## Troubleshooting

### Errori Comuni

#### "Language not supported"

```
Error: Language 'kotlin' not supported
Supported languages: rust, python, javascript, typescript, cpp, go, java, lua, lex
```

**Soluzione**: Usa un linguaggio supportato o crea un parser personalizzato.

#### "Contract not found"

```
Error: Contract 'my-custom-rule' not found
```

**Soluzione**: Verifica che il file YAML sia nella directory `contracts/`.

#### "Keypair not found"

```
Error: Keypair not found at ~/.aether/keypair.json
```

**Soluzione**: Esegui `aether generate-keypair` prima di certificare.

#### "Invalid certificate signature"

```
Error: Certificate signature verification failed
```

**Soluzione**: Verifica di usare la chiave pubblica corretta.

### Debug Mode

```bash
# Abilita debug
RUST_LOG=debug aether validate src/main.rs

# Output verboso
aether validate src/main.rs --format json | jq .
```

### Log File

I log sono salvati in:
- Linux/macOS: `~/.aether/logs/aether.log`
- Windows: `%APPDATA%\aether\logs\aether.log`

---

## Risorse

- [Documentazione API](./API_REFERENCE.md)
- [Architettura](./AETHER_ARCHITECTURE.md)
- [Contratti Registry](./CONTRACTS_REGISTRY.md)
- [GitHub Issues](https://github.com/aether-cloud/aether/issues)
- [Discord Community](https://discord.gg/aether)

---

## Supporto

Per problemi o domande:
1. Consulta la [documentazione](./)
2. Cerca nelle [issues esistenti](https://github.com/aether-cloud/aether/issues)
3. Apri una nuova issue con:
   - Versione Aether (`aether --version`)
   - Comando eseguito
   - Output completo
   - File di configurazione (se rilevante)
