# Aether — Agent Integration

**Version:** 0.1.0
**Implementation Language:** Rust
**Related:** [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md), [AETHER_RUST_IMPLEMENTATION.md](./AETHER_RUST_IMPLEMENTATION.md)

---

## Overview

This document describes how external AI agents integrate with Aether. The goal is seamless integration where any AI coding assistant can use Aether to validate and certify its output.

### Phase 4: Dual-Track Integration (2026)

Based on market research (CodeRabbit Report 2025):
- AI generates **1.7x more issues** overall
- Security issues **2.74x higher** in AI code
- **84%** developers use AI, but only **29%** trust it

**Key Insight:** Market wants validation in workflow, not real-time blocking.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AETHER INTEGRATION (Phase 4 Architecture)                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          DUAL-TRACK SYSTEM                           │   │
│  │                                                                       │   │
│  │  ┌───────────────────────────┐   ┌─────────────────────────────────┐ │   │
│  │  │   PROXY HTTP (Real-time)  │   │   VALIDATORE STANDALONE         │ │   │
│  │  │                           │   │                                 │ │   │
│  │  │   • Intercepta API        │   │   • CLI, Desktop, CI/CD        │ │   │
│  │  │   • Estrae codice         │   │   • Git hooks (pre-commit)     │ │   │
│  │  │   • Valida background     │   │   • UNIVERSALE (tutti agenti)  │ │   │
│  │  │   • Notifiche desktop     │   │                                 │ │   │
│  │  │                           │   │   Compatibilita':               │ │   │
│  │  │   Compatibilita':         │   │   [x] Tutti gli agenti AI      │ │   │
│  │  │   [x] Droid               │   │   [x] Ollama, Copilot          │ │   │
│  │  │   [x] Claude Code         │   │   [x] CI/CD, Git hooks         │ │   │
│  │  │   [x] Cursor              │   │                                 │ │   │
│  │  │   [ ] Ollama, Copilot     │   │                                 │ │   │
│  │  └───────────────────────────┘   └─────────────────────────────────┘ │   │
│  │                                                                       │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐ │   │
│  │  │                     RAG (Apprendimento)                          │ │   │
│  │  │   • Pattern, errori, correzioni -> Migliora nel tempo           │ │   │
│  │  └─────────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AI AGENTS                                         │
│                                                                             │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐    │
│  │  Claude   │ │   GPT-4   │ │  Cursor   │ │  Copilot  │ │  Custom   │    │
│  │   (API)   │ │   (API)   │ │  (IDE)    │ │  (IDE)    │ │  Agents   │    │
│  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └─────┬─────┘    │
│        │             │             │             │             │           │
└────────┼─────────────┼─────────────┼─────────────┼─────────────┼───────────┘
         │             │             │             │             │
         └─────────────┴─────────────┴──────┬──────┴─────────────┘
                                             │
                                             ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AETHER INTERFACES                                    │
│                                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │   CLI       │ │  HTTP API   │ │  LSP        │ │    MCP Server       │   │
│  │             │ │             │ │             │ │                     │   │
│  │ aether val  │ │ POST /api/  │ │ textDocument│ │ aether_validate     │   │
│  │ aether cert │ │             │ │ /diagnostic │ │ aether_certify      │   │
│  └─────────────┘ └─────────────┘ └─────────────┘ │ aether_analyze      │   │
│                                                   └─────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                             │
                                             ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          AETHER CORE                                        │
│                                                                             │
│   Orchestrator │ Validation Engine │ Contract Engine │ Certification       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Integration Methods

### 1. CLI Integration

The simplest integration method — agents execute Aether as a subprocess.

```bash
# Validate a file
aether validate src/enemy.cpp --output json

# Validate with contracts
aether validate src/enemy.cpp --contracts ./contracts/ --output json

# Certify code
aether certify src/enemy.cpp --output cert.json

# Analyze prompt
aether analyze-prompt "Add enemy patrol" --context ./project/
```

#### CLI Output Format

```json
{
  "status": "success",
  "passed": true,
  "certificate_id": "AETHER-2026-03-08-ABC12345",
  "metrics": {
    "errors": 0,
    "warnings": 2,
    "score": 92
  },
  "violations": [],
  "duration_ms": 45
}
```

#### Agent Integration Example (Python)

```python
import subprocess
import json

def validate_with_aether(code: str, language: str = "cpp") -> dict:
    """Validate code using Aether CLI."""
    
    result = subprocess.run(
        ["aether", "validate", "--stdin", "--language", language, "--output", "json"],
        input=code,
        capture_output=True,
        text=True
    )
    
    return json.loads(result.stdout)

def validate_and_iterate(code: str, agent_fix_fn, max_iterations: int = 3) -> tuple[str, dict]:
    """Validate code and iterate with AI agent until it passes."""
    
    for i in range(max_iterations):
        result = validate_with_aether(code)
        
        if result["passed"]:
            return code, result
            
        # Ask agent to fix based on violations
        code = agent_fix_fn(code, result["violations"])
    
    # Max iterations reached, return last result
    return code, result
```

---

### 2. HTTP API Integration

Aether can run as an HTTP server for remote validation.

#### Starting the Server

```bash
aether serve --port 8080 --contracts ./contracts/
```

#### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/validate` | POST | Validate code |
| `/api/v1/certify` | POST | Validate and certify |
| `/api/v1/verify` | POST | Verify certificate |
| `/api/v1/analyze` | POST | Analyze prompt |
| `/api/v1/contracts` | GET | List available contracts |
| `/health` | GET | Health check |

#### Validate Endpoint

```http
POST /api/v1/validate
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "source": "#include <iostream>\nint main() { return 0; }",
  "language": "cpp",
  "file": "main.cpp",
  "contracts": ["CPP-MEM-001", "CPP-SEC-001"],
  "options": {
    "max_iterations": 3,
    "certification_level": "full"
  }
}
```

```json
{
  "request_id": "req-abc123",
  "status": "success",
  "passed": false,
  "validation": {
    "layers": {
      "syntax": { "passed": true },
      "semantic": { "passed": true },
      "logic": { "passed": false },
      "architecture": { "passed": true },
      "style": { "passed": true }
    },
    "violations": [
      {
        "id": "CPP-MEM-001",
        "severity": "error",
        "message": "Raw pointer used for ownership",
        "location": { "line": 5, "column": 5 },
        "suggestion": "Use std::unique_ptr",
        "example_fix": "auto ptr = std::make_unique<int>();"
      }
    ],
    "metrics": {
      "errors": 1,
      "warnings": 0,
      "score": 85
    }
  },
  "feedback": {
    "summary": "1 error found in memory safety",
    "fixes": [
      "Replace raw pointer with std::unique_ptr"
    ],
    "hints": [
      "Smart pointers provide automatic memory management"
    ]
  },
  "certificate": null,
  "duration_ms": 32
}
```

#### Analyze Prompt Endpoint

```http
POST /api/v1/analyze
Content-Type: application/json

{
  "prompt": "Add a patrol behavior to enemies",
  "project_context": {
    "root": "/path/to/project",
    "files": ["src/enemy.cpp", "src/enemy.h"]
  }
}
```

```json
{
  "analysis": {
    "intent": {
      "primary": "CREATE",
      "confidence": 0.92
    },
    "scope": {
      "level": "CLASS",
      "entities": [
        {
          "type": "CLASS",
          "name": "Enemy",
          "file": "src/enemy.h"
        }
      ]
    },
    "domain": {
      "primary": "gameplay",
      "tags": ["ai", "behavior", "enemy"]
    },
    "ambiguities": [
      {
        "type": "VALUE",
        "question": "What are the patrol parameters?",
        "options": ["Use defaults", "Custom waypoints"]
      }
    ],
    "enhanced_prompt": "Create a patrol behavior method for the Enemy class..."
  }
}
```

#### Python Client

```python
import requests

class AetherClient:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url
        self.headers = {"Authorization": f"Bearer {api_key}"}
    
    def validate(self, source: str, language: str, contracts: list[str] = None) -> dict:
        response = requests.post(
            f"{self.base_url}/api/v1/validate",
            headers=self.headers,
            json={
                "source": source,
                "language": language,
                "contracts": contracts or []
            }
        )
        return response.json()
    
    def certify(self, source: str, language: str, agent_info: dict = None) -> dict:
        response = requests.post(
            f"{self.base_url}/api/v1/certify",
            headers=self.headers,
            json={
                "source": source,
                "language": language,
                "agent": agent_info
            }
        )
        return response.json()
    
    def analyze_prompt(self, prompt: str, project_root: str = None) -> dict:
        response = requests.post(
            f"{self.base_url}/api/v1/analyze",
            headers=self.headers,
            json={
                "prompt": prompt,
                "project_context": {"root": project_root} if project_root else None
            }
        )
        return response.json()
```

---

### 3. LSP (Language Server Protocol) Integration

For IDE integration, Aether implements LSP.

#### Features

- **Diagnostics** — Real-time validation errors
- **Code Actions** — Quick fixes for violations
- **Hover** — Show contract details
- **Completion** — Contract-aware suggestions

#### Configuration

```json
// VS Code settings.json
{
  "aether.enable": true,
  "aether.executablePath": "/usr/local/bin/aether",
  "aether.contractsPath": "./contracts/",
  "aether.validateOnSave": true,
  "aether.validateOnChange": false,
  "aether.certificationLevel": "standard"
}
```

#### LSP Server

```rust
// crates/aether-lsp/src/server.rs
use tower_lsp::{LanguageServer, LspService, Client};
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct AetherLanguageServer {
    client: Client,
    orchestrator: Arc<RwLock<Orchestrator>>,
    documents: RwLock<HashMap<String, TextDocument>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for AetherLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions::default())),
                code_action_provider: Some(CodeActionProviderSupport::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.write().await.insert(uri.clone(), TextDocument {
            uri,
            text: params.text_document.text,
            version: params.text_document.version,
        });
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Handle document changes
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // Validate on save
        self.validate_document(&params.text_document.uri).await;
    }
}

impl AetherLanguageServer {
    async fn validate_document(&self, uri: &str) -> Vec<Diagnostic> {
        let docs = self.documents.read().await;
        let doc = match docs.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let orchestrator = self.orchestrator.read().await;
        let result = orchestrator.validate(&doc.text, "rust").await;

        result.violations.into_iter()
            .map(|v| Diagnostic {
                range: v.range.into(),
                severity: Some(v.severity.into()),
                message: v.message,
                source: Some("aether".to_string()),
                code: Some(NumberOrString::String(v.id)),
                ..Default::default()
            })
            .collect()
    }
}
```
```

---

### 4. MCP (Model Context Protocol) Integration

For AI agents that support MCP (like Claude, Factory Droid).

#### MCP Server Definition

```json
{
  "name": "aether",
  "version": "0.1.0",
  "tools": [
    {
      "name": "aether_validate",
      "description": "Validate source code against contracts",
      "inputSchema": {
        "type": "object",
        "properties": {
          "source": {
            "type": "string",
            "description": "Source code to validate"
          },
          "language": {
            "type": "string",
            "description": "Programming language (cpp, rust, python, lex)"
          },
          "contracts": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Contract IDs to apply (optional)"
          }
        },
        "required": ["source", "language"]
      }
    },
    {
      "name": "aether_certify",
      "description": "Validate and generate a certificate",
      "inputSchema": {
        "type": "object",
        "properties": {
          "source": { "type": "string" },
          "language": { "type": "string" },
          "file_path": { "type": "string" }
        },
        "required": ["source", "language"]
      }
    },
    {
      "name": "aether_analyze_prompt",
      "description": "Analyze a user prompt to extract intent and scope",
      "inputSchema": {
        "type": "object",
        "properties": {
          "prompt": {
            "type": "string",
            "description": "User prompt to analyze"
          },
          "project_root": {
            "type": "string",
            "description": "Project root directory (optional)"
          }
        },
        "required": ["prompt"]
      }
    },
    {
      "name": "aether_get_feedback",
      "description": "Get AI-friendly feedback for fixing violations",
      "inputSchema": {
        "type": "object",
        "properties": {
          "violations": {
            "type": "array",
            "description": "Violations from previous validation"
          }
        },
        "required": ["violations"]
      }
    }
  ]
}
```

#### MCP Tool Responses

```json
// aether_validate response
{
  "passed": false,
  "violations": [
    {
      "id": "CPP-MEM-001",
      "severity": "error",
      "message": "Raw pointer 'Enemy*' used for ownership",
      "location": { "line": 42, "column": 5 },
      "suggestion": "Use std::unique_ptr<Enemy>",
      "example_fix": "auto enemy = std::make_unique<Enemy>();"
    }
  ],
  "metrics": { "errors": 1, "warnings": 0, "score": 85 }
}
```

---

## The Aether Flow

### Complete Integration Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         USER REQUEST                                        │
│                                                                             │
│   "Add a patrol behavior to enemies"                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         STEP 1: PROMPT ANALYSIS                             │
│                                                                             │
│   aether_analyze_prompt(prompt, project_context)                           │
│                                                                             │
│   Result:                                                                   │
│   • Intent: CREATE                                                          │
│   • Scope: Enemy class in src/enemy.h                                       │
│   • Domain: gameplay, ai                                                    │
│   • Ambiguities: patrol parameters?                                         │
│   • Enhanced prompt: "Create patrol method for Enemy class..."             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      STEP 2: AI CODE GENERATION                             │
│                                                                             │
│   AI Agent receives:                                                        │
│   • Enhanced prompt                                                         │
│   • Bound context (existing Enemy class, similar patterns)                  │
│   • Relevant contracts (gameplay rules)                                     │
│                                                                             │
│   AI generates code...                                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      STEP 3: VALIDATION                                     │
│                                                                             │
│   aether_validate(source, language, contracts)                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    VALIDATION RESULT                                │   │
│   │                                                                     │   │
│   │   passed: false                                                     │   │
│   │   violations:                                                       │   │
│   │     - CPP-MEM-001: Raw pointer used (error)                        │   │
│   │     - LEX-GP-001: Missing faction property (error)                 │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      STEP 4: FEEDBACK GENERATION                            │
│                                                                             │
│   aether_get_feedback(violations)                                          │
│                                                                             │
│   AI-friendly feedback:                                                     │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   Summary: "2 errors found"                                         │   │
│   │                                                                     │   │
│   │   Fix 1:                                                            │   │
│   │     Issue: Raw pointer 'Enemy*' used for ownership                 │   │
│   │     Fix: Use std::unique_ptr<Enemy>                                │   │
│   │     Example: auto enemy = std::make_unique<Enemy>();               │   │
│   │     Why: Smart pointers prevent memory leaks                       │   │
│   │                                                                     │   │
│   │   Fix 2:                                                            │   │
│   │     Issue: Missing 'faction' property on entity                    │   │
│   │     Fix: Add faction: "Player" or "Enemy"                          │   │
│   │     Why: Required for AI targeting decisions                       │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      STEP 5: ITERATION                                      │
│                                                                             │
│   AI Agent receives feedback and fixes code...                              │
│                                                                             │
│   iteration = 1 / max = 3                                                   │
│                                                                             │
│   → Back to STEP 3 (validate)                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                           (after iterations pass)
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      STEP 6: CERTIFICATION                                  │
│                                                                             │
│   aether_certify(source, language)                                         │
│                                                                             │
│   Result:                                                                   │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │   Certificate ID: AETHER-2026-03-08-ABC12345                       │   │
│   │   Passed: true                                                      │   │
│   │   Score: 98                                                         │   │
│   │   Signature: Ed25519                                                │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COMPLETE                                            │
│                                                                             │
│   • Code is validated                                                       │
│   • Code is certified                                                       │
│   • Certificate is stored                                                   │
│   • Ready for commit/deploy                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## SDK Libraries

### Python SDK

```python
# pip install aether-sdk

from aether import AetherClient, IterationManager

client = AetherClient(api_key="...")

# Simple validation
result = client.validate(source_code, language="cpp")
if not result.passed:
    print(result.feedback.summary)

# With iteration
manager = IterationManager(client, max_iterations=3)

def my_ai_generate(prompt, feedback=None):
    # Your AI generation logic
    if feedback:
        # Use feedback to fix code
        return fixed_code
    return generated_code

result = manager.run(
    prompt="Add patrol to Enemy",
    generate_fn=my_ai_generate,
    language="cpp"
)

if result.passed:
    print(f"Certified: {result.certificate_id}")
```

### TypeScript SDK

```typescript
// npm install @aether/sdk

import { AetherClient, IterationManager } from '@aether/sdk';

const client = new AetherClient({ apiKey: '...' });

// Simple validation
const result = await client.validate({
  source: sourceCode,
  language: 'cpp'
});

// With iteration
const manager = new IterationManager(client, { maxIterations: 3 });

const result = await manager.run({
  prompt: 'Add patrol to Enemy',
  generateFn: async (prompt, feedback) => {
    // Your AI generation logic
    return generatedCode;
  },
  language: 'cpp'
});
```

---

## Agent-Specific Integrations

### Claude (Anthropic)

```python
import anthropic
from aether import AetherClient

anthropic_client = anthropic.Anthropic()
aether = AetherClient()

def generate_with_validation(prompt: str, max_iterations: int = 3) -> str:
    # Analyze prompt first
    analysis = aether.analyze_prompt(prompt)
    
    # Generate with enhanced context
    code = anthropic_client.messages.create(
        model="claude-3-opus",
        messages=[{
            "role": "user",
            "content": analysis.enhanced_prompt
        }]
    ).content[0].text
    
    # Validate and iterate
    for _ in range(max_iterations):
        result = aether.validate(code, language="cpp")
        
        if result.passed:
            cert = aether.certify(code, language="cpp")
            return code, cert
        
        # Fix based on feedback
        code = anthropic_client.messages.create(
            model="claude-3-opus",
            messages=[
                {"role": "user", "content": prompt},
                {"role": "assistant", "content": code},
                {"role": "user", "content": f"Fix these issues:\n{result.feedback.summary}"}
            ]
        ).content[0].text
    
    raise Exception("Max iterations reached")
```

### Cursor / VS Code

```typescript
// Cursor extension integration

import { AetherClient } from '@aether/sdk';

const aether = new AetherClient();

// Validate on save
vscode.workspace.onDidSaveTextDocument(async (doc) => {
  const result = await aether.validate({
    source: doc.getText(),
    language: doc.languageId,
    file: doc.uri.fsPath
  });
  
  if (!result.passed) {
    // Show diagnostics
    const diagnostics = result.violations.map(v => 
      new vscode.Diagnostic(
        new vscode.Range(v.line, v.column, v.line, v.column + 10),
        v.message,
        v.severity === 'error' ? vscode.DiagnosticSeverity.Error 
                               : vscode.DiagnosticSeverity.Warning
      )
    );
    
    diagnosticCollection.set(doc.uri, diagnostics);
  }
});

// Quick fix
vscode.languages.registerCodeActionsProvider('cpp', {
  async provideCodeActions(doc, range) {
    const result = await aether.validate({
      source: doc.getText(),
      language: 'cpp'
    });
    
    return result.violations
      .filter(v => v.example_fix)
      .map(v => ({
        title: v.suggestion,
        kind: vscode.CodeActionKind.QuickFix,
        edit: new vscode.WorkspaceEdit(),
        command: {
          command: 'aether.applyFix',
          arguments: [v.example_fix]
        }
      }));
  }
});
```

---

## Configuration for Agents

### Agent Config File

```yaml
# .aether/agent-config.yaml
version: "1.0"

agent:
  type: "claude-3-opus"
  session_id: "${AETHER_SESSION_ID}"  # From environment
  
validation:
  auto_validate: true
  max_iterations: 3
  fail_action: "ask"  # ask, escalate, auto-fix
  
certification:
  auto_certify: true
  level: "full"
  
feedback:
  format: "detailed"  # minimal, normal, detailed
  include_examples: true
  include_ai_hints: true
  
learning:
  enabled: true
  store_corrections: true
  
logging:
  level: "info"
  path: ".aether/logs/agent.log"
```

---

## Metrics and Monitoring

### Agent Metrics

```json
{
  "agent": "claude-3-opus",
  "period": "2026-03-08",
  "metrics": {
    "total_requests": 156,
    "first_try_pass": 124,
    "iterations_needed": {
      "1": 18,
      "2": 10,
      "3": 4,
      "failed": 0
    },
    "pass_rate": {
      "first_try": 0.79,
      "after_3_iterations": 1.0
    },
    "average_score": 94.2,
    "certificates_issued": 156,
    "violations_by_type": {
      "CPP-MEM-001": 12,
      "CPP-SEC-001": 5,
      "LEX-GP-001": 8
    }
  }
}
```

---

## Summary

Aether integrates with AI agents through:

1. **CLI** — Simple subprocess execution
2. **HTTP API** — Remote validation service
3. **LSP** — IDE integration
4. **MCP** — Native AI agent protocol

The integration flow:
1. Analyze prompt → Enhanced context
2. AI generates code
3. Validate → Pass/Fail
4. If fail → Generate feedback → AI fixes → Retry
5. If pass → Issue certificate

This creates a **trust loop** where AI output is always validated before use.
