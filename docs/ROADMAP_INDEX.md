# Aether — Roadmap Index

**Ultimo Aggiornamento:** 2026-03-19
**Versione:** 2.0 (Autonomous Design)
**Progresso:** 95% completato (Phase 0-8, 11-15 complete, ADR Phase 1.5 + 2 complete)
**Vedi anche:** [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md)

---

## Key Differentiator: Memory-Driven Core

Il **Memory-Driven Core** è la rivoluzione di Aether: non memorizza solo — **configura dinamicamente** i validation layers basandosi sulla knowledge appresa dal progetto.

**Documento:** [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md)

Ogni istanza Aether diventa unica attraverso i LearnedConfig accumulati. Il valore cresce col tempo, rendendo costoso il cambio di validatore.

**ADR:** [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md) — 14 decisioni architetturali per Aether autonomo.

---

## Key Principles (v2.0)

- **AI-Free Core**: Nessuna AI esterna richiesta per validazione (AI opzionale come dizionario)
- **Graph RAG Autonomo**: Attraversa progetti, capisce dipendenze, impara pattern
- **Dubbioso Mode**: Confidence-based validation, chiede quando incerto via MCP
- **TOML Format**: Memoria leggibile e modificabile dall'utente
- **Phase 4 OPZIONALE**: Prompt Analysis richiede LLM, core funziona senza

### Architettura: AI è Opzionale (Decisione Permanente)

**Principio fondamentale:** Aether funzionerà **sempre** autonomamente senza AI. L'AI è e rimarrà una feature extra opzionale.

| Componente | Autonomo | AI Opzionale (futuro) |
|------------|----------|----------------------|
| Syntax validation | ✅ tree-sitter | ❌ |
| Contract validation | ✅ YAML rules | ❌ |
| Graph RAG traversal | ✅ AST + imports | ❌ |
| Memory/Feedback | ✅ File-based | ❌ |
| Confidence scoring | ✅ Statistiche | ❌ |
| suggest_fixes | ⚠️ Regole statiche | ✅ AI per spiegazioni |
| Traduzione messaggi | ⚠️ Hardcoded | ✅ AI dizionario |

**Se AI aggiunta in futuro:**
- Solo come "dizionario" per spiegazioni/traduzioni
- Richiede consenso esplicito utente
- Non modifica codice, solo messaggi
- Core rimane funzionale senza

**Vedi ADR:** [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md) — Decision 1: Aether è Autonomo

---

## Recent Fixes (2026-03-16)

### MCP Server: Stdout Pollution Fix
**Problema:** MCP server stampava log su stdout invece di stderr, rompendo il protocollo JSON-RPC.

**Fix:** Tutti i log ora vanno su stderr. MCP tools funzionanti (13 tools).

**Verifica:** `aether___get_version` restituisce correttamente version, languages_count, tools_count.

---

## Roadmap Principale

La roadmap completa è in questo documento (ROADMAP_INDEX.md).

**Fasi:**
| Fase | Nome | Durata | Stato |
|------|------|--------|-------|
| 0 | Foundation | 2 settimane | ✅ Complete |
| 1 | MVP | 4 settimane | ✅ Complete |
| 2 | Validation | 3 settimane | ✅ Complete |
| 3 | Certification | 2 settimane | ✅ Complete |
| 4 | Standalone Validation | 3 settimane | ✅ Complete |
| 5 | Integration | 3 settimane | ✅ Complete |
| 6 | RAG | 2 settimane | ✅ Complete |
| 7 | Learner | 2 settimane | ✅ Complete |
| 8 | Pre-Guidance | 2 settimane | ✅ Complete |
| 9 | Polish & Launch | 2 settimane | ⚡ In Progress |

| **11** | **Memory System Foundation** | **3 settimane** | **✅ Complete** |
| **12** | **Pattern Discovery MVP** | **3 settimane** | **✅ Complete** |
| **13** | **Intent Inference** | **3 settimane** | **✅ Complete** |
| **14** | **Drift Detection** | **2 settimane** | **✅ Complete** |
| **15** | **Memory System Enhancement** | **3 settimane** | **✅ Complete** |
| **16** | **Memory-Driven Core** | **4 settimane** | **⚡ In Progress** |

**Totale:** ~23 settimane (fase 0-9) + ~16 settimane (fase 11-16) = ~39 settimane

> **Nota**: Phase 4 (Prompt Analysis) è OPZIONALE — richiede LLM esterno. Il core autonomo funziona senza.

---

## Context Rot Research (2026)

**Fonti:** Chroma, Manifold Group, Anthropic

| Finding | Impatto |
|---------|---------|
| Claude Sonnet 4: 99% → 50% accuracy | Context window ≠ memory |
| ChatGPT: ~7±2 items effective memory | Human-like working memory |
| n² attention competition | More tokens = less accuracy |
| RAG: fragile chain, no feedback loop | Silent failures |

**Soluzione Aether:** Hybrid Memory (Knowledge Graph + File-based + Time-series) — **Autonomo, senza AI**

---

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
**Documento:** [AETHER_INTELLIGENCE.md](./AETHER_INTELLIGENCE.md) (consolidato)

- [x] Implementare `UserProfile` e storage TOML
- [x] Implementare `StatsTracker`
- [x] Implementare `MemoryStore`
- [x] Implementare `PatternExtractor`
- [x] Integrare con Pre-Guidance
- [x] Test con scenari reali (aether-learner crate completo)

---

### Pre-Guidance System
**Documento:** [AETHER_INTELLIGENCE.md](./AETHER_INTELLIGENCE.md) (consolidato)

- [x] Implementare `PreGuidance` core
- [x] Integrare con `PromptAnalyzer` esistente
- [x] Integrare con `RagEngine`
- [x] Integrare con `Learner`
- [x] Test di performance (4/4 test passati)

---

### RAG System
**Documento:** [AETHER_INTELLIGENCE.md](./AETHER_INTELLIGENCE.md) (consolidato)

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
- [ ] Definire contratti Prism (shader, memory)
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

## Aether Intelligence (Phase 11-15)

**Documenti:**
- [AETHER_INTELLIGENCE.md](./AETHER_INTELLIGENCE.md) — Design architetturale (aggiornato con Context Rot research)
- [INTELLIGENCE_IMPLEMENTATION.md](./INTELLIGENCE_IMPLEMENTATION.md) — **Piano operativo dettagliato**

Evoluzione di Aether da validatore rule-based a sistema AI autonomo con memoria, apprendimento e capacità di scoprire nuovi pattern.

### Il Problema: Context Rot

Ricerca Chroma/Manifold Group (2026) ha rivelato che gli agenti AI perdono contesto:
- **Claude Sonnet 4**: 99% → 50% accuracy quando il contesto aumenta
- **ChatGPT**: Effective memory di ~7±2 items (come umani!) anche con 128K token
- **RAG tradizionale**: Fragile, snippet ≠ understanding, no feedback loop

**Soluzione:** Hybrid Memory a 4 sub-layers (Knowledge Graph + File-based + Time-series)

### Architettura a 5 Layers

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      AETHER INTELLIGENCE ARCHITECTURE                        │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 5: Drift Detection (Temporal) — Analizza evoluzione codice            │
│  Layer 4: Intent Inference (LLM-lite) — Capisce "perché" il codice esiste    │
│  Layer 3: Pattern Discovery (ML) — Scopre nuovi anti-pattern                 │
│  Layer 2: Memory System (Hybrid) — 4 sub-layers (vedi sotto)                 │
│  Layer 1: Static Analysis (Core) — Validazione rule-based attuale            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Layer 2: Memory System (Hybrid Architecture)

**Nuovo design basato su Context Rot research:**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      AETHER MEMORY SYSTEM                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│  Layer 2A: Code Graph (AST-based)                                           │
│    File → Functions → Calls → Dependencies                                  │
│    "Chi chiama questa funzione?"                                            │
│                                                                              │
│  Layer 2B: Decision Log (Knowledge Graph)                                   │
│    "Perché questo codice esiste?" → Intent                                   │
│    "Questa decisione è ok perché..." → User feedback                        │
│                                                                              │
│  Layer 2C: Validation State (File-based)                                    │
│    Ultima validazione: {file, hash, violations, status}                     │
│    Stato "acceptato/rifiutato" per violations                               │
│                                                                              │
│  Layer 2D: Drift Snapshots (Time-series)                                    │
│    Giorno 1: {metrics} → Giorno 30: {metrics} → Trend                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Nuovo Crate: `aether-intelligence`

```
crates/aether-intelligence/
├── src/
│   ├── memory/     # Layer 2: Memory System (4 sub-layers)
│   │   ├── code_graph.rs      # 2A: AST relationships
│   │   ├── decision_log.rs    # 2B: Knowledge Graph
│   │   ├── validation_state.rs # 2C: File-based state
│   │   └── drift_snapshots.rs # 2D: Time-series
│   ├── patterns/   # Layer 3: Pattern Discovery
│   ├── intent/     # Layer 4: Intent Inference
│   ├── drift/      # Layer 5: Drift Detection
│   └── knowledge/  # API Signatures (Type Stubs + LLM)
```

### Phase 11: Memory System Foundation ✅ COMPLETE (3 settimane, 31 ore)
- [x] 11.1 Setup `aether-intelligence` crate
- [x] 11.2 Layer 2A: Code Graph (AST-based)
  - [x] `CodeNode`, `CodeEdge`, `CodeGraph` structs
  - [x] `who_calls()`, `what_depends_on()`, `impact_analysis()`
- [x] 11.3 Layer 2C: Validation State (File-based)
  - [x] `ProjectState`, `FileState`, `ViolationRecord`
  - [x] TOML persistence
- [x] 11.4 CLI command `aether memory`
- [x] 11.5 Unit tests

**Milestone:** Code Graph e Validation State implementati

### Phase 12: Pattern Discovery MVP ✅ COMPLETE (3 settimane, 42 ore)
- [x] 12.1 Definire `CodeFeatures` struct
- [x] 12.2 Implementare feature extraction (language-specific, rule-based)
- [x] 12.3 Implementare `AnomalyDetector`
- [x] 12.4 Implementare `RuleGenerator`
- [x] 12.5 CLI command `aether discover`
- [x] 12.6 Integration tests (18/18 passing)

**Milestone:** `aether discover src/` trova pattern e anomalie

### Phase 13: Intent Inference ✅ COMPLETE (3 settimane, 36 ore)
- [x] 13.1 Definire `Intent` struct
- [x] 13.2 Implementare prompt templates
- [x] 13.3 Implementare `IntentInferrer` (external API)
- [x] 13.4 Integration with validation
- [x] 13.5 CLI flag `--intent`
- [x] 13.6 Unit tests (23/23 passing)

**Milestone:** `aether validate --intent` usa LLM esterno per inferenza

### Phase 14: Drift Detection ✅ COMPLETE (2 settimane, 34 ore)
- [x] 14.1 Definire `DriftMetrics`
- [x] 14.2 Implementare metric extraction
- [x] 14.3 Implementare `GitIntegration`
- [x] 14.4 Implementare `TrendAnalyzer`
- [x] 14.5 CLI command `aether drift`
- [x] 14.6 Layer 2D: Drift Snapshots structure
- [x] 14.7 Integration tests (all tests passing)

**Milestone:** `aether drift --commits 50` mostra trend degrado

### Phase 15: Memory System Enhancement ✅ COMPLETE (3 settimane, 36 ore)
- [x] 15.1 Layer 2B: Decision Log (Knowledge Graph)
  - [x] `DecisionNode`, `DecisionEdge`, `DecisionLog` structs
  - [x] `why_exists()`, `is_accepted()`, `recall_semantic()`
  - [x] RAG integration per query semantiche
- [x] 15.2 Layer 2D: Drift Snapshots (Time-series)
  - [x] Time-series storage
  - [x] `analyze_trend()` con regression
  - [x] Alerting automatico
- [x] 15.3 API Unificata: `aether recall`
  - [x] `AetherIntelligence` struct
  - [x] `MemoryQuery` enum con 6 query types
- [x] 15.4 Integration con validation pipeline
- [x] 15.5 CLI command `aether memory recall`
  - [x] `why-exists` - Perché esiste questo codice?
  - [x] `is-accepted` - Questa violazione è accettata?
  - [x] `search` - Ricerca semantica
  - [x] `drift-trend` - Analisi trend drift
  - [x] `impact-analysis` - Analisi impatto modifiche
  - [x] `who-calls` - Chi chiama questa funzione?
- [x] 15.6 End-to-end tests (70 test passati)
- [x] 15.7 Documentation (RAG_SYSTEM.md, AETHER_INTELLIGENCE.md)
- [ ] 15.8 Release v0.2.0

**Milestone:** `aether memory recall why-exists src/main.rs:42` restituisce decisioni passate

### Phase 16: Memory-Driven Core (4 settimane) ✅ COMPLETE
- [x] 16.1 Architettura Memory-Driven Core — Documento MEMORY_DRIVEN_CORE.md
- [x] 16.2 LearnedConfig Implementation ✅ **VERIFICATO IN CODICE**
  - [x] `LearnedConfig` struct con thresholds, custom_rules, whitelist, conventions
  - [x] `MemoryStore::load_config()` integration
  - [x] `save_config()` per persistenza
  - [ ] Applicazione dinamica ai validation layers
- [x] 16.3 Feedback Loop ✅ **VERIFICATO IN CODICE**
  - [x] `record_feedback()` per ogni validazione
  - [x] `update_config_from_feedback()` in MemoryStore
  - [ ] Pattern discovery integration (parziale)
  - [ ] Auto-tuning thresholds
- [x] 16.4 Config file `.aether.toml` — Parsing e merge con LearnedConfig ✅
- [x] 16.5 CLI commands: `aether config`, `aether memory apply` ✅
- [x] 16.6 End-to-end tests ✅ **8 test passati**

**Codice verificato:** `crates/aether-intelligence/src/lib.rs` linee 300-400
- `load_config(&project_root) -> Result<LearnedConfig>`
- `save_config(&config) -> Result<()>`
- `record_feedback(&mut config, violations, accepted_ids) -> Result<()>`

**Milestone:** `aether validate` usa LearnedConfig per configurare i layers dinamicamente

### Phase 17: Integration & Polish (2 settimane)
- [ ] Unificare pipeline (5 layers)
- [ ] CLI refactoring
- [ ] Knowledge integration (Type Stubs + LLM)
- [ ] Parallel execution (Rayon)
- [ ] Memory optimization
- [ ] End-to-end tests
- [ ] Benchmark suite
- [ ] Release v0.3.0

**Milestone:** `aether validate` con tutti i 5 layer attivi + Memory-Driven

### Performance Targets

| Mode | Target | Layers |
|------|--------|--------|
| Fast | < 200ms | L1-3 (default) |
| Full (local LLM) | < 1s | L1-5 |
| Full (API) | < 3s | L1-5 + external |

---

## Priorità Correnti

### Alta Priorità
1. ~~**Phase 15: Memory Enhancement**~~ ✅ Complete
2. **Phase 16: Memory-Driven Core** — CLI commands (16.4-16.5)
3. **Phase 9: Polish & Launch** — Performance, docs, release

### Media Priorità (da ADR)
4. **ADR Phase 1.5: Bundled Presets** — TOML presets per linguaggi
5. **ADR Phase 2: Git Integration** — `aether hooks install`
6. **ADR Phase 3: Dubbioso Mode** — MCP questioning

### Bassa Priorità
7. **Multi-Language** — Test end-to-end
8. **Contracts Registry** — Sistema contratti automatici
9. **Phase 17: Integration** — Unificare pipeline 5 layers

### Completate ✅
- Phase 0-8: Tutte complete
- Phase 11-15: Tutte complete
- Phase 16.1-16.3: Architettura + LearnedConfig + Feedback Loop (verificati in codice)

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
│                                                                              │
│  ─── AETHER INTELLIGENCE (v0.2.0) ─────────────────────────────────────────  │
│                                                                              │
│  [✓] Phase 11: Memory Foundation   ───────────────────────────────────────  │
│  [✓] Phase 12: Pattern Discovery   ───────────────────────────────────────  │
│  [✓] Phase 13: Intent Inference    ───────────────────────────────────────  │
│  [✓] Phase 14: Drift Detection     ───────────────────────────────────────  │
│  [✓] Phase 15: Memory Enhancement  ───────────────────────────────────────  │
│  [⚡] Phase 16: Memory-Driven Core  ───────────────────────────────────────  │
│  [ ] Phase 17: Integration & Polish ───────────────────────────────────────  │
│                                                                              │
│  ─── ADR IMPLEMENTATION (autonomous-aether) ──────────────────────────────── │
│                                                                              │
│  [✓] ADR Phase 1: Core Autonomo    ───────────────────────────────────────  │
│  [ ] ADR Phase 1.5: Bundled Presets ───────────────────────────────────────  │
│  [x] ADR Phase 2: Git Integration  ───────────────────────────────────────  │
│  [✓] ADR Phase 3: Dubbioso Mode    ──── Phase 3+4 implemented ─────────  │
│  [✓] ADR Phase 4: CLI Wizard       ───────────────────────────────────────  │
│  [⚡] ADR Phase 5: Polish           ───────────────────────────────────────  │
│                                                                              │
│  Progress: 92% [████████████████████████████████████████████████████░░] 92% │
│  (Phase 0-9: 95% | Phase 11-17: 88% | ADR: 70% | Memory TOML: ❌)          │
│                                                                              │
│  ✓ = Complete   ⚡ = In Progress   ⚠ = Parziale   [ ] = Not Started         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Link Rapidi

| Documento | Descrizione | Stato |
|-----------|-------------|-------|
| [ADR_AUTONOMOUS_AETHER.md](./ADR_AUTONOMOUS_AETHER.md) | **14 decisioni architetturali Aether autonomo** | ✅ ADR |
| [MEMORY_DRIVEN_CORE.md](./MEMORY_DRIVEN_CORE.md) | **Memory-Driven Core Architecture** | 📋 Phase 16 |
| [AETHER_INTELLIGENCE.md](./AETHER_INTELLIGENCE.md) | AI-powered validation (5 layers) | ✅ Updated |
| [AETHER_ARCHITECTURE.md](./AETHER_ARCHITECTURE.md) | Architettura tecnica | ✅ Implementata |
| [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md) | Design complessivo | ✅ Updated |
| [AETHER_CERTIFICATION.md](./AETHER_CERTIFICATION.md) | Sistema certificazione codice | ✅ |
| [AETHER_CONTRACTS.md](./AETHER_CONTRACTS.md) | Contratti di validazione | ✅ |
| [AETHER_SECURITY.md](./AETHER_SECURITY.md) | Security model e policies | ✅ |
| [AETHER_RUST_IMPLEMENTATION.md](./AETHER_RUST_IMPLEMENTATION.md) | Dettagli implementazione Rust | ✅ |
| [USER_GUIDE.md](./USER_GUIDE.md) | Guida utente | ✅ |
| [API_REFERENCE.md](./API_REFERENCE.md) | Riferimento API | ✅ |
| [CONTRACTS_REGISTRY.md](./CONTRACTS_REGISTRY.md) | Sistema contratti automatici | 📋 Da implementare |
| [LANGUAGES.md](./LANGUAGES.md) | Linguaggi supportati (24) | ✅ |
| [LANGUAGE_SCALING_STRATEGY.md](./LANGUAGE_SCALING_STRATEGY.md) | Strategia scaling linguaggi | ✅ |
| [CUSTOM_LANGUAGE_SUPPORT.md](./CUSTOM_LANGUAGE_SUPPORT.md) | 3 livelli per linguaggi custom | ✅ |
| [aether-ui.md](./aether-ui.md) | VS Code Extension UI | ✅ Updated |
| [VSCODE_EXTENSION_PLAN.md](./VSCODE_EXTENSION_PLAN.md) | Piano sviluppo VS Code Extension | 📋 |
| [DUBBIOSO_MODE.md](./DUBBIOSO_MODE.md) | **Hyper-Context Engine + Confidence Intelligence** | 📋 Design |

## Documenti Consolidati

I seguenti documenti sono stati consolidati in **AETHER_INTELLIGENCE.md**:
- ~~AETHER_LEARNER.md~~ → Layer 2 (Semantic Memory)
- ~~AETHER_PRE_GUIDANCE.md~~ → Integrato nel workflow
- ~~AETHER_PROMPT_ANALYZER.md~~ → Integrato nel workflow
- ~~AETHER_RAG.md~~ → Layer 2 (Semantic Memory)

## Documenti Obsoleti (Eliminati 2026-03-18)

- ~~RAG_SYSTEM.md~~ → Sostituito da MEMORY_DRIVEN_CORE.md
- ~~PRIVATE_LAYERS_ARCHITECTURE.md~~ → Integrato in AETHER_ARCHITECTURE.md
- ~~INTELLIGENCE_IMPLEMENTATION.md~~ → Consolidato in AETHER_INTELLIGENCE.md
- ~~AETHER_INTEGRATION.md~~ → Consolidato in AETHER_ARCHITECTURE.md
- ~~AETHER_ROADMAP.md~~ → Consolidato in ROADMAP_INDEX.md

## ADR Implementation Status (ADR_AUTONOMOUS_AETHER.md)

Le seguenti fasi sono definite nell'ADR e devono essere tracciate nella roadmap:

| ADR Phase | Nome | Stato | Note |
|-----------|------|-------|------|
| 1 | Core Autonomo | ✅ | Validation layers, Graph RAG, Memory |
| 1.5 | Bundled Presets | ✅ | TOML presets + CLI commands |
| 2 | Git Integration | ❌ | pre-commit, post-commit hooks |
| 3 | Dubbioso Mode | ⚠️ Parziale | Confidence scoring, manca questioning via MCP |
| 4 | CLI Wizard | ✅ | `aether init` implementato |
| 5 | Polish | ⚡ | Documentazione, VS Code Extension |

### ADR Phase 1.5: Bundled Presets ✅ COMPLETE
- [x] Implementare formato TOML per bundled presets
- [x] Creare `~/.aether/presets/` directory con defaults (10 presets)
- [x] `PresetManager` per load/save/apply presets
- [x] `export_as_preset()` per clean export (no personal info)
- [x] `import_preset()` per import
- [x] CLI: `aether preset list` — Lista presets disponibili
- [x] CLI: `aether preset show <name>` — Dettagli preset
- [x] CLI: `aether preset apply <name>` — Applica al progetto
- [ ] CLI: `aether preset export` — Export current config as preset
- [ ] CLI: `aether preset import <file>` — Import preset from file

### ADR Phase 2: Git Integration ✅
- [x] `aether hooks install` — Installa git hooks nel progetto
- [x] pre-commit hook — Valida staged files (configurabile severity)
- [x] post-commit hook — Aggiorna memoria dopo commit
- [x] pre-push hook — Validazione completa (opzionale)
- [x] Rispetto `.gitignore` automatico

### ADR Phase 3: Dubbioso Mode ✅ Parziale (2026-03-25)
- [x] Confidence scoring nei risultati validazione
- [x] **Phase 3: Confidence filtering (25% threshold)** ✅ Implementato in executor.rs
- [x] **Phase 4: Test file filtering (LOGIC001/LOGIC002)** ✅ Implementato in executor.rs
- [ ] Hyper-Context Engine (Graph RAG + Semantic + Scoring)
- [ ] MCP questioning protocol
- [ ] Thresholds configurabili via TOML
- [ ] Feedback loop completo

**Documento dedicato:** [DUBBIOSO_MODE.md](./DUBBIOSO_MODE.md)

> **Nota:** Phase 3 (confidence filtering) e Phase 4 (test file filtering) sono ora attivi. 172 test passano.

---

## Prossimi Task Immediati

1. ~~**Phase 16.2: LearnedConfig**~~ ✅ Verificato in codice (`memory/mod.rs`)
2. ~~**Phase 16.3: Feedback Loop**~~ ✅ Verificato in codice (`lib.rs:record_feedback()`)
3. **Phase 16.4: Config File** — Implementare `.aether.toml` parsing e merge
4. **Phase 16.5: CLI Commands** — `aether config`, `aether memory apply`
5. **Memory TOML Implementation** — `~/.aether/` structure con `learned_patterns.toml`, `global_whitelist.toml`, `stats.toml`
6. **Memory → Core Loop** — Loop chiuso: validazione → memoria impara → core si adatta → validazione migliore
7. ~~**ADR Phase 1.5: Bundled Presets**~~ ✅ TOML presets per linguaggi comuni
8. ~~**ADR Phase 2: Git Hooks**~~ ✅ `aether hooks install` (hooks.rs completo)
9. **ADR Phase 3: Dubbioso Mode** — MCP questioning
10. **Tier Separation** — Creare crate aether-core e aether-pro
11. **RAG UI CLI** — Implementare `aether memory list/edit/delete`

---

## Future Evolution: Hyper-Context Engine (v2.1+)

**Obiettivo:** Aumentare accuratezza validazione tramite comprensione profonda del codice.

### Fusione di 3 approcci:

| Layer | Funzione | Tecnologia |
|-------|----------|------------|
| **Graph RAG Profondo** | Capisce CONTESTO: chi chiama, dipendenze, architettura | AST traversal multi-livello |
| **Semantic Analysis** | Capisce INTENTO: pattern, anti-pattern, "perché" | Tree-sitter queries |
| **Context Scoring** | Sa QUANDO dubitare: priorità, confidence dinamico | History + graph + semantic |

### Output finale:
```
Violazione: "unwrap() in prod"
├── Confidence: 87%
├── Contesto: chiamata da handle_request() → process() → unwrap()
├── Perché dubbia: 3 chiamate a monte, nessuna gestione errore
└── Domanda MCP: "Questa funzione può fallire? Gestisci l'errore altrove?"
```

### Architettura:
```
Graph RAG ─────┐
               ├──▶ Confidence Intelligence ──▶ Violazioni con % verità
Semantic ──────┤
               │
Scoring ───────┘
```

**Stato:** 📋 Design phase, non implementato
