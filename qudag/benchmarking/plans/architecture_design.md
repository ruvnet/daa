# QuDAG Benchmarking Framework Architecture Design

## Overview

This document provides the detailed architecture design for the QuDAG benchmarking framework, outlining the component structure, interfaces, and implementation patterns.

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Interface                             │
│  (qudag benchmark run | compare | report | monitor)             │
└─────────────────────┬───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                   Benchmark Orchestrator                         │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────────────┐    │
│  │   Config    │ │   Runner     │ │   Task Registry      │    │
│  │  Manager    │ │   Engine     │ │   & Scheduler        │    │
│  └─────────────┘ └──────────────┘ └──────────────────────┘    │
└─────────────────────┬───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                    Execution Layer                               │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────────────┐    │
│  │   Task      │ │   Metrics    │ │   Resource           │    │
│  │  Executor   │ │  Collector   │ │   Monitor            │    │
│  └─────────────┘ └──────────────┘ └──────────────────────┘    │
└─────────────────────┬───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│                    Benchmark Tasks                               │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────────────┐    │
│  │   Crypto    │ │   Network    │ │      DAG             │    │
│  │ Benchmarks  │ │  Benchmarks  │ │   Benchmarks         │    │
│  └─────────────┘ └──────────────┘ └──────────────────────┘    │
└─────────────────────┬───────────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────────┐
│              Storage & Reporting Layer                           │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────────────────┐    │
│  │   Result    │ │   Report     │ │   Visualization      │    │
│  │  Storage    │ │  Generator   │ │     Engine           │    │
│  └─────────────┘ └──────────────┘ └──────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Component Design

### 1. Core Framework Components

#### 1.1 Benchmark Orchestrator
```python
# benchmarking/src/core/orchestrator.py
class BenchmarkOrchestrator:
    """
    Main orchestration component that coordinates all benchmark activities.
    """
    def __init__(self, config_path: Optional[Path] = None):
        self.config_manager = ConfigManager(config_path)
        self.runner_engine = RunnerEngine()
        self.task_registry = TaskRegistry()
        self.result_storage = ResultStorage()
        
    async def run_benchmarks(
        self,
        suite: Optional[str] = None,
        tags: Optional[List[str]] = None,
        parallel: bool = False
    ) -> BenchmarkReport:
        """Execute benchmark suite with specified filters"""
        config = self.config_manager.get_active_config()
        tasks = self.task_registry.get_tasks(suite=suite, tags=tags)
        
        if parallel:
            results = await self.runner_engine.run_parallel(tasks, config)
        else:
            results = await self.runner_engine.run_sequential(tasks, config)
            
        report = self._generate_report(results)
        self.result_storage.save(report)
        return report
```

#### 1.2 Configuration Management
```python
# benchmarking/src/core/config.py
@dataclass
class BenchmarkConfig:
    # Execution settings
    warmup_iterations: int = 10
    test_iterations: int = 100
    timeout_seconds: int = 300
    parallel_workers: int = cpu_count()
    
    # Metric collection
    collect_system_metrics: bool = True
    metrics_sample_rate_hz: float = 10.0
    
    # Environment settings
    qudag_binary_path: Optional[Path] = None
    rust_benchmark_path: Optional[Path] = None
    
    # Output settings
    output_format: Literal["json", "html", "csv"] = "json"
    output_directory: Path = Path("./benchmark_results")
    
    # Performance targets
    performance_targets: Dict[str, float] = field(default_factory=dict)
    
class ConfigManager:
    """Manages benchmark configurations with inheritance and overrides"""
    def __init__(self, config_path: Optional[Path] = None):
        self.base_config = self._load_base_config()
        self.user_config = self._load_user_config(config_path)
        self.env_overrides = self._load_env_overrides()
        
    def get_active_config(self) -> BenchmarkConfig:
        """Merge configurations with proper precedence"""
        config_dict = {
            **asdict(self.base_config),
            **self.user_config,
            **self.env_overrides
        }
        return BenchmarkConfig(**config_dict)
```

#### 1.3 Task Registry and Discovery
```python
# benchmarking/src/core/registry.py
class TaskRegistry:
    """
    Discovers and manages benchmark tasks with dynamic loading.
    """
    def __init__(self):
        self.tasks: Dict[str, BenchmarkTask] = {}
        self._discover_tasks()
        
    def _discover_tasks(self):
        """Auto-discover benchmark tasks from modules"""
        task_modules = [
            "benchmarking.tasks.crypto",
            "benchmarking.tasks.network", 
            "benchmarking.tasks.dag",
            "benchmarking.tasks.cli"
        ]
        
        for module_name in task_modules:
            module = importlib.import_module(module_name)
            for name, obj in inspect.getmembers(module):
                if (inspect.isclass(obj) and 
                    issubclass(obj, BenchmarkTask) and
                    obj != BenchmarkTask):
                    task = obj()
                    self.register(task)
                    
    def register(self, task: BenchmarkTask):
        """Register a benchmark task"""
        if task.name in self.tasks:
            raise ValueError(f"Task {task.name} already registered")
        self.tasks[task.name] = task
        
    def get_tasks(
        self,
        suite: Optional[str] = None,
        tags: Optional[List[str]] = None
    ) -> List[BenchmarkTask]:
        """Get filtered list of tasks"""
        tasks = list(self.tasks.values())
        
        if suite:
            tasks = [t for t in tasks if t.suite == suite]
            
        if tags:
            tasks = [t for t in tasks if any(tag in t.tags for tag in tags)]
            
        return tasks
```

### 2. Execution Layer

#### 2.1 Task Executor
```python
# benchmarking/src/execution/executor.py
class TaskExecutor:
    """
    Executes individual benchmark tasks with isolation and measurement.
    """
    def __init__(self, config: BenchmarkConfig):
        self.config = config
        self.metrics_collector = MetricsCollector()
        
    async def execute_task(self, task: BenchmarkTask) -> TaskResult:
        """Execute a single benchmark task"""
        result = TaskResult(task_name=task.name)
        
        try:
            # Setup phase
            await self._setup_task(task)
            
            # Warmup phase
            if self.config.warmup_iterations > 0:
                await self._warmup_phase(task)
                
            # Measurement phase
            measurements = await self._measurement_phase(task)
            
            # Process results
            result.measurements = measurements
            result.statistics = self._calculate_statistics(measurements)
            result.status = "completed"
            
        except asyncio.TimeoutError:
            result.status = "timeout"
            result.error = "Task exceeded timeout"
            
        except Exception as e:
            result.status = "failed"
            result.error = str(e)
            
        finally:
            # Teardown phase
            await self._teardown_task(task)
            
        return result
        
    async def _measurement_phase(self, task: BenchmarkTask) -> List[Measurement]:
        """Execute task iterations and collect measurements"""
        measurements = []
        
        for i in range(self.config.test_iterations):
            # Start metrics collection
            self.metrics_collector.start_iteration()
            
            # Execute task
            start_time = time.perf_counter()
            custom_metrics = await task.execute()
            end_time = time.perf_counter()
            
            # Stop metrics collection
            system_metrics = self.metrics_collector.stop_iteration()
            
            # Record measurement
            measurement = Measurement(
                iteration=i,
                duration=end_time - start_time,
                custom_metrics=custom_metrics,
                system_metrics=system_metrics
            )
            measurements.append(measurement)
            
            # Adaptive delay between iterations
            await self._adaptive_delay(measurements)
            
        return measurements
```

#### 2.2 Parallel Execution Engine
```python
# benchmarking/src/execution/parallel.py
class ParallelExecutor:
    """
    Manages parallel execution of benchmark tasks.
    """
    def __init__(self, config: BenchmarkConfig):
        self.config = config
        self.executor_pool = ProcessPoolExecutor(
            max_workers=config.parallel_workers
        )
        self.semaphore = asyncio.Semaphore(config.parallel_workers)
        
    async def execute_batch(
        self,
        tasks: List[BenchmarkTask],
        progress_callback: Optional[Callable] = None
    ) -> List[TaskResult]:
        """Execute multiple tasks in parallel"""
        results = []
        
        # Create execution plan
        execution_plan = self._create_execution_plan(tasks)
        
        # Execute tasks in batches
        for batch in execution_plan:
            batch_results = await self._execute_batch(batch, progress_callback)
            results.extend(batch_results)
            
        return results
        
    def _create_execution_plan(
        self,
        tasks: List[BenchmarkTask]
    ) -> List[List[BenchmarkTask]]:
        """Create optimal execution plan based on task characteristics"""
        # Sort tasks by expected duration and resource requirements
        sorted_tasks = sorted(
            tasks,
            key=lambda t: (t.expected_duration, t.resource_intensity),
            reverse=True
        )
        
        # Create balanced batches
        batches = [[] for _ in range(self.config.parallel_workers)]
        batch_times = [0.0] * self.config.parallel_workers
        
        for task in sorted_tasks:
            # Assign to batch with minimum total time
            min_batch_idx = batch_times.index(min(batch_times))
            batches[min_batch_idx].append(task)
            batch_times[min_batch_idx] += task.expected_duration
            
        return [b for b in batches if b]  # Remove empty batches
```

### 3. Metrics Collection System

#### 3.1 Metrics Collector Architecture
```python
# benchmarking/src/metrics/collector.py
class MetricsCollector:
    """
    Collects system and application metrics during benchmark execution.
    """
    def __init__(self):
        self.collectors = {
            'cpu': CPUCollector(),
            'memory': MemoryCollector(),
            'network': NetworkCollector(),
            'disk': DiskIOCollector()
        }
        self.collection_interval = 0.1  # 10Hz
        self.is_collecting = False
        self._collection_task = None
        
    async def start_continuous_collection(self):
        """Start background metric collection"""
        self.is_collecting = True
        self._collection_task = asyncio.create_task(self._collect_loop())
        
    async def _collect_loop(self):
        """Background collection loop"""
        while self.is_collecting:
            timestamp = time.time()
            metrics = {}
            
            for name, collector in self.collectors.items():
                try:
                    metrics[name] = await collector.collect()
                except Exception as e:
                    logger.warning(f"Failed to collect {name} metrics: {e}")
                    
            self._store_metrics(timestamp, metrics)
            await asyncio.sleep(self.collection_interval)
```

#### 3.2 Specialized Collectors
```python
# benchmarking/src/metrics/collectors/cpu.py
class CPUCollector:
    """Collects CPU-related metrics"""
    def __init__(self):
        self.process = psutil.Process()
        self.last_cpu_times = None
        
    async def collect(self) -> Dict[str, float]:
        """Collect current CPU metrics"""
        # System-wide CPU
        cpu_percent = psutil.cpu_percent(interval=0)
        cpu_freq = psutil.cpu_freq()
        
        # Process-specific CPU
        process_cpu = self.process.cpu_percent()
        process_times = self.process.cpu_times()
        
        # Calculate CPU time deltas
        cpu_time_delta = self._calculate_cpu_time_delta(process_times)
        
        return {
            'system_cpu_percent': cpu_percent,
            'process_cpu_percent': process_cpu,
            'cpu_frequency_mhz': cpu_freq.current,
            'user_time_delta': cpu_time_delta['user'],
            'system_time_delta': cpu_time_delta['system'],
            'context_switches': self._get_context_switches()
        }
```

### 4. Benchmark Task Implementation

#### 4.1 Base Task Class
```python
# benchmarking/src/tasks/base.py
class BenchmarkTask(ABC):
    """
    Abstract base class for all benchmark tasks.
    """
    def __init__(self):
        self.name: str = self.__class__.__name__
        self.suite: str = "default"
        self.tags: List[str] = []
        self.description: str = ""
        self.expected_duration: float = 1.0  # seconds
        self.resource_intensity: float = 1.0  # 1-10 scale
        
    @abstractmethod
    async def setup(self) -> None:
        """Setup required before benchmark execution"""
        pass
        
    @abstractmethod
    async def execute(self) -> Dict[str, Any]:
        """Execute the benchmark operation and return custom metrics"""
        pass
        
    @abstractmethod
    async def teardown(self) -> None:
        """Cleanup after benchmark execution"""
        pass
        
    @abstractmethod
    def validate_environment(self) -> ValidationResult:
        """Validate that the environment is suitable for this benchmark"""
        pass
```

#### 4.2 Cryptographic Benchmark Example
```python
# benchmarking/src/tasks/crypto/ml_kem_benchmark.py
class MLKEMBenchmark(BenchmarkTask):
    """Benchmark ML-KEM cryptographic operations"""
    
    def __init__(self):
        super().__init__()
        self.suite = "crypto"
        self.tags = ["ml-kem", "post-quantum", "fast"]
        self.description = "ML-KEM-768 key generation and encapsulation"
        self.expected_duration = 0.002  # 2ms per operation
        
    async def setup(self):
        """Initialize crypto provider"""
        # Import QuDAG crypto module
        self.crypto = await self._initialize_qudag_crypto()
        
    async def execute(self) -> Dict[str, Any]:
        """Execute ML-KEM operations"""
        metrics = {}
        
        # Key generation
        start = time.perf_counter()
        public_key, secret_key = await self.crypto.ml_kem_keygen()
        keygen_time = time.perf_counter() - start
        metrics['keygen_ms'] = keygen_time * 1000
        
        # Encapsulation
        start = time.perf_counter()
        ciphertext, shared_secret = await self.crypto.ml_kem_encapsulate(public_key)
        encap_time = time.perf_counter() - start
        metrics['encapsulate_ms'] = encap_time * 1000
        
        # Decapsulation
        start = time.perf_counter()
        recovered_secret = await self.crypto.ml_kem_decapsulate(ciphertext, secret_key)
        decap_time = time.perf_counter() - start
        metrics['decapsulate_ms'] = decap_time * 1000
        
        # Verify correctness
        assert shared_secret == recovered_secret, "ML-KEM decapsulation failed"
        
        return metrics
        
    async def teardown(self):
        """Cleanup crypto resources"""
        if hasattr(self, 'crypto'):
            await self.crypto.cleanup()
```

### 5. Storage and Reporting

#### 5.1 Result Storage System
```python
# benchmarking/src/storage/storage.py
class ResultStorage:
    """
    Manages storage and retrieval of benchmark results.
    """
    def __init__(self, storage_path: Path):
        self.storage_path = storage_path
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self.metadata_db = self._init_metadata_db()
        
    def save(self, report: BenchmarkReport) -> str:
        """Save benchmark report and return unique ID"""
        report_id = self._generate_report_id()
        report_path = self.storage_path / f"{report_id}.json"
        
        # Save report data
        with open(report_path, 'w') as f:
            json.dump(report.to_dict(), f, indent=2)
            
        # Update metadata database
        self._update_metadata(report_id, report)
        
        return report_id
        
    def get_report(self, report_id: str) -> BenchmarkReport:
        """Retrieve a specific report"""
        report_path = self.storage_path / f"{report_id}.json"
        
        if not report_path.exists():
            raise ValueError(f"Report {report_id} not found")
            
        with open(report_path) as f:
            data = json.load(f)
            
        return BenchmarkReport.from_dict(data)
        
    def query_reports(
        self,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        tags: Optional[List[str]] = None
    ) -> List[ReportMetadata]:
        """Query reports based on criteria"""
        query = "SELECT * FROM reports WHERE 1=1"
        params = []
        
        if start_date:
            query += " AND timestamp >= ?"
            params.append(start_date.isoformat())
            
        if end_date:
            query += " AND timestamp <= ?"
            params.append(end_date.isoformat())
            
        if tags:
            query += " AND tags LIKE ?"
            params.append(f"%{','.join(tags)}%")
            
        cursor = self.metadata_db.execute(query, params)
        return [ReportMetadata(**row) for row in cursor.fetchall()]
```

#### 5.2 Report Generator
```python
# benchmarking/src/reporting/generator.py
class ReportGenerator:
    """
    Generates various report formats from benchmark results.
    """
    def __init__(self):
        self.template_engine = Environment(
            loader=FileSystemLoader('templates')
        )
        
    def generate_html_report(
        self,
        results: List[TaskResult],
        config: BenchmarkConfig
    ) -> str:
        """Generate interactive HTML report"""
        template = self.template_engine.get_template('benchmark_report.html')
        
        # Prepare data for visualization
        charts = {
            'latency_distribution': self._create_latency_chart(results),
            'throughput_timeline': self._create_throughput_chart(results),
            'resource_usage': self._create_resource_chart(results),
            'comparison_matrix': self._create_comparison_chart(results)
        }
        
        # Render template
        html_content = template.render(
            results=results,
            config=config,
            charts=charts,
            summary=self._generate_summary(results)
        )
        
        return html_content
        
    def generate_json_report(
        self,
        results: List[TaskResult],
        config: BenchmarkConfig
    ) -> Dict[str, Any]:
        """Generate structured JSON report"""
        return {
            'metadata': {
                'timestamp': datetime.utcnow().isoformat(),
                'version': '1.0',
                'config': asdict(config)
            },
            'results': [r.to_dict() for r in results],
            'summary': self._generate_summary(results),
            'comparisons': self._generate_comparisons(results)
        }
```

### 6. CLI Integration

#### 6.1 CLI Command Structure
```python
# benchmarking/src/cli/commands.py
@click.group()
def benchmark():
    """QuDAG benchmarking commands"""
    pass

@benchmark.command()
@click.option('--suite', help='Benchmark suite to run')
@click.option('--tags', multiple=True, help='Filter by tags')
@click.option('--config', type=click.Path(), help='Config file path')
@click.option('--parallel', is_flag=True, help='Run in parallel')
@click.option('--output', type=click.Choice(['json', 'html', 'csv']))
def run(suite, tags, config, parallel, output):
    """Run benchmark suite"""
    orchestrator = BenchmarkOrchestrator(config)
    
    # Run benchmarks
    report = asyncio.run(
        orchestrator.run_benchmarks(
            suite=suite,
            tags=list(tags),
            parallel=parallel
        )
    )
    
    # Output results
    if output == 'json':
        click.echo(json.dumps(report.to_dict(), indent=2))
    elif output == 'html':
        html_path = report.save_html()
        click.echo(f"HTML report saved to: {html_path}")
        
@benchmark.command()
@click.argument('report_ids', nargs=-1, required=True)
@click.option('--output', type=click.Path(), help='Output file path')
def compare(report_ids, output):
    """Compare multiple benchmark reports"""
    storage = ResultStorage(Path('./benchmark_results'))
    
    # Load reports
    reports = [storage.get_report(rid) for rid in report_ids]
    
    # Generate comparison
    comparison = ComparisonAnalyzer().analyze(reports)
    
    # Save comparison report
    if output:
        with open(output, 'w') as f:
            json.dump(comparison, f, indent=2)
    else:
        click.echo(json.dumps(comparison, indent=2))
```

### 7. Integration Patterns

#### 7.1 QuDAG Integration
```python
# benchmarking/src/integration/qudag.py
class QuDAGIntegration:
    """
    Integration layer for benchmarking QuDAG components.
    """
    def __init__(self):
        self.qudag_path = self._find_qudag_binary()
        self.rust_bench_path = self._find_rust_benchmarks()
        
    async def run_rust_benchmark(self, module: str) -> Dict[str, Any]:
        """Execute Rust benchmark and parse results"""
        cmd = ['cargo', 'bench', '-p', module, '--', '--output-format', 'json']
        
        process = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
        
        stdout, stderr = await process.communicate()
        
        if process.returncode != 0:
            raise RuntimeError(f"Rust benchmark failed: {stderr.decode()}")
            
        # Parse criterion output
        return self._parse_criterion_output(stdout.decode())
        
    async def start_qudag_node(self, config: Dict[str, Any]) -> QuDAGNode:
        """Start QuDAG node for benchmarking"""
        node = QuDAGNode(self.qudag_path, config)
        await node.start()
        await node.wait_ready()
        return node
```

#### 7.2 Monitoring Integration
```python
# benchmarking/src/monitoring/prometheus.py
class PrometheusExporter:
    """
    Exports benchmark metrics to Prometheus.
    """
    def __init__(self, port: int = 9091):
        self.port = port
        self.registry = CollectorRegistry()
        self._setup_metrics()
        
    def _setup_metrics(self):
        """Define Prometheus metrics"""
        self.latency_histogram = Histogram(
            'qudag_benchmark_latency_seconds',
            'Benchmark operation latency',
            ['operation', 'suite'],
            registry=self.registry
        )
        
        self.throughput_gauge = Gauge(
            'qudag_benchmark_throughput',
            'Benchmark throughput (ops/sec)',
            ['operation', 'suite'],
            registry=self.registry
        )
        
    def record_latency(self, operation: str, suite: str, latency: float):
        """Record latency measurement"""
        self.latency_histogram.labels(
            operation=operation,
            suite=suite
        ).observe(latency)
        
    def start_server(self):
        """Start Prometheus metrics server"""
        start_http_server(self.port, registry=self.registry)
```

## Security Considerations

### 1. Sandboxing
- Execute benchmarks in isolated environments
- Use resource limits (cgroups/ulimit)
- Restrict network access for security-sensitive benchmarks

### 2. Data Protection
- Encrypt stored benchmark results
- Sanitize sensitive data in reports
- Implement access controls for results

### 3. Code Injection Prevention
- Validate all configuration inputs
- Use safe subprocess execution
- Avoid dynamic code execution

## Performance Optimization

### 1. Framework Overhead Minimization
- Use lazy imports for heavy dependencies
- Implement efficient metric collection buffers
- Minimize allocations in hot paths

### 2. Parallel Execution Optimization
- Use process pools for CPU-bound tasks
- Implement work stealing for load balancing
- Optimize task scheduling algorithms

### 3. Memory Efficiency
- Stream large results to disk
- Implement metric aggregation windows
- Use memory-mapped files for large datasets

## Future Enhancements

### 1. Distributed Benchmarking
- Support for multi-node benchmarks
- Distributed result aggregation
- Network partition simulation

### 2. Machine Learning Integration
- Anomaly detection using ML models
- Performance prediction
- Automated optimization suggestions

### 3. Real-time Monitoring
- WebSocket-based live updates
- Interactive dashboard
- Alert system for regressions

This architecture provides a solid foundation for comprehensive benchmarking of the QuDAG system while maintaining flexibility for future enhancements.