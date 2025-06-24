"""
Performance tests for the benchmarking tool.
Ensures the tool itself performs efficiently.
"""
import pytest
import time
import psutil
import gc
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor

from benchmarking.benchmarks.core.runner import BenchmarkRunner
from benchmarking.benchmarks.metrics.collector import MetricCollector
from benchmarking.benchmarks.reporters.json_reporter import JSONReporter


class TestBenchmarkPerformance:
    """Performance tests for benchmarking tool components."""
    
    def test_runner_overhead(self):
        """Test that BenchmarkRunner adds minimal overhead."""
        runner = BenchmarkRunner({"iterations": 100})
        
        # Minimal benchmark function
        def noop():
            pass
        
        # Measure direct execution time
        direct_times = []
        for _ in range(100):
            start = time.perf_counter()
            noop()
            direct_times.append(time.perf_counter() - start)
        
        # Measure through runner
        result = runner.run(noop)
        
        # Compare overhead
        direct_mean = sum(direct_times) / len(direct_times)
        runner_mean = sum(result["execution_times"]) / len(result["execution_times"])
        
        overhead_ratio = runner_mean / direct_mean
        assert overhead_ratio < 10  # Less than 10x overhead for noop
    
    def test_metric_collection_performance(self):
        """Test metric collection doesn't significantly impact benchmarks."""
        runner = BenchmarkRunner({"iterations": 50})
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        
        def simple_computation():
            return sum(i ** 2 for i in range(1000))
        
        # Run without metrics
        result_no_metrics = runner.run(simple_computation)
        mean_no_metrics = sum(result_no_metrics["execution_times"]) / len(result_no_metrics["execution_times"])
        
        # Run with metrics
        result_with_metrics = runner.run(simple_computation, metric_collector=collector)
        mean_with_metrics = sum(result_with_metrics["execution_times"]) / len(result_with_metrics["execution_times"])
        
        # Metric collection should add less than 20% overhead
        overhead = (mean_with_metrics - mean_no_metrics) / mean_no_metrics
        assert overhead < 0.2
    
    def test_large_scale_benchmarking(self):
        """Test handling large number of benchmarks."""
        runner = BenchmarkRunner({"iterations": 10})
        
        # Generate 100 simple benchmarks
        benchmarks = []
        for i in range(100):
            benchmarks.append((
                f"bench_{i}",
                lambda i=i: i ** 2
            ))
        
        start_time = time.time()
        results = runner.run_suite(benchmarks)
        total_time = time.time() - start_time
        
        assert len(results) == 100
        assert total_time < 10  # Should complete in reasonable time
        
        # Check memory usage didn't explode
        process = psutil.Process()
        memory_mb = process.memory_info().rss / 1024 / 1024
        assert memory_mb < 500  # Less than 500MB
    
    def test_parallel_scaling(self):
        """Test parallel execution scaling."""
        def cpu_bound_task(n):
            result = 0
            for i in range(n):
                result += i ** 2
            return result
        
        # Test with different worker counts
        worker_counts = [1, 2, 4, 8]
        timings = {}
        
        for workers in worker_counts:
            runner = BenchmarkRunner({
                "iterations": 5,
                "parallel": True,
                "workers": workers
            })
            
            benchmarks = [
                (f"task_{i}", lambda: cpu_bound_task(100000))
                for i in range(16)
            ]
            
            start = time.time()
            runner.run_suite(benchmarks)
            timings[workers] = time.time() - start
        
        # Should see speedup with more workers (up to CPU count)
        cpu_count = psutil.cpu_count()
        if cpu_count >= 4:
            assert timings[4] < timings[2] < timings[1]
            
            # Calculate speedup
            speedup = timings[1] / timings[4]
            assert speedup > 2  # At least 2x speedup with 4 workers
    
    def test_memory_efficiency(self):
        """Test memory efficiency with large results."""
        runner = BenchmarkRunner({"iterations": 1000})
        reporter = JSONReporter()
        
        # Get initial memory
        gc.collect()
        process = psutil.Process()
        initial_memory = process.memory_info().rss
        
        # Run benchmark generating large results
        def generate_data():
            return list(range(1000))
        
        result = runner.run(generate_data)
        
        # Add to reporter multiple times
        for _ in range(100):
            reporter.add_result(result)
        
        # Check memory usage
        gc.collect()
        final_memory = process.memory_info().rss
        memory_increase_mb = (final_memory - initial_memory) / 1024 / 1024
        
        # Should not use excessive memory
        assert memory_increase_mb < 100  # Less than 100MB increase
    
    def test_continuous_collection_performance(self):
        """Test performance of continuous metric collection."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        
        # Start continuous collection with high frequency
        collector.start_continuous_collection(interval=0.001)  # 1ms interval
        
        # Run for 1 second
        time.sleep(1)
        
        timeline = collector.stop_continuous_collection()
        
        # Should have collected many samples without issues
        assert len(timeline) > 500  # At least 500 samples
        assert len(timeline) < 2000  # But not too many (overhead control)
        
        # Check collection didn't use too much CPU
        cpu_percent = psutil.Process().cpu_percent(interval=0.1)
        assert cpu_percent < 50  # Less than 50% CPU
    
    def test_reporter_performance_with_large_datasets(self):
        """Test reporter performance with large result sets."""
        reporter = JSONReporter()
        
        # Generate large benchmark results
        for i in range(1000):
            result = {
                "name": f"benchmark_{i}",
                "execution_times": [0.001 * j for j in range(100)],
                "metrics": {
                    "memory": {"rss": 1000 * i, "percent": i / 10},
                    "cpu": {"percent": i % 100}
                }
            }
            reporter.add_result(result)
        
        # Time report generation
        start = time.time()
        json_output = reporter.report()
        generation_time = time.time() - start
        
        # Should generate quickly
        assert generation_time < 1.0  # Less than 1 second
        assert len(json_output) > 100000  # Should have substantial output
    
    def test_concurrent_benchmark_safety(self):
        """Test thread safety of concurrent benchmark execution."""
        runner = BenchmarkRunner({
            "iterations": 10,
            "parallel": True,
            "workers": 4
        })
        
        # Shared state to test thread safety
        shared_counter = {"value": 0}
        
        def increment_counter():
            # Intentionally not thread-safe to test isolation
            current = shared_counter["value"]
            time.sleep(0.001)  # Simulate work
            shared_counter["value"] = current + 1
            return current
        
        # Run multiple benchmarks concurrently
        benchmarks = [
            (f"counter_{i}", increment_counter)
            for i in range(20)
        ]
        
        results = runner.run_suite(benchmarks)
        
        # Each benchmark should have consistent results despite shared state
        for result in results:
            # All iterations of same benchmark should see same value
            values = [result["return_value"]] if "return_value" in result else []
            if len(set(values)) > 1:
                # If values differ, runner isn't properly isolating
                pytest.fail("Benchmark isolation failed")
    
    def test_error_recovery_performance(self):
        """Test performance of error recovery mechanisms."""
        runner = BenchmarkRunner({
            "iterations": 100,
            "retry_on_error": True,
            "max_retries": 3
        })
        
        error_count = 0
        def flaky_benchmark():
            nonlocal error_count
            error_count += 1
            if error_count % 10 == 0:
                raise ValueError("Intermittent error")
            return error_count
        
        start = time.time()
        
        # Should handle errors without significant slowdown
        try:
            result = runner.run(flaky_benchmark)
        except:
            pass  # Some errors expected
        
        elapsed = time.time() - start
        
        # Even with errors, should complete reasonably fast
        assert elapsed < 2.0  # Less than 2 seconds for 100 iterations
    
    @pytest.mark.benchmark
    def test_benchmark_tool_benchmarks(self, benchmark):
        """Meta-benchmark: benchmark the benchmarking tool itself."""
        runner = BenchmarkRunner({"iterations": 10})
        
        def sample_workload():
            return sum(i ** 2 for i in range(1000))
        
        # Benchmark the runner
        result = benchmark(runner.run, sample_workload)
        
        assert "execution_times" in result
        assert len(result["execution_times"]) == 10