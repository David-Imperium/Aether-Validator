# Synward Supported Languages

## Overview

Synward validates code quality across multiple programming languages using tree-sitter parsers and custom analyzers.

## Currently Supported (23 public + Prism private)

| Language | Extensions | Parser | Status |
|----------|------------|--------|--------|
| Rust | `.rs` | tree-sitter | ✅ Full |
| Python | `.py`, `.pyw` | tree-sitter | ✅ Full |
| JavaScript | `.js`, `.jsx`, `.mjs`, `.cjs` | tree-sitter | ✅ Full |
| TypeScript | `.ts`, `.tsx`, `.mts`, `.cts` | tree-sitter | ✅ Full |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.h`, `.hxx` | tree-sitter | ✅ Full |
| C | `.c`, `.h` | tree-sitter | ✅ Full |
| Go | `.go` | tree-sitter | ✅ Full |
| Java | `.java` | tree-sitter | ✅ Full |
| Lua | `.lua` | tree-sitter | ✅ Full |
| Bash | `.sh`, `.bash`, `.zsh`, `.ksh` | tree-sitter | ✅ Full |
| Lex | `.lex` | custom | ✅ Full |
| Prism | `.prism` | tree-sitter | ✅ Full |
| GLSL | `.frag`, `.vert`, `.comp`, `.tesc`, `.tese`, `.geom`, `.glsl` | tree-sitter | ✅ Full |
| CSS | `.css` | tree-sitter | ✅ Full |
| HTML | `.html`, `.htm` | tree-sitter | ✅ Full |
| JSON | `.json` | tree-sitter | ✅ Full |
| YAML | `.yaml`, `.yml` | tree-sitter | ✅ Full |
| TOML | `.toml` | tree-sitter-toml-ng | ✅ Full |
| CMake | `.cmake` | tree-sitter-cmake | ✅ Full |
| CUDA | `.cu`, `.cuh` | tree-sitter-cuda | ✅ Full |
| SQL | `.sql`, `.ddl`, `.dml` | tree-sitter | ✅ Full |
| GraphQL | `.graphql`, `.gql` | tree-sitter | ✅ Full |
| Markdown | `.md`, `.markdown`, `.mdown`, `.mkd` | tree-sitter | ✅ Full |
| Notebook | `.ipynb` | tree-sitter | ✅ Full |

## Unsupported Languages Fallback

Synward provides **baseline security validation** even for languages without full AST support. The `FallbackSecurityLayer` applies regex-based pattern matching to detect critical vulnerabilities.

### Supported Fallback Languages

Languages not in the "Currently Supported" list receive security-only validation:
- Kotlin (`.kt`, `.kts`)
- Ruby (`.rb`, `.rake`)
- PHP (`.php`)
- Swift (`.swift`)
- C# (`.cs`)
- Scala (`.scala`, `.sc`)
- R (`.r`, `.R`)
- And any other text-based source files

### Security Patterns (SEC001-SEC010)

| ID | Pattern | Severity | Description |
|----|---------|----------|-------------|
| SEC001 | `password = "` | Error | Hardcoded password detected |
| SEC002 | `api_key = "` | Error | Hardcoded API key detected |
| SEC003 | `secret_key = "` | Error | Hardcoded secret key detected |
| SEC004 | `token = "` | Error | Hardcoded token detected |
| SEC005 | Raw SQL query | Warning | Raw SQL query detected |
| SEC006 | File I/O operations | Warning | File read/write - verify path is not user-controlled |
| SEC007 | Timing comparison | Warning | Potential timing attack on secrets comparison |
| SEC008 | `eval()`, `exec()` | Error | Code execution - potential injection |
| SEC009 | Insecure crypto | Warning | Weak cryptography usage |
| SEC010 | SQL injection | Error | Formatted SQL query - use parameterized queries |

### Example Output (Kotlin)

```kotlin
// File: config.kt
val password = "supersecret123"  // ← SEC001
val apiKey = "sk-abc123xyz"      // ← SEC002
Runtime.getRuntime().exec(cmd)   // ← SEC004
```

**Synward Output:**
```
╔══════════════════════════════════════════════════════════════╗
║  SYNWARD VALIDATION — config.kt                               ║
╠══════════════════════════════════════════════════════════════╣
║  ⚠ [SEC001] Hardcoded password detected                      ║
║     Line 2: val password = "supersecret123"                  ║
║     → Use environment variables or secret management         ║
║                                                              ║
║  ⚠ [SEC002] Hardcoded API key detected                       ║
║     Line 3: val apiKey = "sk-abc123xyz"                      ║
║     → Store secrets in environment variables                 ║
║                                                              ║
║  ⚠ [SEC004] Command execution - verify inputs are sanitized  ║
║     Line 4: Runtime.getRuntime().exec(cmd)                   ║
║     → Validate and sanitize all command inputs               ║
╚══════════════════════════════════════════════════════════════╝
```

### Example Output (Ruby)

```ruby
# File: database.rb
DB.execute("SELECT * FROM users WHERE id = " + user_id)  # ← SEC010
password = "admin123"                                    # ← SEC001
```

**Synward Output:**
```
╔══════════════════════════════════════════════════════════════╗
║  SYNWARD VALIDATION — database.rb                              ║
╠══════════════════════════════════════════════════════════════╣
║  ✗ [SEC010] SQL INJECTION: String concatenation in SQL       ║
║     Line 2: DB.execute("SELECT * FROM users WHERE id = " ... ║
║     → Use parameterized queries                               ║
║                                                              ║
║  ✗ [SEC001] Hardcoded password detected                      ║
║     Line 3: password = "admin123"                            ║
║     → Use environment variables or secret management         ║
╚══════════════════════════════════════════════════════════════╝
```

### Limitations

Fallback mode provides **security-only** validation:
- No AST-based structural analysis
- No language-specific idioms detection
- No style/formatting rules
- No semantic validation

### Adding Full Language Support

To add complete AST-based validation for a new language, see:
**[CUSTOM_LANGUAGE_SUPPORT.md](./CUSTOM_LANGUAGE_SUPPORT.md)**

---

## Not Yet Supported (Technical Issues)

| Language | Extensions | Issue | Possible Solution |
|----------|------------|-------|-------------------|
| SCSS | `.scss`, `.sass` | MSVC incompatible (uses GCC flags) | Wait for upstream fix or use Zig compiler |

## Planned (Custom Implementation)

| Language | Extensions | Parser | Priority | Notes |
|----------|------------|--------|----------|-------|
| Cython | `.pyx`, `.pxd` | tree-sitter | Low | Python extensions |

## Not Planned

| Language | Reason |
|----------|--------|
| Odin | Not used by user |
| Fortran | Legacy, low priority |

## File Statistics (User Projects)

### lex-exploratory
- JavaScript: 9,290 files
- Python: 3,461 files
- TypeScript: 1,510 files
- Rust: 171 files

### Imperium 2.0
- Python: 16,143 files
- C/C++ headers: 20,196 files
- C++: 5,973 files
- C: 1,535 files
- GLSL shaders: 3,567 files
- CMake: 7,891 files

### Ai Personale
- Python: 14,862 files
- JavaScript: 11,398 files
- TypeScript: 1,742 files
- CSS: 256 files

## Adding a New Language

1. Add tree-sitter dependency to `crates/synward-parsers/Cargo.toml`
2. Create parser module in `crates/synward-parsers/src/<lang>.rs`
3. Register parser in `crates/synward-parsers/src/lib.rs`
4. Add language to `LANGUAGES` constant in `crates/synward-cli/src/platforms.rs`
5. Update this document

### Example: Adding CSS

```toml
# Cargo.toml
tree-sitter-css = "0.23"
```

```rust
// css.rs
use crate::tree_sitter::{TreeSitterConverter, parse_source, languages};

pub struct CssParser;

impl Parser for CssParser {
    fn language(&self) -> &str { "css" }
    
    fn extensions(&self) -> &[&str] { &["css"] }
    
    fn parse(&self, source: &str) -> ParseResult<AST> {
        let tree = parse_source(languages::css(), source)?;
        TreeSitterConverter::convert(&tree, source)
    }
}
```

## Validation Rules by Language

Each language has specific validation rules. See `crates/synward-validation/src/rules/` for implementation details.
