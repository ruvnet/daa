# QuDAG Command Structure Summary

## Primary.json Structure

The updated `primary.json` file serves as the main command registry with the following structure:

### Top-level Fields
- `name`: QuDAG Command Registry
- `version`: 1.0.0
- `description`: Primary command registry for QuDAG Protocol development
- `commands`: Organized by category
- `global_contexts`: Shared context files
- `agent_registry`: Agent definitions
- `workflow_registry`: Workflow definitions

### Command Categories

1. **primary_development**: Core development commands
   - tdd-cycle
   - security-audit
   - performance-benchmark
   - integration-test
   - deploy-validate

2. **development_workflow**: Supporting development commands
   - create-test
   - implement-feature
   - refactor-optimize
   - review-security
   - update-docs

3. **specialized**: Domain-specific commands
   - crypto-validate
   - network-simulate
   - dag-visualize
   - fuzz-test

4. **debug**: Debugging and diagnostic commands
   - debug-network
   - debug-consensus
   - debug-performance
   - debug-security

### Command Structure

Each command entry includes:
- `name`: Command identifier
- `description`: Brief description
- `workflow_file`: Reference to workflow .md file (where applicable)
- `command_file`: Reference to detailed command JSON file
- `agent_context`: Reference to agent .md file (where applicable)
- `parameters`: Parameter definitions with types and descriptions
- `example`: Usage example

### Referenced Files

#### Workflow Files
- `workflow/tdd_workflow.md` - Test-Driven Development workflow
- `workflow/security_workflow.md` - Security audit workflow
- `workflow/performance_workflow.md` - Performance benchmarking workflow
- `workflow/deployment_workflow.md` - Deployment validation workflow

#### Agent Context Files
- `agents/crypto_agent.md` - Cryptographic implementations
- `agents/network_agent.md` - P2P networking protocols
- `agents/consensus_agent.md` - DAG consensus mechanisms
- `agents/security_agent.md` - Security analysis
- `agents/performance_agent.md` - Performance optimization
- `agents/integration_agent.md` - Component integration

#### Shared Context Files
- `contexts/test_status.md` - Test execution status
- `contexts/integration_context.md` - Integration test context
- `contexts/security_context.md` - Security audit findings
- `contexts/performance_context.md` - Performance metrics

### Parameter Definition Style

Parameters follow a consistent structure:
```json
{
  "type": "string|integer|boolean|array",
  "description": "Clear description of the parameter",
  "required": true|false,
  "default": "default value if optional",
  "enum": ["allowed", "values"] // for restricted choices
}
```

### Agent Mapping

The TDD cycle command includes agent mapping to route commands to appropriate agents based on module:
- crypto → crypto_agent
- network → network_agent
- dag → consensus_agent
- protocol → integration_agent

This structure ensures clear command organization with proper references to workflows, agents, and contexts for effective multi-agent coordination.