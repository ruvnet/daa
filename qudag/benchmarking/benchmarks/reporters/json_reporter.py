"""
JSON reporter for benchmark results.
Outputs results in JSON format for further processing.
"""
import json
import time
from typing import Dict, Any, Optional, Union

from .reporter import ResultReporter


class JSONReporter(ResultReporter):
    """Reporter that outputs results in JSON format."""
    
    def __init__(self, pretty: bool = True):
        """
        Initialize JSON reporter.
        
        Args:
            pretty: Whether to use pretty printing (indentation)
        """
        super().__init__()
        self.pretty = pretty
    
    def report(self, output_file: Optional[str] = None) -> Union[str, None]:
        """
        Generate JSON report.
        
        Args:
            output_file: Optional file path to write report to
            
        Returns:
            JSON string if no output file, None if written to file
        """
        # Build report structure
        report_data = {
            "metadata": {
                "timestamp": time.time(),
                "iso_timestamp": time.strftime("%Y-%m-%d %H:%M:%S", time.localtime()),
                **self.metadata
            },
            "results": []
        }
        
        # Process each result
        for result in self.results:
            processed_result = self._process_result(result)
            report_data["results"].append(processed_result)
        
        # Add summary if multiple results
        if len(self.results) > 1:
            report_data["summary"] = self._generate_summary()
        
        # Convert to JSON
        if self.pretty:
            json_content = json.dumps(report_data, indent=2, sort_keys=True)
        else:
            json_content = json.dumps(report_data)
        
        # Output
        if output_file:
            with open(output_file, 'w') as f:
                f.write(json_content)
            return None
        else:
            return json_content
    
    def _process_result(self, result: Dict[str, Any]) -> Dict[str, Any]:
        """Process a single result for JSON output."""
        processed = {
            "name": result.get("name", "unknown"),
            "iterations": result.get("iterations", 0)
        }
        
        # Add execution statistics
        if "execution_times" in result:
            times = result["execution_times"]
            stats = self.calculate_statistics(times)
            
            processed["execution_times"] = times
            processed["statistics"] = {
                "mean": stats["mean"],
                "min": stats["min"],
                "max": stats["max"],
                "median": stats["median"],
                "std_dev": stats["std_dev"],
                "percentile_95": stats["percentile_95"]
            }
        
        # Add metrics if present
        if "metrics" in result:
            processed["metrics"] = result["metrics"]
        
        # Add other fields
        for key in ["args", "kwargs", "return_value", "errors", 
                    "completed_iterations", "tag"]:
            if key in result:
                processed[key] = result[key]
        
        return processed
    
    def _generate_summary(self) -> Dict[str, Any]:
        """Generate summary statistics for all results."""
        summary = {
            "total_benchmarks": len(self.results),
            "total_time": 0,
            "total_iterations": 0,
            "benchmarks": []
        }
        
        # Collect benchmark summaries
        for result in self.results:
            bench_summary = {
                "name": result.get("name", "unknown")
            }
            
            if "execution_times" in result:
                times = result["execution_times"]
                bench_summary["mean_time"] = sum(times) / len(times)
                bench_summary["total_time"] = sum(times)
                bench_summary["iterations"] = len(times)
                
                summary["total_time"] += bench_summary["total_time"]
                summary["total_iterations"] += bench_summary["iterations"]
            
            summary["benchmarks"].append(bench_summary)
        
        # Sort benchmarks by mean time
        summary["benchmarks"].sort(key=lambda x: x.get("mean_time", float('inf')))
        
        # Add rankings
        for i, bench in enumerate(summary["benchmarks"]):
            bench["rank"] = i + 1
        
        return summary