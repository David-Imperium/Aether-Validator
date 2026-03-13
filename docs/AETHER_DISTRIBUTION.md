# Aether — Distribution & Pricing

**Version:** 0.1.0  
**Related:** [AETHER_MASTER_DESIGN.md](./AETHER_MASTER_DESIGN.md)

---

## Overview

This document defines how Aether is distributed, priced, and accessed by users.

---

## Distribution Channels

### Primary Channels

| Channel | Format | Target Users |
|---------|--------|---------------|
| **Website** | aether.dev | All users |
| **Docker Hub** | aether/aether | CI/CD, DevOps |
| **Package Managers** | brew, winget, apt | Developers |
| **Cloud API** | api.aether.dev | Enterprise, SaaS users |

### Download Options

```bash
# Direct download (aether.dev/downloads)
curl -sSL https://aether.dev/install.sh | sh

# Homebrew (macOS)
brew install aether

# winget (Windows)
winget install aether

# Docker
docker pull aether/aether:latest
```

---

## Pricing Tiers

### Tier Comparison

| Feature | Free | Pro | Enterprise |
|---------|------|-----|------------|
| **Price** | $0 | $29/mo | $199/mo |
| **Validations/month** | 1,000 | 10,000 | Unlimited |
| **Certifications/month** | 500 | 5,000 | Unlimited |
| **Languages** | C++, Rust | + Lex | + Python, TS, Custom |
| **CLI Access** | ✅ | ✅ | ✅ |
| **HTTP API** | ❌ | ✅ | ✅ |
| **Cloud API** | ❌ | ❌ | ✅ |
| **Custom Contracts** | 5 | 20 | Unlimited |
| **Certificate Signing** | Shared key | Shared key | Custom key |
| **Revocation (CRL)** | ❌ | ✅ | ✅ |
| **Priority Support** | Community | Email | Dedicated + SLA |
| **On-Premise** | ❌ | ❌ | ✅ |
| **Audit Logs** | 7 days | 30 days | Unlimited + SIEM |
| **SSO/SAML** | ❌ | ❌ | ✅ |

---

## Free Tier

**Target:** Individual developers, open source projects, students

**What's Included:**
- CLI tool (full functionality)
- 1,000 validations per month
- 500 certifications per month
- C++, Rust, Lex language support
- 5 custom contracts
- Community support (GitHub discussions)

**What's NOT Included:**
- HTTP API access
- Cloud API access
- Custom signing keys (uses Aether's shared key)
- Revocation checking
- Priority support

**Use Case:**
```bash
# Free tier usage
aether validate src/ --contracts ./my-contracts/
# Works up to 1000 times/month
```

---

## Pro Tier — $29/month

**Target:** Professional developers, small teams, startups

**What's Included:**
- Everything in Free, plus:
- 10,000 validations per month
- 5,000 certifications per month
- HTTP API access
- 20 custom contracts
- Revocation checking (CRL)
- 30-day audit log retention
- Email support (48h response)

**What's NOT Included:**
- Cloud API access
- Custom signing keys
- On-premise deployment
- SSO/SAML

**Use Case:**
```bash
# Pro tier usage
aether validate src/ --contracts ./my-contracts/

# HTTP API access
curl -X POST https://api.aether.dev/v1/validate \
  -H "Authorization: Bearer $AETHER_API_KEY" \
  -d '{"source": "...", "language": "cpp"}'
```

---

## Enterprise Tier — $199/month

**Target:** Companies, enterprises, regulated industries

**What's Included:**
- Everything in Pro, plus:
- Unlimited validations
- Unlimited certifications
- All languages (C++, Rust, Lex, Python, TypeScript, Custom)
- Cloud API access
- Unlimited custom contracts
- Custom signing key (your brand on certificates)
- Revocation with custom CRL
- On-premise deployment option
- Unlimited audit logs + SIEM integration
- SSO/SAML integration
- Dedicated support + 4h SLA
- Annual security audit report
- Custom contract development (upon request)

**Use Case:**
```bash
# Enterprise tier - on-premise
aether serve --port 8080 \
  --signing-key /secure/company-key.pem \
  --contracts /company/contracts/

# Or cloud API
curl -X POST https://api.aether.dev/v1/certify \
  -H "Authorization: Bearer $ENTERPRISE_API_KEY" \
  -d '{"source": "...", "language": "cpp", "sign_with": "company_key"}'
```

---

## Annual Plans (Discount)

| Plan | Monthly Equivalent | Annual Price | Savings |
|------|-------------------|--------------|---------|
| Pro Annual | $29/mo | $290/year | 17% |
| Enterprise Annual | $199/mo | $1,990/year | 17% |

---

## Feature Gates

### Validation Limit Enforcement

```cpp
namespace aether::licensing {

class LicenseManager {
public:
    bool canValidate(const License& license) {
        if (license.tier == Tier::Enterprise) return true;
        
        auto usage = getUsageThisMonth(license);
        auto limit = getValidationLimit(license.tier);
        
        return usage.validations < limit;
    }
    
    void recordValidation(const License& license) {
        m_usageTracker.increment(license.id, UsageType::Validation);
    }
    
private:
    UsageTracker m_usageTracker;
};

}
```

### API Access Gate

```cpp
bool canUseHttpApi(const License& license) {
    return license.tier >= Tier::Pro;
}

bool canUseCloudApi(const License& license) {
    return license.tier >= Tier::Enterprise;
}
```

---

## License Keys

### Format

```
AETHER-XXXX-XXXX-XXXX-XXXX
```

Example: `AETHER-PRO1-ABCD-EFGH-IJKL`

### Validation

```cpp
bool validateLicenseKey(const std::string& key) {
    // Format check
    if (!std::regex_match(key, LICENSE_KEY_PATTERN)) {
        return false;
    }
    
    // Online validation (for Pro+)
    if (isOnlineKey(key)) {
        return validateWithServer(key);
    }
    
    // Offline validation (hash check)
    return verifyKeyHash(key);
}
```

### Activation Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Purchase  │────▶│  Receive    │────▶│  Activate   │
│   (Web)     │     │  License Key│     │  (CLI/Web)  │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                    ┌──────────────────────────┴──────────────────────────┐
                    │                                                     │
                    ▼                                                     ▼
             ┌─────────────┐                                       ┌─────────────┐
             │    Free     │                                       │  Pro/Ent    │
             │  (Offline)  │                                       │  (Online)    │
             └─────────────┘                                       └─────────────┘
```

---

## Usage Tracking

### Free Tier (Offline)

Free tier uses local tracking:

```yaml
# ~/.aether/usage.yaml
month: 2026-03
validations: 847
certifications: 423
last_reset: 2026-03-01T00:00:00Z
```

### Pro/Enterprise (Online)

Pro and Enterprise tiers track usage on server:

```json
{
  "license_key": "AETHER-PRO1-XXXX-XXXX-XXXX",
  "period": "2026-03",
  "validations": 5234,
  "certifications": 2891,
  "limit": 10000
}
```

---

## Billing

### Payment Processing

- **Provider:** Stripe
- **Methods:** Credit card, PayPal, Bank transfer (Enterprise)
- **Currency:** USD, EUR, GBP

### Invoice

Enterprise customers receive monthly invoices:

```
┌─────────────────────────────────────────────────────────────┐
│                      AETHER INVOICE                         │
│                                                             │
│  Customer: Acme Corp                    Date: 2026-03-01   │
│  License: ENTERPRISE-XXXX-XXXX-XXXX-XXXX                   │
│                                                             │
│  ─────────────────────────────────────────────────────────  │
│                                                             │
│  Aether Enterprise (Annual)                    $1,990.00    │
│                                                             │
│  Add-ons:                                                   │
│  - Custom contract development (5 hrs)          $500.00    │
│                                                             │
│  ─────────────────────────────────────────────────────────  │
│                                                             │
│  Subtotal:                                    $2,490.00    │
│  Tax (if applicable):                             $0.00    │
│                                                             │
│  TOTAL:                                       $2,490.00    │
│                                                             │
│  Due: Upon receipt                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Refund Policy

| Period | Refund |
|--------|--------|
| First 14 days | 100% refund, no questions |
| 15-30 days | 50% refund |
| After 30 days | No refund, but can cancel renewal |

---

## Upgrades and Downgrades

### Upgrade

```bash
# Upgrade from Pro to Enterprise
aether license upgrade --to enterprise

# Prorated charge for remaining period
```

### Downgrade

```bash
# Downgrade from Enterprise to Pro
aether license downgrade --to pro

# Features removed at end of billing period
```

---

## Enterprise Add-ons

| Add-on | Price | Description |
|--------|-------|-------------|
| **Custom Contract Development** | $100/hr | We write contracts for your specific needs |
| **On-Premise Setup** | $2,500 one-time | Dedicated engineer helps set up on-premise |
| **Annual Security Audit** | $5,000/year | Third-party security audit + report |
| **SLA Upgrade** | $500/mo | 1-hour response time SLA |
| **Training Session** | $1,000/session | 4-hour training for your team |
| **Custom Language Adapter** | $10,000 one-time | We add support for your custom language |

---

## Educational Discount

| Type | Discount |
|------|----------|
| Student (with .edu email) | 50% off Pro |
| University (department license) | 70% off Enterprise |
| Open Source Projects | Free Pro (apply with GitHub link) |

---

## Trial

| Tier | Trial |
|------|-------|
| Pro | 14 days free |
| Enterprise | 30 days free + demo call |

---

## Partner Program

### Referral

| Referrals | Reward |
|-----------|--------|
| 1-5 | 1 month free Pro |
| 6-10 | 3 months free Pro |
| 11+ | 1 year free Pro + $500 credit |

### Reseller

| Level | Discount | Requirements |
|-------|----------|--------------|
| Bronze | 15% | 5+ customers/year |
| Silver | 25% | 20+ customers/year |
| Gold | 35% | 50+ customers/year |

---

## Summary

Aether uses a **freemium model** with clear upgrade path:

| Tier | Price | Target |
|------|-------|--------|
| **Free** | $0 | Individual developers |
| **Pro** | $29/mo | Professionals, small teams |
| **Enterprise** | $199/mo | Companies, enterprises |

**Key differentiators:**
- Free: CLI only, rate limited
- Pro: + HTTP API, higher limits
- Enterprise: + Cloud API, on-premise, unlimited, support

This makes Aether accessible to everyone while providing clear value for paying customers.
