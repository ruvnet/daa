"""
Unit tests for ResultReporter component.
Tests the result formatting and reporting logic.
"""
import pytest
from unittest.mock import Mock, patch, mock_open, MagicMock
import json
import csv
from pathlib import Path
from typing import Dict, Any, List
from io import StringIO

from benchmarking.benchmarks.reporters.reporter import ResultReporter
from benchmarking.benchmarks.reporters.console import ConsoleReporter
from benchmarking.benchmarks.reporters.json_reporter import JSONReporter
from benchmarking.benchmarks.reporters.html import HTMLReporter
from benchmarking.benchmarks.reporters.csv_reporter import CSVReporter


class TestResultReporter:
    """Test cases for ResultReporter base class and implementations."""
    
    def test_result_reporter_initialization(self):
        """Test ResultReporter base class initialization."""
        reporter = ResultReporter()
        
        assert reporter.results == []
        assert reporter.metadata == {}
        assert hasattr(reporter, 'report')
        assert hasattr(reporter, 'add_result')
    
    def test_add_single_result(self):
        """Test adding a single benchmark result."""
        reporter = ResultReporter()
        
        result = {
            "name": "test_benchmark",
            "iterations": 10,
            "execution_times": [0.1, 0.11, 0.09, 0.1, 0.12],
            "metrics": {"memory": 100, "cpu": 50}
        }
        
        reporter.add_result(result)
        
        assert len(reporter.results) == 1
        assert reporter.results[0] == result
    
    def test_add_multiple_results(self):
        """Test adding multiple benchmark results."""
        reporter = ResultReporter()
        
        results = [
            {"name": f"bench_{i}", "time": i * 0.1} 
            for i in range(5)
        ]
        
        for result in results:
            reporter.add_result(result)
        
        assert len(reporter.results) == 5
        assert reporter.results[2]["name"] == "bench_2"
    
    def test_console_reporter_basic(self):
        """Test basic console reporter output."""
        reporter = ConsoleReporter()
        
        result = {
            "name": "fibonacci",
            "iterations": 100,
            "execution_times": [0.001, 0.002, 0.001, 0.0015, 0.001],
            "return_value": 55
        }
        
        reporter.add_result(result)
        
        with patch('sys.stdout', new=StringIO()) as fake_stdout:
            reporter.report()
            output = fake_stdout.getvalue()
        
        assert "fibonacci" in output
        assert "100 iterations" in output
        assert "Mean:" in output
        assert "Min:" in output
        assert "Max:" in output
    
    def test_console_reporter_with_metrics(self):
        """Test console reporter with metrics display."""
        reporter = ConsoleReporter(show_metrics=True)
        
        result = {
            "name": "matrix_multiply",
            "execution_times": [0.5, 0.6, 0.55],
            "metrics": {
                "memory": {"rss": 1024, "percent": 10.5},
                "cpu": {"percent": 85.2}
            }
        }
        
        reporter.add_result(result)
        
        with patch('sys.stdout', new=StringIO()) as fake_stdout:
            reporter.report()
            output = fake_stdout.getvalue()
        
        assert "Metrics:" in output
        assert "memory" in output
        assert "cpu" in output
        assert "85.2" in output
    
    def test_json_reporter(self):
        """Test JSON reporter output."""
        reporter = JSONReporter()
        
        results = [
            {
                "name": "test1",
                "execution_times": [0.1, 0.2],
                "metrics": {"memory": 100}
            },
            {
                "name": "test2",
                "execution_times": [0.3, 0.4],
                "metrics": {"memory": 200}
            }
        ]
        
        for result in results:
            reporter.add_result(result)
        
        reporter.set_metadata({
            "platform": "Linux",
            "python_version": "3.9.0",
            "timestamp": "2024-01-01T00:00:00"
        })
        
        json_output = reporter.report()
        data = json.loads(json_output)
        
        assert "metadata" in data
        assert "results" in data
        assert len(data["results"]) == 2
        assert data["results"][0]["name"] == "test1"
        assert data["metadata"]["platform"] == "Linux"
    
    def test_json_reporter_file_output(self):
        """Test JSON reporter writing to file."""
        reporter = JSONReporter()
        
        result = {"name": "test", "time": 0.1}
        reporter.add_result(result)
        
        mock_file = mock_open()
        with patch('builtins.open', mock_file):
            reporter.report(output_file="results.json")
        
        mock_file.assert_called_once_with("results.json", 'w')
        handle = mock_file()
        written_data = ''.join(call[0][0] for call in handle.write.call_args_list)
        assert "test" in written_data
    
    def test_html_reporter(self):
        """Test HTML reporter output."""
        reporter = HTMLReporter()
        
        results = [
            {
                "name": "benchmark_1",
                "execution_times": [0.1, 0.15, 0.12],
                "metrics": {"memory": 512}
            }
        ]
        
        reporter.add_result(results[0])
        reporter.set_metadata({"title": "QuDAG Benchmarks"})
        
        html_output = reporter.report()
        
        assert "<html>" in html_output
        assert "QuDAG Benchmarks" in html_output
        assert "benchmark_1" in html_output
        assert "canvas" in html_output  # For charts
        assert "table" in html_output  # For results table
    
    def test_csv_reporter(self):
        """Test CSV reporter output."""
        reporter = CSVReporter()
        
        results = [
            {
                "name": "test1",
                "mean_time": 0.1,
                "min_time": 0.09,
                "max_time": 0.11,
                "memory": 100
            },
            {
                "name": "test2",
                "mean_time": 0.2,
                "min_time": 0.18,
                "max_time": 0.22,
                "memory": 200
            }
        ]
        
        for result in results:
            reporter.add_result(result)
        
        csv_output = reporter.report()
        
        lines = csv_output.strip().split('\n')
        assert len(lines) == 3  # Header + 2 data rows
        assert "name,mean_time,min_time,max_time,memory" == lines[0]
        assert "test1" in lines[1]
        assert "0.2" in lines[2]
    
    def test_comparison_report(self):
        """Test generating comparison reports."""
        reporter = ConsoleReporter()
        
        baseline = {
            "name": "baseline",
            "execution_times": [1.0, 1.1, 1.0]
        }
        
        optimized = {
            "name": "optimized",
            "execution_times": [0.5, 0.6, 0.5]
        }
        
        reporter.add_result(baseline)
        reporter.add_result(optimized)
        
        with patch('sys.stdout', new=StringIO()) as fake_stdout:
            reporter.report_comparison("baseline", "optimized")
            output = fake_stdout.getvalue()
        
        assert "Comparison" in output
        assert "baseline" in output
        assert "optimized" in output
        assert "speedup" in output.lower()
        assert "2" in output  # ~2x speedup
    
    def test_statistical_summary(self):
        """Test statistical summary in reports."""
        reporter = ResultReporter()
        
        result = {
            "name": "stats_test",
            "execution_times": [1.0, 2.0, 3.0, 4.0, 5.0]
        }
        
        stats = reporter.calculate_statistics(result["execution_times"])
        
        assert stats["mean"] == 3.0
        assert stats["min"] == 1.0
        assert stats["max"] == 5.0
        assert stats["median"] == 3.0
        assert "std_dev" in stats
        assert stats["percentile_95"] > 4.0
    
    def test_format_helpers(self):
        """Test formatting helper methods."""
        reporter = ResultReporter()
        
        # Test time formatting
        assert reporter.format_time(0.000001) == "1.00 μs"
        assert reporter.format_time(0.001) == "1.00 ms"
        assert reporter.format_time(1.5) == "1.50 s"
        assert reporter.format_time(90) == "1.50 min"
        
        # Test memory formatting
        assert reporter.format_memory(1024) == "1.00 KB"
        assert reporter.format_memory(1048576) == "1.00 MB"
        assert reporter.format_memory(1073741824) == "1.00 GB"
    
    def test_report_filtering(self):
        """Test filtering results before reporting."""
        reporter = ConsoleReporter()
        
        results = [
            {"name": "fast", "mean_time": 0.1, "tag": "unit"},
            {"name": "slow", "mean_time": 10.0, "tag": "integration"},
            {"name": "medium", "mean_time": 1.0, "tag": "unit"}
        ]
        
        for result in results:
            reporter.add_result(result)
        
        # Filter by tag
        filtered = reporter.filter_results(tag="unit")
        assert len(filtered) == 2
        assert all(r["tag"] == "unit" for r in filtered)
        
        # Filter by performance
        fast_only = reporter.filter_results(max_time=0.5)
        assert len(fast_only) == 1
        assert fast_only[0]["name"] == "fast"
    
    def test_report_aggregation(self):
        """Test aggregating multiple runs of same benchmark."""
        reporter = ResultReporter()
        
        # Multiple runs of same benchmark
        for i in range(3):
            reporter.add_result({
                "name": "repeated_test",
                "run": i + 1,
                "execution_times": [0.1 + i * 0.01] * 5
            })
        
        aggregated = reporter.aggregate_by_name()
        
        assert "repeated_test" in aggregated
        assert aggregated["repeated_test"]["runs"] == 3
        assert "mean_of_means" in aggregated["repeated_test"]