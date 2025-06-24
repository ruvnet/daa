# /command-name

## Purpose
[Brief description of what this command does and when to use it]

## Parameters
- `<parameter1>`: [Description of parameter1 - required/optional, type, constraints]
- `<parameter2>`: [Description of parameter2 - required/optional, type, constraints]
- `[optional-param]`: [Description of optional parameter with square brackets]

## Prerequisites
- [ ] List any required state or conditions
- [ ] Dependencies that must be met
- [ ] Files or modules that must exist

## Execution Steps

### 1. Validation Phase
- Validate all input parameters
- Check prerequisites are met
- Verify system state is appropriate

### 2. Planning Phase
- Analyze the current state
- Create execution plan
- Identify potential risks

### 3. Implementation Phase
- Step 3.1: [Specific action with file paths]
  ```bash
  # Example command or code
  cargo test -p module-name
  ```
- Step 3.2: [Next specific action]
  - Sub-step with details
  - Another sub-step

### 4. Verification Phase
- Run tests to verify changes
- Check for regressions
- Validate success criteria

### 5. Documentation Phase
- Update relevant documentation
- Log changes made
- Create summary report

## Success Criteria
- [ ] All tests pass with >90% coverage
- [ ] No performance regressions detected
- [ ] Security audit shows no vulnerabilities
- [ ] Documentation is updated
- [ ] [Other specific measurable criteria]

## Error Handling
- **Error Type 1**: [Description and recovery steps]
- **Error Type 2**: [Description and recovery steps]
- **Build Failures**: Check dependencies and run `cargo clean`
- **Test Failures**: Review error logs and rollback if needed

## Output
- **Success**: [What the user sees on success]
- **Failure**: [What the user sees on failure]
- **Reports**: [Any generated reports and their locations]

## Example Usage
```
/command-name parameter1 parameter2
```

### Example Scenario
[Concrete example showing the command in action with expected inputs and outputs]

## Related Commands
- `/related-command1`: [How it relates]
- `/related-command2`: [How it relates]

## Workflow Integration
This command is part of the [Workflow Name] workflow and:
- Follows: `/previous-command`
- Precedes: `/next-command`
- Can be run in parallel with: `/parallel-command`

## Agent Coordination
- **Primary Agent**: [Agent name and role]
- **Supporting Agents**: 
  - [Agent 1]: [Specific responsibility]
  - [Agent 2]: [Specific responsibility]

## Notes
- Important considerations or warnings
- Performance implications
- Security considerations

---

# Command Style Guide

## Writing Effective Commands

### 1. Use Imperative Mood
- ✅ "Validate cryptographic implementation"
- ❌ "This validates the cryptographic implementation"
- ✅ "Execute test suite and generate report"
- ❌ "Tests will be executed and a report generated"

### 2. Be Specific and Actionable
- ✅ "Run `cargo test -p qudag-crypto --features ml-kem`"
- ❌ "Run the crypto tests"
- ✅ "Create file at `/workspaces/QuDAG/tests/integration/new_test.rs`"
- ❌ "Make a new test file"

### 3. Include Concrete Examples
Always provide real examples with actual values:
```
# Good example
/tdd-cycle crypto ml_kem_keygen
# This will:
# 1. Create test file: /workspaces/QuDAG/core/crypto/tests/ml_kem_keygen_test.rs
# 2. Run tests and verify failure
# 3. Implement in: /workspaces/QuDAG/core/crypto/src/kem/ml_kem.rs
```

### 4. Reference File Paths Explicitly
- Always use absolute paths starting with `/workspaces/QuDAG/`
- Specify exact file names and locations
- Include file extensions

### 5. Define Clear Success/Failure Conditions
Success criteria must be:
- Measurable: "All tests pass with >95% coverage"
- Specific: "Benchmark shows <10ms latency"
- Verifiable: "No clippy warnings with `cargo clippy -- -D warnings`"

### 6. Structure for Clarity
- Use numbered steps for sequential operations
- Use bullet points for parallel operations
- Use checkboxes for criteria and prerequisites
- Use code blocks for commands and examples

### 7. Error Handling Guidelines
For each potential error:
- Identify the error clearly
- Provide diagnostic steps
- Offer recovery actions
- Include rollback procedures if needed

### 8. Parameter Documentation
- Required parameters: `<parameter>`
- Optional parameters: `[parameter]`
- Multiple values: `<param1|param2|param3>`
- Default values: `[parameter=default]`

### 9. Command Naming Conventions
- Use hyphens for multi-word commands: `/tdd-cycle`
- Start with action verb: `/create-`, `/validate-`, `/deploy-`
- Be descriptive but concise
- Avoid abbreviations unless well-known

### 10. Integration Points
Always specify:
- Which workflows use this command
- Dependencies on other commands
- Parallel execution possibilities
- Required system state

## Template Sections Explained

### Purpose
- One or two sentences maximum
- Focus on the "what" and "why"
- Mention the primary use case

### Parameters
- List all parameters with clear descriptions
- Specify data types and constraints
- Indicate which are required vs optional
- Provide examples of valid values

### Prerequisites
- System state requirements
- Required files or modules
- Environmental dependencies
- Prior commands that must be run

### Execution Steps
- Logical phases (Validation, Planning, Implementation, etc.)
- Specific actions with exact commands
- File paths and locations
- Expected intermediate results

### Success Criteria
- Measurable outcomes
- Specific test results
- Performance metrics
- Quality gates

### Error Handling
- Common failure modes
- Diagnostic procedures
- Recovery strategies
- When to escalate

### Output
- What success looks like to the user
- Generated artifacts and their locations
- Log files and reports
- Next steps guidance

### Example Usage
- Real command invocations
- Typical parameter values
- Expected console output
- Common variations

### Related Commands
- Commands that work together
- Alternative approaches
- Command sequences
- Workflow context

### Agent Coordination
- Which agents are involved
- Their specific responsibilities
- Communication patterns
- Handoff points

## Best Practices

1. **Keep commands focused**: Each command should do one thing well
2. **Make commands idempotent**: Running twice should be safe
3. **Provide progress feedback**: Users should know what's happening
4. **Log important actions**: Enable debugging and auditing
5. **Validate early**: Check parameters before starting work
6. **Clean up on failure**: Don't leave system in inconsistent state
7. **Document assumptions**: Make implicit requirements explicit
8. **Version compatibility**: Note any version-specific behavior
9. **Performance awareness**: Mention if command is long-running
10. **Security considerations**: Highlight any security implications

## Common Patterns

### Test-First Pattern
1. Create/modify test files
2. Run tests (expect failure)
3. Implement feature
4. Run tests (expect success)
5. Refactor if needed
6. Update documentation

### Validation Pattern
1. Check parameter format
2. Verify file existence
3. Validate system state
4. Confirm dependencies
5. Proceed or abort with clear message

### Report Generation Pattern
1. Collect data
2. Analyze results
3. Generate report file
4. Display summary
5. Provide next steps

### Multi-Agent Pattern
1. Primary agent plans
2. Delegate to specialists
3. Coordinate results
4. Integrate outputs
5. Verify consistency
6. Report combined status