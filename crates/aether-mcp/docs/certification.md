# Code Certification

Aether can generate cryptographic certificates for validated code.

## What is Certification?

Certification provides:
1. Proof of validation
2. Cryptographic signature
3. Timestamp
4. Traceability

## Certificate Format

```
AETHER-CERT-1.0
language: rust
sha256: abc123...
signed_by: Developer Name
timestamp: 2024-01-15T10:30:00Z
```

## How to Certify

### Via MCP Tool
```json
{
  "code": "fn main() { println!(\"Hello\"); }",
  "language": "rust",
  "signer": "Developer Name",
  "contracts": ["no_unsafe"]
}
```

### Via CLI
```bash
aether certify src/main.rs --signer "Developer Name"
```

## Verification

Certificates can be verified:
1. Code hash matches SHA256 in certificate
2. Signature is valid
3. No validation errors

## Use Cases

- **Code review**: Certify reviewed code
- **CI/CD**: Require certificates for deployment
- **Compliance**: Document code quality checks
- **Auditing**: Track validation history

## Best Practices

1. Always validate before certifying
2. Use meaningful signer names
3. Include relevant contracts
4. Store certificates with code history
