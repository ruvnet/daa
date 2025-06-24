"""
Integration tests for the benchmarking tool.
Tests how components work together in real scenarios.
"""
import pytest
from unittest.mock import Mock, patch
import json
import time
import tempfile
from pathlib import Path

from benchmarking.benchmarks.core.runner import BenchmarkRunner
from benchmarking.benchmarks.metrics.collector import MetricCollector
from benchmarking.benchmarks.reporters.reporter import ResultReporter
from benchmarking.benchmarks.reporters.json_reporter import JSONReporter
from benchmarking.benchmarks.reporters.console import ConsoleReporter
from benchmarking.cli import BenchmarkCLI


class TestBenchmarkIntegration:
    """Integration tests for benchmarking components."""
    
    def test_full_benchmark_pipeline(self):
        """Test complete benchmark pipeline from execution to reporting."""
        # Setup
        runner = BenchmarkRunner({
            "name": "integration_test",
            "iterations": 5,
            "warmup": 1
        })
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        reporter = JSONReporter()
        
        # Define benchmark
        def sample_workload():
            data = [i ** 2 for i in range(10000)]
            time.sleep(0.01)
            return sum(data)
        
        # Execute
        result = runner.run(sample_workload, metric_collector=collector)
        
        # Verify execution results
        assert result["name"] == "integration_test"
        assert len(result["execution_times"]) == 5
        assert "metrics" in result
        assert "memory" in result["metrics"]
        assert "cpu" in result["metrics"]
        assert result["return_value"] == sum(i ** 2 for i in range(10000))
        
        # Report
        reporter.add_result(result)
        json_output = reporter.report()
        
        # Verify report
        data = json.loads(json_output)
        assert len(data["results"]) == 1
        assert data["results"][0]["name"] == "integration_test"
    
    def test_multiple_benchmarks_with_comparison(self):
        """Test running multiple benchmarks and comparing results."""
        runner = BenchmarkRunner({"iterations": 10, "warmup": 2})
        collector = MetricCollector()
        reporter = ConsoleReporter()
        
        # Define different algorithms to benchmark
        def bubble_sort(arr):
            n = len(arr)
            for i in range(n):
                for j in range(0, n-i-1):
                    if arr[j] > arr[j+1]:
                        arr[j], arr[j+1] = arr[j+1], arr[j]
            return arr
        
        def quick_sort(arr):
            if len(arr) <= 1:
                return arr
            pivot = arr[len(arr) // 2]
            left = [x for x in arr if x < pivot]
            middle = [x for x in arr if x == pivot]
            right = [x for x in arr if x > pivot]
            return quick_sort(left) + middle + quick_sort(right)
        
        # Test data
        test_data = list(range(100, 0, -1))
        
        # Run benchmarks
        benchmarks = [
            ("bubble_sort", lambda: bubble_sort(test_data.copy())),
            ("quick_sort", lambda: quick_sort(test_data.copy()))
        ]
        
        results = runner.run_suite(benchmarks, metric_collector=collector)
        
        # Add to reporter
        for result in results:
            reporter.add_result(result)
        
        # Verify results
        assert len(results) == 2
        assert results[0]["name"] == "bubble_sort"
        assert results[1]["name"] == "quick_sort"
        
        # Quick sort should be faster
        bubble_mean = sum(results[0]["execution_times"]) / len(results[0]["execution_times"])
        quick_mean = sum(results[1]["execution_times"]) / len(results[1]["execution_times"])
        assert quick_mean < bubble_mean
    
    def test_cli_integration(self):
        """Test CLI integration with benchmarking components."""
        with tempfile.TemporaryDirectory() as tmpdir:
            config_file = Path(tmpdir) / "benchmark_config.json"
            config = {
                "benchmarks": [
                    {
                        "name": "test_bench",
                        "module": "test_module",
                        "function": "benchmark_func",
                        "iterations": 3
                    }
                ],
                "output": {
                    "format": "json",
                    "file": str(Path(tmpdir) / "results.json")
                }
            }
            
            with open(config_file, 'w') as f:
                json.dump(config, f)
            
            # Mock the benchmark function
            with patch('benchmarking.cli.import_benchmark') as mock_import:
                mock_import.return_value = lambda: 42
                
                cli = BenchmarkCLI()
                cli.run_from_config(str(config_file))
                
                # Verify results file was created
                results_file = Path(tmpdir) / "results.json"
                assert results_file.exists()
                
                with open(results_file) as f:
                    results = json.load(f)
                    assert len(results["results"]) == 1
                    assert results["results"][0]["name"] == "test_bench"
    
    def test_continuous_monitoring(self):
        """Test continuous metric monitoring during benchmark."""
        runner = BenchmarkRunner({"name": "monitor_test", "iterations": 1})
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        
        def workload_with_phases():
            # Phase 1: Memory allocation
            data1 = [i for i in range(1000000)]
            time.sleep(0.1)
            
            # Phase 2: CPU intensive
            for _ in range(1000000):
                _ = 2 ** 10
            
            # Phase 3: Memory allocation
            data2 = [i ** 2 for i in range(1000000)]
            time.sleep(0.1)
            
            return len(data1) + len(data2)
        
        # Run with continuous monitoring
        collector.start_continuous_collection(interval=0.02)
        result = runner.run(workload_with_phases)
        timeline = collector.stop_continuous_collection()
        
        # Verify timeline data
        assert len(timeline) > 5  # Should have multiple samples
        
        # Memory should increase over time
        memory_values = [s["memory"]["rss"] for s in timeline]
        assert max(memory_values) > min(memory_values)
        
        # Should have captured metrics in result
        result["metrics"]["timeline"] = timeline
        assert "timeline" in result["metrics"]
    
    def test_error_handling_pipeline(self):
        """Test error handling throughout the pipeline."""
        runner = BenchmarkRunner({"name": "error_test", "iterations": 3})
        collector = MetricCollector()
        reporter = JSONReporter()
        
        # Benchmark that fails intermittently
        call_count = 0
        def flaky_benchmark():
            nonlocal call_count
            call_count += 1
            if call_count == 2:
                raise ValueError("Simulated failure")
            return call_count
        
        # Should handle the error
        with pytest.raises(ValueError):
            runner.run(flaky_benchmark)
        
        # Try with error recovery
        runner.config["retry_on_error"] = True
        runner.config["max_retries"] = 3
        
        result = runner.run(flaky_benchmark)
        assert result["completed_iterations"] == 2  # One failed
        assert result["errors"] == 1
    
    def test_parallel_benchmark_execution_integration(self):
        """Test parallel execution with metric collection."""
        runner = BenchmarkRunner({
            "parallel": True,
            "workers": 4,
            "iterations": 5
        })
        
        # Create benchmarks that can run in parallel
        def compute_intensive(n):
            result = 0
            for i in range(n):
                result += i ** 2
            return result
        
        benchmarks = [
            (f"compute_{n}", lambda n=n: compute_intensive(n * 1000))
            for n in range(1, 9)
        ]
        
        start_time = time.time()
        results = runner.run_suite(benchmarks)
        total_time = time.time() - start_time
        
        # Verify all completed
        assert len(results) == 8
        assert all("execution_times" in r for r in results)
        
        # Should be faster than sequential
        sequential_estimate = sum(
            sum(r["execution_times"]) for r in results
        )
        assert total_time < sequential_estimate * 0.5  # At least 2x speedup
    
    def test_memory_benchmark_integration(self):
        """Test benchmarking memory-intensive operations."""
        runner = BenchmarkRunner({"name": "memory_test", "iterations": 3})
        collector = MetricCollector()
        collector.enable_metric("memory")
        
        def memory_intensive():
            # Allocate and deallocate memory
            allocations = []
            for i in range(5):
                allocations.append([0] * (10 ** 6))  # ~8MB each
            
            # Process data
            result = sum(len(a) for a in allocations)
            
            # Clear some allocations
            allocations = allocations[:2]
            
            return result
        
        result = runner.run(memory_intensive, metric_collector=collector)
        
        # Should have memory metrics
        assert "memory" in result["metrics"]
        assert result["metrics"]["memory"]["peak"] > result["metrics"]["memory"]["initial"]
    
    def test_custom_metric_integration(self):
        """Test integration with custom metrics."""
        runner = BenchmarkRunner({"name": "custom_metric", "iterations": 5})
        collector = MetricCollector()
        
        # Add custom metric
        class CacheMetric:
            def __init__(self):
                self.hits = 0
                self.misses = 0
            
            def collect(self):
                total = self.hits + self.misses
                hit_rate = self.hits / total if total > 0 else 0
                return {
                    "hits": self.hits,
                    "misses": self.misses,
                    "hit_rate": hit_rate
                }
            
            def reset(self):
                self.hits = 0
                self.misses = 0
        
        cache_metric = CacheMetric()
        collector.add_metric("cache", cache_metric)
        collector.enable_metric("cache")
        
        def benchmark_with_cache():
            # Simulate cache behavior
            import random
            for _ in range(100):
                if random.random() < 0.7:  # 70% hit rate
                    cache_metric.hits += 1
                else:
                    cache_metric.misses += 1
            return cache_metric.hits
        
        result = runner.run(benchmark_with_cache, metric_collector=collector)
        
        assert "cache" in result["metrics"]
        assert result["metrics"]["cache"]["hit_rate"] > 0.6
        assert result["metrics"]["cache"]["hit_rate"] < 0.8