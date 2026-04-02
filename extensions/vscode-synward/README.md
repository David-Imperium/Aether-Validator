# Synward VS Code Extension

Intelligent code validation with AI-powered contract enforcement.

## Features

- **Real-time Validation**: Automatic validation on save
- **Intelligent Diagnostics**: Severity-based error highlighting
- **Quick Fixes**: 
  - Accept violation with reason
  - Get AI fix suggestions
  - Suppress violations (file/line level)
- **Compliance Dashboard**: View compliance engine status
- **Drift Analysis**: Track code quality trends over time
- **Quality Score**: Status bar indicator

## Supported Languages

Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, Ruby, PHP, Swift, Kotlin

## Commands

| Command | Description |
|---------|-------------|
| `Synward: Validate Current File` | Validate the active file |
| `Synward: Validate Project` | Validate all files in workspace |
| `Synward: Accept Violation` | Accept a violation with documented reason |
| `Synward: Show Compliance Status` | Open compliance dashboard |
| `Synward: Analyze Drift` | Analyze drift for current file |

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `synward.enableValidation` | `true` | Enable automatic validation on save |
| `synward.validationMode` | `balanced` | Validation strictness (strict/balanced/lenient) |
| `synward.dubbiosoMode` | `true` | Enable confidence-based validation |
| `synward.showQualityScore` | `true` | Show quality score in status bar |
| `synward.synwardPath` | `""` | Path to synward binary (empty = use PATH) |

## Contract Tiers

- **INVIOLABLE** (red): Security, memory safety - always blocked
- **STRICT** (orange): Logic, resources - requires explicit acceptance
- **FLEXIBLE** (green): Style, naming - auto-learned

## Installation

1. Install the Synward CLI: `cargo install synward`
2. Install this extension from VS Code Marketplace
3. Configure `synward.synwardPath` if synward is not in PATH

## Development

```bash
npm install
npm run compile
# Press F5 in VS Code to launch extension development host
```

## Packaging

```bash
npm run package
# Creates synward-0.1.0.vsix
```

## License

MIT
