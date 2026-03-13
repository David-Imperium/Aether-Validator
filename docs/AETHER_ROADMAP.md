# Aether — Development Roadmap

**Version:** 0.1.0
**Last Updated:** 2026-03-10
**Implementation Language:** Rust
**Related:** [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md)

---

## Overview

This document provides a detailed development roadmap for Aether, organized by phases with specific milestones, deliverables, and time estimates.

---

## Phase Summary

| Phase | Name | Duration | Goal |
|-------|------|----------|------|
| **0** | Foundation | 2 weeks | Project setup, architecture |
| **1** | MVP | 4 weeks | Basic validation, CLI, C++ support |
| **2** | Validation | 3 weeks | Full 7-layer validation (Military Grade) |
| **3** | Certification | 2 weeks | Certificate generation, signing |
| **4** | Prompt Analysis | 3 weeks | Intent classifier, scope extraction |
| **5** | Integration | 3 weeks | HTTP API, MCP, SDKs |
| **6** | RAG | 2 weeks | Keyword index, semantic search |
| **7** | Learner | 2 weeks | User profile, pattern learning |
| **8** | Pre-Guidance | 2 weeks | Agent context generation, MCP hooks |
| **9** | Polish & Launch | 2 weeks | Testing, docs, deployment |

**Total Estimated Time:** ~25 weeks (6 months)

---

## Phase 0: Foundation (Weeks 1-2)

### Goal
Set up project infrastructure and validate architecture decisions.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M0.1 | Project structure | Directory layout, Cargo workspace |
| M0.2 | Core interfaces | Parser trait, ValidationLayer trait, Contract trait |
| M0.3 | Tree-sitter setup | Integrated tree-sitter-rust parser |
| M0.4 | Test framework | cargo test configured, first tests |
| M0.5 | CI/CD | GitHub Actions for build + test |

### Tasks

```
Week 1:
├── [ ] Create project structure
│   ├── crates/aether-core/
│   ├── crates/aether-parsers/
│   ├── crates/aether-validation/
│   ├── crates/aether-contracts/
│   ├── crates/aether-certification/
│   └── crates/aether-cli/
├── [ ] Cargo.toml for workspace
├── [ ] Define core traits
└── [ ] Set up tree-sitter Rust parser

Week 2:
├── [ ] Implement ParserRegistry
├── [ ] Basic AST traversal
├── [ ] cargo test integration
├── [ ] First unit tests
└── [ ] GitHub Actions workflow
```

### Exit Criteria
- [ ] Project builds on Windows + Linux
- [ ] Tests pass
- [ ] Tree-sitter parses Rust code to AST
- [ ] CI green

---

## Phase 1: MVP (Weeks 3-6)

### Goal
Working CLI that validates Rust code against basic contracts and generates certificates.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M1.1 | Orchestrator | Session management, pipeline execution |
| M1.2 | Contract Engine | YAML loading, pattern matching |
| M1.3 | Basic Contracts | 10 Rust contracts implemented |
| M1.4 | Validation Pipeline | Syntax + Semantic layers |
| M1.5 | Certificate Generator | Ed25519 signing, JSON output |
| M1.6 | CLI | `aether validate`, `aether certify` commands |

### Tasks

```
Week 3:
├── [ ] Implement Orchestrator
│   ├── SessionManager
│   ├── PipelineBuilder
│   └── StateTracker
└── [ ] Basic ValidationContext

Week 4:
├── [ ] Contract Engine
│   ├── ContractRegistry
│   ├── ContractLoader (YAML)
│   └── PatternMatcher (regex + AST)
├── [ ] 5 basic contracts:
│   ├── RUST001: no-unwrap-without-context
│   ├── RUST002: prefer-result-for-errors
│   ├── RUST003: no-clone-unnecessarily
│   ├── RUST004: use-unwrap-or-default
│   └── RUST005: prefer-borrow
└── [ ] Violation reporting

Week 5:
├── [ ] Validation Pipeline
│   ├── SyntaxLayer
│   └── SemanticLayer
├── [ ] 5 more contracts:
│   ├── RUST006: no-expect-without-context
│   ├── RUST007: prefer-slice-patterns
│   ├── RUST008: avoid-allocations-in-loops
│   ├── RUST009: use-iter-for-collection
│   └── RUST010: prefer-const-fn
└── [ ] Metrics calculation

Week 6:
├── [ ] Certificate Generator
│   ├── Certificate struct
│   ├── Signer (ed25519-dalek)
│   └── JSON serialization
├── [ ] CLI Implementation
│   ├── validate command
│   ├── certify command
│   └── JSON output
└── [ ] End-to-end test
```

### Contracts Implemented (Phase 1)

| ID | Name | Severity |
|----|------|----------|
| RUST001 | no-unwrap-without-context | warning |
| RUST002 | prefer-result-for-errors | error |
| RUST003 | no-clone-unnecessarily | warning |
| RUST004 | use-unwrap-or-default | info |
| RUST005 | prefer-borrow | info |
| RUST006 | no-expect-without-context | warning |
| RUST007 | prefer-slice-patterns | info |
| RUST008 | avoid-allocations-in-loops | warning |
| RUST009 | use-iter-for-collection | info |
| RUST010 | prefer-const-fn | info |

### Exit Criteria
- [ ] CLI validates Rust files
- [ ] 10 contracts working
- [ ] Certificates generated and signed
- [ ] E2E test passes

---

## Phase 2: Validation (Weeks 7-9)

### Goal
Complete 5-layer validation pipeline.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M2.1 | Logic Layer | Domain rules validation |
| M2.2 | Architecture Layer | Dependency analysis |
| M2.3 | Style Layer | Naming, formatting |
| M2.4 | Composite Patterns | AND, OR, NOT patterns |
| M2.5 | Violation Feedback | AI-friendly suggestions |

### Tasks

```
Week 7:
├── [ ] LogicLayer implementation
│   ├── Domain rule evaluation
│   └── Constraint checking
├── [ ] ArchitectureLayer implementation
│   ├── Dependency graph building
│   ├── Circular dependency detection
│   └── Layer boundary checking
└── [ ] Tests for both layers

Week 8:
├── [ ] StyleLayer implementation
│   ├── Naming convention checking
│   └── Formatting validation
├── [ ] Composite pattern support
│   ├── AndPattern
│   ├── OrPattern
│   └── NotPattern
└── [ ] Additional contracts (10 more)

Week 9:
├── [ ] Violation feedback system
│   ├── Suggestion generation
│   ├── Example fix generation
│   └── AI hint system
├── [ ] Full pipeline integration tests
└── [ ] Performance optimization
```

### Additional Contracts (Phase 2)

| ID | Name | Severity |
|----|------|----------|
| RUST011 | no-magic-numbers | info |
| RUST012 | avoid-unwrap-in-tests | warning |
| RUST013 | no-unsafe-in-lib | error |
| RUST014 | use-derive-where-possible | info |
| RUST015 | prefer-newtype | warning |
| RUST016 | avoid-deep-nesting | info |
| RUST017 | no-panic-in-lib | error |
| RUST018 | use-dbg-macro | info |
| RUST019 | prefer-enum-variants | info |
| RUST020 | avoid-large-enums | warning |

### Exit Criteria
- [ ] All 5 layers working
- [ ] 20 contracts total
- [ ] Feedback with AI hints
- [ ] Performance < 100ms per file

---

## Phase 3: Certification (Weeks 10-11)

### Goal
Full certification system with revocation.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M3.1 | Certificate Storage | File and database backends |
| M3.2 | Certificate Registry | Query and management |
| M3.3 | Revocation System | CRL generation and checking |
| M3.4 | Audit Logging | All operations logged |
| M3.5 | Verification API | Public verification endpoint |

### Tasks

```
Week 10:
├── [ ] CertificateStorage
│   ├── FileBackend
│   └── DatabaseBackend (SQLite)
├── [ ] CertificateRegistry
│   ├── Store/Query/Delete
│   └── Index by hash, file, date
└── [ ] Tests

Week 11:
├── [ ] Revocation system
│   ├── RevocationList (CRL)
│   ├── RevocationChecker
│   └── CLI command: aether revoke
├── [ ] Audit logging
│   ├── AuditLogger
│   └── AuditEntry storage
├── [ ] Verification API
│   └── CLI command: aether verify
└── [ ] Integration tests
```

### Exit Criteria
- [ ] Certificates stored and queryable
- [ ] Revocation working
- [ ] Audit logs complete
- [ ] Verification validates signature + revocation

---

## Phase 4: Dual-Track Validation (Weeks 12-14)

### Goal
Implement simplified validation architecture with Proxy (real-time notifications) + Standalone Validator (universal coverage).

### Market Context (CodeRabbit Report 2025)
- AI generates **1.7x more issues** overall
- Security issues **2.74x higher** in AI code
- **84%** developers use AI, but only **29%** trust it
- Market wants validation in workflow (CI/CD, PR review), not real-time blocking

### Architecture

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

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M4.1 | Proxy HTTP Enhancement | Code extraction from API responses |
| M4.2 | Desktop Notifications | System notifications for validation issues |
| M4.3 | Standalone Validator Polish | CLI, Desktop App, CI/CD integration |
| M4.4 | Git Hooks | pre-commit validation hook |
| M4.5 | RAG Integration | Learning from corrections |

### Compatibility Matrix

**Proxy (Real-time):**
- ✅ Droid, Claude Code, Cursor (configurable via env vars)
- ❌ Ollama local, GitHub Copilot (proprietary channels)

**Standalone Validator:**
- ✅ All AI agents (universal)
- ✅ CI/CD pipelines
- ✅ Git hooks
- ✅ Manual validation

### Tasks

```
Week 12:
├── [ ] Proxy HTTP Enhancement
│   ├── Extract code from API responses
│   ├── Support multiple response formats
│   └── Background validation queue
├── [ ] Desktop Notifications
│   ├── Windows toast notifications
│   ├── macOS notifications
│   └── Linux libnotify
└── [ ] Tests

Week 13:
├── [ ] Standalone Validator Polish
│   ├── CLI improvements
│   ├── Desktop App UI refinements
│   └── Validation report export
├── [ ] CI/CD Integration
│   ├── GitHub Action
│   ├── GitLab CI template
│   └── Jenkins plugin
└── [ ] Tests

Week 14:
├── [ ] Git Hooks
│   ├── pre-commit hook
│   ├── pre-push hook
│   └── Hook installation script
├── [ ] RAG Integration
│   ├── Store validation patterns
│   ├── Learn from corrections
│   └── Improve suggestions
├── [ ] End-to-end tests
└── [ ] Documentation
```

### Exit Criteria
- [ ] Proxy intercepts and validates compatible agents
- [ ] Desktop notifications show validation issues
- [ ] Standalone validator works with all agents
- [ ] Git hooks prevent invalid commits
- [ ] RAG improves validation over time

---

## Phase 5: Integration (Weeks 15-17)

### Goal
Full integration with external systems.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M5.1 | HTTP API | REST endpoints |
| M5.2 | MCP Server | Model Context Protocol |
| M5.3 | Python SDK | Client library |
| M5.4 | Rust Adapter | Second language support |
| M5.5 | Lex Adapter | Third language support |

### Tasks

```
Week 15:
├── [ ] HTTP API server
│   ├── /api/v1/validate
│   ├── /api/v1/certify
│   ├── /api/v1/verify
│   ├── /api/v1/analyze
│   └── Authentication
├── [ ] API documentation (OpenAPI)
└── [ ] Tests

Week 16:
├── [ ] MCP Server
│   ├── Tool definitions
│   ├── aether_validate tool
│   ├── aether_certify tool
│   └── aether_analyze tool
├── [ ] Python SDK
│   ├── AetherClient class
│   ├── validate(), certify(), analyze()
│   └── IterationManager
└── [ ] Tests

Week 17:
├── [ ] Rust adapter
│   ├── Tree-sitter Rust parser
│   ├── 10 Rust contracts
│   └── Integration tests
├── [ ] Lex adapter
│   ├── Lex parser (from Aegis)
│   ├── 10 Lex contracts
│   └── Integration tests
└── [ ] Full integration tests
```

### Rust Contracts (Phase 5)

| ID | Name | Severity |
|----|------|----------|
| RUST001 | no-unwrap | warning |
| RUST002 | no-expect-without-context | warning |
| RUST003 | prefer-borrow | info |
| RUST004 | no-clone-unnecessarily | warning |
| RUST005 | use-result-for-errors | error |

### Lex Contracts (Phase 5)

| ID | Name | Severity |
|----|------|----------|
| LEX001 | entity-requires-faction | error |
| LEX002 | health-positive | error |
| LEX003 | valid-reference | error |
| LEX004 | no-circular-dependencies | error |
| LEX005 | valid-type-annotation | warning |

### Exit Criteria
- [ ] HTTP API working
- [ ] MCP server integrates with AI agents
- [ ] Python SDK usable
- [ ] Rust and Lex adapters working

---

## Phase 6: RAG (Weeks 18-19)

### Goal
Hybrid RAG system for documentation search.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M6.1 | Keyword Index | TF-IDF/BM25 search |
| M6.2 | Semantic Search | Cached embeddings search |
| M6.3 | Pattern Library | Learned patterns storage |
| M6.4 | Index CLI | `aether index` command |
| M6.5 | Search API | `/api/v1/search` endpoint |

### Tasks

```
Week 18:
├── [ ] KeywordIndex implementation
│   ├── TF-IDF calculation
│   ├── BM25 ranking
│   └── Index persistence
├── [ ] Document indexer
│   ├── Markdown parser
│   ├── Code extractor
│   └── Config parser
└── [ ] Tests

Week 19:
├── [ ] SemanticSearch implementation
│   ├── Local embedding model
│   ├── Embedding cache
│   └── Cosine similarity
├── [ ] PatternLibrary
│   ├── Pattern storage
│   ├── Frequency tracking
│   └── Relevance scoring
├── [ ] CLI command: aether index
├── [ ] CLI command: aether search
└── [ ] Integration tests
```

### Exit Criteria
- [ ] Keyword search < 10ms
- [ ] Semantic search < 50ms (cached)
- [ ] Documents indexed from project
- [ ] CLI commands working

---

## Phase 7: Learner (Weeks 20-21)

### Goal
User learning system for personalized guidance.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M7.1 | User Profile | Preferences, statistics |
| M7.2 | Stats Tracker | Validation statistics |
| M7.3 | Memory Store | Corrections, lessons |
| M7.4 | Pattern Extractor | Code pattern extraction |
| M7.5 | Learning API | `/api/v1/learn` endpoint |

### Tasks

```
Week 20:
├── [ ] UserProfile implementation
│   ├── Language statistics
│   ├── Violation records
│   └── Code style preferences
├── [ ] StatsTracker
│   ├── Validation tracking
│   ├── Error prediction
│   └── Success rate calculation
└── [ ] Tests

Week 21:
├── [ ] MemoryStore
│   ├── Memory types
│   ├── Persistence
│   └── Search
├── [ ] PatternExtractor
│   ├── Correction analysis
│   ├── Style extraction
│   └── Frequency tracking
├── [ ] CLI command: aether learn
└── [ ] Integration tests
```

### Exit Criteria
- [ ] User profile persists
- [ ] Violations tracked per user
- [ ] Patterns extracted from corrections
- [ ] Guidance hints generated

---

## Phase 8: Pre-Guidance (Weeks 22-23)

### Goal
Preventive context system for agents.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M8.1 | PreGuidance Core | Context generation |
| M8.2 | MCP Hook | Agent response interception |
| M8.3 | PreToolUse Hook | File write validation |
| M8.4 | Context Formatter | Agent-friendly output |
| M8.5 | Pre-Guidance API | `/api/v1/guide` endpoint |

### Tasks

```
Week 22:
├── [ ] PreGuidance implementation
│   ├── Prompt analysis integration
│   ├── RAG integration
│   ├── Learner integration
│   └── Context assembly
├── [ ] ContextFormatter
│   ├── Markdown output
│   ├── Warning prioritization
│   └── Source references
└── [ ] Tests

Week 23:
├── [ ] MCP Hook
│   ├── Pre-process hook
│   ├── Context injection
│   └── Metadata tracking
├── [ ] PreToolUse Hook
│   ├── File write interception
│   ├── Validation blocking
│   └── Error reporting
├── [ ] CLI command: aether guide
├── [ ] MCP tool: aether_pre_guidance
└── [ ] End-to-end tests
```

### Exit Criteria
- [ ] Pre-Guidance generates context < 70ms
- [ ] MCP Hook intercepts agent requests
- [ ] PreToolUse blocks invalid writes
- [ ] Iteration reduction measurable

---

## Phase 9: Polish & Launch (Weeks 24-25)

### Goal
Production-ready release.

### Milestones

| Milestone | Description | Deliverable |
|-----------|-------------|-------------|
| M6.1 | Performance Testing | Benchmarks met |
| M6.2 | Security Review | Vulnerabilities fixed |
| M6.3 | Documentation | User docs complete |
| M6.4 | Website | aethercloud.dev live |
| M6.5 | Release | v1.0.0 binaries published |

### Tasks

```
Week 18:
├── [ ] Performance testing
│   ├── Benchmark suite
│   ├── Profiling
│   └── Optimization
├── [ ] Security review
│   ├── Dependency audit
│   ├── Code review
│   └── Penetration testing
└── [ ] Bug fixes

Week 19:
├── [ ] Documentation
│   ├── User guide
│   ├── API reference
│   ├── Examples
│   └── FAQ
├── [ ] Website setup
│   ├── Landing page
│   ├── Pricing page
│   ├── Download page
│   └── Blog (first post)
├── [ ] Release preparation
│   ├── Version tagging
│   ├── Binary builds (Win, Linux, Mac)
│   ├── Docker image
│   └── Package manager submissions
└── [ ] v1.0.0 release
```

### Exit Criteria
- [ ] All performance targets met
- [ ] No known security vulnerabilities
- [ ] Documentation complete
- [ ] Website live
- [ ] Binaries downloadable

---

## Release Schedule

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AETHER RELEASE SCHEDULE                           │
│                                                                             │
│  2026 Q1 (Mar-Apr)         2026 Q2 (May-Jun)        2026 Q3 (Jul-Sep)     │
│  ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐    │
│  │ Phase 0: Found. │      │ Phase 2: Valid. │      │ Phase 4: Prompt │    │
│  │ Phase 1: MVP    │      │ Phase 3: Cert.  │      │ Phase 5: Integ. │    │
│  │                 │      │                 │      │                 │    │
│  │ v0.1.0-alpha    │─────▶│ v0.3.0-beta    │─────▶│ v0.5.0-rc       │    │
│  └─────────────────┘      └─────────────────┘      └─────────────────┘    │
│                                        │                                   │
│                                        ▼                                   │
│  2026 Q4 (Oct-Nov)                 ┌─────────────────┐                    │
│  ┌─────────────────┐               │ Phase 6: RAG    │                    │
│  │ Phase 8: Pre-   │               │ Phase 7: Learner│                    │
│  │ Guidance        │◀──────────────│                 │                    │
│  │                 │               │ v0.7.0-rc       │                    │
│  │ v0.9.0-rc       │               └─────────────────┘                    │
│  └────────┬────────┘                                                      │
│           │                                                                │
│           ▼                                                                │
│  2026 Q4 (Dec)                                                             │
│  ┌─────────────────┐                                                      │
│  │ Phase 9: Polish │                                                      │
│  │                 │                                                      │
│  │ v1.0.0 RELEASE  │                                                      │
│  └─────────────────┘                                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Release Versions

| Version | Phase | Description |
|---------|-------|-------------|
| **v0.1.0-alpha** | Phase 1 | MVP, internal testing |
| **v0.2.0-alpha** | Phase 2 mid | Full 7-layer validation |
| **v0.3.0-beta** | Phase 3 end | Certification complete |
| **v0.4.0-beta** | Phase 4 mid | Prompt analysis |
| **v0.5.0-rc** | Phase 5 end | Integration complete (HTTP, MCP, SDK) |
| **v0.6.0-rc** | Phase 6 end | RAG system |
| **v0.7.0-rc** | Phase 7 end | Learner system |
| **v0.9.0-rc** | Phase 8 end | Pre-Guidance system |
| **v1.0.0** | Phase 9 end | First stable release |

---

## Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| tree-sitter-rust issues | Medium | High | Use syn as backup parser |
| Ed25519 complexity | Low | Medium | Use ed25519-dalek crate |
| Performance issues | Medium | Medium | Profile early, optimize hot paths |
| API design changes | Medium | Low | Version API from start |
| Scope creep | High | High | Stick to roadmap, defer features |

---

## Success Metrics

### Technical Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Validation latency | < 100ms | Benchmark suite |
| Memory usage | < 50MB | Profiling |
| Test coverage | > 80% | Code coverage tools |
| False positive rate | < 5% | Manual review |

### Business Metrics (Post-Launch)

| Metric | 6-month Target |
|--------|----------------|
| Downloads | 5,000 |
| Registered users | 500 |
| Paying customers | 50 |
| MRR | $5,000 |

---

## Dependencies

| Dependency | Purpose | Phase Required |
|------------|---------|----------------|
| tree-sitter | Parsing | Phase 0 |
| syn | Rust AST parsing | Phase 0 |
| nom | Parser combinators | Phase 0 |
| serde | Serialization | Phase 0 |
| serde_json | JSON handling | Phase 0 |
| serde_yaml | Config | Phase 1 |
| ed25519-dalek | Signing | Phase 1 |
| rusqlite | Storage | Phase 3 |
| axum/actix-web | HTTP API | Phase 5 |
| pyo3 | Python SDK | Phase 5 |

---

## Conclusion

This roadmap provides a clear path from foundation to v1.0.0 release. The phased approach allows for:

1. **Early value delivery** — MVP usable after Phase 1
2. **Risk reduction** — Core components built first
3. **Flexibility** — Can adjust based on feedback
4. **Quality focus** — Dedicated polish phase before launch
5. **Pre-Guidance advantage** — Unique value proposition with proactive guidance

### Key Differentiators

| Feature | Traditional Validators | Aether |
|---------|------------------------|--------|
| Validation | Reactive (after code) | Proactive (before code) |
| Learning | None | Learns from user patterns |
| Context | None | RAG-powered documentation |
| Iterations | 2-3 average | 1-1.5 average |
| User-specific | No | Personalized guidance |

**Estimated release:** December 2026
