# Certification Example

## Example 1: Basic Certification

**Input code:**
```rust
fn main() {
    println!("Hello, world!");
}
```

**MCP Tool Call:**
```json
{
  "code": "fn main() {\n    println!(\"Hello, world!\");\n}",
  "language": "rust",
  "signer": "Alice Developer",
  "contracts": []
}
```

**Output:**
```json
{
  "passed": true,
  "certificate": "AETHER-CERT-1.0\nlanguage: rust\nsha256: 3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c183da925bfb169d964d3\nsigned_by: Alice Developer\ntimestamp: 2024-01-15T10:30:00Z\n",
  "signature": "ed25519:QWxpY2UgRGV2ZWxvcGVy",
  "errors": []
}
```

## Example 2: Certification with Contracts

**Input code:**
```rust
/// Adds two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**MCP Tool Call:**
```json
{
  "code": "/// Adds two numbers\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
  "language": "rust",
  "signer": "Bob Reviewer",
  "contracts": ["documentation", "no_panic"]
}
```

**Output:**
```json
{
  "passed": true,
  "certificate": "AETHER-CERT-1.0\nlanguage: rust\nsha256: f4d5e6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5\nsigned_by: Bob Reviewer\ntimestamp: 2024-01-15T11:00:00Z\ncontracts: documentation, no_panic\n",
  "signature": "ed25519:Qm9iIFJldmlld2Vy",
  "errors": []
}
```

## Example 3: Failed Certification

**Input code (with syntax error):**
```rust
fn main( {
    // Missing closing parenthesis
}
```

**MCP Tool Call:**
```json
{
  "code": "fn main( {\n    // Missing closing parenthesis\n}",
  "language": "rust",
  "signer": "Charlie Developer",
  "contracts": []
}
```

**Output:**
```json
{
  "passed": false,
  "certificate": null,
  "signature": null,
  "errors": [
    {
      "id": "parse_error",
      "message": "Expected ) after function parameters",
      "line": 1,
      "column": 9,
      "layer": "syntax",
      "is_new": true
    }
  ]
}
```

## Verification Process

To verify a certificate:

1. **Parse the certificate** to extract fields
2. **Hash the code** and compare with SHA256 in certificate
3. **Verify timestamp** is valid
4. **Check signer** matches expected identity

```bash
# CLI verification
aether verify src/main.rs --cert certificate.txt
```
