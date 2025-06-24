# debug-performance

Profile performance bottlenecks

## Usage

```
/debug-performance
```

## Parameters

### component (optional)
- **Type**: string
- **Description**: Optional component to profile

### duration (optional)
- **Type**: string
- **Description**: Profile duration
- **Default**: 60s

### tool (optional)
- **Type**: string
- **Description**: Profiling tool to use
- **Default**: perf

### output_format (optional)
- **Type**: string
- **Description**: Output format for profiling data
- **Default**: report

## Examples

```
/debug-performance
/debug-performance --component crypto
/debug-performance --duration 300s
/debug-performance --tool flamegraph --output_format flamegraph
/debug-performance --component memory --tool heaptrack
```

## Output Format

```
1. CPU Usage
2. Memory Profile
3. I/O Statistics
4. Hot Spots
5. Optimization Suggestions
6. Call Graph Analysis
7. Lock Contention Analysis
8. Memory Leak Detection
```

## Error Handling

- **invalid_component**: Component validation error
- **profile_error**: Profiling failure details
- **resource_limit**: Resource limitation warning

## Agent Context

- **Primary Agent**: `agents/performance_agent.md`

