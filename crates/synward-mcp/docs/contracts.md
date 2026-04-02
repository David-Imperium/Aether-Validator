# Validation Contracts

Contracts are rules that validated code must satisfy.

## Built-in Contracts

### Security Contracts

#### no_unsafe
Disallows unsafe code blocks.

**Applies to:** Rust

**Example violation:**
```rust
unsafe {
    // unsafe operations
}
```

#### no_panic
Disallows code that can panic.

**Applies to:** Rust

**Example violation:**
```rust
let x = vec![1, 2, 3];
let y = x[10]; // Can panic
```

### Style Contracts

#### documentation
Requires public items to have documentation.

**Applies to:** Rust, Python, JavaScript

**Example violation:**
```rust
pub fn my_function() { // Missing doc comment
}
```

#### naming
Enforces naming conventions.

**Applies to:** All languages

**Example violation (Rust):**
```rust
fn myFunction() {} // Should be snake_case
```

### Maintainability Contracts

#### complexity
Limits function complexity.

**Applies to:** All languages

**Example violation:**
```rust
fn deeply_nested() {
    if a {
        if b {
            if c {
                if d {
                    if e {
                        // Too deep
                    }
                }
            }
        }
    }
}
```

## Using Contracts

### Via MCP Tool
```json
{
  "file_path": "src/main.rs",
  "contracts": "no_unsafe,documentation"
}
```

### Via CLI
```bash
synward validate src/main.rs --contracts no_unsafe,documentation
```

## Custom Contracts

You can define custom contracts in a `.synward/contracts/` directory.

```rust
// .synward/contracts/my_contract.rs
pub fn check(code: &str, ast: &ASTNode) -> Vec<ValidationError> {
    // Custom validation logic
}
```
