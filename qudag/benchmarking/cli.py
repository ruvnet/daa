"""
Command-line interface for the QuDAG benchmarking tool.
Provides easy access to benchmarking functionality.
"""
import argparse
import json
import sys
import importlib
import os
from typing import Dict, Any, Callable, Optional
from pathlib import Path

from benchmarking.benchmarks.core import BenchmarkRunner
from benchmarking.benchmarks.metrics import MetricCollector
from benchmarking.benchmarks.reporters import (
    ConsoleReporter, JSONReporter, HTMLReporter, CSVReporter
)


class BenchmarkCLI:
    """CLI for running benchmarks."""
    
    def __init__(self):
        """Initialize CLI."""
        self.parser = self._create_parser()
    
    def _create_parser(self) -> argparse.ArgumentParser:
        """Create argument parser."""
        parser = argparse.ArgumentParser(
            description="QuDAG Benchmarking Tool",
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog="""
Examples:
  # Run a single benchmark
  python -m benchmarking.cli run mymodule.benchmark_func --iterations 100
  
  # Run benchmarks from config file
  python -m benchmarking.cli run-config benchmark_config.json
  
  # Run with metrics collection
  python -m benchmarking.cli run mymodule.benchmark_func --metrics memory,cpu
  
  # Export results to different formats
  python -m benchmarking.cli run mymodule.benchmark_func --format json --output results.json
  python -m benchmarking.cli run mymodule.benchmark_func --format html --output results.html
            """
        )
        
        subparsers = parser.add_subparsers(dest="command", help="Commands")
        
        # Run command
        run_parser = subparsers.add_parser("run", help="Run a single benchmark")
        run_parser.add_argument("benchmark", help="Module path to benchmark function (e.g., mymodule.benchmark_func)")
        run_parser.add_argument("-n", "--iterations", type=int, default=10, help="Number of iterations")
        run_parser.add_argument("-w", "--warmup", type=int, default=0, help="Number of warmup iterations")
        run_parser.add_argument("-t", "--timeout", type=float, help="Timeout in seconds")
        run_parser.add_argument("-m", "--metrics", help="Comma-separated list of metrics to collect")
        run_parser.add_argument("-f", "--format", choices=["console", "json", "html", "csv"], default="console", help="Output format")
        run_parser.add_argument("-o", "--output", help="Output file path")
        run_parser.add_argument("--parallel", action="store_true", help="Enable parallel execution")
        run_parser.add_argument("--workers", type=int, default=4, help="Number of parallel workers")
        
        # Run from config
        config_parser = subparsers.add_parser("run-config", help="Run benchmarks from config file")
        config_parser.add_argument("config", help="Path to configuration JSON file")
        config_parser.add_argument("-f", "--format", choices=["console", "json", "html", "csv"], help="Override output format")
        config_parser.add_argument("-o", "--output", help="Override output file")
        
        # Compare command
        compare_parser = subparsers.add_parser("compare", help="Compare benchmark results")
        compare_parser.add_argument("baseline", help="Path to baseline results JSON")
        compare_parser.add_argument("comparison", help="Path to comparison results JSON")
        compare_parser.add_argument("-f", "--format", choices=["console", "html"], default="console", help="Output format")
        compare_parser.add_argument("-o", "--output", help="Output file path")
        
        # List metrics command
        list_parser = subparsers.add_parser("list-metrics", help="List available metrics")
        
        return parser
    
    def run(self, args: Optional[list] = None):
        """Run the CLI."""
        parsed_args = self.parser.parse_args(args)
        
        if not parsed_args.command:
            self.parser.print_help()
            return
        
        if parsed_args.command == "run":
            self._run_benchmark(parsed_args)
        elif parsed_args.command == "run-config":
            self._run_from_config(parsed_args)
        elif parsed_args.command == "compare":
            self._compare_results(parsed_args)
        elif parsed_args.command == "list-metrics":
            self._list_metrics()
    
    def _run_benchmark(self, args):
        """Run a single benchmark."""
        # Import benchmark function
        try:
            benchmark_func = self._import_benchmark(args.benchmark)
        except Exception as e:
            print(f"Error importing benchmark: {e}", file=sys.stderr)
            sys.exit(1)
        
        # Configure runner
        config = {
            "name": args.benchmark,
            "iterations": args.iterations,
            "warmup": args.warmup,
            "timeout": args.timeout,
            "parallel": args.parallel,
            "workers": args.workers
        }
        
        runner = BenchmarkRunner(config)
        
        # Configure metrics
        collector = None
        if args.metrics:
            collector = MetricCollector()
            for metric in args.metrics.split(","):
                collector.enable_metric(metric.strip())
        
        # Run benchmark
        try:
            result = runner.run(benchmark_func, metric_collector=collector)
        except Exception as e:
            print(f"Error running benchmark: {e}", file=sys.stderr)
            sys.exit(1)
        
        # Report results
        self._report_results([result], args.format, args.output)
    
    def run_from_config(self, config_path: str):
        """Public method to run from config (for testing)."""
        with open(config_path) as f:
            config = json.load(f)
        
        results = []
        
        for bench_config in config.get("benchmarks", []):
            # Import benchmark function
            module_path = bench_config["module"]
            func_name = bench_config["function"]
            benchmark_func = self._import_benchmark(f"{module_path}.{func_name}")
            
            # Configure runner
            runner_config = {
                "name": bench_config.get("name", func_name),
                "iterations": bench_config.get("iterations", 10),
                "warmup": bench_config.get("warmup", 0),
                "timeout": bench_config.get("timeout", None)
            }
            
            runner = BenchmarkRunner(runner_config)
            
            # Run benchmark
            result = runner.run(benchmark_func)
            results.append(result)
        
        # Output results
        output_config = config.get("output", {})
        format_type = output_config.get("format", "console")
        output_file = output_config.get("file", None)
        
        self._report_results(results, format_type, output_file)
    
    def _run_from_config(self, args):
        """Run benchmarks from configuration file."""
        try:
            self.run_from_config(args.config)
        except Exception as e:
            print(f"Error running from config: {e}", file=sys.stderr)
            sys.exit(1)
    
    def _compare_results(self, args):
        """Compare benchmark results."""
        try:
            # Load result files
            with open(args.baseline) as f:
                baseline_data = json.load(f)
            
            with open(args.comparison) as f:
                comparison_data = json.load(f)
            
            # Extract results
            baseline_results = baseline_data.get("results", [])
            comparison_results = comparison_data.get("results", [])
            
            if not baseline_results or not comparison_results:
                print("Error: No results found in files", file=sys.stderr)
                sys.exit(1)
            
            # Create comparison report
            if args.format == "console":
                reporter = ConsoleReporter()
                for result in baseline_results + comparison_results:
                    reporter.add_result(result)
                
                # Find matching benchmarks and compare
                for baseline in baseline_results:
                    for comparison in comparison_results:
                        if baseline.get("name") == comparison.get("name"):
                            reporter.report_comparison(
                                baseline["name"] + "_baseline",
                                comparison["name"] + "_comparison"
                            )
            
        except Exception as e:
            print(f"Error comparing results: {e}", file=sys.stderr)
            sys.exit(1)
    
    def _list_metrics(self):
        """List available metrics."""
        collector = MetricCollector()
        
        print("Available metrics:")
        for metric_name in sorted(collector.available_metrics.keys()):
            print(f"  - {metric_name}")
    
    def _import_benchmark(self, path: str) -> Callable:
        """Import a benchmark function from module path."""
        parts = path.split(".")
        module_path = ".".join(parts[:-1])
        func_name = parts[-1]
        
        module = importlib.import_module(module_path)
        return getattr(module, func_name)
    
    def import_benchmark(self, path: str) -> Callable:
        """Public method for importing benchmarks (for testing)."""
        return self._import_benchmark(path)
    
    def _report_results(self, results: list, format_type: str, output_file: Optional[str]):
        """Generate and output report."""
        # Create reporter
        if format_type == "console":
            reporter = ConsoleReporter(show_metrics=True)
        elif format_type == "json":
            reporter = JSONReporter()
        elif format_type == "html":
            reporter = HTMLReporter()
        elif format_type == "csv":
            reporter = CSVReporter()
        else:
            raise ValueError(f"Unknown format: {format_type}")
        
        # Add results
        for result in results:
            reporter.add_result(result)
        
        # Generate report
        reporter.report(output_file)


def main():
    """Entry point for CLI."""
    cli = BenchmarkCLI()
    cli.run()


if __name__ == "__main__":
    main()