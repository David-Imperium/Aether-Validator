# Aether MCP Tools Reference

**Version:** 0.1.0
**Protocol:** MCP 2024-11-05
**Tools Count:** 27

---

## Tool Overview

| Tool | Description |
|------|-------------|
| `get_version` | Get Aether version and capabilities |
| `list_languages` | List all supported languages |
| `list_contracts` | List available validation contracts |
| `get_language_info` | Get info for a specific language |
| `analyze_code` | Analyze code structure and AST |
| `get_metrics` | Calculate code metrics |
| `validate_file` | Validate a source file |
| `batch_validate` | Validate multiple files |
| `suggest_fixes` | AI-powered fix suggestions |
| `certify_code` | Cryptographic code certification |
| `build_graph` | Build dependency graph from directory |
| `who_calls` | Find callers of a function |
| `impact_analysis` | Analyze impact of modifications |
| `file_dependencies` | Get dependencies of a file |
| `file_dependents` | Get files that depend on this file |
| `get_context` | Get complete function context |
| `find_call_chain` | Find call chain between two functions |
| `memory_recall` | Search for similar patterns in memory |
| `memory_store` | Store new pattern in memory |
| `save_state` | Save validation state |
| `load_state` | Load validation state |
| `accept_violation` | Accept violation with justification |
| `analyze_scope` | Analyze variable scopes and detect shadowing/unused variables |
| `infer_types` | Infer types for variables and expressions |
| `get_confidence` | Get confidence score for code quality |
| `watch_start` | Start directory watch |
| `watch_check` | Check for file changes |
| `watch_stop` | Stop directory watch |

---

## Tool Details

### get_version

Returns Aether version, supported languages count, and tools count.

**Input Schema:**
```json
{}
```

**Response:**
```json
{
  "version": "0.1.0",
  "name": "Aether",
  "languages_count": 24,
  "tools_count": 24
}
```

---

### list_languages

Lists all supported programming languages with extensions.

**Input Schema:**
```json
{}
```

**Response:**
```json
[
  {
    "language": "rust",
    "extensions": [".rs"],
    "supported": true,
    "features": ["parsing", "validation", "analysis"]
  },
  // ... 22 more
]
```

---

### list_contracts

Lists available validation contracts.

**Input Schema:**
```json
{}
```

**Response:**
```json
{
  "contracts": [
    {
      "name": "no_unsafe",
      "category": "security",
      "description": "No unsafe code blocks"
    },
    {
      "name": "no_panic",
      "category": "reliability",
      "description": "No panic or unwrap"
    },
    {
      "name": "documentation",
      "category": "style",
      "description": "Public items must have docs"
    },
    {
      "name": "complexity",
      "category": "maintainability",
      "description": "Limit function complexity"
    },
    {
      "name": "naming",
      "category": "style",
      "description": "Follow naming conventions"
    }
  ]
}
```

---

### get_language_info

Returns supported features for a specific language.

**Input Schema:**
```json
{
  "language": "string (required)"
}
```

**Response:**
```json
{
  "language": "rust",
  "extensions": [".rs"],
  "supported": true,
  "features": ["parsing", "validation", "analysis"],
  "tree_sitter": true
}
```

---

### analyze_code

Analyzes code structure and returns AST statistics.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)"
}
```

**Response:**
```json
{
  "language": "rust",
  "total_nodes": 15,
  "node_types": [
    {"node_type": "Function", "count": 2},
    {"node_type": "Let", "count": 5},
    {"node_type": "Call", "count": 3}
  ],
  "max_depth": 3
}
```

---

### get_metrics

Calculates code metrics including complexity.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)"
}
```

**Response:**
```json
{
  "language": "rust",
  "lines_of_code": 42,
  "blank_lines": 8,
  "comment_lines": 5,
  "total_nodes": 120,
  "max_depth": 4,
  "functions": 5,
  "classes": 0,
  "complexity_estimate": 8
}
```

---

### validate_file

Validates a source file against all validation layers.

**Input Schema:**
```json
{
  "file_path": "string (required)",
  "language": "string | null (optional, auto-detected)",
  "contracts": "string | null (optional)",
  "dubbioso_mode": "boolean | null (optional)"
}
```

**Response:**
```json
{
  "passed": true,
  "errors": [],
  "warnings": [],
  "language": "rust",
  "layers": {
    "syntax": true,
    "semantic": true,
    "logic": true,
    "security": true,
    "contracts": true,
    "style": true
  }
}
```

---

### batch_validate

Validates multiple files with progress reporting.

**Input Schema:**
```json
{
  "file_paths": ["string"] (required),
  "contracts": "string | null (optional)"
}
```

**Response:**
```json
{
  "total": 10,
  "passed": 8,
  "failed": 2,
  "results": [
    {"file": "src/main.rs", "passed": true},
    {"file": "src/lib.rs", "passed": false, "errors": 3}
  ]
}
```

---

### suggest_fixes

Generates AI-powered fix suggestions using MCP sampling.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)",
  "errors": ["string"] (required)
}
```

**Response:**
```json
{
  "suggestions": [
    {
      "error": "SYNTAX001",
      "suggestion": "Add semicolon at line 5",
      "confidence": 0.95
    }
  ]
}
```

---

### certify_code

Generates cryptographic certificate (Ed25519) for validated code.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)",
  "signer": "string (required)",
  "contracts": ["string"] (optional, default [])
}
```

**Response:**
```json
{
  "certificate": {
    "hash": "sha256:...",
    "signature": "ed25519:...",
    "signer": "Developer",
    "timestamp": "2026-03-15T12:00:00Z",
    "contracts": ["no_unsafe"]
  }
}
```

---

### build_graph

Builds a dependency graph from a directory.

**Input Schema:**
```json
{
  "directory": "string (required)",
  "extensions": "string | null (optional)"
}
```

**Response:**
```json
{
  "nodes": [
    {"id": "src/main.rs", "type": "file"},
    {"id": "src/lib.rs", "type": "file"}
  ],
  "edges": [
    {"from": "src/main.rs", "to": "src/lib.rs"}
  ],
  "stats": {
    "total_files": 15,
    "total_dependencies": 23
  }
}
```

---

### who_calls

Finds all callers of a function.

**Input Schema:**
```json
{
  "function": "string (required)",
  "file": "string | null (optional)"
}
```

**Response:**
```json
{
  "function": "parse_config",
  "callers": [
    {"file": "src/main.rs", "line": 42, "caller": "main"},
    {"file": "src/cli.rs", "line": 15, "caller": "run_cli"}
  ]
}
```

---

### impact_analysis

Analyzes the impact of modifying a function.

**Input Schema:**
```json
{
  "function": "string (required)",
  "file": "string | null (optional)"
}
```

**Response:**
```json
{
  "function": "parse_config",
  "direct_callers": 3,
  "transitive_callers": 12,
  "affected_files": [
    "src/main.rs",
    "src/cli.rs",
    "src/config.rs"
  ],
  "risk_level": "medium"
}
```

---

### file_dependencies

Returns all dependencies of a file.

**Input Schema:**
```json
{
  "file": "string (required)",
  "max_depth": "integer | null (optional)"
}
```

**Response:**
```json
{
  "file": "src/main.rs",
  "dependencies": [
    {"file": "src/lib.rs", "depth": 1},
    {"file": "src/config.rs", "depth": 2}
  ],
  "max_depth": 2
}
```

---

### file_dependents

Returns all files that depend on this file.

**Input Schema:**
```json
{
  "file": "string (required)",
  "max_depth": "integer | null (optional)"
}
```

**Response:**
```json
{
  "file": "src/lib.rs",
  "dependents": [
    {"file": "src/main.rs", "depth": 1},
    {"file": "src/cli.rs", "depth": 1}
  ],
  "max_depth": 1
}
```

---

### get_context

Returns complete context for a function including callers and callees.

**Input Schema:**
```json
{
  "function": "string (required)",
  "file": "string | null (optional)",
  "max_depth": "integer | null (optional)"
}
```

**Response:**
```json
{
  "function": "parse_config",
  "file": "src/config.rs",
  "callers": [
    {"function": "main", "file": "src/main.rs", "line": 42}
  ],
  "callees": [
    {"function": "read_file", "file": "src/utils.rs", "line": 10}
  ],
  "context_depth": 2
}
```

---

### find_call_chain

Finds a call chain between two functions.

**Input Schema:**
```json
{
  "from_function": "string (required)",
  "from_file": "string (required)",
  "to_function": "string (required)",
  "to_file": "string (required)"
}
```

**Response:**
```json
{
  "found": true,
  "chain": [
    {"function": "main", "file": "src/main.rs"},
    {"function": "process_input", "file": "src/process.rs"},
    {"function": "parse_config", "file": "src/config.rs"}
  ],
  "length": 3
}
```

---

### memory_recall

Searches for similar patterns in memory.

**Input Schema:**
```json
{
  "query": "string (required)",
  "limit": "integer | null (optional)"
}
```

**Response:**
```json
{
  "results": [
    {
      "code": "fn parse_config() -> Result<Config> {...}",
      "language": "rust",
      "similarity": 0.85,
      "tags": ["config", "parsing"]
    }
  ],
  "total": 5
}
```

---

### memory_store

Stores a new pattern in memory.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)",
  "memory_type": "string | null (optional)",
  "tags": ["string"] (optional)"
}
```

**Response:**
```json
{
  "stored": true,
  "id": "mem_abc123",
  "timestamp": "2026-03-15T12:00:00Z"
}
```

---

### save_state

Saves validation state for a project.

**Input Schema:**
```json
{
  "project_root": "string (required)"
}
```

**Response:**
```json
{
  "saved": true,
  "project_root": "/path/to/project",
  "violations_count": 5,
  "timestamp": "2026-03-15T12:00:00Z"
}
```

---

### load_state

Loads validation state for a project.

**Input Schema:**
```json
{
  "project_root": "string (required)"
}
```

**Response:**
```json
{
  "loaded": true,
  "project_root": "/path/to/project",
  "violations": [
    {"id": "v001", "file": "src/main.rs", "status": "accepted"}
  ],
  "timestamp": "2026-03-15T12:00:00Z"
}
```

---

### accept_violation

Accepts a violation with justification.

**Input Schema:**
```json
{
  "project_root": "string (required)",
  "violation_id": "string (required)",
  "reason": "string (required)",
  "file": "string | null (optional)",
  "line": "integer | null (optional)"
}
```

**Response:**
```json
{
  "accepted": true,
  "violation_id": "v001",
  "reason": "Legacy code, will refactor in Q2",
  "timestamp": "2026-03-15T12:00:00Z"
}
```

---

### watch_start

Starts watching a directory for file changes.

**Input Schema:**
```json
{
  "directory": "string (required)",
  "extensions": "string | null (optional)"
}
```

**Response:**
```json
{
  "watch_id": 1,
  "directory": "./src",
  "extensions": ["rs"]
}
```

---

### watch_check

Checks for file changes since last check.

**Input Schema:**
```json
{
  "watch_id": "integer (required, min 0)"
}
```

**Response:**
```json
{
  "watch_id": 1,
  "changed_files": ["src/main.rs", "src/lib.rs"],
  "deleted_files": ["src/old.rs"]
}
```

---

### watch_stop

Stops watching a directory.

**Input Schema:**
```json
{
  "watch_id": "integer (required, min 0)"
}
```

**Response:**
```json
{
  "watch_id": 1,
  "stopped": true
}
```

---

## MCP Protocol Requirements

### Initialize Request

Clients MUST include `clientInfo` in the initialize request:

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "client-name",
      "version": "1.0.0"
    }
  },
  "id": 1
}
```

### Server Response

The server responds with:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {},
      "resources": {},
      "prompts": {}
    },
    "serverInfo": {
      "name": "aether-mcp",
      "version": "1.2.0"
    },
    "instructions": "Aether MCP Server - Code validation, analysis, and certification..."
  }
}
```

### Initialized Notification

After receiving the initialize response, clients MUST send:

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized",
  "params": {}
}
```

---

### analyze_scope

Analyzes variable scopes and detects shadowing/unused variables.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)"
}
```

**Response:**
```json
{
  "scope_count": 3,
  "symbols": [
    {
      "name": "x",
      "kind": "variable",
      "scope_path": "global",
      "line": 0
    }
  ],
  "unused_variables": [],
  "shadowing": []
}
```

---

### infer_types

Infers types for variables and expressions without annotations.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)"
}
```

**Response:**
```json
{
  "types": {
    "string_expr": "String",
    "int_expr": "Integer",
    "bool_expr": "Boolean"
  },
  "errors": [],
  "violations": []
}
```

---

### get_confidence

Gets confidence score for code quality and generates clarifying questions.

**Input Schema:**
```json
{
  "code": "string (required)",
  "language": "string (required)",
  "violations": "array (optional)"
}
```

**Response:**
```json
{
  "confidence": 0.85,
  "level": "Good",
  "questions": []
}
```

---

## Error Handling

Tools return errors in the standard MCP format:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Missing required field: file_path"
  }
}
```

---

## Supported Languages (24)

| Language | Extensions |
|----------|------------|
| Rust | .rs |
| Python | .py, .pyw |
| JavaScript | .js, .jsx, .mjs, .cjs |
| TypeScript | .ts, .tsx, .mts, .cts |
| C | .c, .h |
| C++ | .cpp, .cc, .cxx, .hpp, .hxx |
| CUDA | .cu, .cuh |
| Go | .go |
| Java | .java |
| Lua | .lua |
| Bash | .sh, .bash, .zsh, .ksh |
| Lex | .lex |
| Prism | .prism |
| GLSL | .frag, .vert, .comp, .glsl |
| CSS | .css |
| HTML | .html, .htm |
| JSON | .json |
| YAML | .yaml, .yml |
| TOML | .toml |
| CMake | .cmake |
| SQL | .sql, .ddl, .dml |
| GraphQL | .graphql, .gql |
| Markdown | .md, .markdown, .mdown, .mkd |
| Notebook | .ipynb |
