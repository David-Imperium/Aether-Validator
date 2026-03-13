# Aether — Stato Attuale

**Aggiornato:** 2026-03-12
**Versione:** v0.1.0-alpha

---

## Roadmap

**La roadmap ufficiale è in `docs/ROADMAP_INDEX.md`**

Qui solo task in corso o da fare.

---

## Task In Corso

### Phase 10: Proxy + Watcher
- [x] Phase 1: Foundation
- [x] Phase 2: API Handlers + Test
- [x] Phase 3: Desktop App (Tauri) - UI base funzionante
- [ ] Phase 4: Sandbox per intercettazione agenti AI

---

## Phase 4: Architettura Semplificata

### Insight dal Mercato (2025-2026)

Dati chiave da ricerche recenti:
- 84% sviluppatori usa AI, solo 29% si fida
- AI genera **1.7x più problemi**
- Security issues **2.74x più alti**
- Error handling **2x più problematico**

Il mercato cerca strumenti di **validazione nel workflow** (CI/CD, PR review, git hooks), non necessariamente intercettazione in tempo reale.

### Architettura

Aether si compone di due parti:

```
┌─────────────────────────────────────────────────────────────┐
│                    AETHER VALIDATION                         │
├─────────────────────────┬───────────────────────────────────┤
│   PROXY (real-time)     │   VALIDATORE STANDALONE           │
│   Notifiche immediate   │   Universale, CI/CD, manuale      │
├─────────────────────────┴───────────────────────────────────┤
│                      RAG (apprendimento)                     │
│   - Salva patterns, errori, correzioni                      │
│   - Suggerimenti basati su storico                          │
└─────────────────────────────────────────────────────────────┘
```

### 1. Proxy HTTP (Notifiche Real-time)

**Funzione:** Intercetta risposte API, estrae codice, valida, notifica.

```
API Response → Proxy → Estrae codice → Valida → Notifica
                                                    ↓
                                              Agente scrive
```

**Cosa fa:**
1. Intercetta risposta API (OpenAI, Anthropic, ecc.)
2. Estrae blocchi di codice markdown
3. Passa ad Aether per validazione
4. Salva risultato nella RAG
5. Invia notifica desktop

**Notifica esempio:**
```
⚠️ Aether: 3 issues in generated code
- SYNTAX002: Missing semicolon (line 45)
- SECURITY001: Hardcoded credential pattern
- STYLE004: Line too long (line 78)
```

**Compatibilità:**
- ✅ Droid (configurabile via env)
- ✅ Claude Code (configurabile)
- ✅ Cursor (parzialmente)
- ❌ Ollama locale
- ❌ GitHub Copilot (canali proprietari)

### 2. Validatore Standalone (Universale)

**Funzione:** Validazione su richiesta o integrata in workflow.

**Modi d'uso:**
- CLI: `aether validate src/`
- Desktop app: drag-and-drop o selezione directory
- CI/CD: GitHub Actions, GitLab CI
- Git hooks: pre-commit, pre-push

**Funziona con:**
- Qualsiasi agente AI
- Codice scritto manualmente
- Code review

### 3. RAG (Apprendimento)

**Funzione:** Memoria persistente per migliorare nel tempo.

**Cosa salva:**
- Tipi di errori ricorrenti
- Pattern problematici
- Correzioni applicate
- Decisioni dell'utente (ignora/fix)

**Benefici:**
- Suggerimenti contestuali
- Riduzione falsi positivi
- Adattamento al codebase specifico

### Piano Implementazione

**Fase 4a: Proxy Enhancement**
- [ ] Estrazione codice da risposte API (markdown code blocks)
- [ ] Supporto formati OpenAI/Anthropic
- [ ] Sistema notifiche desktop
- [ ] Integrazione RAG

**Fase 4b: Standalone Polish**
- [ ] CLI completo con tutte le opzioni
- [ ] Desktop app per validazione manuale
- [ ] Git hooks installer
- [ ] GitHub Action

**Fase 4c: RAG Integration**
- [ ] Storage persistente
- [ ] Query contestuali
- [ ] Feedback loop (correzioni)

---

## Task Da Fare

### Phase 4a: Proxy Enhancement
- [ ] Parser per markdown code blocks nelle risposte API
- [ ] Supporto formato risposta OpenAI (chat.completions)
- [ ] Supporto formato risposta Anthropic (messages)
- [ ] Sistema notifiche desktop (native)
- [ ] Collegamento con RAG esistente

### Phase 4b: Standalone Polish
- [ ] CLI `aether validate` con output formattato
- [ ] Desktop app: validazione manuale directory
- [ ] Installer git hooks (`aether install-hooks`)
- [ ] GitHub Action pubblicata

### Phase 4c: RAG Integration
- [ ] Salvataggio risultati validazione
- [ ] Query per pattern ricorrenti
- [ ] Suggerimenti basati su storico
- [ ] UI per visualizzare apprendimento

### Future
- [ ] Contracts Registry auto-update
- [ ] Multi-language support (Python, JS, Go)
- [ ] Private Layers (Prism)

---

## Note Tecniche

### Phase 10 Phase 3 Completata (2026-03-12)

**Desktop App (Tauri):**
- ✅ UI con Setup Wizard, Validate, Settings, Status tabs
- ✅ Finestra in primo piano all'avvio
- ✅ Avvio automatico watcher/proxy quando si seleziona un tool
- ✅ Status polling ogni 2 secondi
- ✅ Validazione manuale nel tab Validate
- ✅ Build con Vite per frontend
- ✅ Proxy porta configurabile

**Problemi noti:**
- Porta 8080 spesso occupata (Docker)
- Watcher è un semplice file monitor, non intercetta scritture

**Build:**
- `cargo tauri build --debug` - Build debug
- `C:\lex-exploratory\Aether\target\debug\aether-desktop.exe` - Eseguibile

### Phase 9 Completata (2026-03-12)

**Performance Testing:**
| Test | Tempo | Target |
|------|-------|--------|
| Full pipeline (small) | 29 µs | < 100ms ✅ |
| Full pipeline (medium) | 21 µs | < 100ms ✅ |
| Full pipeline (large) | 65 µs | < 100ms ✅ |

**Security Review:**
- ✅ Cargo audit: 1 warning (fxhash non mantenuto, dip. indiretta)
- ✅ Nessun codice unsafe nel progetto
- ✅ TLS configurato correttamente (rustls + webpki)
- ✅ Error handling sicuro

### Phase 10 Phase 2 Completata (2026-03-12)

**Proxy Testing:**
- ✅ 23 test totali (15 e2e + 8 real_api_test)
- ✅ Validator con 5 layers (Syntax, AST, Logic, Security, Style)
- ✅ Script per test manuali: `scripts/start-proxy.bat`, `scripts/test-proxy.ps1`
- ✅ Documentazione test in `SETUP.md`

**Zero warning** ✅
**213 test passano** ✅
