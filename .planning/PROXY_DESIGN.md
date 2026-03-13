# Aether Proxy — Design Document

> Versione: 2.0
> Data: 2025-03-12
> Status: Approved

## 1. Visione

Aether diventa un **sistema di validazione intelligente** che protegge il codice generato da agenti AI tramite due metodi complementari:

### Modalità Operative

| Modalità | Metodo | Scenario | Come l'agente vede errori |
|----------|--------|----------|---------------------------|
| **Real-time (Cloud)** | Proxy HTTP | API OpenAI/Anthropic | Iniettati nella risposta HTTP |
| **Real-time (Local)** | File Watcher | Modelli locali (Ollama, LM Studio) | Commenti annotati nel file |
| **Manuale** | Desktop App | Codice esistente | UI con report interattivo |

---

## 2. Architettura

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AETHER SYSTEM                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐         │
│  │  Desktop App    │     │  Proxy Server   │     │  File Watcher   │         │
│  │  (Tauri)        │     │  (localhost)    │     │  (background)   │         │
│  │                 │     │                 │     │                 │         │
│  │  • UI Setup     │     │  • OpenAI API   │     │  • Monitora FS  │         │
│  │  • Drag&Drop    │     │  • Anthropic    │     │  • Annota file  │         │
│  │  • Reports      │     │  • Inietta err. │     │  • Auto-fix     │         │
│  │  • Settings     │     │                 │     │                 │         │
│  └────────┬────────┘     └────────┬────────┘     └────────┬────────┘         │
│           │                       │                       │                  │
│           └───────────────────────┼───────────────────────┘                  │
│                                   ▼                                          │
│                        ┌─────────────────────┐                               │
│                        │    AETHER CORE      │                               │
│                        │                     │                               │
│                        │  • Parsers          │                               │
│                        │  • Validation       │                               │
│                        │  • Contracts        │                               │
│                        │  • Certification    │                               │
│                        └─────────────────────┘                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Modalità 1: Proxy HTTP (API Cloud)

### 3.1 Come Funziona

Per agenti che usano API cloud (OpenAI, Anthropic):

```
Agente AI                    Aether Proxy                    API LLM
    │                             │                             │
    │  POST /v1/chat/completions  │                             │
    │────────────────────────────►│                             │
    │                             │                             │
    │                             │  Forward request            │
    │                             │────────────────────────────►│
    │                             │                             │
    │                             │  Response with code         │
    │                             │◄────────────────────────────│
    │                             │                             │
    │                             │  ┌───────────────────────┐  │
    │                             │  │ 1. Scan per codice    │  │
    │                             │  │    ```rust ... ```    │  │
    │                             │  │                       │  │
    │                             │  │ 2. Validate con core  │  │
    │                             │  │                       │  │
    │                             │  │ 3. Se errori:         │  │
    │                             │  │    → Inietta nella    │  │
    │                             │  │      risposta         │  │
    │                             │  └───────────────────────┘  │
    │                             │                             │
    │  Response (modificata)      │                             │
    │◄────────────────────────────│                             │
```

### 3.2 Iniezione Errori

Quando Aether trova errori, li inietta nella risposta HTTP:

```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "Ecco il codice:\n\n```rust\nfn process() { ... }\n```"
    }
  }],
  "_aether": {
    "blocked": true,
    "errors": [
      {
        "id": "LOGIC001",
        "severity": "error",
        "message": "Implicit unwrap",
        "line": 2,
        "suggestion": "Use match or explicit error handling"
      }
    ]
  }
}
```

---

## 4. Modalità 2: File Watcher (Modelli Locali)

### 4.1 Come Funziona

Per agenti che usano modelli locali (Ollama, LM Studio) o quando il proxy non è applicabile:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         FILE WATCHER FLOW                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   1. Aether avvia watcher sulla cartella progetto                       │
│                                                                          │
│   2. Agente scrive: src/main.rs                                          │
│                                                                          │
│   3. Aether rileva il cambiamento (< 100ms)                             │
│                                                                          │
│   4. Aether valida e trova errori                                       │
│                                                                          │
│   5. Aether ANNOTA il file con commenti                                 │
│                                                                          │
│   6. L'agente rilegge il file → Vede gli errori                         │
│                                                                          │
│   7. L'agente corregge e riscrive                                       │
│                                                                          │
│   8. Aether valida → Se OK, rimuove i commenti                          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Formato Annotazioni

```rust
// ═══════════════════════════════════════════════════════════════
// ⚠️ AETHER VALIDATION FAILED
// ═══════════════════════════════════════════════════════════════
// 
// 🔴 CRITICAL (2)
//   LOGIC001: Implicit unwrap at line 15
//     💡 Use match or explicit error handling
//   SEC001: Hardcoded secret at line 42
//     💡 Use environment variable instead
//
// 🟡 WARNING (1)
//   STYLE003: Function exceeds 50 lines
//     💡 Consider splitting into smaller functions
//
// Run: aether fix <file> --apply
// ═══════════════════════════════════════════════════════════════
```

### 4.3 Configurazione Watcher

```yaml
# .aether/watcher.yaml
enabled: true
paths:
  - "src/**/*.rs"
  - "lib/**/*.py"
  - "app/**/*.ts"
exclude:
  - "target/**"
  - "node_modules/**"
  - ".git/**"
  
languages:
  rust: true
  python: true
  typescript: true

on_error: annotate    # annotate | block | warn
on_warning: annotate
auto_fix: false

notify: true
```

---

## 5. Setup Wizard (Desktop App)

```
┌────────────────────────────────────────────────────────────────┐
│                     SETUP WIZARD                                │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Benvenuto in Aether!                                        │
│                                                                 │
│  2. Modalità di intercettazione:                               │
│     [x] Proxy HTTP (per API cloud: OpenAI, Anthropic)          │
│     [x] File Watcher (per modelli locali: Ollama, LM Studio)   │
│                                                                 │
│  3. Linguaggi da validare:                                      │
│     [x] Rust                                                    │
│     [x] Python                                                  │
│     [ ] JavaScript/TypeScript                                   │
│     [ ] C++                                                     │
│                                                                 │
│  4. Severità:                                                   │
│     ( ) Basic    - Solo errori critici                          │
│     (•) Standard - Errori + Warning                             │
│     ( ) Strict   - Military grade                               │
│                                                                 │
│  5. Auto-fix:                                                   │
│     [x] Tenta correzione automatica                             │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Aether configurerà:                                      │   │
│  │   • Proxy: localhost:8080 (per API cloud)               │   │
│  │   • Watcher: monitora cartelle progetto                  │   │
│  │                                                          │   │
│  │ Tutto rimane locale. Nessun dato esce dal computer.      │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  [ Applica ]                                                    │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## 6. Componenti Tecnici

### 6.1 Proxy Server

```rust
// crates/aether-proxy/src/lib.rs

pub struct AetherProxy {
    port: u16,
    api_handlers: Vec<Box<dyn ApiHandler>>,
    core: Arc<AetherCore>,
}

pub trait ApiHandler {
    fn matches(&self, request: &HttpRequest) -> bool;
    fn process_response(&self, response: HttpResponse) -> HttpResponse;
}

pub struct OpenAiHandler;
pub struct AnthropicHandler;
```

### 6.2 File Watcher

```rust
// crates/aether-watcher/src/lib.rs

pub struct FileWatcher {
    paths: Vec<PathBuf>,
    exclude: Vec<GlobPattern>,
    core: Arc<AetherCore>,
}

impl FileWatcher {
    pub fn start(&self) -> Result<()>;
    pub fn stop(&self) -> Result<()>;
    
    fn on_file_changed(&self, path: &Path) {
        let code = fs::read_to_string(path)?;
        let report = self.core.validate(&code, detect_language(path))?;
        
        if report.has_errors() {
            self.annotate_file(path, &report)?;
        } else {
            self.remove_annotations(path)?;
        }
    }
}
```

### 6.3 Workspace

```toml
# Cargo.toml
members = [
    "crates/aether-core",
    "crates/aether-parsers",
    "crates/aether-validation",
    "crates/aether-contracts",
    "crates/aether-certification",
    "crates/aether-api",
    "crates/aether-sdk",
    "crates/aether-cli",
    "crates/aether-proxy",      # ← NUOVO
    "crates/aether-watcher",    # ← NUOVO
    "crates/aether-desktop",    # ← NUOVO (Tauri)
]
```

---

## 7. Roadmap

### Phase 1: Foundation ✅ COMPLETATO
- [x] `aether-proxy` crate base
- [x] `aether-watcher` crate base  
- [x] Code scanner (regex per ```code```)
- [x] File annotation system
- [x] Integrazione aether-validation (SyntaxLayer, ASTLayer)

### Phase 2: API Handlers (In Corso)
- [x] OpenAI handler
- [x] Anthropic handler
- [x] HTTPS/SSL handling (rustls)
- [x] Error injection nel contenuto (visibile all'agente AI) ✅ NEW
- [ ] Test con Droid/Claude Code

### Phase 3: Desktop App (Da fare)
- [ ] Tauri setup
- [ ] Setup wizard
- [ ] System tray

### Phase 4: Polish (Da fare)
- [ ] Auto-configuration
- [ ] HTTPS/SSL handling (rustls)
- [ ] Documentation
- [ ] Installer

---

## 8. Considerazioni

### Privacy
- Tutto locale, nessun dato esce dal computer
- SQLite per storage locale
- No telemetry

### Performance
- Proxy overhead: < 50ms
- File validation: < 200ms
- Watcher latency: < 100ms

### Language Detection
- Nel proxy: usa il tag del code block (\`\`\`rust)
- Nel watcher: usa l'estensione del file

---

## 9. Prossimi Passi

1. Creare `aether-proxy` crate
2. Creare `aether-watcher` crate
3. Implementare code scanner
4. Implementare file annotation
5. Test con Droid
