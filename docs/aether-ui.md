# Synward — VS Code Extension

**Versione:** 3.0
**Aggiornato:** 2026-03-19
**Vedi anche:** [ADR_AUTONOMOUS_SYNWARD.md](./ADR_AUTONOMOUS_SYNWARD.md)

---

## Panoramica

Synward fornisce validazione in tempo reale tramite una **VS Code Extension** che sfrutta le API native dell'editor.

**Interfacce disponibili:**
| Interfaccia | Uso principale |
|-------------|----------------|
| **VS Code Extension** | Real-time validation, quick fixes |
| **CLI TUI** (`synward tui`) | Vim/Neovim, SSH, CI |
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
- Quick access a `synward tui`

---

## Architettura Extension

```
synward-vscode/
├── src/
│   ├── extension.ts         # Entry point
│   ├── validationProvider.ts # Diagnostics
│   ├── codeActions.ts       # Quick fixes
│   ├── treeView.ts          # Sidebar panels
│   └── synwardCli.ts         # Bridge to CLI
├── package.json             # Manifest
└── README.md
```

### Comunicazione con Synward CLI

L'estensione chiama `synward` CLI con `--json` per ottenere output strutturato:

```bash
synward validate src/main.rs --lang rust --format json
```

Output JSON mappato su VS Code Diagnostics:

```typescript
interface SynwardResult {
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
  "synward.enabled": true,
  "synward.validateOnSave": true,
  "synward.validateOnType": true,
  "synward.severity": {
    "error": "error",
    "warning": "warning",
    "info": "info"
  },
  "synward.languages": ["rust", "python", "typescript"],
  "synward.configPath": null
}
```

### .synward.toml

L'estensione legge `.synward.toml` nella root del workspace:

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
SYNWARD
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
│  🤔 Synward è dubbioso                   │
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
| `synward validate --format json` | Diagnostics |
| `synward config --show` | Load settings |
| `synward memory list --format json` | Tree view data |
| `synward tui` | Open TUI from command |

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
cd Synward && cargo build --release

# Install extension (from VSIX)
code --install-extension synward-vscode-x.x.x.vsix

# O in development
cd synward-vscode && npm install && npm run compile
# F5 in VS Code to launch Extension Development Host
```

---

## Comandi VS Code

| Command | Description |
|---------|-------------|
| `synward.validate` | Validate current file |
| `synward.validateAll` | Validate all open files |
| `synward.openConfig` | Open .synward.toml |
| `synward.openTui` | Open TUI in terminal |
| `synward.showMemory` | Show memory panel |
| `synward.acceptViolation` | Accept current violation |
