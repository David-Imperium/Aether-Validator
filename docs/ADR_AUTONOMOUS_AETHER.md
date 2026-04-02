# ADR: Synward Autonomo (AI-free Architecture)

**Status:** Proposed  
**Date:** 2026-03-18  
**Authors:** David + Droid

---

## Context

Synward è nato come validatore di codice universale. Durante lo sviluppo è emersa la necessità di chiarire:

1. **Dipendenza da AI esterna** - Synward deve funzionare autonomamente?
2. **Come impara** - Graph RAG + Memory = apprendimento senza AI?
3. **Integrazione Git** - Feedback implicito tramite hooks?
4. **Interfaccia "dubbiosa"** - Confidence-based questioning?

---

## Decision

### 1. Synward è Autonomo (AI-free Core)

**Principio:** Synward NON richiede AI esterna per funzionare. L'AI è opzionale e limitata a funzioni di supporto ("dizionario").

| Componente | Autonomo | Richiede AI |
|------------|----------|-------------|
| Syntax validation | ✅ tree-sitter | ❌ |
| Contract validation | ✅ YAML rules | ❌ |
| Graph RAG traversal | ✅ AST + imports | ❌ |
| Memory/Feedback | ✅ File-based | ❌ |
| Git integration | ✅ Hooks | ❌ |
| Confidence scoring | ✅ Statistiche | ❌ |
| suggest_fixes | ⚠️ Regole statiche | ✅ (opzionale) |
| Traduzione messaggi | ⚠️ Hardcoded | ✅ (opzionale) |

**AI esterna opzionale serve solo per:**
- Traduzione/estensione messaggi (dizionario)
- Spiegazioni dettagliate di termini
- Maybe: generazione documentazione

---

### 2. Apprendimento Autonomo

**Graph RAG + Memory = Learning Loop**

```
┌─────────────────────────────────────────────────────────────┐
│                     LEARNING CYCLE                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Valida file A                                           │
│       ↓                                                     │
│  2. Trova violazione in funzione foo()                      │
│       ↓                                                     │
│  3. Graph RAG: chi chiama foo()? Dove è definita?           │
│       ↓                                                     │
│  4. Attraversa: import → file B → definizione foo()         │
│       ↓                                                     │
│  5. Controlla B → capisce contesto completo                 │
│       ↓                                                     │
│  6. Se ancora dubbioso → chiede via MCP                     │
│       ↓                                                     │
│  7. Agente/Developer risponde                               │
│       ↓                                                     │
│  8. Memory salva: pattern + esito → confidence update       │
│       ↓                                                     │
│  9. Prossima volta: confidence più alto                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Memory Structure:**

```
~/.synward/                    # Memoria globale utente
├── learned_patterns.toml     # Pattern imparati cross-project (leggibile!)
├── global_whitelist.toml     # Whitelist globale
├── stats.toml                # Statistiche aggregate
└── presets/                  # Bundled presets (sola lettura)
    ├── rust-security.toml
    ├── python-style.toml
    └── ...

project/.synward/              # Memoria locale (override)
├── learned_config.toml       # Config imparata per progetto
├── validation_state.toml     # Stato validazione file
└── cache/                    # Cache AST (binario, nascosto)
```

---

### 14. Memoria Formato e Sharing

#### Formato: TOML (leggibile e modificabile)

```toml
# learned_patterns.toml - Pattern imparati da Synward

# Questo file è auto-generato ma può essere modificato manualmente
# Synward lo ricaricherà alla prossima validazione

[[patterns]]
id = "UNWRAP001"
rule = "no_panic"
confidence = 0.85
times_seen = 12
times_accepted = 8
times_fixed = 4
first_seen = 2026-03-10T14:30:00Z
last_seen = 2026-03-18T09:15:00Z

[[patterns]]
id = "STYLE003"
rule = "naming_convention"
confidence = 0.72
times_seen = 5
times_accepted = 5
times_fixed = 0
note = "Questo pattern è sempre accettato, considerare whitelist"
```

#### Vantaggi TOML:
- ✅ Commenti supportati
- ✅ Leggibile da umani
- ✅ Modificabile con qualsiasi editor
- ✅ Git-friendly (diff chiari)
- ✅ Meno errori di indentazione vs YAML

#### Bundled Presets (Distribuzione Pubblica)

Synward include preset pre-impostati, puliti, senza info personali:

```
~/.synward/presets/           # Bundled con Synward
├── rust-security.toml       # Regole sicurezza Rust
├── rust-style.toml          # Style conventions Rust
├── python-security.toml
├── python-style.toml
├── javascript-best.toml
└── ...
```

**Contenuto preset (anonimo, solo pattern):**

```toml
# rust-security.toml - Bundled preset
# Versione: 1.0.0
# Aggiornato: 2026-03-01

[preset]
name = "Rust Security Best Practices"
language = "rust"
category = "security"

[[rules]]
id = "UNSAFE001"
pattern = "unsafe\\s+\\{"
message = "Unsafe block detected"
severity = "warning"
confidence_base = 0.7  # Confidence iniziale

[[rules]]
id = "UNWRAP001"
pattern = "\\.unwrap\\(\\)"
message = "Potential panic: unwrap without check"
severity = "error"
confidence_base = 0.8
```

#### Tier-based Memory Sharing

| Tier | Memoria | Sharing |
|------|---------|---------|
| **Free** | Locale singolo utente | `~/.synward/` |
| **Team** | Condivisa nel team | Cloud sync automatico |
| **Enterprise** | Condivisa + audit | Cloud + policies + backup |

**Free (attuale):**
```
User A → ~/.synward/          # Solo locale
User B → ~/.synward/          # Separato, non condiviso
```

**Team (futuro):**
```
Team X → shared.synward.io/team-x/
         ├── learned_patterns.toml   # Sync automatico
         ├── global_whitelist.toml
         └── policies.toml           # Regole del team

User A (Team X) → locale + sync con team
User B (Team X) → locale + sync con team
```

**Enterprise (futuro):**
```
Company Y → enterprise.synward.io/
            ├── teams/
            │   ├── frontend/
            │   └── backend/
            ├── policies/            # Compliance rules
            ├── audit.log            # Chi ha fatto cosa
            └── backup/              # Versioning memoria
```

#### Export/Import (per sharing manuale)

```bash
# Esporta pattern imparati (pulito, no path personali)
$ synward memory export --clean > my-patterns.toml

# Il file esportato contiene solo:
# - Pattern ID
# - Regola
# - Confidence
# - Statistiche aggregate
# NO: file path, snippet codice, timestamp utente

# Importa pattern da qualcun altro
$ synward memory import friend-patterns.toml --merge
# --merge: combina con esistenti
# --replace: sovrascrive
```

#### CLI per gestire memoria

```bash
# Visualizza memoria
$ synward memory show
  Pattern     | Confidence | Times Seen | Status
  ------------|------------|------------|--------
  UNWRAP001   | 85%        | 12         | Learning
  STYLE003    | 72%        | 5          | Accepted

# Modifica interattiva
$ synward memory edit
  Open editor with learned_patterns.toml

# Reset memoria (mantieni presets)
$ synward memory reset --keep-presets

# Condividi con team (Team tier)
$ synward memory push
$ synward memory pull
```

---

### 3. Git Integration

**Target:** Developer normali + Agenti AI

#### 3.1 Git Hooks

| Hook | Funzione | Configurabile |
|------|----------|---------------|
| `pre-commit` | Valida staged files | Severity level blocco |
| `post-commit` | Analizza commit, aggiorna memoria | On/off |
| `pre-push` | Validazione completa prima di push | On/off |

**Configurazione severity:**

```toml
# synward.toml
[git]
pre_commit = true
block_on = ["error"]  # ["error", "warning", "style"]
post_commit = true    # Aggiorna memoria dopo commit
pre_push = false      # Validazione completa
```

**Flusso feedback implicito:**

```
Developer committa codice
    ↓
pre-commit: Synward valida
    ↓
Se errori critici (configurabile) → blocca commit
    ↓
Developer fixa O configurA whitelist
    ↓
Se procede senza fixare certe violazioni →
    ↓
post-commit: Synward nota pattern "accettato implicitamente"
    ↓
Dopo N volte → suggerisce whitelist automatica
    ↓
Developer conferma → Memory salva
```

#### 3.2 Per Agenti AI

Gli agenti usano Synward via MCP:

```
Agente → synward.validate_file(path)
       ← violations[{id, rule, confidence, message}]
       → fixa
       → synward.accept_violation(id) // "questa è ok"
       ← Memory aggiornata
```

---

### 4. "Dubbioso" Mode

**Principio:** Synward non dice "questo è sbagliato", dice "questo è problematico con X% confidence".

**Dettagli implementativi:** Vedi [DUBBIOSO_MODE.md](./DUBBIOSO_MODE.md)

**Componenti chiave:**
- Confidence scoring basato su statistiche
- Hyper-Context Engine (Graph RAG + Semantic + Scoring)
- MCP questioning protocol per feedback
- Thresholds configurabili in `.synward.toml`

---

### 5. Deployment Targets

| Target | Priority | Usecase |
|--------|----------|---------|
| **MCP Tool** | Primario | Agenti AI (Droid, Claude, etc.) |
| **CLI** | Secondario | Developer da terminale |
| **VS Code Extension** | Terziario | Developer GUI |
| **Git Hooks** | Integrato | CI/CD + workflow |

---

### 6. Multi-Lingua

**Auto-detect per file:**

```
project/
├── src/
│   ├── main.rs       → Rust
│   ├── parser.py     → Python
│   └── utils.js      → JavaScript
└── contracts/
    └── security.yaml → YAML (config)
```

Ogni file viene validato con il parser appropriato. Contratti YAML definiscono regole per linguaggi specifici.

---

### 7. Privacy

**Regola:** Codice NON esce mai dal machine.

- 100% locale
- Nessuna telemetria
- AI dizionario opzionale richiede consenso esplicito e manda solo messaggi, non codice

---

### 8. Configurazione Progetto

**Struttura:**

```
project/
├── synward.toml           # Config utente (editabile)
├── .synward/              # Dati generati (non toccare)
│   ├── learned_config.json
│   ├── validation_state.json
│   └── cache/
├── .gitignore            # Synward rispetta automaticamente
└── src/
```

**synward.toml esempio:**

```toml
# Synward Configuration
# Generato da: synward init

[project]
name = "my-project"
languages = ["rust", "python"]  # Auto-detect se omesso

[validation]
# Tutti i layer attivi di default
syntax = true
semantic = true
contracts = true
style = true
security = true

[git]
pre_commit = true
block_on = ["error"]  # ["error", "warning", "style"]
post_commit = true
pre_push = false

[doubt]
ask_threshold = 0.5
warn_threshold = 0.3

[memory]
global = true        # Usa memoria globale ~/.synward/
local_override = true # Permetti override locale

[ignore]
# File aggiuntivi da ignorare (oltre a .gitignore)
patterns = ["*_generated.rs", "vendor/**"]
```

---

### 9. CLI Interattiva

**Wizard setup:**

```bash
$ synward init

Welcome to Synward! Let's configure your project.

? Which languages do you use? (auto-detect from files)
  ✓ Rust (.rs)
  ✓ Python (.py)
  ✗ JavaScript
  ✗ Go

? Git integration:
  ✓ pre-commit hook (validate before commit)
  ✓ post-commit hook (learn from commits)
  ✗ pre-push hook (full validation)
  
? Block commits on:
  ◉ Errors only
  ○ Errors + Warnings
  ○ Never block (informational only)

? Memory sharing:
  ◉ Use global memory + local overrides
  ○ Isolated per-project
  
? Done! Created synward.toml

Run 'synward validate' to check your code.
```

---

### 10. Certification (Ed25519)

**Status:** Opzionale, avanzato

**Use cases:**
- Team enterprise con audit requirements
- CI/CD pipelines che verificano provenienza
- Synward come "gate" per produzione

**Default:** Disabilitato per uso generale, abilitabile in config.

```toml
[certification]
enabled = false  # Advanced feature
signer = "team-name"
```

---

### 11. Validation Layers

**Tutti attivi di default, configurabili:**

| Layer | Funzione | Default |
|-------|----------|---------|
| Syntax | Parsing, errori sintassi | ON |
| Semantic | Tipi, scope, riferimenti | ON |
| Contracts | Regole YAML custom | ON |
| Style | Formatting, naming | ON |
| Security | Unsafe, panics, injection | ON |
| Logic | Pattern problem detection | ON |

---

### 12. Output Format

**Auto-detect basato su contesto:**

| Contesto | Output |
|----------|--------|
| MCP Tool | JSON strutturato |
| CLI (TTY) | Pretty print con colori |
| CLI (pipe) | JSON (per script) |
| VS Code Extension | Via CLI, JSON |

---

### 13. Falsi Positivi

**Strategia:**

1. Accumula statistiche per progetto
2. Pattern ricorrente → suggerisce whitelist
3. Chiede conferma prima di ignorare automaticamente
4. Confidence si aggiorna basandosi su feedback

```
After 5 commits with same violation not fixed:

? I noticed you keep committing with UNWRAP001 in parser.rs.
  Should I add this to the project whitelist?
  
  [y] Yes, always ignore for this pattern
  [n] No, keep warning
  [d] Show details first
```

---

## Consequences

### Positive
- Synward funziona offline, senza API keys
- Privacy totale: codice non esce mai
- Apprendimento autonomo via Graph RAG + Memory
- Interfaccia "dubbiosa" riduce falsi positivi
- Git integration per developer normali e agenti

### Negative
- Apprendimento iniziale più lento senza AI
- Suggerimenti meno "intelligenti" senza sampling
- Richiede feedback umano per tuning iniziale

### Mitigations
- Preset contratti per linguaggi comuni
- CLI wizard semplifica setup
- Memory globale accelera learning cross-project
- AI dizionario opzionale per chi vuole

---

## Implementation Phases

### Phase 1: Core Autonomo ✅ (in progress)
- [x] Validation layers (syntax, semantic, contracts, style, security)
- [x] Graph RAG traversal
- [ ] Memory-Driven Core (fix compilazione in corso)
- [ ] Confidence scoring
- [ ] Convertire memoria da JSON a TOML

### Phase 1.5: Memory Format & Bundled Presets
- [ ] Implementare formato TOML per memoria
- [ ] Creare bundled presets per linguaggi comuni
- [ ] CLI: synward memory show/edit/export/import
- [ ] Pulizia automatica info personali in export

### Phase 2: Git Integration
- [ ] pre-commit hook
- [ ] post-commit feedback
- [ ] .gitignore respect

### Phase 3: "Dubbioso" Mode
**Dettagli:** Vedi [DUBBIOSO_MODE.md](./DUBBIOSO_MODE.md)

- [ ] Hyper-Context Engine (Graph RAG + Semantic + Scoring)
- [ ] MCP questioning protocol
- [ ] Thresholds configurabili
- [ ] Feedback loop completo

### Phase 4: CLI Wizard
- [ ] synward init (wizard interattivo)
- [ ] synward config
- [ ] synward validate

### Phase 5: Polish
- [ ] VS Code Extension integration
- [ ] Documentation
- [ ] AI dizionario opzionale

---

## References

- [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md)
- [SYNWARD_ARCHITECTURE.md](./SYNWARD_ARCHITECTURE.md)
- [SYNWARD_INTELLIGENCE.md](./SYNWARD_INTELLIGENCE.md)
