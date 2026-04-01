# Aether VS Code Extension

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
| `Aether: Validate Current File` | Validate the active file |
| `Aether: Validate Project` | Validate all files in workspace |
| `Aether: Accept Violation` | Accept a violation with documented reason |
| `Aether: Show Compliance Status` | Open compliance dashboard |
| `Aether: Analyze Drift` | Analyze drift for current file |

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `aether.enableValidation` | `true` | Enable automatic validation on save |
| `aether.validationMode` | `balanced` | Validation strictness (strict/balanced/lenient) |
| `aether.dubbiosoMode` | `true` | Enable confidence-based validation |
| `aether.showQualityScore` | `true` | Show quality score in status bar |
| `aether.aetherPath` | `""` | Path to aether binary (empty = use PATH) |

## Contract Tiers

- **INVIOLABLE** (red): Security, memory safety - always blocked
- **STRICT** (orange): Logic, resources - requires explicit acceptance
- **FLEXIBLE** (green): Style, naming - auto-learned

## Installation

1. Install the Aether CLI: `cargo install aether`
2. Install this extension from VS Code Marketplace
3. Configure `aether.aetherPath` if aether is not in PATH

## Development

```bash
npm install
npm run compile
# Press F5 in VS Code to launch extension development host
```

## Packaging

```bash
npm run package
# Creates aether-0.1.0.vsix
```

## License

MIT
