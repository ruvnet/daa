# QuDAG Benchmarking Metrics Specification

## Overview

This document provides a comprehensive specification of all metrics collected by the QuDAG benchmarking framework, including definitions, collection methods, and analysis approaches.

## Metric Categories

### 1. Performance Metrics

#### 1.1 Latency Metrics
```python
@dataclass
class LatencyMetric:
    operation: str              # Operation name
    start_time: float          # Unix timestamp
    end_time: float            # Unix timestamp
    duration_ms: float         # Duration in milliseconds
    success: bool              # Operation success status
    error_type: Optional[str]  # Error classification if failed
```

**Collection Points:**
- Cryptographic operations (keygen, encrypt, sign, verify)
- Network operations (connect, send, receive)
- DAG operations (validate, consensus, finalize)
- CLI command execution

**Statistical Measures:**
- Min, Max, Mean, Median
- Standard Deviation
- Percentiles (P50, P75, P90, P95, P99, P99.9)
- Histogram buckets (exponential boundaries)

#### 1.2 Throughput Metrics
```python
@dataclass
class ThroughputMetric:
    operation: str             # Operation type
    count: int                 # Number of operations
    duration_seconds: float    # Time window
    rate_per_second: float     # Operations per second
    bytes_processed: Optional[int]  # Data volume if applicable
    bandwidth_mbps: Optional[float] # Megabits per second
```

**Key Throughput Measurements:**
- Messages per second (network layer)
- Transactions per second (DAG consensus)
- Vertices processed per second
- Bytes encrypted/decrypted per second
- Signatures generated/verified per second

### 2. Resource Utilization Metrics

#### 2.1 CPU Metrics
```python
@dataclass
class CPUMetric:
    timestamp: float           # Collection time
    cpu_percent: float         # Total CPU usage (0-100)
    cpu_per_core: List[float]  # Per-core usage
    user_time: float           # User space CPU time
    system_time: float         # Kernel space CPU time
    idle_time: float           # Idle CPU time
    iowait_time: float         # I/O wait time
    context_switches: int      # Context switch count
    interrupts: int            # Interrupt count
```

**Collection Method:**
```python
import psutil

def collect_cpu_metrics() -> CPUMetric:
    cpu_times = psutil.cpu_times()
    return CPUMetric(
        timestamp=time.time(),
        cpu_percent=psutil.cpu_percent(interval=0.1),
        cpu_per_core=psutil.cpu_percent(percpu=True),
        user_time=cpu_times.user,
        system_time=cpu_times.system,
        idle_time=cpu_times.idle,
        iowait_time=cpu_times.iowait,
        context_switches=psutil.cpu_stats().ctx_switches,
        interrupts=psutil.cpu_stats().interrupts
    )
```

#### 2.2 Memory Metrics
```python
@dataclass
class MemoryMetric:
    timestamp: float          # Collection time
    rss_bytes: int           # Resident Set Size
    vms_bytes: int           # Virtual Memory Size
    heap_used: int           # Heap memory used
    heap_free: int           # Heap memory free
    gc_collections: Dict[int, int]  # GC collection counts by generation
    page_faults: int         # Page fault count
    swap_usage: int          # Swap usage in bytes
```

**Memory Profiling Points:**
- Before/after benchmark runs
- Peak memory during operations
- Memory allocation patterns
- Memory leak detection

#### 2.3 Network Metrics
```python
@dataclass
class NetworkMetric:
    timestamp: float         # Collection time
    bytes_sent: int          # Total bytes sent
    bytes_recv: int          # Total bytes received
    packets_sent: int        # Total packets sent
    packets_recv: int        # Total packets received
    connections_active: int  # Active connection count
    connections_failed: int  # Failed connection attempts
    bandwidth_utilization: float  # Percentage of available bandwidth
```

#### 2.4 Disk I/O Metrics
```python
@dataclass
class DiskIOMetric:
    timestamp: float         # Collection time
    read_bytes: int          # Bytes read from disk
    write_bytes: int         # Bytes written to disk
    read_ops: int            # Read operations count
    write_ops: int           # Write operations count
    io_wait_time: float      # Time spent waiting for I/O
```

### 3. Application-Specific Metrics

#### 3.1 Cryptographic Performance
```python
@dataclass
class CryptoMetric:
    algorithm: str           # Algorithm name (ML-KEM, ML-DSA, etc.)
    operation: str           # Operation type (keygen, encrypt, sign, etc.)
    key_size: int            # Key size in bits
    input_size: int          # Input data size in bytes
    duration_ms: float       # Operation duration
    operations_per_sec: float # Throughput
    cpu_cycles: Optional[int] # CPU cycles if available
```

**Specific Measurements:**
- ML-KEM-768: keygen, encapsulate, decapsulate
- ML-DSA: keygen, sign, verify
- HQC: encrypt, decrypt (128/192/256-bit)
- BLAKE3: hash computation
- Quantum fingerprinting: generation, verification

#### 3.2 DAG Consensus Metrics
```python
@dataclass
class ConsensusMetric:
    round_number: int        # Consensus round
    vertices_processed: int  # Vertices in this round
    conflicts_detected: int  # Number of conflicts
    resolution_time_ms: float # Conflict resolution time
    finality_time_ms: float  # Time to finality
    byzantine_nodes: int     # Byzantine nodes detected
    network_partitions: int  # Partition events
```

**QR-Avalanche Specific:**
- Query rounds to convergence
- Confidence threshold changes
- Parent selection efficiency
- Fork detection rate

#### 3.3 Network Layer Metrics
```python
@dataclass
class P2PMetric:
    peer_count: int          # Connected peers
    peer_churn_rate: float   # Peer turnover rate
    discovery_time_ms: float # Peer discovery time
    handshake_time_ms: float # Handshake duration
    message_latency_ms: float # Message propagation time
    routing_hops: int        # Routing hop count
    circuit_count: int       # Active onion circuits
```

**Dark Addressing Metrics:**
```python
@dataclass
class DarkAddressMetric:
    operation: str           # register, resolve, shadow
    domain_type: str         # .dark or .shadow
    resolution_time_ms: float # Time to resolve
    fingerprint_size: int    # Quantum fingerprint size
    ttl_seconds: Optional[int] # TTL for shadow addresses
    cache_hit: bool          # Resolution from cache
```

### 4. Benchmark Framework Metrics

#### 4.1 Execution Metrics
```python
@dataclass
class ExecutionMetric:
    task_name: str           # Benchmark task name
    warmup_iterations: int   # Warmup iteration count
    test_iterations: int     # Test iteration count
    total_duration_s: float  # Total execution time
    setup_time_ms: float     # Setup phase duration
    teardown_time_ms: float  # Teardown phase duration
    failed_iterations: int   # Failed iteration count
```

#### 4.2 Statistical Quality Metrics
```python
@dataclass
class QualityMetric:
    coefficient_of_variation: float  # CV = stddev/mean
    outlier_count: int              # Outlier detection
    confidence_interval_95: Tuple[float, float]  # 95% CI
    sample_size_adequate: bool      # Statistical power check
    distribution_normality: float   # Normality test p-value
```

## Metric Collection Implementation

### 1. Collector Architecture
```python
class MetricsCollector:
    def __init__(self):
        self.metrics_queue = asyncio.Queue()
        self.aggregators = {}
        self.storage_backend = MetricsStorage()
        
    async def start_collection(self):
        """Start background metric collection tasks"""
        tasks = [
            self._collect_system_metrics(),
            self._process_metrics_queue(),
            self._periodic_aggregation()
        ]
        await asyncio.gather(*tasks)
        
    async def record_metric(self, metric: Any):
        """Record a metric to the queue"""
        await self.metrics_queue.put({
            'timestamp': time.time(),
            'metric': metric,
            'tags': self._get_context_tags()
        })
```

### 2. Sampling Strategies

#### High-Frequency Metrics (10Hz)
- CPU usage
- Memory usage
- Active connections
- Queue depths

#### Medium-Frequency Metrics (1Hz)
- Network I/O
- Disk I/O
- GC statistics
- Thread counts

#### Low-Frequency Metrics (0.1Hz)
- System load average
- Disk space
- Process count
- Temperature sensors

### 3. Aggregation Methods

```python
class MetricAggregator:
    def __init__(self, window_size_seconds=60):
        self.window_size = window_size_seconds
        self.buckets = defaultdict(list)
        
    def add_sample(self, metric_name: str, value: float, timestamp: float):
        """Add a sample to the aggregation window"""
        bucket_key = int(timestamp / self.window_size)
        self.buckets[metric_name].append({
            'value': value,
            'timestamp': timestamp,
            'bucket': bucket_key
        })
        
    def get_statistics(self, metric_name: str) -> Dict[str, float]:
        """Calculate statistics for a metric"""
        values = [s['value'] for s in self.buckets[metric_name]]
        if not values:
            return {}
            
        return {
            'count': len(values),
            'sum': sum(values),
            'min': min(values),
            'max': max(values),
            'mean': statistics.mean(values),
            'median': statistics.median(values),
            'stddev': statistics.stdev(values) if len(values) > 1 else 0,
            'p95': np.percentile(values, 95),
            'p99': np.percentile(values, 99)
        }
```

## Metric Storage and Export

### 1. Time-Series Storage
```python
@dataclass
class TimeSeriesPoint:
    metric_name: str
    timestamp: float
    value: float
    tags: Dict[str, str]
    
class TimeSeriesStorage:
    def write_point(self, point: TimeSeriesPoint):
        """Write metric point to storage"""
        
    def query_range(self, metric: str, start: float, end: float) -> List[TimeSeriesPoint]:
        """Query metrics in time range"""
```

### 2. Export Formats

#### JSON Export
```json
{
  "benchmark_run": {
    "id": "run_12345",
    "start_time": "2024-01-15T10:00:00Z",
    "end_time": "2024-01-15T10:05:00Z",
    "config": {...},
    "metrics": {
      "latency": {
        "ml_kem_keygen": {
          "samples": 1000,
          "mean_ms": 1.94,
          "p95_ms": 2.15,
          "p99_ms": 2.28
        }
      },
      "throughput": {
        "messages_per_second": 5284.3
      },
      "resources": {
        "peak_memory_mb": 184,
        "avg_cpu_percent": 45.2
      }
    }
  }
}
```

#### Prometheus Format
```prometheus
# HELP qudag_latency_seconds Operation latency in seconds
# TYPE qudag_latency_seconds histogram
qudag_latency_seconds_bucket{operation="ml_kem_keygen",le="0.001"} 0
qudag_latency_seconds_bucket{operation="ml_kem_keygen",le="0.002"} 512
qudag_latency_seconds_bucket{operation="ml_kem_keygen",le="0.005"} 987
qudag_latency_seconds_bucket{operation="ml_kem_keygen",le="+Inf"} 1000
qudag_latency_seconds_sum{operation="ml_kem_keygen"} 1.94
qudag_latency_seconds_count{operation="ml_kem_keygen"} 1000
```

## Metric Analysis

### 1. Performance Baselines
```python
class PerformanceBaseline:
    def __init__(self):
        self.baselines = self._load_baselines()
        
    def compare_to_baseline(self, metric: str, value: float) -> Dict[str, Any]:
        """Compare metric to established baseline"""
        baseline = self.baselines.get(metric)
        if not baseline:
            return {"status": "no_baseline"}
            
        deviation_percent = ((value - baseline['mean']) / baseline['mean']) * 100
        
        return {
            "status": "regression" if deviation_percent > 10 else "normal",
            "baseline_mean": baseline['mean'],
            "current_value": value,
            "deviation_percent": deviation_percent
        }
```

### 2. Anomaly Detection
```python
class AnomalyDetector:
    def __init__(self, sensitivity=3.0):
        self.sensitivity = sensitivity  # Standard deviations
        self.history = defaultdict(deque)
        
    def is_anomaly(self, metric: str, value: float) -> bool:
        """Detect if value is anomalous"""
        history = list(self.history[metric])
        if len(history) < 30:
            return False
            
        mean = statistics.mean(history)
        stddev = statistics.stdev(history)
        z_score = abs((value - mean) / stddev)
        
        return z_score > self.sensitivity
```

### 3. Correlation Analysis
```python
class CorrelationAnalyzer:
    def analyze_correlations(self, metrics: Dict[str, List[float]]) -> Dict[str, float]:
        """Find correlations between metrics"""
        correlations = {}
        
        for metric1, values1 in metrics.items():
            for metric2, values2 in metrics.items():
                if metric1 >= metric2:
                    continue
                    
                correlation = np.corrcoef(values1, values2)[0, 1]
                if abs(correlation) > 0.7:  # Strong correlation
                    correlations[f"{metric1}_{metric2}"] = correlation
                    
        return correlations
```

## Metric Visualization

### 1. Real-time Dashboard
```python
class MetricsDashboard:
    def __init__(self):
        self.figures = {}
        self.update_interval = 1.0  # seconds
        
    def create_latency_histogram(self, data: List[float]) -> plotly.Figure:
        """Create latency distribution histogram"""
        fig = go.Figure(data=[
            go.Histogram(x=data, nbinsx=50)
        ])
        fig.update_layout(
            title="Latency Distribution",
            xaxis_title="Latency (ms)",
            yaxis_title="Count"
        )
        return fig
        
    def create_throughput_timeseries(self, timestamps: List[float], 
                                    values: List[float]) -> plotly.Figure:
        """Create throughput over time chart"""
        fig = go.Figure(data=[
            go.Scatter(x=timestamps, y=values, mode='lines')
        ])
        fig.update_layout(
            title="Throughput Over Time",
            xaxis_title="Time",
            yaxis_title="Operations/sec"
        )
        return fig
```

### 2. Comparison Visualizations
```python
def create_comparison_chart(baseline: Dict, current: Dict) -> plotly.Figure:
    """Create comparison chart between baseline and current results"""
    metrics = list(baseline.keys())
    baseline_values = [baseline[m] for m in metrics]
    current_values = [current[m] for m in metrics]
    
    fig = go.Figure(data=[
        go.Bar(name='Baseline', x=metrics, y=baseline_values),
        go.Bar(name='Current', x=metrics, y=current_values)
    ])
    fig.update_layout(barmode='group')
    return fig
```

## Metric Best Practices

### 1. Collection Guidelines
- Minimize observer effect
- Use appropriate sampling rates
- Implement backpressure handling
- Ensure thread safety
- Handle metric overflow

### 2. Storage Guidelines
- Implement retention policies
- Use compression for historical data
- Index frequently queried metrics
- Implement data aggregation
- Plan for scalability

### 3. Analysis Guidelines
- Establish baselines early
- Account for warmup effects
- Consider environmental factors
- Use statistical significance
- Document metric definitions

## Performance Impact

The metrics collection system itself should have minimal impact:
- CPU overhead: <2%
- Memory overhead: <10MB
- Network overhead: <1KB/s
- Disk I/O: <100KB/s

Regular benchmarking of the benchmarking system ensures these targets are met.