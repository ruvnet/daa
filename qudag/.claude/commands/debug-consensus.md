# debug-consensus

Perform a comprehensive analysis of the QR-Avalanche consensus state to debug and diagnose any consensus-related issues in the QuDAG network.

## Debugging Instructions

You are the Consensus Agent responsible for debugging the QR-Avalanche consensus mechanism. Conduct a thorough analysis following these steps:

1. **Initial Consensus State Assessment**
   - Check the current consensus round and network participation
   - Verify Byzantine fault tolerance threshold (f < n/3)
   - Assess overall consensus health score
   - Identify any immediate red flags or anomalies

2. **Voting Analysis Deep Dive**
   - Trace vote propagation across the network
   - Verify vote signatures and validity
   - Analyze vote aggregation and counting logic
   - Identify missing votes or non-participating nodes
   - Detect and investigate any conflicting votes

3. **DAG State Examination**
   - Count total nodes vs finalized nodes
   - Identify pending and orphaned nodes
   - Detect fork branches and their weights
   - Verify DAG consistency and integrity
   - Check for any structural anomalies

4. **Finality Investigation**
   - Verify finality rules are being followed
   - Check checkpoint consistency across nodes
   - Analyze any reorganizations or rollbacks
   - Measure time-to-finality metrics
   - Identify finality bottlenecks or delays

5. **Performance Analysis**
   - Measure consensus throughput (TPS)
   - Analyze vote propagation latency (p50, p99)
   - Calculate average round duration
   - Assess message overhead per round
   - Identify performance bottlenecks

## Debug Focus Areas

When debugging, you can focus on specific areas:
- `voting` - Deep dive into vote propagation and validation
- `finality` - Analyze finality achievement and delays
- `forks` - Investigate fork detection and resolution
- `performance` - Profile consensus performance metrics
- `conflicts` - Focus on conflicting votes or states
- `all` - Comprehensive analysis of all areas

## Debugging Process

1. **Data Collection Phase**
   - Gather consensus logs for the specified timeframe
   - Collect vote messages and their propagation paths
   - Record finality checkpoints and transitions
   - Monitor resource usage (CPU, memory, network)

2. **Analysis Phase**
   - Check consensus invariants:
     * Safety: No conflicting finalized blocks
     * Liveness: Consensus progresses under synchrony
     * Performance: Sub-second finality achieved
   - Verify QR-Avalanche algorithm correctness
   - Validate Byzantine node behavior bounds

3. **Diagnosis Phase**
   - Identify root causes of any issues found
   - Trace problematic transactions or votes
   - Pinpoint network communication failures
   - Detect any malicious or faulty nodes

4. **Recommendation Phase**
   - Suggest immediate fixes for critical issues
   - Propose optimizations for performance
   - Recommend configuration changes
   - Identify areas needing further investigation

## Expected Debug Output

Generate a comprehensive debug report containing:

1. **QR-Avalanche Consensus Status**
   - Algorithm implementation status
   - Current round and network size
   - Byzantine threshold verification
   - Overall consensus health score

2. **Detailed Voting Analysis**
   - Active voter participation rate
   - Vote distribution across options
   - List of missing or delayed votes
   - Conflict detection and resolution

3. **DAG State Tracking Report**
   - Node counts (total, finalized, pending, orphaned)
   - Fork branch analysis
   - DAG growth patterns
   - Structural integrity verification

4. **Finality Information**
   - Last finalized block details
   - Current finality depth
   - Average time to finality
   - Finality queue analysis

5. **Performance Metrics**
   - Consensus transactions per second
   - Vote latency percentiles
   - Round duration statistics
   - Message overhead analysis

## Troubleshooting Common Issues

- **Stalled Consensus**: Check for network partitions or insufficient voters
- **Slow Finality**: Analyze vote propagation delays and network latency
- **Fork Storms**: Investigate conflicting transactions and resolution logic
- **High Message Overhead**: Review vote aggregation and gossip parameters
- **Byzantine Behavior**: Verify threshold bounds and node authentication

## Debug Options

- Specify a consensus round number for historical analysis
- Focus on a specific node's perspective
- Enable verbose logging for detailed traces
- Set custom timeframes for analysis (e.g., '5m', '1h', 'all')

Remember to correlate findings across different components and provide actionable insights for resolving any consensus issues discovered.