# ADR: Dubbioso Mode — Confidence-Based Validation

**Status**: ✅ IMPLEMENTED
**Date**: 2026-03-19
**Phase**: ADR Phase 3

## Context

Synward validation traditionally uses strict rule-based checks. However, some violations require human judgment because:
- Context matters (code purpose, project conventions)
- False positives waste developer time
- Learning from user decisions improves accuracy over time

Dubbioso Mode adds confidence-based validation that:
1. Calculates confidence score for each violation
2. Asks user when confidence is low
3. Learns from responses to improve future decisions

## Implementation

### Task 1: Graph RAG Multi-Livello ✅
**File**: `crates/synward-intelligence/src/memory/code_graph/rag.rs`

Functions implemented:
- `file_dependencies_deep(file, depth)` — Recursively find all dependencies
- `file_dependents_deep(file, depth)` — Recursively find all dependents
- `find_call_chain(from, to)` — Find path between two functions
- `context_score(file)` — Calculate importance based on connections

### Task 2: Tree-sitter Semantic Analysis ✅
**File**: `crates/synward-intelligence/src/semantic.rs`

Pattern-based semantic analysis:
- `SemanticAnalyzer` — Analyzes code patterns without full tree-sitter parsing
- `SemanticContext` — Intent detection (data_flow, error_handling, initialization, etc.)
- `analyze_semantic_patterns(code, language)` — Extract semantic information

### Task 3: Context Scoring Algorithm ✅
**File**: `crates/synward-intelligence/src/dubbioso.rs`

Confidence calculation:
- `DubbiosoAnalyzer` — Combines graph + semantic + heuristics
- `ConfidenceResult` — confidence, level, uncertainty_reasons, questions
- `ConfidenceLevel` — AutoAccept (≥95%), Good (≥80%), Warn (≥60%), Ask (<60%)

Formula:
```
confidence = (graph_context * 0.3) + (semantic_confidence * 0.4) + (heuristics * 0.3)
```

### Task 4: MCP Question Protocol ✅
**File**: `crates/synward-intelligence/src/mcp_questions.rs`

Interactive questioning:
- `McpQuestionManager` — Creates and tracks questions
- `McpQuestion` — id, violation, context, suggested_actions, memory_impact
- `McpResponse` — question_id, answer, message
- `process_response()` — Updates memory based on answer

### Task 5: Threshold Configuration ✅
**File**: `crates/synward-intelligence/src/memory/project_config.rs`

`.synward.toml` configuration:
```toml
[dubbioso]
ask_threshold = 0.60      # Below this, ask user
warn_threshold = 0.80     # Below this, warn
auto_accept_threshold = 0.95  # Above this, auto-accept
permanent_after = 5       # Pattern becomes permanent after N accepts
max_context_depth = 10    # Max depth for graph traversal
```

### Task 6: Memory Pattern Persistence ✅
**File**: `crates/synward-intelligence/src/dubbioso_patterns.rs`

Pattern learning:
- `DubbiosoPattern` — id, pattern, language, accept/reject counts
- `DubbiosoPatternStore` — Load/save to `.synward/patterns.json`
- Pattern becomes permanent after `permanent_after` accepts
- Confidence adjustment: `+(accepts/total)*0.3` or `-(rejects/total)*0.3`

### Task 7: Feedback Loop Integration ✅
**File**: `crates/synward-intelligence/src/dubbioso_validator.rs`

Complete integration:
- `DubbiosoValidator` — Combines analyzer + question_manager + pattern_store
- `validate(violation)` — Check whitelist → permanent → analyze → classify
- `process_response(response)` — Update pattern store based on answer

## Flow

```
┌─────────────────┐
│  Code Violation │
└────────┬────────┘
         ▼
┌─────────────────┐
│ Whitelist Check │──Yes──► ACCEPT
└────────┬────────┘
         │ No
         ▼
┌─────────────────┐
│ Permanent Check │──Yes──► ACCEPT (learned)
└────────┬────────┘
         │ No
         ▼
┌─────────────────┐
│ Confidence Calc │
│ Graph + Semantic│
└────────┬────────┘
         ▼
    ┌────────┐
    │ Level? │
    └────┬───┘
    ┌────┼────┐
    ▼    ▼    ▼
 AutoAccept Warn Ask
    │    │    │
    ▼    ▼    ▼
 ACCEPT  ▼  Question
         │    │
         │    ▼
         │  Response
         │    │
         │    ▼
         │  Update Store
         │    │
         └────┴──► Next Violation
```

## Configuration

Default thresholds (`.synward.toml`):
```toml
[dubbioso]
ask_threshold = 0.60
warn_threshold = 0.80
auto_accept_threshold = 0.95
permanent_after = 5
max_context_depth = 10
```

## Files Created/Modified

| File | Type | Description |
|------|------|-------------|
| `dubbioso.rs` | Created | Confidence analyzer |
| `mcp_questions.rs` | Created | MCP question protocol |
| `dubbioso_patterns.rs` | Created | Pattern persistence |
| `dubbioso_validator.rs` | Created | Integration layer |
| `semantic.rs` | Modified | Pattern-based analysis |
| `project_config.rs` | Modified | DubbiosoSection |
| `.synward.toml` | Modified | [dubbioso] config |
| `lib.rs` | Modified | Module exports |

## Consequences

**Positive:**
- Reduces false positives by learning from user decisions
- Context-aware validation considers code purpose
- Persistent patterns don't require repeated questions

**Negative:**
- Initial learning period with more questions
- Pattern store requires disk space (~KB per pattern)
- Confidence calculation adds overhead to validation

## Future Enhancements

1. **Project-wide patterns** — Share patterns across team via Git
2. **Cloud sync** — Sync patterns across machines (Pro tier)
3. **LLM integration** — Use LLM for better confidence scoring
4. **Explanation generation** — Generate explanations for low-confidence decisions
