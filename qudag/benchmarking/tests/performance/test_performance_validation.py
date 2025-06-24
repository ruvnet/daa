"""Performance validation tests for the benchmarking framework."""

import asyncio
import time
import resource
import gc
from concurrent.futures import ProcessPoolExecutor
from typing import List, Dict, Any
import statistics

import pytest
import psutil

from benchmarking.src.performance_validator import (
    PerformanceValidator,
    PerformanceBaseline,
    PerformanceRegression,
    ResourceMonitor,
    BenchmarkProfiler
)


class TestPerformanceValidator:
    """Test performance validation functionality."""
    
    def test_validator_initialization(self):
        """Test performance validator initialization."""
        thresholds = {
            "max_memory_mb": 512,
            "max_cpu_percent": 80,
            "max_latency_ms": 100,
            "min_throughput": 1000
        }
        
        validator = PerformanceValidator(thresholds)
        
        assert validator.thresholds == thresholds
        assert validator.violations == []
        
    @pytest.mark.asyncio
    async def test_validate_memory_usage(self):
        """Test memory usage validation."""
        validator = PerformanceValidator({"max_memory_mb": 100})
        
        # Function that uses memory
        def memory_intensive_task():
            data = bytearray(50 * 1024 * 1024)  # 50MB
            return len(data)
            
        # Should pass
        result = await validator.validate_memory(memory_intensive_task)
        assert result.passed is True
        assert result.memory_used_mb < 100
        
        # Function that exceeds limit
        def memory_excessive_task():
            data = bytearray(150 * 1024 * 1024)  # 150MB
            return len(data)
            
        # Should fail
        result = await validator.validate_memory(memory_excessive_task)
        assert result.passed is False
        assert result.memory_used_mb > 100
        assert len(validator.violations) == 1
        
    @pytest.mark.asyncio
    async def test_validate_cpu_usage(self):
        """Test CPU usage validation."""
        validator = PerformanceValidator({"max_cpu_percent": 50})
        
        # CPU-intensive function
        async def cpu_intensive_task(duration=0.1):
            start = time.time()
            while time.time() - start < duration:
                # Busy loop
                _ = sum(i**2 for i in range(1000))
            
        # Run and validate
        result = await validator.validate_cpu(cpu_intensive_task, duration=0.1)
        
        assert result.cpu_percent >= 0
        assert result.duration > 0
        
        # Note: CPU validation is tricky in test environments
        # We mainly check that monitoring works
        
    @pytest.mark.asyncio
    async def test_validate_latency(self):
        """Test latency validation."""
        validator = PerformanceValidator({"max_latency_ms": 50})
        
        # Fast function
        async def fast_function():
            await asyncio.sleep(0.01)  # 10ms
            return "done"
            
        result = await validator.validate_latency(fast_function)
        assert result.passed is True
        assert result.latency_ms < 50
        
        # Slow function
        async def slow_function():
            await asyncio.sleep(0.1)  # 100ms
            return "done"
            
        result = await validator.validate_latency(slow_function)
        assert result.passed is False
        assert result.latency_ms > 50
        
    @pytest.mark.asyncio
    async def test_validate_throughput(self):
        """Test throughput validation."""
        validator = PerformanceValidator({"min_throughput": 100})
        
        # High throughput function
        async def high_throughput_task():
            operations = 0
            start = time.time()
            while time.time() - start < 0.1:
                operations += 1
                if operations % 10 == 0:
                    await asyncio.sleep(0)  # Yield control
            return operations
            
        result = await validator.validate_throughput(
            high_throughput_task,
            operation_count_getter=lambda r: r
        )
        
        assert result.throughput > 100
        assert result.passed is True
        
    def test_generate_validation_report(self):
        """Test validation report generation."""
        validator = PerformanceValidator({
            "max_memory_mb": 100,
            "max_latency_ms": 50
        })
        
        # Add some mock violations
        validator.violations = [
            {"type": "memory", "threshold": 100, "actual": 150, "test": "test1"},
            {"type": "latency", "threshold": 50, "actual": 75, "test": "test2"}
        ]
        
        report = validator.generate_report()
        
        assert report["total_validations"] >= 0
        assert report["violations_count"] == 2
        assert len(report["violations"]) == 2
        assert report["passed"] is False


class TestPerformanceBaseline:
    """Test performance baseline management."""
    
    def test_baseline_creation(self):
        """Test creating performance baseline."""
        metrics = {
            "transaction_throughput": 1000,
            "consensus_latency_ms": 50,
            "memory_usage_mb": 256,
            "cpu_usage_percent": 45
        }
        
        baseline = PerformanceBaseline("v1.0", metrics)
        
        assert baseline.version == "v1.0"
        assert baseline.metrics == metrics
        assert baseline.timestamp is not None
        
    def test_baseline_comparison(self):
        """Test comparing against baseline."""
        baseline = PerformanceBaseline("v1.0", {
            "throughput": 1000,
            "latency_ms": 50
        })
        
        # Current metrics within tolerance
        current = {
            "throughput": 950,  # 5% degradation
            "latency_ms": 52    # 4% increase
        }
        
        comparison = baseline.compare(current, tolerance=0.1)
        
        assert comparison["passed"] is True
        assert comparison["throughput"]["degradation"] == 0.05
        assert comparison["latency_ms"]["degradation"] == 0.04
        
        # Current metrics exceed tolerance
        current_bad = {
            "throughput": 800,  # 20% degradation
            "latency_ms": 65    # 30% increase
        }
        
        comparison = baseline.compare(current_bad, tolerance=0.1)
        
        assert comparison["passed"] is False
        assert len(comparison["failures"]) == 2
        
    def test_baseline_persistence(self, tmp_path):
        """Test saving and loading baseline."""
        baseline = PerformanceBaseline("v1.0", {
            "metric1": 100,
            "metric2": 200
        })
        
        # Save baseline
        file_path = tmp_path / "baseline.json"
        baseline.save(file_path)
        
        assert file_path.exists()
        
        # Load baseline
        loaded = PerformanceBaseline.load(file_path)
        
        assert loaded.version == "v1.0"
        assert loaded.metrics == baseline.metrics
        
    def test_baseline_history(self):
        """Test maintaining baseline history."""
        history = PerformanceBaseline.create_history()
        
        # Add baselines
        history.add(PerformanceBaseline("v1.0", {"tps": 1000}))
        history.add(PerformanceBaseline("v1.1", {"tps": 1100}))
        history.add(PerformanceBaseline("v1.2", {"tps": 1050}))
        
        assert len(history) == 3
        assert history.get_latest().version == "v1.2"
        assert history.get("v1.0").metrics["tps"] == 1000
        
        # Analyze trends
        trend = history.analyze_trend("tps")
        assert trend["direction"] == "improving"  # 1000 -> 1100 -> 1050
        assert trend["total_change"] == 0.05  # 5% improvement


class TestPerformanceRegression:
    """Test performance regression detection."""
    
    def test_regression_detection(self):
        """Test detecting performance regressions."""
        detector = PerformanceRegression(threshold=0.1)  # 10% threshold
        
        # Add historical data
        historical_data = [
            {"timestamp": 1, "latency": 50, "throughput": 1000},
            {"timestamp": 2, "latency": 52, "throughput": 980},
            {"timestamp": 3, "latency": 51, "throughput": 990},
            {"timestamp": 4, "latency": 53, "throughput": 970},
            {"timestamp": 5, "latency": 50, "throughput": 1000}
        ]
        
        for data in historical_data:
            detector.add_measurement(data)
            
        # Check for regression in latest data
        current = {"latency": 60, "throughput": 850}  # Significant degradation
        
        regressions = detector.detect(current)
        
        assert len(regressions) > 0
        assert any(r["metric"] == "latency" for r in regressions)
        assert any(r["metric"] == "throughput" for r in regressions)
        
    def test_statistical_regression_analysis(self):
        """Test statistical regression analysis."""
        detector = PerformanceRegression(method="statistical")
        
        # Generate data with trend
        measurements = []
        for i in range(20):
            measurements.append({
                "timestamp": i,
                "latency": 50 + i * 0.5 + (i % 3),  # Increasing trend with noise
                "throughput": 1000 - i * 10         # Decreasing trend
            })
            
        for m in measurements:
            detector.add_measurement(m)
            
        analysis = detector.analyze_trends()
        
        assert analysis["latency"]["trend"] == "increasing"
        assert analysis["latency"]["confidence"] > 0.8
        assert analysis["throughput"]["trend"] == "decreasing"
        assert analysis["throughput"]["slope"] < 0
        
    def test_anomaly_detection(self):
        """Test anomaly detection in performance data."""
        detector = PerformanceRegression(anomaly_detection=True)
        
        # Normal measurements
        for i in range(10):
            detector.add_measurement({
                "latency": 50 + (i % 2) * 2,  # 50 or 52
                "throughput": 1000 + (i % 3) * 10  # 1000, 1010, or 1020
            })
            
        # Anomalous measurement
        anomaly = {"latency": 150, "throughput": 500}  # Way off normal
        
        is_anomaly = detector.is_anomaly(anomaly)
        anomaly_scores = detector.get_anomaly_scores(anomaly)
        
        assert is_anomaly is True
        assert anomaly_scores["latency"] > 3.0  # More than 3 std devs
        assert anomaly_scores["throughput"] > 3.0


class TestResourceMonitor:
    """Test resource monitoring functionality."""
    
    @pytest.mark.asyncio
    async def test_monitor_initialization(self):
        """Test resource monitor initialization."""
        monitor = ResourceMonitor(
            sample_interval=0.1,
            metrics=["cpu", "memory", "io"]
        )
        
        assert monitor.sample_interval == 0.1
        assert "cpu" in monitor.metrics
        assert monitor.is_monitoring is False
        
    @pytest.mark.asyncio
    async def test_continuous_monitoring(self):
        """Test continuous resource monitoring."""
        monitor = ResourceMonitor(sample_interval=0.05)
        
        # Start monitoring
        monitor_task = asyncio.create_task(monitor.start())
        
        # Let it collect some samples
        await asyncio.sleep(0.2)
        
        # Stop monitoring
        await monitor.stop()
        await monitor_task
        
        samples = monitor.get_samples()
        
        assert len(samples) >= 3
        assert all("timestamp" in s for s in samples)
        assert all("cpu_percent" in s for s in samples)
        assert all("memory_mb" in s for s in samples)
        
    @pytest.mark.asyncio
    async def test_monitor_context_manager(self):
        """Test monitor as context manager."""
        monitor = ResourceMonitor(sample_interval=0.05)
        
        async with monitor:
            # Simulate some work
            data = bytearray(10 * 1024 * 1024)  # 10MB
            await asyncio.sleep(0.15)
            del data
            
        summary = monitor.get_summary()
        
        assert summary["duration"] > 0.15
        assert summary["samples_collected"] >= 3
        assert "cpu_percent" in summary["averages"]
        assert "memory_mb" in summary["peaks"]
        
    @pytest.mark.asyncio
    async def test_resource_alerts(self):
        """Test resource usage alerts."""
        monitor = ResourceMonitor(
            alerts={
                "cpu_percent": 90,
                "memory_mb": 1000
            }
        )
        
        # Mock high resource usage
        triggered_alerts = []
        
        def alert_handler(alert):
            triggered_alerts.append(alert)
            
        monitor.on_alert = alert_handler
        
        # Simulate high CPU
        monitor._current_samples = [
            {"cpu_percent": 95, "memory_mb": 500}
        ]
        
        await monitor._check_alerts()
        
        assert len(triggered_alerts) == 1
        assert triggered_alerts[0]["metric"] == "cpu_percent"
        assert triggered_alerts[0]["threshold"] == 90
        assert triggered_alerts[0]["actual"] == 95


class TestBenchmarkProfiler:
    """Test benchmark profiling capabilities."""
    
    def test_profiler_initialization(self):
        """Test profiler initialization."""
        profiler = BenchmarkProfiler(
            profile_cpu=True,
            profile_memory=True,
            profile_io=True
        )
        
        assert profiler.profile_cpu is True
        assert profiler.profile_memory is True
        assert profiler.profile_io is True
        
    @pytest.mark.asyncio
    async def test_function_profiling(self):
        """Test profiling function execution."""
        profiler = BenchmarkProfiler()
        
        # Function to profile
        async def test_function():
            # CPU work
            result = sum(i**2 for i in range(1000))
            
            # Memory allocation
            data = bytearray(1024 * 1024)  # 1MB
            
            # Async I/O simulation
            await asyncio.sleep(0.01)
            
            return result
            
        profile_data = await profiler.profile(test_function)
        
        assert profile_data["function_name"] == "test_function"
        assert profile_data["execution_time"] > 0
        assert "cpu_time" in profile_data
        assert "memory_peak" in profile_data
        assert profile_data["result"] == sum(i**2 for i in range(1000))
        
    @pytest.mark.asyncio
    async def test_call_graph_generation(self):
        """Test call graph generation for profiled code."""
        profiler = BenchmarkProfiler(generate_callgraph=True)
        
        async def parent_function():
            await child_function_a()
            await child_function_b()
            
        async def child_function_a():
            await asyncio.sleep(0.001)
            
        async def child_function_b():
            await asyncio.sleep(0.001)
            
        profile_data = await profiler.profile(parent_function)
        
        assert "call_graph" in profile_data
        assert len(profile_data["call_graph"]["nodes"]) >= 3
        assert any(n["name"] == "parent_function" for n in profile_data["call_graph"]["nodes"])
        
    @pytest.mark.asyncio
    async def test_memory_allocation_tracking(self):
        """Test tracking memory allocations."""
        profiler = BenchmarkProfiler(track_allocations=True)
        
        async def memory_test():
            allocations = []
            for i in range(5):
                # Allocate different sizes
                allocations.append(bytearray(i * 1024 * 1024))
                await asyncio.sleep(0.001)
            return len(allocations)
            
        profile_data = await profiler.profile(memory_test)
        
        assert "memory_allocations" in profile_data
        assert len(profile_data["memory_allocations"]) >= 5
        assert profile_data["total_allocated_mb"] > 10  # 0+1+2+3+4 = 10MB minimum
        
    def test_profiler_report_generation(self, tmp_path):
        """Test generating profiler reports."""
        profiler = BenchmarkProfiler()
        
        # Add mock profile data
        profiler.profiles = [
            {
                "function_name": "test_func_1",
                "execution_time": 0.1,
                "cpu_time": 0.08,
                "memory_peak": 100
            },
            {
                "function_name": "test_func_2",
                "execution_time": 0.2,
                "cpu_time": 0.15,
                "memory_peak": 200
            }
        ]
        
        # Generate report
        report_path = profiler.generate_report(
            output_dir=tmp_path,
            format="html"
        )
        
        assert report_path.exists()
        assert report_path.suffix == ".html"
        
        # Verify report content
        content = report_path.read_text()
        assert "test_func_1" in content
        assert "test_func_2" in content
        assert "Performance Profile Report" in content


class TestPerformanceOptimization:
    """Test performance optimization helpers."""
    
    @pytest.mark.asyncio
    async def test_parallel_execution_optimization(self):
        """Test optimizing parallel execution."""
        from benchmarking.src.optimization import ParallelExecutor
        
        # Tasks with different execution times
        async def task(duration):
            await asyncio.sleep(duration)
            return duration
            
        tasks = [
            lambda: task(0.01),
            lambda: task(0.02),
            lambda: task(0.015),
            lambda: task(0.025),
            lambda: task(0.01)
        ]
        
        # Sequential execution
        start = time.time()
        sequential_results = []
        for t in tasks:
            sequential_results.append(await t())
        sequential_time = time.time() - start
        
        # Parallel execution
        executor = ParallelExecutor(max_workers=3)
        start = time.time()
        parallel_results = await executor.execute_all(tasks)
        parallel_time = time.time() - start
        
        assert len(parallel_results) == len(sequential_results)
        assert set(parallel_results) == set(sequential_results)
        assert parallel_time < sequential_time * 0.6  # At least 40% faster
        
    @pytest.mark.asyncio
    async def test_batch_processing_optimization(self):
        """Test batch processing optimization."""
        from benchmarking.src.optimization import BatchProcessor
        
        # Simulate processing overhead
        processing_count = {"value": 0}
        
        async def process_items(items):
            processing_count["value"] += 1
            await asyncio.sleep(0.01)  # Fixed overhead
            return [item * 2 for item in items]
            
        processor = BatchProcessor(
            process_func=process_items,
            batch_size=10
        )
        
        # Process many items
        items = list(range(100))
        results = await processor.process_all(items)
        
        assert len(results) == 100
        assert results == [i * 2 for i in range(100)]
        assert processing_count["value"] == 10  # 100 items / batch_size 10
        
    def test_cache_optimization(self):
        """Test caching optimization for repeated operations."""
        from benchmarking.src.optimization import MemoizedFunction
        
        call_count = {"value": 0}
        
        @MemoizedFunction(max_size=100)
        def expensive_function(n):
            call_count["value"] += 1
            return sum(i**2 for i in range(n))
            
        # First calls
        result1 = expensive_function(1000)
        result2 = expensive_function(2000)
        
        assert call_count["value"] == 2
        
        # Cached calls
        result1_cached = expensive_function(1000)
        result2_cached = expensive_function(2000)
        
        assert call_count["value"] == 2  # No additional calls
        assert result1 == result1_cached
        assert result2 == result2_cached
        
        # Cache statistics
        stats = expensive_function.cache_stats()
        assert stats["hits"] == 2
        assert stats["misses"] == 2
        assert stats["hit_rate"] == 0.5