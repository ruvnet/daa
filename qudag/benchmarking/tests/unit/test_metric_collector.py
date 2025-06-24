"""
Unit tests for MetricCollector component.
Tests the metric collection and aggregation logic.
"""
import pytest
from unittest.mock import Mock, patch, MagicMock
import psutil
import time
from typing import Dict, Any, List

from benchmarking.benchmarks.metrics.collector import MetricCollector
from benchmarking.benchmarks.metrics.memory import MemoryMetric
from benchmarking.benchmarks.metrics.cpu import CPUMetric
from benchmarking.benchmarks.metrics.latency import LatencyMetric


class TestMetricCollector:
    """Test cases for MetricCollector class."""
    
    def test_metric_collector_initialization(self):
        """Test MetricCollector initialization with default metrics."""
        collector = MetricCollector()
        
        assert collector.metrics == {}
        assert collector.is_collecting == False
        assert len(collector.available_metrics) > 0
        assert "memory" in collector.available_metrics
        assert "cpu" in collector.available_metrics
        assert "latency" in collector.available_metrics
    
    def test_add_metric(self):
        """Test adding custom metrics to collector."""
        collector = MetricCollector()
        
        class CustomMetric:
            def collect(self):
                return {"custom_value": 42}
        
        collector.add_metric("custom", CustomMetric())
        
        assert "custom" in collector.available_metrics
        metrics = collector.collect_all()
        assert metrics["custom"]["custom_value"] == 42
    
    def test_collect_memory_metrics(self):
        """Test memory metric collection."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        
        # Allocate some memory
        data = [i for i in range(1000000)]
        
        metrics = collector.collect("memory")
        
        assert "rss" in metrics
        assert "vms" in metrics
        assert "percent" in metrics
        assert metrics["rss"] > 0
        assert 0 <= metrics["percent"] <= 100
        
        del data  # Cleanup
    
    def test_collect_cpu_metrics(self):
        """Test CPU metric collection."""
        collector = MetricCollector()
        collector.enable_metric("cpu")
        
        # Do some CPU work
        for _ in range(1000000):
            _ = 2 ** 10
        
        metrics = collector.collect("cpu")
        
        assert "percent" in metrics
        assert "user_time" in metrics
        assert "system_time" in metrics
        assert 0 <= metrics["percent"] <= 100
    
    def test_collect_latency_metrics(self):
        """Test latency metric collection."""
        collector = MetricCollector()
        collector.enable_metric("latency")
        
        # Simulate operations with different latencies
        collector.start_operation("fast_op")
        time.sleep(0.01)
        collector.end_operation("fast_op")
        
        collector.start_operation("slow_op")
        time.sleep(0.1)
        collector.end_operation("slow_op")
        
        metrics = collector.collect("latency")
        
        assert "fast_op" in metrics
        assert "slow_op" in metrics
        assert metrics["fast_op"]["avg"] < metrics["slow_op"]["avg"]
        assert metrics["fast_op"]["count"] == 1
    
    def test_continuous_collection(self):
        """Test continuous metric collection during benchmark."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        
        # Start continuous collection
        collector.start_continuous_collection(interval=0.01)
        
        # Simulate workload
        time.sleep(0.05)
        data = [i ** 2 for i in range(10000)]
        
        # Stop collection
        timeline = collector.stop_continuous_collection()
        
        assert len(timeline) >= 3  # Should have multiple samples
        assert all("timestamp" in sample for sample in timeline)
        assert all("memory" in sample for sample in timeline)
        assert all("cpu" in sample for sample in timeline)
    
    def test_metric_aggregation(self):
        """Test aggregating metrics over multiple runs."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        
        # Collect metrics multiple times
        samples = []
        for i in range(5):
            data = [j for j in range(i * 1000)]
            samples.append(collector.collect("memory"))
            del data
        
        aggregated = collector.aggregate_metrics(samples)
        
        assert "memory" in aggregated
        assert "min" in aggregated["memory"]["rss"]
        assert "max" in aggregated["memory"]["rss"]
        assert "avg" in aggregated["memory"]["rss"]
        assert "std" in aggregated["memory"]["rss"]
    
    def test_metric_filtering(self):
        """Test filtering specific metrics."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        collector.enable_metric("latency")
        
        # Collect only memory and cpu
        metrics = collector.collect_all(filter=["memory", "cpu"])
        
        assert "memory" in metrics
        assert "cpu" in metrics
        assert "latency" not in metrics
    
    def test_metric_reset(self):
        """Test resetting metric collectors."""
        collector = MetricCollector()
        collector.enable_metric("latency")
        
        # Add some latency data
        collector.start_operation("test_op")
        time.sleep(0.01)
        collector.end_operation("test_op")
        
        metrics_before = collector.collect("latency")
        assert "test_op" in metrics_before
        
        # Reset
        collector.reset()
        
        metrics_after = collector.collect("latency")
        assert "test_op" not in metrics_after
    
    def test_custom_metric_integration(self):
        """Test integrating custom metrics with standard ones."""
        collector = MetricCollector()
        
        class NetworkMetric:
            def __init__(self):
                self.bytes_sent = 0
                self.bytes_received = 0
            
            def collect(self):
                # Simulate network activity
                self.bytes_sent += 1024
                self.bytes_received += 2048
                return {
                    "bytes_sent": self.bytes_sent,
                    "bytes_received": self.bytes_received,
                    "total": self.bytes_sent + self.bytes_received
                }
        
        collector.add_metric("network", NetworkMetric())
        collector.enable_metric("network")
        collector.enable_metric("memory")
        
        metrics = collector.collect_all()
        
        assert "network" in metrics
        assert "memory" in metrics
        assert metrics["network"]["total"] == 3072
    
    def test_metric_export(self):
        """Test exporting metrics in different formats."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        collector.enable_metric("cpu")
        
        metrics = collector.collect_all()
        
        # Test JSON export
        json_export = collector.export_json(metrics)
        assert isinstance(json_export, str)
        
        # Test CSV export
        csv_export = collector.export_csv(metrics)
        assert isinstance(csv_export, str)
        assert "metric,value" in csv_export
    
    def test_metric_thresholds(self):
        """Test setting and checking metric thresholds."""
        collector = MetricCollector()
        collector.enable_metric("memory")
        
        # Set threshold
        collector.set_threshold("memory", "percent", max_value=80)
        
        metrics = collector.collect("memory")
        violations = collector.check_thresholds(metrics)
        
        # Depending on system state, may or may not have violations
        assert isinstance(violations, list)
        if violations:
            assert all("metric" in v and "threshold" in v for v in violations)
    
    @patch('psutil.Process')
    def test_metric_collection_error_handling(self, mock_process):
        """Test error handling in metric collection."""
        collector = MetricCollector()
        collector.enable_metric("cpu")
        
        # Simulate psutil error
        mock_process.side_effect = psutil.NoSuchProcess(999)
        
        metrics = collector.collect("cpu", ignore_errors=True)
        
        assert metrics == {}  # Should return empty dict on error
        
        # Test without ignoring errors
        with pytest.raises(psutil.NoSuchProcess):
            collector.collect("cpu", ignore_errors=False)