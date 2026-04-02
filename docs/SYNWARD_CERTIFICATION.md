# Synward — Certification System

**Version:** 0.2.0  
**Related:** [SYNWARD_MASTER_DESIGN.md](./SYNWARD_MASTER_DESIGN.md)

---

## Overview

The Certification System provides cryptographic proof that code has been validated by Synward. A certificate is a verifiable, tamper-proof record that code meets specific quality standards.

**Purpose:** Enable trust in AI-generated code through cryptographic verification.

---

## Why Certification?

### The Trust Problem

```
Without Certification:
┌─────────┐      ┌─────────┐      ┌─────────┐
│   AI    │─────▶│  Code   │─────▶│Production│
│ Agent   │      │ (maybe  │      │  ???     │
└─────────┘      │  good?) │      └─────────┘
                 └─────────┘

With Certification:
┌─────────┐      ┌─────────┐      ┌─────────┐      ┌─────────┐
│   AI    │─────▶│ Synward  │─────▶│Certified│─────▶│Production│
│ Agent   │      │ Validate│      │  Code   │      │  Safe!   │
└─────────┘      └─────────┘      └─────────┘      └─────────┘
                        │
                        ▼
                 ┌─────────────┐
                 │Certificate  │
                 │(verifiable) │
                 └─────────────┘
```

### Use Cases

1. **CI/CD Gates** — Only certified code can merge/deploy
2. **Audit Trail** — Prove what was validated, when, by whom
3. **Commercial Trust** — "Certified by Synward" as quality mark
4. **Regulatory Compliance** — Verifiable quality for regulated industries
5. **Supply Chain Security** — Verify dependencies are certified

---

## Certificate Structure

### Full Certificate

```json
{
  "version": "1.0",
  "certificate_id": "SYNWARD-2026-03-08-ABC12345",
  
  "timestamp": "2026-03-08T23:45:00Z",
  "expires": "2027-03-08T23:45:00Z",
  
  "subject": {
    "type": "source_code",
    "language": "cpp",
    "file": "src/enemy.cpp",
    "hash": {
      "algorithm": "SHA-256",
      "value": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    "size_bytes": 4521,
    "line_count": 156
  },
  
  "validation": {
    "passed": true,
    "layers": ["syntax", "semantic", "logic", "architecture", "style"],
    "contracts_applied": [
      "CPP-MEM-001",
      "CPP-MEM-002",
      "CPP-SEC-001",
      "LEX-GP-001"
    ],
    "violations": [],
    "metrics": {
      "errors": 0,
      "warnings": 0,
      "infos": 2,
      "score": 98
    },
    "duration_ms": 45
  },
  
  "agent": {
    "type": "claude-3-opus",
    "version": "20240229",
    "session_id": "session-abc123",
    "prompt_hash": "f4d5e6..."
  },
  
  "project": {
    "name": "lex-game",
    "version": "0.5.0",
    "commit": "a1b2c3d4e5f6..."
  },
  
  "issuer": {
    "name": "Synward",
    "version": "0.1.0",
    "instance_id": "synward-prod-01"
  },
  
  "signature": {
    "algorithm": "Ed25519",
    "public_key": "MCowBQYDK2VwAy...",
    "value": "304402200a3b4c5d6e7f..."
  }
}
```

### Minimal Certificate (for embedding)

```json
{
  "v": "1.0",
  "id": "SYNWARD-2026-03-08-ABC12345",
  "ts": "2026-03-08T23:45:00Z",
  "hash": "e3b0c44298fc...",
  "pass": true,
  "sig": "304402200a3b..."
}
```

---

## Certificate Fields Reference

### Root Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | Yes | Certificate format version |
| `certificate_id` | string | Yes | Unique certificate identifier |
| `timestamp` | ISO8601 | Yes | When certificate was issued |
| `expires` | ISO8601 | No | Certificate expiration (optional) |
| `subject` | object | Yes | What is being certified |
| `validation` | object | Yes | Validation details |
| `agent` | object | No | AI agent information (if applicable) |
| `project` | object | No | Project context |
| `issuer` | object | Yes | Certificate issuer |
| `signature` | object | Yes | Cryptographic signature |

### Subject Fields

| Field | Type | Description |
|-------|------|-------------|
| `type` | enum | source_code, config, data, binary |
| `language` | string | Programming language |
| `file` | string | File path (relative to project) |
| `hash.algorithm` | string | Hash algorithm used |
| `hash.value` | string | Hash of the content |
| `size_bytes` | number | File size |
| `line_count` | number | Number of lines |

### Validation Fields

| Field | Type | Description |
|-------|------|-------------|
| `passed` | bool | Overall pass/fail |
| `layers` | string[] | Validation layers executed |
| `contracts_applied` | string[] | Contract IDs applied |
| `violations` | object[] | Any violations found |
| `metrics` | object | Quality metrics |
| `duration_ms` | number | Validation duration |

### Agent Fields

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Agent type (claude, gpt, cursor, etc.) |
| `version` | string | Agent version |
| `session_id` | string | Agent session identifier |
| `prompt_hash` | string | Hash of the original prompt |

### Signature Fields

| Field | Type | Description |
|-------|------|-------------|
| `algorithm` | string | Signature algorithm |
| `public_key` | string | Public key (base64) |
| `value` | string | Signature value (base64) |

---

## Cryptographic Details

### Signing Algorithm

Synward uses **Ed25519** for digital signatures:

- Fast signature generation and verification
- Small key sizes (32 bytes private, 32 bytes public)
- 64-byte signatures
- Strong security guarantees

### Key Management

```cpp
namespace synward::certification {

class KeyManager {
public:
    // Generate new key pair
    static KeyPair generateKeyPair();
    
    // Load/save keys
    static KeyPair loadKeyPair(const std::string& path);
    static void saveKeyPair(const KeyPair& keys, const std::string& path);
    
    // Get public key for verification
    static PublicKey loadPublicKey(const std::string& path);
};

struct KeyPair {
    std::vector<uint8_t> privateKey;
    std::vector<uint8_t> publicKey;
};

}
```

### Signature Process

```
1. Build canonical JSON of certificate (without signature field)
2. Hash with SHA-256
3. Sign hash with Ed25519 private key
4. Add signature to certificate
```

```cpp
namespace synward::certification {

class Signer {
public:
    Signer(const KeyPair& keys);
    
    // Sign a certificate
    void sign(Certificate& cert);
    
    // Verify a certificate
    bool verify(const Certificate& cert) const;
    
private:
    KeyPair m_keys;
    
    // Create canonical JSON representation
    std::string canonicalize(const Certificate& cert);
};

}
```

---

## Certificate Generation

### Certificate Generator

```cpp
namespace synward::certification {

class CertificateGenerator {
public:
    CertificateGenerator(const KeyPair& keys);
    
    // Generate certificate for validation result
    Certificate generate(
        const ValidationResult& result,
        const ValidationContext& ctx
    );
    
    // Generate minimal certificate
    Certificate generateMinimal(
        const ValidationResult& result,
        const ValidationContext& ctx
    );
    
    // Verify certificate
    bool verify(const Certificate& cert) const;
    
private:
    KeyPair m_keys;
    std::unique_ptr<Signer> m_signer;
    IdGenerator m_idGenerator;
};

struct Certificate {
    std::string version;
    std::string certificateId;
    std::chrono::system_clock::time_point timestamp;
    std::optional<std::chrono::system_clock::time_point> expires;
    
    Subject subject;
    ValidationInfo validation;
    std::optional<AgentInfo> agent;
    std::optional<ProjectInfo> project;
    IssuerInfo issuer;
    Signature signature;
    
    // Serialization
    std::string toJson() const;
    std::string toJsonMinified() const;
    static Certificate fromJson(const std::string& json);
};

}
```

### Certificate ID Format

```
SYNWARD-YYYY-MM-DD-XXXXXXXX

SYNWARD      — Fixed prefix
YYYY-MM-DD  — Date of issuance
XXXXXXXX    — 8-character random hex

Example: SYNWARD-2026-03-08-4F3A2B91
```

---

## Certificate Storage

### Storage Options

| Storage | Use Case |
|---------|----------|
| **File** | Local development, git-committed |
| **Database** | Centralized team/server |
| **Blockchain** | Immutable public record (advanced) |

### File Storage

```
project/
├── src/
│   └── enemy.cpp
├── .synward/
│   └── certificates/
│       └── 2026-03/
│           └── SYNWARD-2026-03-08-4F3A2B91.json
```

### Certificate Registry

```cpp
namespace synward::certification {

class CertificateRegistry {
public:
    // Store certificate
    void store(const Certificate& cert);
    
    // Retrieve by ID
    std::optional<Certificate> getById(const std::string& id);
    
    // Retrieve by file hash
    std::vector<Certificate> getByHash(const std::string& hash);
    
    // Retrieve latest for file
    std::optional<Certificate> getLatestForFile(const std::string& file);
    
    // Verify chain (if using certificate hierarchies)
    bool verifyChain(const Certificate& cert);
    
private:
    StorageBackend m_storage;
    CertificateIndex m_index;
};

}
```

---

## Verification

### Verification Process

```
1. Parse certificate JSON
2. Verify signature
3. Check timestamp (not expired)
4. Verify content hash matches actual code
5. Check issuer is trusted
6. Return verification result
```

### Verifier

```cpp
namespace synward::certification {

struct VerificationResult {
    bool valid;
    std::vector<std::string> errors;
    std::vector<std::string> warnings;
    
    // Details
    std::string certificateId;
    std::string subjectFile;
    std::chrono::system_clock::time_point issuedAt;
    bool hashMatches;
    bool signatureValid;
    bool notExpired;
    bool issuerTrusted;
};

class Verifier {
public:
    Verifier();
    
    // Add trusted public keys
    void addTrustedKey(const std::string& issuer, const PublicKey& key);
    
    // Verify certificate
    VerificationResult verify(
        const Certificate& cert,
        const std::string& actualContent = ""
    );
    
    // Verify certificate file
    VerificationResult verifyFile(const std::string& certPath);
    
private:
    std::map<std::string, PublicKey> m_trustedKeys;
    std::unique_ptr<Signer> m_signer;
};

}
```

---

## Audit Log

All certificate operations are logged for auditing.

### Audit Entry

```json
{
  "timestamp": "2026-03-08T23:45:00Z",
  "event": "certificate_issued",
  "certificate_id": "SYNWARD-2026-03-08-ABC12345",
  "file": "src/enemy.cpp",
  "agent": "claude-3-opus",
  "validation_result": "pass",
  "issuer": "synward-prod-01",
  "ip_address": "192.168.1.100",
  "user": "david"
}
```

### Audit Logger

```cpp
namespace synward::certification {

enum class AuditEvent {
    CertificateIssued,
    CertificateVerified,
    CertificateExpired,
    CertificateRevoked,
    KeyGenerated,
    KeyRotated
};

struct AuditEntry {
    std::chrono::system_clock::time_point timestamp;
    AuditEvent event;
    std::string certificateId;
    std::map<std::string, std::string> details;
};

class AuditLogger {
public:
    void log(AuditEvent event, const std::map<std::string, std::string>& details);
    
    // Query audit log
    std::vector<AuditEntry> query(const AuditQuery& query);
    
    // Export for compliance
    void exportToJson(const std::string& path);
    void exportToCsv(const std::string& path);
    
private:
    StorageBackend m_storage;
};

}
```

---

## Certificate Revocation

Sometimes certificates need to be revoked (e.g., vulnerability discovered).

### Revocation List

```json
{
  "version": "1.0",
  "updated": "2026-03-08T12:00:00Z",
  "revoked": [
    {
      "certificate_id": "SYNWARD-2026-03-07-XYZ12345",
      "reason": "security_vulnerability",
      "revoked_at": "2026-03-08T11:00:00Z",
      "details": "CVE-2026-XXXXX"
    }
  ]
}
```

### Revocation Checker

```cpp
namespace synward::certification {

class RevocationChecker {
public:
    // Load revocation list
    void loadFromUrl(const std::string& url);
    void loadFromFile(const std::string& path);
    
    // Check if certificate is revoked
    bool isRevoked(const std::string& certificateId);
    std::optional<RevocationInfo> getRevocationInfo(const std::string& certificateId);
    
private:
    std::map<std::string, RevocationInfo> m_revoked;
};

}
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/synward-verify.yml
name: Synward Verification

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Synward
        run: |
          curl -sSL https://get.synward.dev | sh
          
      - name: Verify Certificates
        run: |
          synward verify-changed --fail-on-missing
          
      - name: Validate New Code
        run: |
          synward validate src/ --certify
          
      - name: Upload Certificates
        uses: actions/upload-artifact@v4
        with:
          name: synward-certificates
          path: .synward/certificates/
```

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Get changed files
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(cpp|h|rs|py)$')

if [ -z "$CHANGED_FILES" ]; then
    exit 0
fi

# Validate and certify
echo "Running Synward validation..."
for FILE in $CHANGED_FILES; do
    RESULT=$(synward validate "$FILE" --certify --output json)
    PASSED=$(echo "$RESULT" | jq -r '.passed')
    
    if [ "$PASSED" != "true" ]; then
        echo "Validation failed for $FILE:"
        echo "$RESULT" | jq -r '.violations[]'
        exit 1
    fi
done

echo "All files validated and certified."
exit 0
```

---

## API Endpoints

### HTTP API

```
POST /api/v1/certify
Content-Type: application/json

{
  "source": "... code ...",
  "language": "cpp",
  "contracts": ["CPP-MEM-001"],
  "agent": {
    "type": "claude",
    "session_id": "..."
  }
}

Response:
{
  "certificate": { ... },
  "validation": { ... }
}
```

```
POST /api/v1/verify
Content-Type: application/json

{
  "certificate": { ... },
  "source": "... code ..."
}

Response:
{
  "valid": true,
  "checks": {
    "signature": true,
    "hash": true,
    "not_expired": true,
    "not_revoked": true
  }
}
```

### MCP Tool

```json
{
  "tool": "synward_certify",
  "params": {
    "source": "...",
    "language": "cpp"
  }
}
```

---

## Certificate Levels

### Certification Tiers

| Level | Requirements | Use Case |
|-------|--------------|----------|
| `basic` | Syntax only | Quick validation |
| `standard` | Syntax + Semantic | Regular development |
| `full` | All 5 layers | Production release |
| `enterprise` | Full + custom audits | Regulated industries |

### Configuration

```yaml
# .synward/config.yaml
certification:
  level: full
  
  levels:
    basic:
      layers: [syntax]
    standard:
      layers: [syntax, semantic]
    full:
      layers: [syntax, semantic, logic, architecture, style]
    enterprise:
      layers: [syntax, semantic, logic, architecture, style]
      additional_checks:
        - security_audit
        - compliance_check
```

---

## Summary

The Certification System provides:

1. **Cryptographic Proof** — Ed25519 signatures
2. **Verifiable Records** — Anyone can verify
3. **Audit Trail** — Complete history
4. **Revocation** — Handle compromised certificates
5. **CI/CD Integration** — Automated gates
6. **Multiple Levels** — Basic to enterprise

Certificates transform Synward from a validation tool into a **trust infrastructure** for AI-generated code.
