"""Shared pytest fixtures and utilities for QuDAG benchmarking tests."""

import asyncio
import json
import os
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Protocol
from unittest.mock import Mock, AsyncMock

import pytest
import pytest_asyncio
from pytest_benchmark.fixture import BenchmarkFixture


# Test data directory
TEST_DATA_DIR = Path(__file__).parent / "test_data"
TEST_DATA_DIR.mkdir(exist_ok=True)


@dataclass
class BenchmarkResult:
    """Container for benchmark results."""
    name: str
    duration: float
    iterations: int
    memory_peak: int
    cpu_percent: float
    metadata: Dict[str, Any]


class QuDAGProtocol(Protocol):
    """Protocol defining QuDAG interface for testing."""
    
    async def connect(self, address: str) -> bool:
        """Connect to QuDAG node."""
        ...
    
    async def submit_transaction(self, data: bytes) -> str:
        """Submit transaction to DAG."""
        ...
    
    async def query_dag(self, query: Dict[str, Any]) -> List[Dict]:
        """Query DAG state."""
        ...
    
    async def get_metrics(self) -> Dict[str, Any]:
        """Get performance metrics."""
        ...


@pytest.fixture(scope="session")
def event_loop():
    """Create event loop for async tests."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def mock_qudag():
    """Mock QuDAG instance for unit tests."""
    mock = Mock(spec=QuDAGProtocol)
    
    # Configure mock behaviors
    mock.connect = AsyncMock(return_value=True)
    mock.submit_transaction = AsyncMock(return_value="tx_hash_12345")
    mock.query_dag = AsyncMock(return_value=[
        {"id": "vertex1", "height": 0, "parents": []},
        {"id": "vertex2", "height": 1, "parents": ["vertex1"]}
    ])
    mock.get_metrics = AsyncMock(return_value={
        "transactions_per_second": 1000,
        "latency_ms": 50,
        "memory_mb": 256,
        "cpu_percent": 45.5
    })
    
    return mock


@pytest.fixture
def benchmark_config():
    """Benchmark configuration fixture."""
    return {
        "warmup_iterations": 10,
        "test_iterations": 100,
        "timeout_seconds": 30,
        "memory_limit_mb": 1024,
        "cpu_limit_percent": 80,
        "parallel_workers": 4,
        "transaction_sizes": [100, 1000, 10000, 100000],  # bytes
        "batch_sizes": [1, 10, 100, 1000],
        "network_delays": [0, 10, 50, 100, 500],  # milliseconds
    }


@pytest.fixture
def test_data_generator():
    """Generate test data for benchmarks."""
    def generate(size: int, pattern: str = "random") -> bytes:
        if pattern == "random":
            return os.urandom(size)
        elif pattern == "sequential":
            return bytes(range(size % 256)) * (size // 256 + 1)
        elif pattern == "sparse":
            data = bytearray(size)
            for i in range(0, size, 10):
                data[i] = i % 256
            return bytes(data)
        else:
            raise ValueError(f"Unknown pattern: {pattern}")
    
    return generate


@pytest.fixture
def performance_monitor():
    """Monitor performance metrics during tests."""
    class PerformanceMonitor:
        def __init__(self):
            self.metrics = []
            self.start_time = None
            
        def start(self):
            self.start_time = time.perf_counter()
            self.metrics = []
            
        def record(self, metric_name: str, value: float):
            elapsed = time.perf_counter() - self.start_time
            self.metrics.append({
                "timestamp": elapsed,
                "metric": metric_name,
                "value": value
            })
            
        def get_summary(self) -> Dict[str, Any]:
            if not self.metrics:
                return {}
                
            summary = {}
            metric_groups = {}
            
            for metric in self.metrics:
                name = metric["metric"]
                if name not in metric_groups:
                    metric_groups[name] = []
                metric_groups[name].append(metric["value"])
                
            for name, values in metric_groups.items():
                summary[name] = {
                    "min": min(values),
                    "max": max(values),
                    "mean": sum(values) / len(values),
                    "count": len(values)
                }
                
            return summary
    
    return PerformanceMonitor()


@pytest.fixture
def mock_dag_state():
    """Mock DAG state for testing."""
    return {
        "vertices": {
            "genesis": {
                "id": "genesis",
                "height": 0,
                "parents": [],
                "timestamp": 1000000000,
                "data": b"genesis block"
            }
        },
        "tips": ["genesis"],
        "height": 0,
        "total_vertices": 1
    }


@pytest.fixture
async def simulated_network_conditions():
    """Simulate various network conditions."""
    class NetworkSimulator:
        def __init__(self):
            self.delay_ms = 0
            self.packet_loss = 0.0
            self.bandwidth_mbps = 1000
            
        async def apply_delay(self):
            """Apply network delay."""
            if self.delay_ms > 0:
                await asyncio.sleep(self.delay_ms / 1000.0)
                
        def should_drop_packet(self) -> bool:
            """Determine if packet should be dropped."""
            import random
            return random.random() < self.packet_loss
            
        def throttle_bandwidth(self, data_size: int) -> float:
            """Calculate transfer time based on bandwidth."""
            if self.bandwidth_mbps <= 0:
                return 0
            # Convert to seconds
            return (data_size * 8) / (self.bandwidth_mbps * 1_000_000)
    
    return NetworkSimulator()


@pytest.fixture
def benchmark_reporter(tmp_path):
    """Generate benchmark reports."""
    class BenchmarkReporter:
        def __init__(self, output_dir: Path):
            self.output_dir = output_dir
            self.results = []
            
        def add_result(self, result: BenchmarkResult):
            self.results.append(result)
            
        def generate_report(self) -> Path:
            report_path = self.output_dir / "benchmark_report.json"
            report_data = {
                "timestamp": time.time(),
                "results": [
                    {
                        "name": r.name,
                        "duration": r.duration,
                        "iterations": r.iterations,
                        "memory_peak": r.memory_peak,
                        "cpu_percent": r.cpu_percent,
                        "metadata": r.metadata
                    }
                    for r in self.results
                ]
            }
            
            with open(report_path, "w") as f:
                json.dump(report_data, f, indent=2)
                
            return report_path
    
    return BenchmarkReporter(tmp_path)


@pytest.fixture
def cleanup_after_test():
    """Cleanup resources after test."""
    resources = []
    
    def register(resource):
        resources.append(resource)
    
    yield register
    
    # Cleanup
    for resource in resources:
        if hasattr(resource, "close"):
            resource.close()
        elif hasattr(resource, "cleanup"):
            resource.cleanup()


# Markers for test organization
def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers", "requires_qudag: mark test as requiring QuDAG connection"
    )
    config.addinivalue_line(
        "markers", "stress_test: mark test as stress test"
    )
    config.addinivalue_line(
        "markers", "data_driven: mark test as data-driven test"
    )