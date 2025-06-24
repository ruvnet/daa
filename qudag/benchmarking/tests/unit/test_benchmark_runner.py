"""
Unit tests for BenchmarkRunner component.
Tests the core benchmark execution logic.
"""
import pytest
from unittest.mock import Mock, patch, MagicMock
import time
from typing import Dict, Any, List

from benchmarking.benchmarks.core.runner import BenchmarkRunner
from benchmarking.benchmarks.metrics.collector import MetricCollector


class TestBenchmarkRunner:
    """Test cases for BenchmarkRunner class."""
    
    def test_benchmark_runner_initialization(self):
        """Test BenchmarkRunner can be initialized with configuration."""
        config = {
            "name": "test_benchmark",
            "iterations": 10,
            "warmup": 2,
            "timeout": 30
        }
        runner = BenchmarkRunner(config)
        
        assert runner.name == "test_benchmark"
        assert runner.iterations == 10
        assert runner.warmup == 2
        assert runner.timeout == 30
        assert runner.results == []
    
    def test_run_single_benchmark(self):
        """Test running a single benchmark function."""
        runner = BenchmarkRunner({"name": "test", "iterations": 3, "warmup": 1})
        
        def sample_benchmark():
            time.sleep(0.01)
            return 42
        
        result = runner.run(sample_benchmark)
        
        assert result["name"] == "test"
        assert result["iterations"] == 3
        assert "execution_times" in result
        assert len(result["execution_times"]) == 3
        assert all(t > 0.01 for t in result["execution_times"])
        assert result["return_value"] == 42
    
    def test_run_with_warmup(self):
        """Test benchmark warmup phase."""
        runner = BenchmarkRunner({"name": "test", "iterations": 5, "warmup": 2})
        call_count = 0
        
        def counting_benchmark():
            nonlocal call_count
            call_count += 1
            return call_count
        
        result = runner.run(counting_benchmark)
        
        # Should be called warmup + iterations times
        assert call_count == 7  # 2 warmup + 5 iterations
        assert len(result["execution_times"]) == 5  # Only iteration times recorded
    
    def test_run_with_timeout(self):
        """Test benchmark timeout handling."""
        runner = BenchmarkRunner({"name": "test", "iterations": 1, "timeout": 0.1})
        
        def slow_benchmark():
            time.sleep(0.5)
        
        from benchmarking.benchmarks.core.runner import TimeoutError as BenchmarkTimeoutError
        with pytest.raises(BenchmarkTimeoutError):
            runner.run(slow_benchmark)
    
    def test_run_with_exception(self):
        """Test handling of benchmark exceptions."""
        runner = BenchmarkRunner({"name": "test", "iterations": 1})
        
        def failing_benchmark():
            raise ValueError("Benchmark failed")
        
        with pytest.raises(ValueError) as exc_info:
            runner.run(failing_benchmark)
        
        assert str(exc_info.value) == "Benchmark failed"
    
    def test_run_with_arguments(self):
        """Test running benchmark with arguments."""
        runner = BenchmarkRunner({"name": "test", "iterations": 3})
        
        def parameterized_benchmark(x, y, z=10):
            return x + y + z
        
        result = runner.run(parameterized_benchmark, 5, 3, z=20)
        
        assert result["return_value"] == 28
        assert "args" in result
        assert result["args"] == (5, 3)
        assert result["kwargs"] == {"z": 20}
    
    def test_run_with_metric_collector(self):
        """Test integration with MetricCollector."""
        runner = BenchmarkRunner({"name": "test", "iterations": 3})
        collector = MetricCollector()
        
        def benchmark_with_metrics():
            return {"custom_metric": 100}
        
        result = runner.run(benchmark_with_metrics, metric_collector=collector)
        
        assert "metrics" in result
        assert "memory_usage" in result["metrics"] or "custom_metric" in result["metrics"]
    
    def test_run_multiple_benchmarks(self):
        """Test running multiple benchmarks in sequence."""
        runner = BenchmarkRunner({"name": "suite", "iterations": 2})
        
        benchmarks = [
            ("bench1", lambda: 1),
            ("bench2", lambda: 2),
            ("bench3", lambda: 3)
        ]
        
        results = runner.run_suite(benchmarks)
        
        assert len(results) == 3
        assert results[0]["name"] == "bench1"
        assert results[0]["return_value"] == 1
        assert results[2]["name"] == "bench3"
        assert results[2]["return_value"] == 3
    
    def test_parallel_benchmark_execution(self):
        """Test parallel execution of benchmarks."""
        runner = BenchmarkRunner({
            "name": "parallel",
            "iterations": 3,
            "parallel": True,
            "workers": 2
        })
        
        def slow_benchmark(n):
            time.sleep(0.1)
            return n * 2
        
        benchmarks = [(f"bench{i}", lambda i=i: slow_benchmark(i)) for i in range(4)]
        
        start_time = time.time()
        results = runner.run_suite(benchmarks)
        elapsed = time.time() - start_time
        
        # Should be faster than sequential (1.6s for 4x0.4s) due to parallelism
        assert elapsed < 1.0  # Allow more time for parallel overhead
        assert len(results) == 4
        assert all(r["return_value"] == i * 2 for i, r in enumerate(results))
    
    def test_benchmark_context_manager(self):
        """Test using BenchmarkRunner as context manager."""
        config = {"name": "context_test", "iterations": 1}
        
        with BenchmarkRunner(config) as runner:
            result = runner.run(lambda: 42)
            assert result["return_value"] == 42
        
        # Should have cleanup performed
        assert runner._cleaned_up
    
    def test_custom_timer_function(self):
        """Test using custom timer function."""
        custom_timer_called = False
        
        def custom_timer():
            nonlocal custom_timer_called
            custom_timer_called = True
            return time.perf_counter()
        
        runner = BenchmarkRunner({
            "name": "test",
            "iterations": 1,
            "timer": custom_timer
        })
        
        runner.run(lambda: None)
        assert custom_timer_called
    
    def test_benchmark_comparison(self):
        """Test comparing multiple benchmark results."""
        runner = BenchmarkRunner({"name": "comparison", "iterations": 10})
        
        def fast_algo():
            return sum(range(100))
        
        def slow_algo():
            time.sleep(0.01)
            return sum(range(100))
        
        fast_result = runner.run(fast_algo)
        slow_result = runner.run(slow_algo)
        
        comparison = runner.compare([fast_result, slow_result])
        
        assert comparison["fastest"] == "comparison"  # fast_algo
        assert comparison["slowest"] == "comparison"  # slow_algo
        assert comparison["speedup"] > 1.0