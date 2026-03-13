# Aether — Prompt Analyzer

**Version:** 0.1.0  
**Related:** [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md)

---

## Overview

The Prompt Analyzer is the first line of defense against AI errors. It analyzes user prompts to extract **intent**, **scope**, **domain**, and **ambiguities** before code generation begins.

**Goal:** Reduce AI errors by ensuring the AI understands exactly what the user wants.

---

## Why Prompt Analysis Matters

Without analysis:
```
User: "Fix the enemy code"
AI: *fixes wrong file, changes wrong function, breaks something else*
```

With analysis:
```
User: "Fix the enemy code"
Analyzer: 
  - Intent: FIX
  - Scope: Which file? Which enemy? What's broken?
  - Ambiguity detected → Ask clarifying questions
AI: *fixes exactly what was needed*
```

---

## Analysis Components

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PROMPT ANALYZER                                     │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        INPUT                                        │   │
│  │  • User prompt (natural language)                                  │   │
│  │  • Project context (files, structure, patterns)                    │   │
│  │  • Conversation history (previous turns)                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     ANALYSIS PIPELINE                               │   │
│  │                                                                     │   │
│  │   ┌───────────┐   ┌───────────┐   ┌───────────┐   ┌───────────┐    │   │
│  │   │  Intent   │──▶│   Scope   │──▶│  Domain   │──▶│ Ambiguity │    │   │
│  │   │Classifier │   │ Extractor │   │  Mapper   │   │ Detector  │    │   │
│  │   └───────────┘   └───────────┘   └───────────┘   └───────────┘    │   │
│  │         │               │               │               │          │   │
│  │         └───────────────┴───────────────┴───────────────┘          │   │
│  │                                 │                                   │   │
│  │                                 ▼                                   │   │
│  │                        ┌───────────────┐                           │   │
│  │                        │   Context     │                           │   │
│  │                        │   Binder      │                           │   │
│  │                        └───────────────┘                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        OUTPUT                                       │   │
│  │  • Structured intent (enum)                                         │   │
│  │  • Scope definition (files, functions, classes)                     │   │
│  │  • Domain classification (tags)                                     │   │
│  │  • Ambiguities detected (questions to ask)                          │   │
│  │  • Bound context (relevant code, patterns)                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Intent Classification

Determine what action the user wants to perform.

### Intent Types

| Intent | Description | Example |
|--------|-------------|---------|
| `CREATE` | Create new code | "Add an enemy class" |
| `MODIFY` | Change existing code | "Update the player speed" |
| `FIX` | Fix a bug or error | "Fix the crash in Enemy::update" |
| `REFACTOR` | Restructure without behavior change | "Extract this into a function" |
| `DELETE` | Remove code | "Delete the old system" |
| `EXPLAIN` | Understand code | "How does the AI work?" |
| `SEARCH` | Find something | "Where is the player defined?" |
| `TEST` | Create or run tests | "Add tests for Enemy" |
| `DOCUMENT` | Add documentation | "Document this function" |

### Classification Algorithm

```cpp
namespace aether::prompt {

enum class Intent {
    Create,
    Modify,
    Fix,
    Refactor,
    Delete,
    Explain,
    Search,
    Test,
    Document,
    Unknown
};

struct IntentResult {
    Intent primary;
    float confidence;
    std::vector<Intent> alternatives;
};

class IntentClassifier {
public:
    IntentResult classify(const std::string& prompt);
    
private:
    // Keywords for each intent
    std::map<Intent, std::vector<std::string>> m_keywords;
    std::map<Intent, std::vector<std::string>> m_patterns;
    
    // ML model (optional, for advanced classification)
    std::unique_ptr<IntentModel> m_model;
};

// Example keyword mappings
const std::map<Intent, std::vector<std::string>> DEFAULT_KEYWORDS = {
    {Intent::Create, {"add", "create", "new", "implement", "build", "write", "make"}},
    {Intent::Modify, {"change", "update", "modify", "alter", "set", "adjust", "edit"}},
    {Intent::Fix, {"fix", "bug", "error", "crash", "issue", "problem", "broken", "repair"}},
    {Intent::Refactor, {"refactor", "restructure", "reorganize", "extract", "move", "rename"}},
    {Intent::Delete, {"delete", "remove", "eliminate", "drop", "clear"}},
    {Intent::Explain, {"how", "what", "why", "explain", "describe", "understand"}},
    {Intent::Search, {"where", "find", "search", "locate", "show me"}},
    {Intent::Test, {"test", "spec", "coverage", "assert", "verify"}},
    {Intent::Document, {"document", "comment", "doc", "documentation", "readme"}}
};

}
```

---

## 2. Scope Extraction

Determine what parts of the codebase are affected.

### Scope Levels

| Level | Description | Example |
|-------|-------------|---------|
| `FILE` | Single file | "Update enemy.cpp" |
| `FUNCTION` | Single function | "Fix the update method" |
| `CLASS` | Single class | "Add method to Enemy" |
| `MODULE` | Multiple related files | "Refactor the AI system" |
| `PROJECT` | Entire project | "Upgrade to C++20" |

### Scope Entities

```cpp
namespace aether::prompt {

struct ScopeEntity {
    enum Type { File, Function, Class, Module, Namespace, Variable };
    Type type;
    std::string name;
    std::string file;
    int line = -1;
    float confidence;
};

struct ScopeResult {
    ScopeLevel level;
    std::vector<ScopeEntity> entities;
    bool isAmbiguous;
};

class ScopeExtractor {
public:
    ScopeExtractor(const ProjectContext& project);
    
    ScopeResult extract(const std::string& prompt, Intent intent);
    
private:
    const ProjectContext& m_project;
    SymbolIndex m_symbolIndex;
    
    // Extract file references
    std::vector<ScopeEntity> extractFiles(const std::string& prompt);
    
    // Extract function/class references
    std::vector<ScopeEntity> extractSymbols(const std::string& prompt);
    
    // Infer scope from intent
    std::vector<ScopeEntity> inferFromContext(Intent intent);
};

}
```

### Scope Inference Examples

```
Prompt: "Fix the crash"
Project: Has recent errors in enemy.cpp line 42
→ Scope: enemy.cpp, Enemy::update()

Prompt: "Add jumping"
Project: Player class exists
→ Scope: player.cpp, Player class

Prompt: "Refactor the AI system"
Project: Has ai/ directory with multiple files
→ Scope: ai/ module
```

---

## 3. Domain Mapping

Classify the technical domain of the request.

### Domain Categories

| Domain | Tags | Examples |
|--------|------|----------|
| `gameplay` | mechanics, ai, combat | "Add enemy patrol", "Fix damage calculation" |
| `ui` | hud, menu, input | "Add health bar", "Fix button click" |
| `graphics` | rendering, shaders | "Add bloom effect", "Fix texture loading" |
| `audio` | sound, music | "Add footstep sounds", "Fix music loop" |
| `networking` | multiplayer, sync | "Add chat", "Fix desync" |
| `data` | persistence, config | "Save player progress", "Load settings" |
| `performance` | optimization, memory | "Reduce lag", "Fix memory leak" |
| `security` | auth, encryption | "Add login", "Encrypt saves" |
| `build` | cmake, packaging | "Fix build error", "Add library" |
| `testing` | unit, integration | "Add tests for Enemy" |

### Domain Classifier

```cpp
namespace aether::prompt {

struct DomainResult {
    std::string primary;
    std::vector<std::string> secondary;
    std::vector<std::string> tags;
    float confidence;
};

class DomainMapper {
public:
    DomainMapper();
    
    DomainResult map(const std::string& prompt, const ScopeResult& scope);
    
private:
    std::map<std::string, std::vector<std::string>> m_domainKeywords;
    std::map<std::string, std::vector<std::string>> m_tagKeywords;
};

const std::map<std::string, std::vector<std::string>> DOMAIN_KEYWORDS = {
    {"gameplay", {"enemy", "player", "weapon", "damage", "health", "ai", "pathfinding"}},
    {"ui", {"menu", "button", "hud", "ui", "interface", "click", "input"}},
    {"graphics", {"render", "shader", "texture", "mesh", "particle", "light"}},
    {"audio", {"sound", "music", "audio", "sfx", "volume"}},
    {"networking", {"multiplayer", "server", "client", "sync", "network", "online"}},
    {"data", {"save", "load", "file", "config", "json", "database"}},
    {"performance", {"fast", "slow", "optimize", "lag", "fps", "memory"}},
    {"security", {"auth", "login", "password", "encrypt", "secure"}},
    {"build", {"build", "compile", "cmake", "link", "library", "error"}},
    {"testing", {"test", "spec", "mock", "assert", "coverage"}}
};

}
```

---

## 4. Ambiguity Detection

Identify unclear or underspecified parts of the request.

### Ambiguity Types

| Type | Description | Example | Question |
|------|-------------|---------|----------|
| `SCOPE` | What to modify | "Fix the bug" | "Which bug in which file?" |
| `VALUE` | What value to use | "Add speed" | "What speed value?" |
| `LOCATION` | Where to add | "Add logging" | "Which functions need logging?" |
| `BEHAVIOR` | How it should work | "Make it better" | "What does 'better' mean?" |
| `DEPENDENCY` | What depends on this | "Delete this" | "This is used by X, Y, Z" |
| `CONFLICT` | Multiple interpretations | "Update player" | "Player class or Player file?" |

### Ambiguity Detector

```cpp
namespace aether::prompt {

struct Ambiguity {
    enum Type { Scope, Value, Location, Behavior, Dependency, Conflict };
    Type type;
    std::string description;
    std::string question;          // Question to ask user
    std::vector<std::string> options;  // Suggested answers
    float severity;                // 0.0-1.0, how critical
};

class AmbiguityDetector {
public:
    std::vector<Ambiguity> detect(
        const std::string& prompt,
        Intent intent,
        const ScopeResult& scope,
        const DomainResult& domain,
        const ProjectContext& project
    );
    
private:
    // Detect missing scope information
    std::vector<Ambiguity> detectScopeAmbiguity(
        Intent intent,
        const ScopeResult& scope,
        const ProjectContext& project
    );
    
    // Detect missing value specifications
    std::vector<Ambiguity> detectValueAmbiguity(const std::string& prompt);
    
    // Detect potential conflicts
    std::vector<Ambiguity> detectConflicts(
        const ScopeResult& scope,
        const ProjectContext& project
    );
};

}
```

### Ambiguity Resolution

When ambiguities are detected, Aether can:

1. **Ask the user** — Generate clarifying questions
2. **Use heuristics** — Make reasonable assumptions
3. **Check history** — Look at previous interactions

```cpp
struct ClarificationRequest {
    std::string message;
    std::vector<Ambiguity> ambiguities;
    std::vector<std::string> suggestedAnswers;
};

// Example output
ClarificationRequest {
    message: "I need some clarification before proceeding:",
    ambiguities: [
        {
            type: Ambiguity::Scope,
            description: "Multiple 'Enemy' classes found",
            question: "Which Enemy class do you want to modify?",
            options: ["Enemy (gameplay/enemy.h)", "Enemy (ai/enemy.h)"]
        },
        {
            type: Ambiguity::Value,
            description: "Speed value not specified",
            question: "What speed value should the enemy have?",
            options: ["Use default (5.0)", "Custom value"]
        }
    ]
}
```

---

## 5. Context Binding

Retrieve relevant code context for the AI.

### Context Sources

| Source | What It Provides |
|--------|------------------|
| **Target Code** | The code being modified |
| **Related Code** | Dependencies, callers, callees |
| **Pattern Library** | Project-specific patterns |
| **Similar Code** | Similar implementations in project |
| **Documentation** | Comments, docs, READMEs |

### Context Binder

```cpp
namespace aether::prompt {

struct BoundContext {
    // Primary context
    std::string targetCode;
    std::string targetFile;
    SourceRange targetRange;
    
    // Related context
    std::vector<std::string> relatedFiles;
    std::map<std::string, std::string> relatedCode;  // file -> code
    
    // Pattern context
    std::vector<CodePattern> relevantPatterns;
    
    // Similar code
    std::vector<CodeExample> similarExamples;
    
    // Metadata
    std::string language;
    std::string framework;
    std::vector<std::string> conventions;
};

class ContextBinder {
public:
    ContextBinder(const ProjectContext& project);
    
    BoundContext bind(
        Intent intent,
        const ScopeResult& scope,
        const DomainResult& domain
    );
    
private:
    const ProjectContext& m_project;
    PatternLibrary m_patterns;
    CodeSearchIndex m_searchIndex;
    
    // Retrieve code for scope
    std::string getTargetCode(const ScopeResult& scope);
    
    // Find related files
    std::vector<std::string> findRelatedFiles(const ScopeResult& scope);
    
    // Find similar implementations
    std::vector<CodeExample> findSimilarCode(
        const std::string& target,
        const DomainResult& domain
    );
};

}
```

---

## Complete Analysis Output

```cpp
namespace aether::prompt {

struct PromptAnalysis {
    // Original input
    std::string originalPrompt;
    
    // Classification
    IntentResult intent;
    ScopeResult scope;
    DomainResult domain;
    
    // Ambiguities
    std::vector<Ambiguity> ambiguities;
    bool needsClarification;
    ClarificationRequest clarification;
    
    // Bound context
    BoundContext context;
    
    // AI-ready prompt enhancement
    std::string enhancedPrompt;
    
    // Metadata
    std::string analysisId;
    std::chrono::milliseconds processingTime;
};

class PromptAnalyzer {
public:
    PromptAnalyzer(const ProjectContext& project);
    
    PromptAnalysis analyze(const std::string& prompt);
    
private:
    IntentClassifier m_intentClassifier;
    ScopeExtractor m_scopeExtractor;
    DomainMapper m_domainMapper;
    AmbiguityDetector m_ambiguityDetector;
    ContextBinder m_contextBinder;
    
    const ProjectContext& m_project;
};

}
```

---

## Example Analysis

### Input

```
Prompt: "Add a patrol behavior to enemies"
Project: lex-game/
```

### Analysis

```json
{
  "originalPrompt": "Add a patrol behavior to enemies",
  
  "intent": {
    "primary": "CREATE",
    "confidence": 0.92,
    "alternatives": ["MODIFY"]
  },
  
  "scope": {
    "level": "CLASS",
    "entities": [
      {
        "type": "CLASS",
        "name": "Enemy",
        "file": "gameplay/enemy.h",
        "confidence": 0.88
      }
    ],
    "isAmbiguous": false
  },
  
  "domain": {
    "primary": "gameplay",
    "secondary": ["ai"],
    "tags": ["enemy", "behavior", "ai", "movement"],
    "confidence": 0.95
  },
  
  "ambiguities": [
    {
      "type": "VALUE",
      "description": "Patrol parameters not specified",
      "question": "What are the patrol parameters?",
      "options": [
        "Use default (random waypoints, 5s wait)",
        "Specify waypoints manually",
        "Use defined patrol zones"
      ],
      "severity": 0.4
    }
  ],
  
  "needsClarification": true,
  
  "clarification": {
    "message": "Before I add patrol behavior, I need to know:",
    "ambiguities": ["..."],
    "suggestedAnswers": ["..."]
  },
  
  "context": {
    "targetCode": "class Enemy : public Entity { ... }",
    "targetFile": "gameplay/enemy.h",
    "relatedFiles": [
      "gameplay/enemy.cpp",
      "ai/behavior_tree.h",
      "ai/movement_system.h"
    ],
    "relevantPatterns": [
      {
        "name": "Enemy behavior pattern",
        "example": "class FlyingEnemy : public Enemy { void updateBehavior() override; }"
      }
    ],
    "similarExamples": [
      {
        "file": "ai/patrol_behavior.cpp",
        "description": "Existing patrol implementation for NPCs"
      }
    ],
    "language": "cpp",
    "framework": "prism-engine"
  },
  
  "enhancedPrompt": "Create a patrol behavior method for the Enemy class in gameplay/enemy.h. The project uses the Prism engine and has existing patrol behavior in ai/patrol_behavior.cpp that can be used as reference. Add the method declaration and implement it in gameplay/enemy.cpp. The method should integrate with the existing behavior tree system."
}
```

---

## Learning from User Corrections

The analyzer learns from how users refine their prompts:

```cpp
namespace aether::prompt {

class PromptLearner {
public:
    // Record a user's clarification
    void recordClarification(
        const std::string& originalPrompt,
        const Ambiguity& ambiguity,
        const std::string& userResponse
    );
    
    // Record successful analysis
    void recordSuccess(const PromptAnalysis& analysis);
    
    // Record failed analysis (user corrected)
    void recordFailure(
        const PromptAnalysis& analysis,
        const std::string& correction
    );
    
    // Get learned patterns
    std::vector<PromptPattern> getLearnedPatterns();
};

}
```

---

## Integration with Validation

The prompt analysis feeds into validation:

```
1. User Prompt
      │
      ▼
2. Prompt Analysis
      │
      ├─ Intent → Determines validation rules
      ├─ Scope → Determines files to validate
      ├─ Domain → Determines domain contracts
      └─ Context → Provides expected patterns
      │
      ▼
3. Code Generation (by AI)
      │
      ▼
4. Validation
      │
      ├─ Check against domain contracts
      ├─ Check against project patterns
      └─ Check scope is respected
      │
      ▼
5. Result
```

---

## Configuration

```yaml
# .aether/prompt-config.yaml
version: "1.0"

intent:
  confidence_threshold: 0.7
  ask_if_below: true
  
scope:
  max_files: 10
  infer_from_errors: true
  prefer_recent_files: true
  
domain:
  auto_detect: true
  fallback: "general"
  
ambiguity:
  auto_resolve_low_severity: true  # severity < 0.3
  max_questions: 3
  timeout_seconds: 60
  
context:
  max_related_files: 5
  max_similar_examples: 3
  include_comments: true
  
learning:
  enabled: true
  storage: ".aether/learned-prompts.yaml"
```

---

## Summary

The Prompt Analyzer:

1. **Classifies Intent** — What does the user want?
2. **Extracts Scope** — What code is affected?
3. **Maps Domain** — What technical area?
4. **Detects Ambiguity** — What's unclear?
5. **Binds Context** — What relevant code exists?

This structured understanding enables AI agents to generate better code on the first try, reducing the need for iteration.
