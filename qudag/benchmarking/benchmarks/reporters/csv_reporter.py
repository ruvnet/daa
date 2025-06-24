"""
CSV reporter for benchmark results.
Outputs results in CSV format for spreadsheet analysis.
"""
import csv
import io
from typing import Dict, Any, Optional, Union, List

from .reporter import ResultReporter


class CSVReporter(ResultReporter):
    """Reporter that outputs results in CSV format."""
    
    def __init__(self, delimiter: str = ','):
        """
        Initialize CSV reporter.
        
        Args:
            delimiter: CSV delimiter character
        """
        super().__init__()
        self.delimiter = delimiter
    
    def report(self, output_file: Optional[str] = None) -> Union[str, None]:
        """
        Generate CSV report.
        
        Args:
            output_file: Optional file path to write report to
            
        Returns:
            CSV string if no output file, None if written to file
        """
        # Determine all fields across all results
        all_fields = self._get_all_fields()
        
        # Create CSV content
        output = io.StringIO()
        writer = csv.DictWriter(
            output, 
            fieldnames=all_fields, 
            delimiter=self.delimiter
        )
        
        # Write header
        writer.writeheader()
        
        # Write data rows
        for result in self.results:
            row = self._flatten_result(result)
            writer.writerow(row)
        
        csv_content = output.getvalue()
        
        # Output
        if output_file:
            with open(output_file, 'w', newline='') as f:
                f.write(csv_content)
            return None
        else:
            return csv_content
    
    def _get_all_fields(self) -> List[str]:
        """Get all unique fields from results."""
        fields = set()
        
        # Standard fields
        fields.update([
            "name", "iterations", "mean_time", "min_time", 
            "max_time", "median_time", "std_dev", "percentile_95"
        ])
        
        # Collect all metric fields
        for result in self.results:
            if "metrics" in result:
                metric_fields = self._get_metric_fields(result["metrics"])
                fields.update(metric_fields)
        
        # Sort fields for consistent output
        return sorted(list(fields))
    
    def _get_metric_fields(self, metrics: Dict[str, Any], prefix: str = "") -> List[str]:
        """Recursively get all metric field names."""
        fields = []
        
        for key, value in metrics.items():
            field_name = f"{prefix}{key}" if prefix else key
            
            if isinstance(value, dict):
                # Recurse for nested metrics
                nested_fields = self._get_metric_fields(value, f"{field_name}.")
                fields.extend(nested_fields)
            else:
                fields.append(field_name)
        
        return fields
    
    def _flatten_result(self, result: Dict[str, Any]) -> Dict[str, Any]:
        """Flatten a result dictionary for CSV output."""
        row = {
            "name": result.get("name", ""),
            "iterations": result.get("iterations", 0)
        }
        
        # Add statistics if execution times available
        if "execution_times" in result:
            stats = self.calculate_statistics(result["execution_times"])
            row.update({
                "mean_time": stats["mean"],
                "min_time": stats["min"],
                "max_time": stats["max"],
                "median_time": stats["median"],
                "std_dev": stats["std_dev"],
                "percentile_95": stats["percentile_95"]
            })
        
        # Add flattened metrics
        if "metrics" in result:
            flattened_metrics = self._flatten_dict(result["metrics"])
            row.update(flattened_metrics)
        
        return row
    
    def _flatten_dict(self, d: Dict[str, Any], parent_key: str = "") -> Dict[str, Any]:
        """Flatten nested dictionary."""
        items = []
        
        for k, v in d.items():
            new_key = f"{parent_key}{k}" if parent_key else k
            
            if isinstance(v, dict):
                items.extend(self._flatten_dict(v, f"{new_key}.").items())
            else:
                items.append((new_key, v))
        
        return dict(items)