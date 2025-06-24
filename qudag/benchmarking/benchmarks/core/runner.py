"""
Core benchmark runner implementation.
Handles benchmark execution, timing, and result collection.
"""
import time
import threading
import signal
from typing import Callable, Dict, Any, List, Tuple, Optional, Union
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor, as_completed
from contextlib import contextmanager
import statistics


class TimeoutError(Exception):
    """Raised when a benchmark exceeds its timeout limit."""
    pass


class BenchmarkRunner:
    """Core benchmark execution engine."""
    
    def __init__(self, config: Dict[str, Any]):
        """
        Initialize BenchmarkRunner with configuration.
        
        Args:
            config: Configuration dictionary with keys:
                - name: Benchmark name
                - iterations: Number of iterations to run
                - warmup: Number of warmup iterations (default: 0)
                - timeout: Timeout in seconds (default: None)
                - parallel: Enable parallel execution (default: False)
                - workers: Number of parallel workers (default: 4)
                - timer: Timer function to use (default: time.perf_counter)
                - retry_on_error: Retry failed benchmarks (default: False)
                - max_retries: Maximum retry attempts (default: 3)
        """
        self.name = config.get("name", "benchmark")
        self.iterations = config.get("iterations", 10)
        self.warmup = config.get("warmup", 0)
        self.timeout = config.get("timeout", None)
        self.parallel = config.get("parallel", False)
        self.workers = config.get("workers", 4)
        self.timer = config.get("timer", time.perf_counter)
        self.retry_on_error = config.get("retry_on_error", False)
        self.max_retries = config.get("max_retries", 3)
        self.results = []
        self._cleaned_up = False
        self.config = config
    
    def run(self, benchmark_func: Callable, *args, 
            metric_collector: Optional[Any] = None, **kwargs) -> Dict[str, Any]:
        """
        Run a single benchmark function.
        
        Args:
            benchmark_func: Function to benchmark
            *args: Positional arguments for benchmark function
            metric_collector: Optional MetricCollector instance
            **kwargs: Keyword arguments for benchmark function
            
        Returns:
            Dictionary containing benchmark results
        """
        # Warmup phase
        for _ in range(self.warmup):
            benchmark_func(*args, **kwargs)
        
        execution_times = []
        return_value = None
        errors = 0
        completed_iterations = 0
        
        # Run benchmark iterations
        for i in range(self.iterations):
            try:
                if self.timeout:
                    result = self._run_with_timeout(
                        benchmark_func, args, kwargs, self.timeout
                    )
                    exec_time, return_value = result
                else:
                    start_time = self.timer()
                    return_value = benchmark_func(*args, **kwargs)
                    exec_time = self.timer() - start_time
                
                execution_times.append(exec_time)
                completed_iterations += 1
                
            except Exception as e:
                errors += 1
                if self.retry_on_error and errors <= self.max_retries:
                    # Retry the iteration
                    i -= 1
                    continue
                else:
                    # Re-raise the exception
                    raise
        
        # Collect metrics if collector provided
        metrics = {}
        if metric_collector:
            if hasattr(benchmark_func, "__call__") and hasattr(return_value, "__iter__") and isinstance(return_value, dict):
                # If benchmark returns metrics, merge them
                metrics.update(return_value)
            
            # Collect system metrics
            all_metrics = metric_collector.collect_all()
            metrics.update(all_metrics)
        
        result = {
            "name": self.name,
            "iterations": self.iterations,
            "execution_times": execution_times,
            "return_value": return_value,
            "args": args,
            "kwargs": kwargs,
            "completed_iterations": completed_iterations,
            "errors": errors
        }
        
        if metrics:
            result["metrics"] = metrics
        
        self.results.append(result)
        return result
    
    def _run_with_timeout(self, func: Callable, args: tuple, 
                         kwargs: dict, timeout: float) -> Tuple[float, Any]:
        """Run function with timeout."""
        result = [None, None]  # [execution_time, return_value]
        exception = [None]
        
        def target():
            try:
                start_time = self.timer()
                result[1] = func(*args, **kwargs)
                result[0] = self.timer() - start_time
            except Exception as e:
                exception[0] = e
        
        thread = threading.Thread(target=target)
        thread.daemon = True
        thread.start()
        thread.join(timeout)
        
        if thread.is_alive():
            # Timeout occurred
            raise TimeoutError(f"Benchmark exceeded timeout of {timeout}s")
        
        if exception[0]:
            raise exception[0]
        
        return result[0], result[1]
    
    def run_suite(self, benchmarks: List[Tuple[str, Callable]], 
                  metric_collector: Optional[Any] = None) -> List[Dict[str, Any]]:
        """
        Run a suite of benchmarks.
        
        Args:
            benchmarks: List of (name, function) tuples
            metric_collector: Optional MetricCollector instance
            
        Returns:
            List of benchmark results
        """
        if self.parallel:
            return self._run_parallel(benchmarks, metric_collector)
        else:
            return self._run_sequential(benchmarks, metric_collector)
    
    def _run_sequential(self, benchmarks: List[Tuple[str, Callable]], 
                       metric_collector: Optional[Any] = None) -> List[Dict[str, Any]]:
        """Run benchmarks sequentially."""
        results = []
        
        for name, func in benchmarks:
            # Update name for this benchmark
            original_name = self.name
            self.name = name
            
            result = self.run(func, metric_collector=metric_collector)
            results.append(result)
            
            # Restore original name
            self.name = original_name
        
        return results
    
    def _run_parallel(self, benchmarks: List[Tuple[str, Callable]], 
                     metric_collector: Optional[Any] = None) -> List[Dict[str, Any]]:
        """Run benchmarks in parallel."""
        results = []
        
        with ThreadPoolExecutor(max_workers=self.workers) as executor:
            # Create a new runner for each parallel task to avoid conflicts
            futures = {}
            
            for name, func in benchmarks:
                runner = BenchmarkRunner(self.config.copy())
                runner.name = name
                future = executor.submit(runner.run, func, metric_collector=metric_collector)
                futures[future] = name
            
            # Collect results in order
            for future in as_completed(futures):
                result = future.result()
                results.append(result)
        
        # Sort results to maintain order
        results.sort(key=lambda r: benchmarks.index(
            next((b for b in benchmarks if b[0] == r["name"]), None)
        ))
        
        return results
    
    def compare(self, results: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Compare multiple benchmark results.
        
        Args:
            results: List of benchmark results to compare
            
        Returns:
            Comparison statistics
        """
        if len(results) < 2:
            raise ValueError("Need at least 2 results to compare")
        
        # Calculate mean execution times
        mean_times = []
        for result in results:
            times = result["execution_times"]
            mean_time = statistics.mean(times)
            mean_times.append((result["name"], mean_time))
        
        # Sort by execution time
        mean_times.sort(key=lambda x: x[1])
        
        fastest_name, fastest_time = mean_times[0]
        slowest_name, slowest_time = mean_times[-1]
        
        return {
            "fastest": fastest_name,
            "slowest": slowest_name,
            "speedup": slowest_time / fastest_time,
            "rankings": mean_times
        }
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit with cleanup."""
        self._cleaned_up = True
        return False
    
    def calculate_statistics(self, times: List[float]) -> Dict[str, float]:
        """Calculate statistical summary of execution times."""
        if not times:
            return {}
        
        sorted_times = sorted(times)
        n = len(sorted_times)
        
        return {
            "mean": statistics.mean(sorted_times),
            "min": min(sorted_times),
            "max": max(sorted_times),
            "median": statistics.median(sorted_times),
            "std_dev": statistics.stdev(sorted_times) if n > 1 else 0,
            "percentile_95": sorted_times[int(n * 0.95)] if n > 0 else sorted_times[-1]
        }