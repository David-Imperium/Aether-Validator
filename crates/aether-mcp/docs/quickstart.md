# Aether Quick Start Guide

## What is Aether?

Aether is a code validation and certification tool that provides:
- Multi-language parsing (23 languages)
- Syntax validation with detailed error reporting
- AST analysis and metrics
- Cryptographic code certification (Ed25519)
- MCP-native tool integration

## MCP Configuration

Add to `~/.factory/mcp.json`:
```json
{
  "mcpServers": {
    "aether": {
      "type": "stdio",
      "command": "/path/to/aether-mcp",
      "args": []
    }
  }
}
```

## Available MCP Tools (12)

### validate_file
Validate a single file:
```json
{
  "file_path": "src/main.rs",
  "language": "rust",
  "contracts": "no_unsafe,documentation"
}
```

### batch_validate
Validate multiple files:
```json
{
  "file_paths": ["src/main.rs", "src/lib.rs"],
  "contracts": "no_panic"
}
```

### analyze_code
Get AST statistics:
```json
{
  "code": "fn main() { println!(\"Hello\"); }",
  "language": "rust"
}
```

### get_metrics
Get code metrics:
```json
{
  "code": "...",
  "language": "rust"
}
```

### certify_code
Generate cryptographic certificate:
```json
{
  "code": "...",
  "language": "rust",
  "signer": "Developer Name",
  "contracts": []
}
```

### suggest_fixes
Get AI-powered fix suggestions:
```json
{
  "code": "...",
  "language": "rust",
  "errors": ["SYNTAX001: Missing semicolon"]
}
```

### get_language_info
Get supported features for a language:
```json
{
  "language": "rust"
}
```

### list_languages
List all supported languages (23):
```json
{}
```

### list_contracts
List available validation contracts:
```json
{}
```

### get_version
Get Aether version and capabilities:
```json
{}
```

### watch_start / watch_check / watch_stop
Monitor directory for changes:
```json
{"directory": "./src", "extensions": "rs"}
```

## Using via CLI

```bash
# Validate a file
aether validate src/main.rs

# Analyze code
aether analyze src/main.rs

# Certify code
aether certify src/main.rs --signer "Developer"
```

## Integration with Droid Skill

The `validate` skill uses Aether automatically when validating code files.
