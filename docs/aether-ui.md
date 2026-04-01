# Aether — VS Code Extension

**Versione:** 3.0
**Aggiornato:** 2026-03-19
**Vedi anche:** [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md)

---

## Panoramica

Aether fornisce validazione in tempo reale tramite una **VS Code Extension** che sfrutta le API native dell'editor.

**Interfacce disponibili:**
| Interfaccia | Uso principale |
|-------------|----------------|
| **VS Code Extension** | Real-time validation, quick fixes |
| **CLI TUI** (`aether tui`) | Vim/Neovim, SSH, CI |
| **CLI commands** | Scripting, automazione |

---

## VS Code Extension Features

### Diagnostics (Real-time)
- Sottolineature rosse/gialle nell'editor per errori/warning
- Integrato nel Problems panel di VS Code
- Aggiornamento automatico mentre scrivi

### Code Actions (Quick Fix)
- `Cmd+.` o click lampadina per suggerimenti
- Fix automatici con anteprima
- "Fix all in file" per errori ripetuti

### Tree View (Sidebar)
- Memory Browser: pattern appresi, decision log
- Accepted Violations: errori ignorati con motivazione

### Status Bar
- Contatore errori/warning
- Confidence indicator
- Quick access a `aether tui`

---

## Architettura Extension

```
aether-vscode/
├── src/
│   ├── extension.ts         # Entry point
│   ├── validationProvider.ts # Diagnostics
│   ├── codeActions.ts       # Quick fixes
│   ├── treeView.ts          # Sidebar panels
│   └── aetherCli.ts         # Bridge to CLI
├── package.json             # Manifest
└── README.md
```

### Comunicazione con Aether CLI

L'estensione chiama `aether` CLI con `--json` per ottenere output strutturato:

```bash
aether validate src/main.rs --lang rust --format json
```

Output JSON mappato su VS Code Diagnostics:

```typescript
interface AetherResult {
  violations: Array<{
    id: string;
    severity: "error" | "warning" | "info";
    message: string;
    line: number;
    column: number;
    endLine: number;
    endColumn: number;
    fix?: {
      title: string;
      edits: Array<{
        range: [number, number, number, number];
        newText: string;
      }>;
    };
  }>;
}
```

---

## Configurazione

### Settings (VS Code)

```json
{
  "aether.enabled": true,
  "aether.validateOnSave": true,
  "aether.validateOnType": true,
  "aether.severity": {
    "error": "error",
    "warning": "warning",
    "info": "info"
  },
  "aether.languages": ["rust", "python", "typescript"],
  "aether.configPath": null
}
```

### .aether.toml

L'estensione legge `.aether.toml` nella root del workspace:

```toml
[validation]
languages = ["rust", "python"]
severity = "warning"

[rules.custom]
my-rule = { pattern = "todo!", severity = "info" }

[thresholds]
max_complexity = 15
max_function_lines = 50
```

---

## Memory Panel (Tree View)

### Panel Structure

```
AETHER
├── 📊 Project Status
│   └─ Confidence: 0.82
├── 📝 Learned Patterns (12)
│   ├─ avoid_unwrap
│   ├─ prefer_result
│   └─ ...
├── ✅ Accepted Violations (3)
│   ├─ UNWRAP001 in main.rs:52
│   └─ ...
└── 🤔 Pending Questions (1)
```

### Actions

- Click su pattern → mostra dettaglio
- Click su violation → salta al file
- Right-click → Edit/Delete/Export

---

## Dubbioso Mode

Quando confidence < threshold, l'estensione mostra un popup:

```
┌─────────────────────────────────────────┐
│  🤔 Aether è dubbioso                   │
│                                         │
│  Confidence: 0.48 / 0.5                 │
│                                         │
│  "La funzione process_data ha 127       │
│   righe. È intenzionale?"               │
│                                         │
│  [Intenzionale]  [Segnala]  [Whitelist] │
└─────────────────────────────────────────┘
```

---

## CLI Commands Reference

Comandi usati dall'estensione:

| Comando | Uso |
|---------|-----|
| `aether validate --format json` | Diagnostics |
| `aether config --show` | Load settings |
| `aether memory list --format json` | Tree view data |
| `aether tui` | Open TUI from command |

---

## Comparison: VS Code vs TUI

| Feature | VS Code Extension | CLI TUI |
|---------|-------------------|---------|
| Real-time validation | ✅ | ❌ |
| Diagnostics inline | ✅ | ❌ |
| Quick Fix (Cmd+.) | ✅ | ❌ |
| Problems panel | ✅ | ❌ |
| Memory browser | ✅ (Tree View) | ✅ |
| Config editor | ✅ (Settings UI) | ✅ |
| SSH/Remote | ❌ | ✅ |
| Vim/Neovim | ❌ | ✅ |
| CI/CD | ❌ | ✅ |

---

## Installazione

```bash
# Build CLI
cd Aether && cargo build --release

# Install extension (from VSIX)
code --install-extension aether-vscode-x.x.x.vsix

# O in development
cd aether-vscode && npm install && npm run compile
# F5 in VS Code to launch Extension Development Host
```

---

## Comandi VS Code

| Command | Description |
|---------|-------------|
| `aether.validate` | Validate current file |
| `aether.validateAll` | Validate all open files |
| `aether.openConfig` | Open .aether.toml |
| `aether.openTui` | Open TUI in terminal |
| `aether.showMemory` | Show memory panel |
| `aether.acceptViolation` | Accept current violation |
