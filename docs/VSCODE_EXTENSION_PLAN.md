# Aether VS Code Extension — Piano Implementazione

**Data:** 2026-03-19
**Status:** Piano
**Prerequisito:** CLI `aether` con `--format json` (gia implementato)

---

## Stato Attuale del CLI

Il CLI `aether validate` supporta gia output JSON strutturato:

```bash
aether validate src/main.rs --format json
```

```json
{
  "passed": false,
  "language": "rust",
  "file": "src/main.rs",
  "total_violations": 3,
  "validation_violations": 2,
  "contract_violations": 1,
  "violations": [
    {
      "source": "validation",
      "layer": "syntax",
      "id": "SYN001",
      "severity": "Error",
      "message": "Missing semicolon",
      "line": 42,
      "suggestion": "Add ';' at end of statement"
    },
    {
      "source": "contract",
      "contract_id": "RUST_001",
      "contract_name": "no-unwrap",
      "id": "CONTRACT_RUST_001",
      "severity": "Warning",
      "message": "Use of unwrap() in production code",
      "line": 58
    }
  ]
}
```

### Comandi CLI Disponibili

| Comando | Flag | Uso Extension |
|---------|------|---------------|
| `aether validate <path>` | `--format json`, `--lang`, `--severity`, `--accept`, `--reason` | Diagnostics |
| `aether config <path>` | `--show`, `--init` | Settings |
| `aether memory` | `list`, `recall`, subcommands | Tree View |
| `aether discover <path>` | `--format json`, `--lang` | Pattern discovery |
| `aether drift` | `--format json`, `--commits` | Trend analysis |
| `aether tui` | `<path>` | Open in terminal |
| `aether analyze <file>` | `--format json` | AST analysis |

---

## Struttura Progetto

```
Aether/aether-vscode/
├── .vscode/
│   └── launch.json            # F5 per Extension Development Host
├── src/
│   ├── extension.ts           # Entry point (activate/deactivate)
│   ├── aetherCli.ts           # Bridge: spawn CLI, parse JSON
│   ├── diagnostics.ts         # JSON violations → DiagnosticCollection
│   ├── codeActions.ts         # Quick Fix da campo "suggestion"
│   ├── statusBar.ts           # Contatore errori/warning
│   └── commands.ts            # Registrazione comandi VS Code
├── package.json               # Extension manifest
├── tsconfig.json
└── esbuild.js                 # Bundler (raccomandato da VS Code team)
```

---

## Fasi di Sviluppo

### Fase 1 — Scaffold + Diagnostics (MVP)

**Obiettivo:** Validazione real-time con sottolineature nell'editor e Problems panel.

**Cosa fare:**
- Setup progetto TypeScript con esbuild
- `package.json` manifest con `activationEvents`, `contributes.languages`
- `aetherCli.ts`: spawn `aether validate <file> --format json`, parse stdout
- `diagnostics.ts`: mappa violations → `vscode.Diagnostic` con severity, range, message
- Trigger: `onDidSaveTextDocument` + `onDidOpenTextDocument`
- Setting: `aether.executablePath` (default: `aether` nel PATH)

**Mapping:**
```typescript
// violation.severity → DiagnosticSeverity
"Error"   → DiagnosticSeverity.Error
"Warning" → DiagnosticSeverity.Warning
"Info"    → DiagnosticSeverity.Information
"Style"   → DiagnosticSeverity.Hint

// violation.line → Range (VS Code usa 0-based)
new Range(line - 1, 0, line - 1, Number.MAX_VALUE)

// violation.source → Diagnostic.source
"aether"
```

**Risultato:** Apri un file, salva, vedi errori Aether nel Problems panel e sottolineati nell'editor.

---

### Fase 2 — Code Actions + Status Bar

**Obiettivo:** Quick Fix con suggerimenti e feedback visivo nella status bar.

**Code Actions:**
- Se violation ha campo `suggestion`, offri Code Action "Aether: Apply Fix"
- `CodeActionKind.QuickFix`
- "Fix all Aether issues in file" come azione aggregata

**Status Bar:**
- Item a sinistra: `$(error) 3 $(warning) 5` con colori
- Click → apre Problems panel filtrato su Aether
- Aggiornato ad ogni validazione

**Comandi Command Palette:**
- `Aether: Validate Current File`
- `Aether: Validate All Open Files`
- `Aether: Open TUI` → apre terminale integrato con `aether tui`

---

### Fase 3 — Config + Settings

**Obiettivo:** Configurazione dall'editor.

**VS Code Settings (`contributes.configuration`):**
```json
{
  "aether.enabled": true,
  "aether.executablePath": "aether",
  "aether.validateOnSave": true,
  "aether.validateOnType": false,
  "aether.validateOnOpen": true,
  "aether.severity": "warning",
  "aether.languages": ["rust", "python", "typescript", "javascript", "cpp", "go", "java", "lua"]
}
```

**Comandi:**
- `Aether: Open Config` → apre `.aether.toml` nel workspace
- `Aether: Init Config` → chiama `aether config --init`, poi apre il file creato

---

### Fase 4 — Memory Tree View (futura)

**Obiettivo:** Sidebar con memoria del progetto.

**Tree View (`contributes.viewsContainers` + `contributes.views`):**
```
AETHER (sidebar icon)
├── Project Status
│   └── Confidence: 0.82
├── Learned Patterns (12)
│   ├── avoid_unwrap
│   ├── prefer_result
│   └── ...
├── Accepted Violations (3)
│   ├── UNWRAP001 in main.rs:52
│   └── ...
└── Pending Questions (1)
```

**Azioni:**
- Click su violation → `vscode.window.showTextDocument` al file/riga
- Right-click → Accept/Reject/Edit
- Refresh periodico

**Data source:** `aether memory list --format json` (da implementare nel CLI se mancante)

---

### Fase 5 — Dubbioso Mode (futura)

**Obiettivo:** Popup interattivo quando Aether e incerto.

**Implementazione:**
- `vscode.window.showInformationMessage` con bottoni
- "Aether e dubbioso: La funzione X ha 127 righe. E intenzionale?"
- Bottoni: `[Intenzionale]` `[Segnala]` `[Whitelist]`
- Risposta inviata al CLI: `aether validate --accept <id> --reason "intentional"`

---

## Stack Tecnico

| Scelta | Motivazione |
|--------|-------------|
| TypeScript | Standard per VS Code extensions |
| esbuild | Bundle veloce, raccomandato da VS Code |
| child_process.spawn | Comunicazione con CLI, zero dipendenze |
| Zero npm runtime deps | Extension leggera, nessun node_modules |
| VS Code API only | Diagnostics, CodeActions, TreeView, StatusBar nativi |

---

## Linguaggi Supportati

L'extension attiva Aether per i file con queste estensioni (configurabile):

| Language ID | Extensions |
|-------------|-----------|
| rust | `.rs` |
| python | `.py` |
| typescript | `.ts`, `.tsx` |
| javascript | `.js`, `.jsx` |
| cpp | `.cpp`, `.cc`, `.h`, `.hpp` |
| c | `.c` |
| go | `.go` |
| java | `.java` |
| lua | `.lua` |
| lex | `.lex` |

---

## Stima Tempi

| Fase | Effort | Dipendenze |
|------|--------|------------|
| Fase 1 (MVP) | 2-3 ore | Nessuna |
| Fase 2 (Actions + Bar) | 1-2 ore | Fase 1 |
| Fase 3 (Config) | 1 ora | Fase 1 |
| Fase 4 (Tree View) | 3-4 ore | CLI `memory --format json` |
| Fase 5 (Dubbioso) | 1-2 ore | Fase 1 |

**MVP (Fase 1) in ~3 ore.**

---

## Note

- L'extension non ha bisogno di Language Server Protocol (LSP) — il CLI e sufficiente
- Se in futuro servisse validate-on-type (real-time), si puo aggiungere debouncing
- L'extension funziona anche con VS Code Remote (SSH, Containers) se `aether` e nel PATH remoto
- Per pubblicare: `vsce package` genera `.vsix`, installabile con `code --install-extension`
