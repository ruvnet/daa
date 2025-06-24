# dag-visualize

Generate a comprehensive visualization of the QuDAG network's DAG state, showcasing the QR-Avalanche consensus algorithm's current state, including node voting patterns, finality progression, and network topology.

## Visualization Instructions

You are the Consensus Agent responsible for visualizing the QuDAG's state. Generate a detailed visualization that clearly shows:

1. **DAG Structure Visualization**
   - Display the current DAG topology with nodes and edges
   - Show confidence scores and edge weights for each connection
   - Highlight finalized vs pending nodes with different colors
   - Mark any detected forks or conflicting branches

2. **QR-Avalanche Consensus State**
   - Visualize the current voting round and participant nodes
   - Show vote distribution and progress toward finality threshold
   - Display Byzantine fault tolerance status (current vs maximum)
   - Indicate any consensus conflicts or anomalies

3. **Performance Metrics Dashboard**
   - Graph showing throughput (messages/second) over time
   - Finality latency distribution (p50, p99 percentiles)
   - DAG growth rate and node creation frequency
   - Fork occurrence frequency and resolution times

## Visualization Options

Choose one of these visualization states:
- `current` - Real-time DAG state with live consensus data
- `snapshot-{id}` - Historical DAG state at specific snapshot
- `consensus` - Focus on voting patterns and consensus flow
- `forks` - Highlight fork detection and resolution process
- `metrics` - Performance metrics and statistics view
- `finality` - Finality progression and checkpoint visualization

Output formats available:
- `svg` - Scalable vector graphics (default, best for web viewing)
- `png` - Raster image format (good for reports)
- `dot` - GraphViz DOT format (for further processing)
- `json` - Raw data in JSON format (for programmatic use)

Additional options:
- Set visualization depth (default: 10 levels)
- Highlight specific aspects: votes, conflicts, or finalized nodes
- Filter by time range or node subset

## Expected Output

Generate a visualization that includes:

1. **Visualization File**: Create the output file in the requested format
2. **Node Statistics**: Total nodes, finalized count, pending count, orphaned nodes
3. **Edge Statistics**: Total edges, validated edges, weight distribution analysis
4. **QR-Avalanche Status**:
   - Current consensus round number
   - Voting progress (votes received / votes required)
   - Current finality depth in the DAG
   - Active fork count and resolution status
5. **Performance Metrics**:
   - Current throughput in messages per second
   - Finality latency percentiles (p50, p99)
   - DAG growth rate in nodes per second

## Visualization Quality Checks

Ensure the visualization:
- Clearly distinguishes between finalized and pending nodes
- Shows vote flow and consensus progression
- Identifies any network partitions or conflicts
- Provides actionable insights for consensus health
- Is readable and properly labeled

## Error Handling

Handle these potential issues:
- Invalid state parameter: Validate the requested state exists
- Rendering failures: Fall back to simpler format if needed
- Format errors: Check supported output formats
- Depth limits: Cap at available data depth
- Missing consensus data: Indicate which data is unavailable

Remember to analyze the current consensus health and provide recommendations if any issues are detected in the visualization.