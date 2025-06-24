# Security Context

## Purpose
Track security audit status and findings

## Schema
```json
{
  "security_status": {
    "crypto": {
      "audits": [],
      "vulnerabilities": {
        "critical": 0,
        "high": 0,
        "medium": 0,
        "low": 0
      },
      "last_audit": null
    },
    "network": {
      "audits": [],
      "vulnerabilities": {
        "critical": 0,
        "high": 0,
        "medium": 0,
        "low": 0
      },
      "last_audit": null
    },
    "consensus": {
      "audits": [],
      "vulnerabilities": {
        "critical": 0,
        "high": 0,
        "medium": 0,
        "low": 0
      },
      "last_audit": null
    }
  }
}
```

## Update Protocol
1. Run security audit
2. Log findings by severity
3. Track remediation status
4. Schedule follow-up audits

## Access Patterns
- Pre-audit status check
- Vulnerability tracking
- Audit result recording
- Remediation monitoring