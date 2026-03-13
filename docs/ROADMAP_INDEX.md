# Aether — Roadmap Index

**Ultimo Aggiornamento:** 2026-03-12
**Versione:** 0.1.0
**Progresso:** 95% completato (Phase 9 in corso, Proxy+Watcher Phase 1 complete, Prism Self-Hosting Phase 0.9 Lexer Fix)

---

## Roadmap Principale

La roadmap completa di sviluppo è in **[AETHER_ROADMAP.md](./AETHER_ROADMAP.md)**.

**Fasi:**
| Fase | Nome | Durata | Stato |
|------|------|--------|-------|
| 0 | Foundation | 2 settimane | ✅ Complete |
| 1 | MVP | 4 settimane | ✅ Complete |
| 2 | Validation | 3 settimane | ✅ Complete |
| 3 | Certification | 2 settimane | ✅ Complete |
| 4 | Dual-Track Validation | 3 settimane | ⚡ In Progress |
| 5 | Integration | 3 settimane | ✅ Complete |
| 6 | RAG | 2 settimane | ✅ Complete |
| 7 | Learner | 2 settimane | ✅ Complete |
| 8 | Pre-Guidance | 2 settimane | ✅ Complete |
| 9 | Polish & Launch | 2 settimane | ⚡ In Progress |
| 10 | Proxy + Watcher | 2 settimane | ⚡ Phase 1 Complete |

**Totale:** ~27 settimane

---

## Proxy + Watcher System

**Documento:** [PROXY_DESIGN.md](../.planning/PROXY_DESIGN.md)

### Phase 4: Dual-Track Validation ⚡ IN PROGRESS

**Architettura Semplificata (2026):**
```
┌─────────────────────────────────────────────────────────────────┐
│                    AETHER VALIDATION                             │
├─────────────────────────┬───────────────────────────────────────┤
│   PROXY (Real-time)     │   VALIDATORE STANDALONE               │
│   Notifiche immediate   │   Universale, CI/CD, manuale          │
├─────────────────────────┴───────────────────────────────────────┤
│                      RAG (Apprendimento)                         │
└─────────────────────────────────────────────────────────────────┘
```

**Market Context (CodeRabbit Report 2025):**
- AI generates **1.7x more issues** overall
- Security issues **2.74x higher** in AI code
- **84%** developers use AI, only **29%** trust it

**Compatibilità:**
- **Proxy:** ✅ Droid, Claude Code, Cursor | ❌ Ollama, Copilot
- **Standalone:** ✅ Tutti gli agenti, CI/CD, Git hooks

### Phase 1: Foundation ✅ COMPLETE
- [x] `aether-proxy` crate base (hyper 1.0)
- [x] `aether-watcher` crate base (notify)
- [x] Code scanner (regex per code blocks)
- [x] File annotation system
- [x] Integrazione aether-validation (SyntaxLayer + ASTLayer)

### Phase 2: API Handlers ✅ COMPLETE
- [x] OpenAI handler
- [x] Anthropic handler
- [x] HTTPS/SSL handling (rustls)
- [x] Error injection nel contenuto (visibile all'agente AI)
- [x] Test end-to-end con mock server (wiremock)
- [x] Binario eseguibile con CLI
- [x] Test con API reali (Droid/Claude Code) — scripts e documentazione pronti

### Phase 3: Desktop App (Da fare)
- [ ] Tauri setup
- [ ] Setup wizard
- [ ] System tray

### Phase 4: Polish (Da fare)
- [ ] Auto-configuration
- [ ] Documentation
- [ ] Installer

---

## Prossimi Passi per Componente

### Core Architecture
**Documento:** [AETHER_ARCHITECTURE.md](./AETHER_ARCHITECTURE.md)

- [x] Implementare Core — Orchestrator, session management, pipeline (Rust)
- [x] Implementare Rust Parser — syn integration (151 test passati)
- [x] Implementare Contract Engine — YAML loading, pattern matching
- [x] Validation Pipeline — 7 layers (Syntax, Semantic, Logic, Security, Private, Style, Architecture)

---

### Learner System
**Documento:** [AETHER_LEARNER.md](./AETHER_LEARNER.md)

- [x] Implementare `UserProfile` e storage JSON
- [x] Implementare `StatsTracker`
- [x] Implementare `MemoryStore`
- [x] Implementare `PatternExtractor`
- [x] Integrare con Pre-Guidance
- [x] Test con scenari reali (aether-learner crate completo)

---

### Pre-Guidance System
**Documento:** [AETHER_PRE_GUIDANCE.md](./AETHER_PRE_GUIDANCE.md)

- [x] Implementare `PreGuidance` core
- [x] Integrare con `PromptAnalyzer` esistente
- [x] Integrare con `RagEngine`
- [x] Integrare con `Learner`
- [x] Implementare MCP Hook
- [x] Implementare PreToolUse Hook
- [x] Test di performance (4/4 test passati)

---

### RAG System
**Documento:** [AETHER_RAG.md](./AETHER_RAG.md)

- [x] Implementare `KeywordIndex` con TF-IDF/BM25
- [x] Implementare `SemanticSearch` con FastEmbed (BGE-small-en-v1.5)
- [x] Implementare `PatternLibrary`
- [ ] Creare CLI per indicizzazione
- [x] Integrare con Pre-Guidance
- [x] Test performance (2/2 test passati)

---

### Multi-Language Support
**Documento:** [MULTI_LANGUAGE_PLAN.md](./MULTI_LANGUAGE_PLAN.md)

- [x] Approvazione piano — Confermare opzione tree-sitter
- [x] Setup infrastruttura — Parser Rust con syn, contratti YAML
- [x] Implementare Python — Parser Python completo
- [x] Implementare JavaScript — Parser JavaScript completo
- [x] Implementare TypeScript — Parser TypeScript completo
- [x] Implementare C++ — Parser C++ completo
- [x] Implementare Go — Parser Go completo
- [x] Implementare Java — Parser Java completo
- [x] Implementare Lua — Parser Lua completo
- [x] Implementare Lex — Parser Lex completo (10/10 test passati)
- [x] ParserRegistry — `with_defaults()` con tutti i 9 parser
- [ ] Test end-to-end — Validare con progetti reali

---

### Language Scaling Strategy
**Documento:** [LANGUAGE_SCALING_STRATEGY.md](./LANGUAGE_SCALING_STRATEGY.md)

Strategia per scalare a 50+ linguaggi con tree-sitter.

---

### Private Layers (Prism)
**Documento:** [PRIVATE_LAYERS_ARCHITECTURE.md](./PRIVATE_LAYERS_ARCHITECTURE.md)

**Status:** Documentato, non implementato

- [ ] Creare directory `private/aether-prism-layer/`
- [ ] Definire contratti Prism (shader, memory, neural)
- [ ] Integrare con validation pipeline

---

### Contracts Registry
**Documento:** [CONTRACTS_REGISTRY.md](./CONTRACTS_REGISTRY.md)

**Status:** Architettura documentata, implementazione da iniziare

- [ ] Creare repository `aether-ai/contracts`
- [ ] Implementare `ContractLoader` in Rust (auto-update da registry)
- [ ] Creare TUI con `ratatui` (selezione linguaggi, piattaforme, livelli)
- [ ] Implementare generatori per ogni piattaforma:
  - [ ] Claude Code / Droid
  - [ ] VS Code
  - [ ] Cursor
  - [ ] Neovim
  - [ ] Zed
  - [ ] JetBrains
  - [ ] Gemini CLI
  - [ ] Antigravity
- [ ] Setup CI/CD per aggiornamento contratti
- [ ] Documentare API del registry
- [ ] Creare contratti iniziali per Rust, C++, Prism, Lua

---

## Priorità Correnti

### Alta Priorità
1. **Phase 10: Proxy + Watcher** — Test API reali, HTTPS, Desktop app
2. **Phase 9: Polish & Launch** — Performance, docs, release
3. **Contracts Registry** — Implementazione sistema contratti automatici

### Media Priorità
4. **Multi-Language** — Test end-to-end
5. **Private Layers** — Prism integration

### Completate ✅
- Phase 0-8: Tutte complete
- Phase 10: Proxy + Watcher Phase 1 completa

---

## Stato Implementazione

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        IMPLEMENTATION STATUS                                 │
│                                                                              │
│  [✓] Phase 0: Foundation           ───────────────────────────────────────  │
│  [✓] Phase 1: MVP                  ───────────────────────────────────────  │
│  [✓] Phase 2: Validation           ───────────────────────────────────────  │
│  [✓] Phase 3: Certification        ───────────────────────────────────────  │
│  [⚡] Phase 4: Dual-Track Validation ────────────────────────────────────── │
│  [✓] Phase 5: Integration          ───────────────────────────────────────  │
│  [✓] Phase 6: RAG                  ───────────────────────────────────────  │
│  [✓] Phase 7: Learner              ───────────────────────────────────────  │
│  [✓] Phase 8: Pre-Guidance         ───────────────────────────────────────  │
│  [⚡] Phase 9: Polish & Launch      ───────────────────────────────────────  │
│  [⚡] Phase 10: Proxy + Watcher     ───────────────────────────────────────  │
│                                                                              │
│  Progress: 92% [████████████████████████████████████████░░░░░░░░░] 92%      │
│                                                                              │
│  ✓ = Complete   ⚡ = In Progress   [ ] = Not Started                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Link Rapidi

| Documento | Descrizione | Stato |
|-----------|-------------|-------|
| [AETHER_ROADMAP.md](./AETHER_ROADMAP.md) | Roadmap dettagliata per fasi | ✅ |
| [AETHER_ARCHITECTURE.md](./AETHER_ARCHITECTURE.md) | Architettura tecnica | ✅ Implementata |
| [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md) | Design complessivo | ✅ |
| [USER_GUIDE.md](./USER_GUIDE.md) | Guida utente | ✅ Nuovo |
| [API_REFERENCE.md](./API_REFERENCE.md) | Riferimento API | ✅ Nuovo |
| [CONTRACTS_REGISTRY.md](./CONTRACTS_REGISTRY.md) | Sistema contratti automatici | 📋 Da implementare |
| [PRIVATE_LAYERS_ARCHITECTURE.md](./PRIVATE_LAYERS_ARCHITECTURE.md) | Layer privati (Prism) | 📋 Documentato |

## Prossimi Task Immediati

1. **Phase 10: Proxy + Watcher** — Test con API reali, HTTPS, Desktop app
2. **Phase 9: Polish & Launch** — Performance, docs, release
3. **Contracts Registry** — Implementare ContractLoader + TUI
4. **Multi-Language** — Test end-to-end con progetti reali
5. **Private Layers** — Prism integration
