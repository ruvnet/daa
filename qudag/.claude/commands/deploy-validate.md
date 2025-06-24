# deploy-validate

Validate deployment configuration and execute comprehensive deployment tests

## Usage

```
/deploy-validate [options]
```

## Parameters

### environment (optional)
- **Type**: string
- **Description**: Target environment (dev, staging, prod)
- **Default**: dev

### run_tests (optional)
- **Type**: boolean
- **Description**: Run full test suite before deployment
- **Default**: true

### security_audit (optional)
- **Type**: boolean
- **Description**: Perform security audit
- **Default**: true

### performance_check (optional)
- **Type**: boolean
- **Description**: Verify performance benchmarks
- **Default**: true

## Examples

```
/deploy-validate
/deploy-validate --environment staging --run_tests true
/deploy-validate --environment prod --security_audit true --performance_check true
/deploy-validate --environment dev --run_tests false
```

## Validation Criteria

### Development Environment
**Tests:**
- Unit Tests: Required - All must pass
- Integration Tests: Required - Core tests must pass
- Security Tests: Optional - Run if available
- Performance Tests: Optional - Baseline only

**Checks:**
- Configuration: Basic validation
- Dependencies: Version compatibility
- Resources: Minimal requirements
- Network: Local connectivity

**Requirements:**
- Test Coverage: >=80%
- Security Score: >=7/10
- Performance Baseline: Established

### Staging Environment
**Tests:**
- Unit Tests: Required - 100% pass rate
- Integration Tests: Required - All must pass
- Security Tests: Required - All must pass
- Performance Tests: Required - Meet targets
- Load Tests: Required - Handle expected load

**Checks:**
- Configuration: Full validation with secrets
- Dependencies: Production versions
- Resources: Production-like specs
- Network: Multi-node testing
- Monitoring: Metrics collection enabled

**Requirements:**
- Test Coverage: >=90%
- Security Score: >=9/10
- Performance Targets: Met or exceeded
- Load Capacity: 2x expected traffic

### Production Environment
**Tests:**
- Unit Tests: Required - 100% pass rate
- Integration Tests: Required - All must pass
- Security Tests: Required - All must pass
- Performance Tests: Required - Exceed targets
- Load Tests: Required - 3x capacity
- Chaos Tests: Required - Resilience verified

**Checks:**
- Configuration: Production-ready with backups
- Dependencies: Locked versions
- Resources: Auto-scaling configured
- Network: Load balancing ready
- Monitoring: Full observability
- Rollback: Tested and ready
- Documentation: Complete and current

**Requirements:**
- Test Coverage: >=95%
- Security Score: 10/10
- Performance Targets: Exceeded by 20%
- Load Capacity: 3x expected traffic
- Zero Downtime: Blue-green deployment ready
- Disaster Recovery: RTO < 15 minutes

## Deployment Checklist

### Pre-deployment
- All tests passing
- Security audit completed
- Performance benchmarks met
- Configuration validated
- Rollback plan tested
- Monitoring configured
- Documentation updated

### Deployment
- Health checks passing
- Gradual rollout started
- Metrics within bounds
- No error spike detected
- User traffic normal

### Post-deployment
- All services healthy
- Performance nominal
- No security alerts
- User feedback positive
- Logs clean

## Validation Steps

1. **Configuration Validation**
   ```bash
   cargo build --release
   cargo test --all-features
   ./scripts/validate_config.sh ${environment}
   ```

2. **Security Validation**
   ```bash
   cargo audit
   cargo fuzz run crypto_fuzz -- -max_total_time=60
   ./scripts/security_scan.sh
   ```

3. **Performance Validation**
   ```bash
   cargo bench --bench '*'
   ./scripts/performance_check.sh ${environment}
   python3 scripts/analyze_benchmarks.py
   ```

4. **Integration Validation**
   ```bash
   cargo test --test '*integration*'
   ./tools/simulator/run_scenarios.sh
   docker-compose -f docker-compose.test.yml up --abort-on-container-exit
   ```

## Output Format

```
## Deployment Validation Report
### Environment: ${environment}

### 1. Test Results
- Unit Tests: ${unit_test_results}
- Integration Tests: ${integration_test_results}
- Security Tests: ${security_test_results}
- Performance Tests: ${performance_test_results}

### 2. Configuration Status
- Config Validation: ${config_status}
- Environment Variables: ${env_status}
- Secrets Management: ${secrets_status}

### 3. Security Assessment
- Vulnerability Scan: ${vuln_scan_results}
- Dependency Audit: ${audit_results}
- Security Score: ${security_score}/10

### 4. Performance Metrics
- Throughput: ${throughput_metrics}
- Latency: ${latency_metrics}
- Resource Usage: ${resource_metrics}

### 5. Deployment Readiness
- Overall Status: ${readiness_status}
- Risk Assessment: ${risk_level}
- Recommendations: ${recommendations}

### 6. Pre-flight Checklist
${checklist_status}
```

## Decision Gates

### Continue Deployment
- All required tests passing
- Security score meets threshold
- Performance targets achieved
- No critical issues found

### Abort Deployment
- Any required test failing
- Security vulnerabilities found
- Performance regression detected
- Configuration errors present

## Error Handling

- **config_error**: Configuration validation failed: ${error_details}
- **invalid_environment**: Valid environments: dev, staging, prod
- **security_failure**: Security validation failed: ${security_issues}
- **test_failure**: Test suite failed: ${failed_tests}
- **performance_regression**: Performance below threshold: ${metrics}
- **dependency_conflict**: Dependency issues found: ${conflicts}

## Post Actions

- Generate deployment report
- Archive test results
- Update deployment log
- Notify team of status
- Prepare rollback if needed

## Agent Context

- **Workflow Reference**: `workflow/deployment_workflow.md`