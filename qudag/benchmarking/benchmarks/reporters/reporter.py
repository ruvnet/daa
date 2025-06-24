"""
Base reporter class for benchmark results.
Provides common functionality for all report formats.
"""
import statistics
from typing import Dict, Any, List, Optional, Union
from abc import ABC, abstractmethod


class ResultReporter(ABC):
    """Base class for benchmark result reporters."""
    
    def __init__(self):
        """Initialize reporter with empty results."""
        self.results = []
        self.metadata = {}
    
    def add_result(self, result: Dict[str, Any]):
        """
        Add a benchmark result to the reporter.
        
        Args:
            result: Dictionary containing benchmark results
        """
        self.results.append(result)
    
    def set_metadata(self, metadata: Dict[str, Any]):
        """
        Set metadata for the report.
        
        Args:
            metadata: Dictionary containing report metadata
        """
        self.metadata = metadata
    
    @abstractmethod
    def report(self, output_file: Optional[str] = None) -> Union[str, None]:
        """
        Generate the report.
        
        Args:
            output_file: Optional file path to write report to
            
        Returns:
            Report content as string, or None if written to file
        """
        pass
    
    def calculate_statistics(self, times: List[float]) -> Dict[str, float]:
        """
        Calculate statistical summary of execution times.
        
        Args:
            times: List of execution times
            
        Returns:
            Dictionary with statistical measures
        """
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
    
    def format_time(self, seconds: float) -> str:
        """
        Format time duration in human-readable format.
        
        Args:
            seconds: Time in seconds
            
        Returns:
            Formatted time string
        """
        if seconds < 1e-6:
            return f"{seconds * 1e9:.2f} ns"
        elif seconds < 1e-3:
            return f"{seconds * 1e6:.2f} μs"
        elif seconds < 1:
            return f"{seconds * 1e3:.2f} ms"
        elif seconds < 60:
            return f"{seconds:.2f} s"
        else:
            return f"{seconds / 60:.2f} min"
    
    def format_memory(self, bytes_value: int) -> str:
        """
        Format memory size in human-readable format.
        
        Args:
            bytes_value: Memory size in bytes
            
        Returns:
            Formatted memory string
        """
        if bytes_value < 1024:
            return f"{bytes_value} B"
        elif bytes_value < 1024 ** 2:
            return f"{bytes_value / 1024:.2f} KB"
        elif bytes_value < 1024 ** 3:
            return f"{bytes_value / 1024 ** 2:.2f} MB"
        else:
            return f"{bytes_value / 1024 ** 3:.2f} GB"
    
    def filter_results(self, tag: Optional[str] = None, 
                      max_time: Optional[float] = None) -> List[Dict[str, Any]]:
        """
        Filter results based on criteria.
        
        Args:
            tag: Filter by tag
            max_time: Filter by maximum execution time
            
        Returns:
            Filtered list of results
        """
        filtered = self.results
        
        if tag:
            filtered = [r for r in filtered if r.get("tag") == tag]
        
        if max_time:
            filtered = [r for r in filtered 
                       if r.get("mean_time", float('inf')) <= max_time]
        
        return filtered
    
    def aggregate_by_name(self) -> Dict[str, Dict[str, Any]]:
        """
        Aggregate multiple runs of the same benchmark.
        
        Returns:
            Dictionary with aggregated results by benchmark name
        """
        aggregated = {}
        
        # Group results by name
        for result in self.results:
            name = result.get("name", "unknown")
            if name not in aggregated:
                aggregated[name] = {
                    "runs": 0,
                    "all_times": [],
                    "all_metrics": []
                }
            
            aggregated[name]["runs"] += 1
            
            if "execution_times" in result:
                aggregated[name]["all_times"].extend(result["execution_times"])
            
            if "metrics" in result:
                aggregated[name]["all_metrics"].append(result["metrics"])
        
        # Calculate aggregated statistics
        for name, data in aggregated.items():
            if data["all_times"]:
                # Calculate mean of means for multiple runs
                run_means = []
                times_per_run = len(data["all_times"]) // data["runs"]
                
                for i in range(data["runs"]):
                    start_idx = i * times_per_run
                    end_idx = (i + 1) * times_per_run
                    run_times = data["all_times"][start_idx:end_idx]
                    if run_times:
                        run_means.append(statistics.mean(run_times))
                
                if run_means:
                    data["mean_of_means"] = statistics.mean(run_means)
                    data["std_of_means"] = statistics.stdev(run_means) if len(run_means) > 1 else 0
        
        return aggregated