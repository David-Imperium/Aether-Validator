# Synward — Contract System

**Version:** 0.2.0  
**Related:** [SYNWARD_MASTER_DESIGN.md](./SYNWARD_MASTER_DESIGN.md)

---

## Overview

The Contract System is the heart of Synward's validation logic. It provides a declarative way to define rules that code must follow, independent of the programming language.

---

## Core Concepts

### What is a Contract?

A **contract** is a formal declaration of a code requirement:

- **ID** — Unique identifier (e.g., `CPP-MEM-001`)
- **Name** — Human-readable name
- **Description** — What the contract checks
- **Severity** — How serious is violation
- **Pattern** — What to look for
- **Suggestion** — How to fix it
- **AI Hint** — Context for AI agents

### Contract Domains

Contracts are organized by **domain** — the area of concern:

| Domain | Examples |
|--------|----------|
| `memory-safety` | Raw pointers, leaks, RAII |
| `security` | SQL injection, XSS, secrets |
| `performance` | Unnecessary copies, N+1 queries |
| `architecture` | Circular deps, layer violations |
| `style` | Naming, formatting |
| `logic` | Preconditions, invariants |
| `domain-specific` | Game rules, API contracts |

### Contract Loading

Contracts are loaded by the **ContractLayer** from the filesystem:

| Location | Description |
|----------|-------------|
| `~/.synward/contracts/{language}/*.yaml` | User contracts (default) |
| `./contracts/{language}/*.yaml` | Project-local contracts |
| Built-in registry | Core contracts bundled with Synward |

**ContractLayer** is the first layer in the validation pipeline. It:
1. Loads YAML contracts for the target language
2. Evaluates regex patterns against source code
3. Converts matches to violations with severity mapping
4. Caches loaded contracts for performance

**CLI Usage:**
```bash
# Default: loads from ~/.synward/contracts/
synward validate src/main.py --lang python

# Custom contracts directory
synward validate src/main.py --lang python --contracts ./my-contracts/
```

---

## Contract Definition Language (CDL)

Contracts are defined in YAML for readability and AI-friendliness.

### Basic Structure

```yaml
# contracts/cpp/memory.contracts.yaml
meta:
  version: "1.0"
  language: cpp
  domain: memory-safety
  author: Synward Team

contracts:
  - id: CPP-MEM-001
    name: no-raw-pointers-owning
    description: "Owning raw pointers are forbidden. Use smart pointers."
    severity: error
    
    # Detection
    pattern:
      type: ast-match
      query: |
        (pointer_type declarator: (identifier) @var)
        (assignment_expression left: @var right: (call_expression function: "new"))
    
    # Remediation
    suggestion: "Use std::unique_ptr<T> or std::shared_ptr<T>"
    example_fix:
      before: "Enemy* enemy = new Enemy();"
      after: "auto enemy = std::make_unique<Enemy>();"
    
    # AI context
    ai_hint: |
      Smart pointers automatically manage memory lifecycle:
      - unique_ptr: Single ownership, zero overhead
      - shared_ptr: Shared ownership, reference counted
    
    # Metadata
    tags: [memory, raii, modern-cpp]
    references:
      - "https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#r20-use-unique_ptr-or-shared_ptr-to-represent-ownership"
```

### Contract Fields Reference

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `id` | Yes | string | Unique contract ID |
| `name` | Yes | string | Short identifier |
| `description` | Yes | string | Full description |
| `severity` | Yes | enum | error, warning, info, hint |
| `pattern` | Yes | object | Detection pattern |
| `suggestion` | No | string | How to fix |
| `example_fix` | No | object | Before/after code |
| `ai_hint` | No | string | Context for AI |
| `tags` | No | string[] | Searchable tags |
| `references` | No | string[] | Documentation links |
| `enabled` | No | bool | Default: true |
| `deprecated` | No | bool | Mark as deprecated |

---

## Pattern Types

### 1. AST Match

Match against the Abstract Syntax Tree:

```yaml
pattern:
  type: ast-match
  query: |
    (call_expression
      function: (identifier) @func
      (#eq? @func "malloc"))
```

### 2. Text Regex

Simple text-based matching:

```yaml
pattern:
  type: regex
  pattern: "\\b(malloc|free|realloc)\\s*\\("
  flags: [case-sensitive]
```

**Multiline Matching:** The `.` character in regex does NOT match newlines. For patterns spanning multiple lines, use:

| Pattern | Description |
|---------|-------------|
| `[\s\S]*?` | Match any character including newlines (non-greedy) |
| `[\s\S]+?` | Match one or more characters including newlines |
| `(?s)` | Enable DOTALL mode (makes `.` match newlines) |

**Example - Detect silent exception catching:**
```yaml
# WRONG - doesn't match across lines
pattern: "except.*:\s*pass"

# CORRECT - matches multiline code
pattern: "except[^:]*:[\s\S]*?pass"

# ALSO CORRECT - using DOTALL mode
pattern: "(?s)except.*?:\s*pass"
```

**Real-world example:**
```yaml
contracts:
  - id: PYERR001
    name: silent-exception-caught
    description: "Silent exception caught - errors are hidden"
    severity: error
    pattern: "except[^:]*:[\\s\\S]*?pass"
    suggestion: "Log or handle the exception properly"

  - id: PYERR002
    name: print-in-except
    description: "Using print() in exception handler - use logging"
    severity: warning
    pattern: "(?s)except.*?:\\s*print"
    suggestion: "Use logging.error() or logging.exception() instead"
```

### 3. Semantic Query

Query semantic information (types, symbols):

```yaml
pattern:
  type: semantic-query
  query: |
    SELECT var.name, var.type
    FROM variables var
    WHERE var.type LIKE "%*" 
      AND var.is_owning = true
      AND var.type NOT LIKE "unique_ptr%"
      AND var.type NOT LIKE "shared_ptr%"
```

### 4. Composite

Combine patterns with logic:

```yaml
pattern:
  type: composite
  operator: and
  patterns:
    - type: ast-match
      query: "(pointer_type)"
    - type: not
      pattern:
        type: regex
        pattern: "const\\s*\\*"
```

### 5. Custom

Language-specific custom check:

```yaml
pattern:
  type: custom
  evaluator: "cpp.check_circular_include"
  params:
    max_depth: 10
```

---

## Severity Levels

| Level | Meaning | Behavior |
|-------|---------|----------|
| `error` | Must fix | Blocks certification |
| `warning` | Should fix | Flagged but can certify |
| `info` | FYI | Informational only |
| `hint` | Suggestion | Style improvement |

### Severity Thresholds

```yaml
# .synward/config.yaml
thresholds:
  error_limit: 0       # 0 errors = fail
  warning_limit: 10    # >10 warnings = fail
  score_minimum: 80    # score < 80 = fail
```

---

## Contract Examples

### Memory Safety (C++)

```yaml
# contracts/cpp/memory-safety.contracts.yaml
meta:
  language: cpp
  domain: memory-safety

contracts:
  - id: CPP-MEM-001
    name: no-raw-pointers-owning
    description: "Use smart pointers for ownership"
    severity: error
    pattern:
      type: ast-match
      query: |
        (pointer_type) @ptr
        (assignment_expression 
          left: @ptr 
          right: (call_expression function: "new"))
    suggestion: "Use std::unique_ptr or std::shared_ptr"
    
  - id: CPP-MEM-002
    name: no-malloc-free
    description: "Use RAII containers instead of malloc/free"
    severity: warning
    pattern:
      type: regex
      pattern: "\\b(malloc|free|realloc|calloc)\\s*\\("
    suggestion: "Use std::vector, std::string, or smart pointers"
    
  - id: CPP-MEM-003
    name: initialize-members
    description: "All class members should be initialized"
    severity: warning
    pattern:
      type: semantic-query
      query: |
        SELECT c.name, m.name
        FROM classes c
        JOIN members m ON c.id = m.class_id
        LEFT JOIN initializers i ON m.id = i.member_id
        WHERE i.id IS NULL AND m.has_default_value = false
    suggestion: "Initialize member in constructor or use default member initializer"
```

### Security (C++)

```yaml
# contracts/cpp/security.contracts.yaml
meta:
  language: cpp
  domain: security

contracts:
  - id: CPP-SEC-001
    name: no-strcpy
    description: "strcpy is unsafe, use strncpy or std::string"
    severity: error
    pattern:
      type: ast-match
      query: '(call_expression function: (identifier) @func (#eq? @func "strcpy"))'
    suggestion: "Use std::string or strncpy with bounds checking"
    
  - id: CPP-SEC-002
    name: no-printf-user-input
    description: "User input must not be used as format string"
    severity: error
    pattern:
      type: semantic-query
      query: |
        SELECT call.location
        FROM call_expressions call
        WHERE call.function IN ('printf', 'fprintf', 'sprintf')
          AND call.args[0].source = 'user_input'
    suggestion: "Use %s with the user input as argument, not as format string"
    
  - id: CPP-SEC-003
    name: no-hardcoded-secrets
    description: "No hardcoded passwords or API keys"
    severity: error
    pattern:
      type: regex
      pattern: '(?i)(password|api_key|secret|token)\s*=\s*"[^"]{8,}"'
    suggestion: "Load secrets from environment variables or secure storage"
```

### Rust Ownership

```yaml
# contracts/rust/ownership.contracts.yaml
meta:
  language: rust
  domain: ownership

contracts:
  - id: RUST-OWN-001
    name: no-unnecessary-clone
    description: "Avoid unnecessary .clone() calls"
    severity: warning
    pattern:
      type: ast-match
      query: |
        (method_call_expression
          method: (identifier) @method
          (#eq? @method "clone"))
    suggestion: "Consider using references instead of cloning"
    
  - id: RUST-OWN-002
    name: prefer-borrow
    description: "Prefer &T over T when ownership not needed"
    severity: info
    pattern:
      type: semantic-query
      query: |
        SELECT param.name, param.type
        FROM function_params param
        WHERE param.is_consume = true
          AND param.usage_count < param.move_count
    suggestion: "This parameter is only read, consider making it a reference"
```

### Lex DSL (Game)

```yaml
# contracts/lex/gameplay.contracts.yaml
meta:
  language: lex
  domain: gameplay

contracts:
  - id: LEX-GP-001
    name: entity-requires-faction
    description: "Every entity must have a faction property"
    severity: error
    pattern:
      type: semantic-query
      query: |
        SELECT e.name, e.type
        FROM entities e
        WHERE e.type IN ('unit', 'building', 'character')
          AND e.properties NOT CONTAINS 'faction'
    suggestion: "Add: faction: \"Player\" or \"Enemy\""
    ai_hint: |
      Faction is required for:
      - AI targeting decisions
      - Alliance calculations
      - Win/lose conditions
    
  - id: LEX-GP-002
    name: health-positive
    description: "Health values must be positive"
    severity: error
    pattern:
      type: semantic-query
      query: |
        SELECT e.name, e.health
        FROM entities e
        WHERE e.health <= 0
    suggestion: "Set health to a positive value (e.g., 100)"
    
  - id: LEX-GP-003
    name: damage-valid-range
    description: "Damage must be reasonable (1-1000)"
    severity: warning
    pattern:
      type: semantic-query
      query: |
        SELECT e.name, e.damage
        FROM entities e
        WHERE e.damage < 1 OR e.damage > 1000
    suggestion: "Adjust damage to reasonable range"
```

---

## Contract Registry

The Contract Registry manages all loaded contracts.

```cpp
namespace synward::contracts {

class ContractRegistry {
public:
    // Loading
    void loadFromDirectory(const std::string& path);
    void loadFromFile(const std::string& path);
    
    // Querying
    std::vector<const Contract*> getAllContracts() const;
    std::vector<const Contract*> getContractsForDomain(const std::string& domain) const;
    std::vector<const Contract*> getContractsForLanguage(const std::string& lang) const;
    const Contract* getContractById(const std::string& id) const;
    
    // Filtering
    void enableContract(const std::string& id);
    void disableContract(const std::string& id);
    void setSeverityOverride(const std::string& id, Severity severity);
    
private:
    std::map<std::string, std::unique_ptr<Contract>> m_contracts;
    std::map<std::string, std::vector<std::string>> m_domainIndex;
    std::map<std::string, std::vector<std::string>> m_languageIndex;
};

}
```

---

## Rule Evaluator

The Rule Evaluator executes contract patterns against the AST.

```cpp
namespace synward::contracts {

class RuleEvaluator {
public:
    RuleEvaluator(const ASTNode* ast, const ValidationContext& ctx);
    
    // Evaluate a single contract
    std::vector<Violation> evaluate(const Contract& contract);
    
    // Evaluate multiple contracts
    std::vector<Violation> evaluateAll(const std::vector<const Contract*>& contracts);
    
private:
    const ASTNode* m_ast;
    const ValidationContext& m_ctx;
    std::unique_ptr<PatternMatcher> m_matcher;
    std::unique_ptr<SemanticQuerier> m_querier;
};

// Pattern matchers for different pattern types
class IPatternMatcher {
public:
    virtual ~IPatternMatcher() = default;
    virtual std::vector<Match> match(const Pattern& pattern, const ASTNode* ast) = 0;
};

class ASTMatcher : public IPatternMatcher { /* tree-sitter queries */ };
class RegexMatcher : public IPatternMatcher { /* regex matching */ };
class SemanticQuerier : public IPatternMatcher { /* symbol table queries */ };
class CompositeMatcher : public IPatternMatcher { /* combine matchers */ };

}
```

---

## Violation Reporting

```cpp
namespace synward::contracts {

class ViolationReporter {
public:
    // Generate human-readable report
    std::string generateTextReport(const std::vector<Violation>& violations);
    
    // Generate JSON report
    json generateJsonReport(const std::vector<Violation>& violations);
    
    // Generate AI-friendly feedback
    AIFeedback generateAIFeedback(const std::vector<Violation>& violations);
    
private:
    SourceFormatter m_formatter;
    ExampleGenerator m_exampleGenerator;
};

struct AIFeedback {
    std::string summary;                          // "3 errors found in memory safety"
    std::vector<std::string> fixes;               // Specific fixes to apply
    std::vector<std::string> hints;               // Context for understanding
    std::map<std::string, std::string> examples;  // Before/after code
};

}
```

---

## Project-Specific Contracts

Projects can define their own contracts:

```
my-project/
├── .synward/
│   ├── config.yaml
│   └── contracts/
│       ├── my-cpp-rules.contracts.yaml
│       └── my-game-rules.contracts.yaml
```

```yaml
# my-project/.synward/contracts/my-cpp-rules.contracts.yaml
meta:
  language: cpp
  domain: project-specific
  project: my-game

contracts:
  - id: MYGAME-001
    name: use-game-allocator
    description: "Use GameAllocator for all game objects"
    severity: error
    pattern:
      type: ast-match
      query: '(call_expression function: "new" (type_identifier) @type (#not-match? @type "std::.*"))'
    suggestion: "Use GAME_NEW(Type) instead of new Type"
```

---

## Contract Inheritance

Contracts can inherit from base contracts:

```yaml
contracts:
  - id: BASE-SEC-001
    name: no-plaintext-secrets
    abstract: true
    description: "Secrets must not be in plaintext"
    severity: error
    pattern:
      type: regex
      pattern: '(?i)(password|secret)\s*=\s*"'

  - id: CPP-SEC-005
    name: no-plaintext-secrets-cpp
    extends: BASE-SEC-001
    language: cpp
    suggestion: "Use SecureString or load from Keychain"
    
  - id: PY-SEC-005
    name: no-plaintext-secrets-python
    extends: BASE-SEC-001
    language: python
    suggestion: "Use os.environ.get() or python-dotenv"
```

---

## Contract Versioning

```yaml
meta:
  version: "2.0"
  compatibility: ">=1.0 <3.0"

contracts:
  - id: CPP-MEM-001
    name: no-raw-pointers-owning
    version: "2.0"
    deprecated: false
    # ...
    
  - id: CPP-MEM-001
    name: no-raw-pointers-owning
    version: "1.0"
    deprecated: true
    deprecation_message: "Use version 2.0 with improved detection"
    # old pattern...
```

---

## Learning from Corrections

Synward can learn from how developers fix violations:

```cpp
namespace synward::contracts {

class ContractLearner {
public:
    // Record a correction made by developer
    void recordCorrection(
        const Violation& violation,
        const std::string& originalCode,
        const std::string& fixedCode
    );
    
    // Generate learned patterns
    std::vector<LearnedPattern> extractPatterns();
    
    // Suggest new contracts based on patterns
    std::vector<ContractSuggestion> suggestContracts();
    
private:
    CorrectionHistory m_history;
    PatternMiner m_miner;
};

struct LearnedPattern {
    std::string pattern;
    float confidence;
    int occurrences;
    std::vector<std::string> examples;
};

}
```

---

## Contract Testing

Each contract should have tests:

```yaml
# contracts/cpp/memory-safety.contracts.test.yaml
contract: CPP-MEM-001
name: no-raw-pointers-owning

tests:
  - name: detects-raw-new
    input: |
      Enemy* enemy = new Enemy();
    expected:
      - violation: true
        severity: error
        
  - name: allows-unique-ptr
    input: |
      auto enemy = std::make_unique<Enemy>();
    expected:
      - violation: false
        
  - name: allows-non-owning-pointer
    input: |
      Enemy* enemy = getEnemy();
    expected:
      - violation: false
```

---

## Summary

The Contract System provides:

1. **Declarative Rules** — YAML-based contract definitions
2. **Multiple Pattern Types** — AST, regex, semantic, composite
3. **AI-Friendly Output** — Hints and examples for AI agents
4. **Extensibility** — Project-specific contracts
5. **Learning** — Extract patterns from corrections

This enables Synward to validate code against any set of requirements, making it truly universal.
