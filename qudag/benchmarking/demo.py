#!/usr/bin/env python3
"""
Demo script showing how to use the QuDAG benchmarking tool.
"""
import time
from benchmarks import (
    BenchmarkRunner, 
    MetricCollector,
    ConsoleReporter,
    JSONReporter
)


def fibonacci(n):
    """Calculate fibonacci number (recursive - slow for demo)."""
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)


def fibonacci_iterative(n):
    """Calculate fibonacci number (iterative - fast)."""
    if n <= 1:
        return n
    a, b = 0, 1
    for _ in range(2, n+1):
        a, b = b, a + b
    return b


def matrix_multiply(size=100):
    """Simple matrix multiplication benchmark."""
    import random
    # Create random matrices
    A = [[random.random() for _ in range(size)] for _ in range(size)]
    B = [[random.random() for _ in range(size)] for _ in range(size)]
    
    # Multiply
    C = [[0 for _ in range(size)] for _ in range(size)]
    for i in range(size):
        for j in range(size):
            for k in range(size):
                C[i][j] += A[i][k] * B[k][j]
    
    return C


def demo_basic_benchmark():
    """Demonstrate basic benchmarking."""
    print("=== Basic Benchmark Demo ===\n")
    
    # Create runner
    runner = BenchmarkRunner({
        "name": "fibonacci_comparison",
        "iterations": 5,
        "warmup": 1
    })
    
    # Run benchmarks
    print("Running fibonacci(20) - recursive...")
    recursive_result = runner.run(lambda: fibonacci(20))
    
    print("Running fibonacci(20) - iterative...")
    iterative_result = runner.run(lambda: fibonacci_iterative(20))
    
    # Report results
    reporter = ConsoleReporter()
    reporter.add_result(recursive_result)
    reporter.add_result(iterative_result)
    reporter.report()
    
    # Compare
    comparison = runner.compare([recursive_result, iterative_result])
    print(f"\nSpeedup: {comparison['speedup']:.2f}x")


def demo_with_metrics():
    """Demonstrate benchmarking with metric collection."""
    print("\n\n=== Benchmark with Metrics Demo ===\n")
    
    # Create runner and metric collector
    runner = BenchmarkRunner({
        "name": "matrix_multiply",
        "iterations": 3
    })
    
    collector = MetricCollector()
    collector.enable_metric("memory")
    collector.enable_metric("cpu")
    
    # Run benchmark with metrics
    print("Running matrix multiplication benchmark with metrics...")
    result = runner.run(lambda: matrix_multiply(50), metric_collector=collector)
    
    # Report with metrics
    reporter = ConsoleReporter(show_metrics=True)
    reporter.add_result(result)
    reporter.report()


def demo_parallel_execution():
    """Demonstrate parallel benchmark execution."""
    print("\n\n=== Parallel Execution Demo ===\n")
    
    # Create parallel runner
    runner = BenchmarkRunner({
        "parallel": True,
        "workers": 4,
        "iterations": 3
    })
    
    # Define benchmarks
    benchmarks = [
        ("fib_10", lambda: fibonacci(10)),
        ("fib_15", lambda: fibonacci(15)),
        ("fib_20", lambda: fibonacci(20)),
        ("matrix_20", lambda: matrix_multiply(20)),
        ("matrix_30", lambda: matrix_multiply(30)),
        ("matrix_40", lambda: matrix_multiply(40))
    ]
    
    print("Running 6 benchmarks in parallel...")
    start = time.time()
    results = runner.run_suite(benchmarks)
    elapsed = time.time() - start
    
    print(f"Completed in {elapsed:.2f} seconds")
    
    # Save results to JSON
    json_reporter = JSONReporter()
    for result in results:
        json_reporter.add_result(result)
    
    json_reporter.set_metadata({
        "title": "Parallel Benchmark Results",
        "total_time": elapsed
    })
    
    with open("demo_results.json", "w") as f:
        f.write(json_reporter.report())
    
    print("Results saved to demo_results.json")


if __name__ == "__main__":
    demo_basic_benchmark()
    demo_with_metrics()
    demo_parallel_execution()
    
    print("\n\nDemo complete! You can also use the CLI:")
    print("  python -m benchmarking.cli run demo.fibonacci --iterations 10")
    print("  python -m benchmarking.cli list-metrics")