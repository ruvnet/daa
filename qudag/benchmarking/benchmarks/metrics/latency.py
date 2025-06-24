"""
Latency metric collector for benchmarks.
Tracks operation latencies during benchmark execution.
"""
import time
from typing import Dict, Any, List
from collections import defaultdict
import statistics


class LatencyMetric:
    """Collects latency metrics for operations."""
    
    def __init__(self):
        """Initialize latency metric collector."""
        self.operation_timings = defaultdict(list)
        self.active_operations = {}
    
    def start_operation(self, operation_name: str) -> float:
        """
        Start timing an operation.
        
        Args:
            operation_name: Name of the operation
            
        Returns:
            Start timestamp
        """
        start_time = time.perf_counter()
        
        # Store active operation
        if operation_name not in self.active_operations:
            self.active_operations[operation_name] = []
        
        self.active_operations[operation_name].append(start_time)
        return start_time
    
    def end_operation(self, operation_name: str) -> float:
        """
        End timing an operation.
        
        Args:
            operation_name: Name of the operation
            
        Returns:
            Operation duration in seconds
        """
        end_time = time.perf_counter()
        
        if operation_name in self.active_operations and self.active_operations[operation_name]:
            start_time = self.active_operations[operation_name].pop(0)
            duration = end_time - start_time
            self.operation_timings[operation_name].append(duration)
            return duration
        
        return 0.0
    
    def collect(self) -> Dict[str, Any]:
        """
        Collect latency statistics for all operations.
        
        Returns:
            Dictionary containing latency metrics for each operation
        """
        results = {}
        
        for operation_name, timings in self.operation_timings.items():
            if timings:
                results[operation_name] = {
                    "count": len(timings),
                    "avg": statistics.mean(timings),
                    "min": min(timings),
                    "max": max(timings),
                    "median": statistics.median(timings),
                    "p95": self._percentile(timings, 0.95),
                    "p99": self._percentile(timings, 0.99),
                    "std_dev": statistics.stdev(timings) if len(timings) > 1 else 0
                }
        
        return results
    
    def _percentile(self, data: List[float], percentile: float) -> float:
        """Calculate percentile of data."""
        if not data:
            return 0.0
        
        sorted_data = sorted(data)
        index = int(len(sorted_data) * percentile)
        
        if index >= len(sorted_data):
            return sorted_data[-1]
        
        return sorted_data[index]
    
    def reset(self):
        """Reset metric collector state."""
        self.operation_timings.clear()
        self.active_operations.clear()