"""
Metric collection system for benchmarks.
Collects and aggregates performance metrics during benchmark execution.
"""
import time
import threading
import psutil
import json
import csv
import statistics
from typing import Dict, Any, List, Optional, Union
from collections import defaultdict
import io


class MetricCollector:
    """Collects and manages performance metrics during benchmarks."""
    
    def __init__(self):
        """Initialize MetricCollector with default metrics."""
        self.metrics = {}
        self.is_collecting = False
        self.available_metrics = {}
        self._continuous_thread = None
        self._continuous_data = []
        self._continuous_interval = 0.1
        self._continuous_stop = False
        self._thresholds = {}
        self._operation_timings = defaultdict(list)
        
        # Initialize default metrics
        self._initialize_default_metrics()
    
    def _initialize_default_metrics(self):
        """Initialize default metric collectors."""
        from .memory import MemoryMetric
        from .cpu import CPUMetric
        from .latency import LatencyMetric
        
        self.available_metrics["memory"] = MemoryMetric()
        self.available_metrics["cpu"] = CPUMetric()
        self.available_metrics["latency"] = LatencyMetric()
    
    def add_metric(self, name: str, metric_instance: Any):
        """Add a custom metric collector."""
        self.available_metrics[name] = metric_instance
    
    def enable_metric(self, name: str):
        """Enable a specific metric for collection."""
        if name not in self.available_metrics:
            raise ValueError(f"Unknown metric: {name}")
        self.metrics[name] = self.available_metrics[name]
    
    def collect(self, metric_name: str, ignore_errors: bool = True) -> Dict[str, Any]:
        """
        Collect data for a specific metric.
        
        Args:
            metric_name: Name of the metric to collect
            ignore_errors: Whether to ignore collection errors
            
        Returns:
            Collected metric data
        """
        if metric_name not in self.available_metrics:
            return {}
        
        try:
            metric = self.available_metrics[metric_name]
            
            # Handle latency metric specially
            if metric_name == "latency":
                return self._collect_latency_metrics()
            
            return metric.collect()
        except Exception as e:
            if ignore_errors:
                return {}
            raise
    
    def collect_all(self, filter: Optional[List[str]] = None) -> Dict[str, Any]:
        """
        Collect all enabled metrics.
        
        Args:
            filter: Optional list of metric names to collect
            
        Returns:
            Dictionary of all collected metrics
        """
        results = {}
        
        metrics_to_collect = self.metrics.keys()
        if filter:
            metrics_to_collect = [m for m in metrics_to_collect if m in filter]
        
        for metric_name in metrics_to_collect:
            data = self.collect(metric_name)
            if data:
                results[metric_name] = data
        
        return results
    
    def start_continuous_collection(self, interval: float = 0.1):
        """Start continuous metric collection in background."""
        if self.is_collecting:
            return
        
        self.is_collecting = True
        self._continuous_interval = interval
        self._continuous_data = []
        self._continuous_stop = False
        
        def collect_loop():
            while not self._continuous_stop:
                timestamp = time.time()
                metrics = self.collect_all()
                self._continuous_data.append({
                    "timestamp": timestamp,
                    **metrics
                })
                time.sleep(self._continuous_interval)
        
        self._continuous_thread = threading.Thread(target=collect_loop)
        self._continuous_thread.daemon = True
        self._continuous_thread.start()
    
    def stop_continuous_collection(self) -> List[Dict[str, Any]]:
        """Stop continuous collection and return timeline data."""
        if not self.is_collecting:
            return []
        
        self._continuous_stop = True
        if self._continuous_thread:
            self._continuous_thread.join(timeout=1)
        
        self.is_collecting = False
        return self._continuous_data
    
    def start_operation(self, operation_name: str):
        """Start timing an operation for latency metrics."""
        self._operation_timings[operation_name].append({
            "start": time.perf_counter(),
            "end": None
        })
    
    def end_operation(self, operation_name: str):
        """End timing an operation."""
        if operation_name in self._operation_timings:
            # Find the last unfinished operation
            for timing in reversed(self._operation_timings[operation_name]):
                if timing["end"] is None:
                    timing["end"] = time.perf_counter()
                    break
    
    def _collect_latency_metrics(self) -> Dict[str, Any]:
        """Collect latency metrics from operation timings."""
        results = {}
        
        for op_name, timings in self._operation_timings.items():
            # Calculate latencies for completed operations
            latencies = []
            for timing in timings:
                if timing["end"] is not None:
                    latencies.append(timing["end"] - timing["start"])
            
            if latencies:
                results[op_name] = {
                    "avg": statistics.mean(latencies),
                    "min": min(latencies),
                    "max": max(latencies),
                    "count": len(latencies)
                }
        
        return results
    
    def aggregate_metrics(self, samples: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Aggregate multiple metric samples."""
        if not samples:
            return {}
        
        aggregated = {}
        
        # Group values by metric and field
        metric_values = defaultdict(lambda: defaultdict(list))
        
        for sample in samples:
            for metric_name, metric_data in sample.items():
                if isinstance(metric_data, dict):
                    for field, value in metric_data.items():
                        if isinstance(value, (int, float)):
                            metric_values[metric_name][field].append(value)
        
        # Calculate statistics for each metric field
        for metric_name, fields in metric_values.items():
            aggregated[metric_name] = {}
            for field, values in fields.items():
                if values:
                    aggregated[metric_name][field] = {
                        "min": min(values),
                        "max": max(values),
                        "avg": statistics.mean(values),
                        "std": statistics.stdev(values) if len(values) > 1 else 0
                    }
        
        return aggregated
    
    def reset(self):
        """Reset all metric collectors."""
        self._operation_timings.clear()
        for metric in self.available_metrics.values():
            if hasattr(metric, "reset"):
                metric.reset()
    
    def set_threshold(self, metric_name: str, field: str, 
                     max_value: Optional[float] = None,
                     min_value: Optional[float] = None):
        """Set threshold for a metric field."""
        if metric_name not in self._thresholds:
            self._thresholds[metric_name] = {}
        
        self._thresholds[metric_name][field] = {
            "max": max_value,
            "min": min_value
        }
    
    def check_thresholds(self, metrics: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Check if metrics violate any thresholds."""
        violations = []
        
        for metric_name, metric_data in metrics.items():
            if metric_name not in self._thresholds:
                continue
            
            for field, value in metric_data.items():
                if field not in self._thresholds[metric_name]:
                    continue
                
                threshold = self._thresholds[metric_name][field]
                
                if threshold["max"] is not None and value > threshold["max"]:
                    violations.append({
                        "metric": metric_name,
                        "field": field,
                        "value": value,
                        "threshold": threshold["max"],
                        "type": "max"
                    })
                
                if threshold["min"] is not None and value < threshold["min"]:
                    violations.append({
                        "metric": metric_name,
                        "field": field,
                        "value": value,
                        "threshold": threshold["min"],
                        "type": "min"
                    })
        
        return violations
    
    def export_json(self, metrics: Dict[str, Any]) -> str:
        """Export metrics as JSON string."""
        return json.dumps(metrics, indent=2)
    
    def export_csv(self, metrics: Dict[str, Any]) -> str:
        """Export metrics as CSV string."""
        output = io.StringIO()
        writer = csv.writer(output)
        
        # Write header
        writer.writerow(["metric", "value"])
        
        # Flatten metrics and write rows
        def flatten_dict(d, parent_key=""):
            items = []
            for k, v in d.items():
                new_key = f"{parent_key}.{k}" if parent_key else k
                if isinstance(v, dict):
                    items.extend(flatten_dict(v, new_key))
                else:
                    items.append((new_key, v))
            return items
        
        for metric, value in flatten_dict(metrics):
            writer.writerow([metric, value])
        
        return output.getvalue()