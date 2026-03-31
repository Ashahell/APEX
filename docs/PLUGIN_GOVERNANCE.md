# Plugin Governance and Marketplace

## Overview
- This document defines the governance framework for APEX plugins/skills, including submission, review, trust levels, and revocation policies.
- It ensures plugin quality, security, and compatibility with the APEX ecosystem.

## Plugin Lifecycle

### 1. Development
- Plugins follow the SKILL.md format with YAML frontmatter
- Must include: name, version, description, author, tier, inputSchema, outputSchema
- Must pass security scanning before submission

### 2. Submission
- Submit via Skills Hub or direct API
- Required metadata:
  - Plugin name (unique)
  - Version (semver)
  - Description
  - Author information
  - Category/tags
  - Input/output schemas
  - Security declarations

### 3. Review Process
- **Automated Checks**:
  - Security scan for dangerous patterns
  - Schema validation
  - Dependency audit
  - Performance benchmark
- **Manual Review** (for Trusted/Verified):
  - Code review by maintainers
  - Security assessment
  - Compatibility testing

### 4. Publication
- Published to Skills Hub marketplace
- Assigned trust level based on review
- Available for installation

### 5. Updates
- Version bumps require re-review
- Security patches expedited
- Breaking changes require major version bump

### 6. Deprecation
- 30-day notice before deprecation
- Migration guide provided
- Old versions remain available for 90 days

## Trust Levels

| Level | Requirements | Permissions | Badge |
|-------|-------------|-------------|-------|
| **Community** | Basic submission, automated checks pass | Install, execute | 🟢 |
| **Trusted** | Manual review passed, security audit | Install, execute, recommend | 🔵 |
| **Verified** | Official APEX team verification | Install, execute, featured, pre-installed | ✅ |

### Trust Level Progression

```
Community → Trusted → Verified
   (30 days)   (90 days)
```

- **Community → Trusted**: 30 days without security issues, 100+ installs
- **Trusted → Verified**: 90 days without issues, 1000+ installs, team review

## Security Requirements

### Mandatory
- No hardcoded secrets or credentials
- Input validation for all parameters
- Error handling without information leakage
- Resource limits (memory, CPU, timeout)

### Prohibited
- Network access without explicit declaration
- File system access outside sandbox
- Process spawning without approval
- Cryptocurrency mining
- Data exfiltration

### Security Scanning
- All plugins scanned for:
  - Known vulnerability patterns
  - Dangerous API calls
  - Obfuscated code
  - Network callbacks
  - File system traversal

## Plugin Signing

### ed25519 Signatures
- All official plugins signed with ed25519
- Signature verification on installation
- Signature expiry: 365 days
- Key rotation: Annual

### Verification Process
1. Plugin downloaded from marketplace
2. Signature extracted from metadata
3. Public key retrieved from APEX key server
4. Signature verified against plugin content
5. Installation proceeds only if valid

## Revocation Policy

### Grounds for Revocation
- Security vulnerability discovered
- Malicious behavior detected
- Policy violation
- Author request
- Abandoned plugin (2+ years)

### Revocation Process
1. Issue identified and verified
2. Plugin marked as revoked in marketplace
3. Installed plugins receive revocation notice
4. Users prompted to uninstall or update
5. Revocation logged in audit trail

### Emergency Revocation
- Immediate revocation for critical security issues
- All instances notified within 1 hour
- Automatic uninstallation option provided

## Marketplace Governance

### Listing Criteria
- Must pass security scan
- Must have valid metadata
- Must follow SKILL.md format
- Must not duplicate existing plugins (unless improved)

### Ranking Algorithm
- Download count (30%)
- User ratings (25%)
- Trust level (20%)
- Recency (15%)
- Compatibility (10%)

### User Reviews
- Verified installers only
- 1-5 star rating
- Text review (optional)
- Flagging system for abuse

## Compliance

### Data Privacy
- No personal data collection without consent
- GDPR compliance for EU users
- Data processing disclosure required

### Licensing
- Open source preferred (MIT, Apache 2.0)
- Commercial licenses allowed with disclosure
- License must be specified in metadata

## Contacts

- **Plugin Review Team**: @plugin-reviewers
- **Security Team**: @security-team
- **Marketplace Maintainers**: @marketplace-team

---

## Last Updated

- Phase 7: Ecosystem Growth
- Version: 1.0
- Date: 2026-03-31
