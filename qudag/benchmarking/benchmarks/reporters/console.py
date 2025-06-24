"""
Console reporter for benchmark results.
Outputs formatted results to stdout.
"""
import sys
import statistics
from typing import Dict, Any, List, Optional, Union

from .reporter import ResultReporter


class ConsoleReporter(ResultReporter):
    """Reporter that outputs results to console."""
    
    def __init__(self, show_metrics: bool = False):
        """
        Initialize console reporter.
        
        Args:
            show_metrics: Whether to display detailed metrics
        """
        super().__init__()
        self.show_metrics = show_metrics
    
    def report(self, output_file: Optional[str] = None) -> Union[str, None]:
        """Generate console report."""
        output_lines = []
        
        # Header
        output_lines.append("=" * 80)
        output_lines.append("BENCHMARK RESULTS")
        output_lines.append("=" * 80)
        
        if self.metadata:
            output_lines.append("\nMetadata:")
            for key, value in self.metadata.items():
                output_lines.append(f"  {key}: {value}")
        
        # Results
        for result in self.results:
            output_lines.extend(self._format_result(result))
        
        # Summary
        if len(self.results) > 1:
            output_lines.append("\n" + "-" * 80)
            output_lines.append("SUMMARY")
            output_lines.append("-" * 80)
            output_lines.extend(self._format_summary())
        
        output_lines.append("=" * 80)
        
        # Output
        output_content = "\n".join(output_lines)
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(output_content)
            return None
        else:
            print(output_content)
            return output_content
    
    def _format_result(self, result: Dict[str, Any]) -> List[str]:
        """Format a single benchmark result."""
        lines = []
        
        lines.append(f"\n{result.get('name', 'Unknown Benchmark')}")
        lines.append("-" * len(result.get('name', 'Unknown Benchmark')))
        
        # Execution statistics
        if "execution_times" in result:
            times = result["execution_times"]
            stats = self.calculate_statistics(times)
            
            lines.append(f"  Iterations: {result.get('iterations', len(times))}")
            lines.append(f"  Mean: {self.format_time(stats['mean'])}")
            lines.append(f"  Min: {self.format_time(stats['min'])}")
            lines.append(f"  Max: {self.format_time(stats['max'])}")
            lines.append(f"  Median: {self.format_time(stats['median'])}")
            lines.append(f"  Std Dev: {self.format_time(stats['std_dev'])}")
            lines.append(f"  95th percentile: {self.format_time(stats['percentile_95'])}")
        
        # Show metrics if enabled
        if self.show_metrics and "metrics" in result:
            lines.append("\n  Metrics:")
            self._format_metrics(result["metrics"], lines, indent=4)
        
        return lines
    
    def _format_metrics(self, metrics: Dict[str, Any], lines: List[str], indent: int = 0):
        """Format metrics recursively."""
        indent_str = " " * indent
        
        for key, value in metrics.items():
            if isinstance(value, dict):
                lines.append(f"{indent_str}{key}:")
                self._format_metrics(value, lines, indent + 2)
            elif isinstance(value, (int, float)):
                if key.endswith("_bytes") or key in ["rss", "vms", "peak", "initial"]:
                    lines.append(f"{indent_str}{key}: {self.format_memory(int(value))}")
                elif key.endswith("_percent") or key == "percent":
                    lines.append(f"{indent_str}{key}: {value:.1f}%")
                else:
                    lines.append(f"{indent_str}{key}: {value:.4f}")
            else:
                lines.append(f"{indent_str}{key}: {value}")
    
    def _format_summary(self) -> List[str]:
        """Format summary of all results."""
        lines = []
        
        # Ranking by mean execution time
        results_with_mean = []
        for result in self.results:
            if "execution_times" in result:
                mean_time = statistics.mean(result["execution_times"])
                results_with_mean.append((result["name"], mean_time))
        
        results_with_mean.sort(key=lambda x: x[1])
        
        lines.append("\nRanking by execution time:")
        for i, (name, mean_time) in enumerate(results_with_mean, 1):
            lines.append(f"  {i}. {name}: {self.format_time(mean_time)}")
        
        # Overall statistics
        if results_with_mean:
            all_times = []
            for result in self.results:
                if "execution_times" in result:
                    all_times.extend(result["execution_times"])
            
            if all_times:
                lines.append("\nOverall statistics:")
                lines.append(f"  Total benchmarks: {len(self.results)}")
                lines.append(f"  Total executions: {len(all_times)}")
                lines.append(f"  Total time: {self.format_time(sum(all_times))}")
        
        return lines
    
    def report_comparison(self, baseline_name: str, comparison_name: str):
        """
        Generate comparison report between two benchmarks.
        
        Args:
            baseline_name: Name of baseline benchmark
            comparison_name: Name of comparison benchmark
        """
        baseline = None
        comparison = None
        
        for result in self.results:
            if result.get("name") == baseline_name:
                baseline = result
            elif result.get("name") == comparison_name:
                comparison = result
        
        if not baseline or not comparison:
            print("Error: Could not find both benchmarks for comparison")
            return
        
        print("\n" + "=" * 80)
        print(f"COMPARISON: {baseline_name} vs {comparison_name}")
        print("=" * 80)
        
        # Calculate statistics
        baseline_mean = statistics.mean(baseline["execution_times"])
        comparison_mean = statistics.mean(comparison["execution_times"])
        
        speedup = baseline_mean / comparison_mean
        improvement = (baseline_mean - comparison_mean) / baseline_mean * 100
        
        print(f"\n{baseline_name} (baseline):")
        print(f"  Mean time: {self.format_time(baseline_mean)}")
        
        print(f"\n{comparison_name}:")
        print(f"  Mean time: {self.format_time(comparison_mean)}")
        
        print(f"\nSpeedup: {speedup:.2f}x")
        
        if improvement > 0:
            print(f"Improvement: {improvement:.1f}% faster")
        else:
            print(f"Regression: {-improvement:.1f}% slower")
        
        print("=" * 80)