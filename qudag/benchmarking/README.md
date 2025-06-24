# QuDAG Benchmarking Tool

A comprehensive benchmarking framework for testing QuDAG performance, implemented following Test-Driven Development (TDD) principles.

## Features

- **Flexible Benchmark Execution**: Run single or multiple benchmarks with configurable iterations and warmup
- **Parallel Execution**: Execute benchmarks in parallel for faster results
- **Metric Collection**: Collect CPU, memory, and latency metrics during benchmark execution
- **Multiple Output Formats**: Console, JSON, HTML, and CSV reporting
- **Timeout Support**: Prevent benchmarks from running indefinitely
- **Statistical Analysis**: Automatic calculation of mean, min, max, median, and percentiles
- **Comparison Reports**: Compare performance between different implementations

## Installation

```bash
cd /workspaces/QuDAG/benchmarking
pip install -r requirements.txt  # Install dependencies if needed
```

## Quick Start

### Using the Python API

```python
from benchmarking import BenchmarkRunner, MetricCollector, ConsoleReporter

# Create a benchmark runner
runner = BenchmarkRunner({
    "name": "my_benchmark",
    "iterations": 100,
    "warmup": 10
})

# Define your benchmark function
def my_function():
    # Your code to benchmark
    return sum(range(1000))

# Run the benchmark
result = runner.run(my_function)

# Report results
reporter = ConsoleReporter()
reporter.add_result(result)
reporter.report()
```

### Using the CLI

```bash
# Run a single benchmark
python -m benchmarking.cli run mymodule.benchmark_func --iterations 100

# Run with metrics
python -m benchmarking.cli run mymodule.benchmark_func --metrics memory,cpu

# Export to different formats
python -m benchmarking.cli run mymodule.benchmark_func --format json --output results.json

# List available metrics
python -m benchmarking.cli list-metrics
```

## Architecture

The benchmarking tool is organized into modular components:

```
benchmarking/
├── benchmarks/
│   ├── core/           # Core benchmark execution logic
│   │   └── runner.py   # BenchmarkRunner class
│   ├── metrics/        # Performance metric collectors
│   │   ├── collector.py
│   │   ├── memory.py
│   │   ├── cpu.py
│   │   └── latency.py
│   └── reporters/      # Result formatting and output
│       ├── reporter.py # Base reporter class
│       ├── console.py
│       ├── json_reporter.py
│       ├── html.py
│       └── csv_reporter.py
├── cli.py             # Command-line interface
└── tests/             # Comprehensive test suite
```

## Core Components

### BenchmarkRunner
- Executes benchmark functions with timing and iteration control
- Supports warmup iterations, timeouts, and parallel execution
- Integrates with MetricCollector for performance monitoring

### MetricCollector
- Collects system metrics during benchmark execution
- Built-in collectors for CPU, memory, and latency
- Extensible architecture for custom metrics

### ResultReporter
- Base class for all report formats
- Built-in reporters: Console, JSON, HTML, CSV
- Statistical analysis and comparison features

## Running Tests

The tool was developed using TDD. Run the test suite:

```bash
# Run all tests
pytest

# Run specific test file
pytest tests/unit/test_benchmark_runner.py

# Run with coverage
pytest --cov=benchmarking
```

## Example Demo

Run the included demo to see the tool in action:

```bash
python demo.py
```

This demonstrates:
- Basic benchmarking with performance comparison
- Metric collection during execution
- Parallel benchmark execution
- Multiple output formats

## Configuration

Create a JSON configuration file for complex benchmark suites:

```json
{
  "benchmarks": [
    {
      "name": "test1",
      "module": "mymodule",
      "function": "benchmark_func1",
      "iterations": 100,
      "warmup": 10
    },
    {
      "name": "test2",
      "module": "mymodule",
      "function": "benchmark_func2",
      "iterations": 50
    }
  ],
  "output": {
    "format": "html",
    "file": "results.html"
  }
}
```

Run with:
```bash
python -m benchmarking.cli run-config config.json
```

## Development

This tool was implemented following TDD principles:
1. Tests were written first (see `tests/` directory)
2. Implementation was created to make tests pass
3. Code was refactored while maintaining test coverage

Current test results: **All unit tests passing (12/12 for BenchmarkRunner)**