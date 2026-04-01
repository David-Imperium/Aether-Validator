# Validation Example

## Example 1: Simple Rust Validation

**Input file (src/main.rs):**
```rust
fn main() {
    let x = 1;
    println!("Hello, world!");
}
```

**MCP Tool Call:**
```json
{
  "file_path": "src/main.rs",
  "language": "rust"
}
```

**Output:**
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

## Example 2: Validation with Contracts

**Input file (src/lib.rs):**
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

unsafe fn dangerous() {
    // unsafe operations
}
```

**MCP Tool Call:**
```json
{
  "file_path": "src/lib.rs",
  "contracts": "no_unsafe,documentation"
}
```

**Output:**
```json
{
  "passed": false,
  "errors": [
    {
      "id": "unsafe_code",
      "message": "Unsafe code block found",
      "line": 5,
      "column": 1,
      "layer": "security",
      "is_new": true
    },
    {
      "id": "missing_doc",
      "message": "Public function 'add' lacks documentation",
      "line": 1,
      "column": 1,
      "layer": "style",
      "is_new": true
    }
  ],
  "warnings": [],
  "language": "rust",
  "layers": {
    "syntax": true,
    "semantic": true,
    "logic": true,
    "security": false,
    "contracts": false,
    "style": false
  }
}
```

## Example 3: Python Validation

**Input file (main.py):**
```python
def greet(name):
    print(f"Hello, {name}!")

if __name__ == "__main__":
    greet("World")
```

**MCP Tool Call:**
```json
{
  "file_path": "main.py",
  "language": "python"
}
```

**Output:**
```json
{
  "passed": true,
  "errors": [],
  "warnings": [],
  "language": "python",
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
