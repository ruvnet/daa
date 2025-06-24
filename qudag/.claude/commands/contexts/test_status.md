# Test Status Context

## Purpose
Track test execution status and coverage across components

## Schema
```json
{
  "test_status": {
    "unit_tests": {
      "crypto": {
        "total": 0,
        "passed": 0,
        "failed": 0,
        "coverage": 0
      },
      "network": {
        "total": 0,
        "passed": 0,
        "failed": 0,
        "coverage": 0
      },
      "consensus": {
        "total": 0,
        "passed": 0,
        "failed": 0,
        "coverage": 0
      }
    },
    "integration_tests": {
      "total": 0,
      "passed": 0,
      "failed": 0,
      "coverage": 0
    },
    "security_tests": {
      "total": 0,
      "passed": 0,
      "failed": 0,
      "coverage": 0
    }
  }
}
```

## Update Protocol
1. Run tests via respective commands
2. Parse test output for results
3. Update relevant status fields
4. Track coverage changes

## Access Patterns
- Read before running tests
- Update after test completion
- Query for coverage metrics
- Monitor test failures