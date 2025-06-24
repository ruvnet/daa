"""
HTML reporter for benchmark results.
Generates interactive HTML reports with charts.
"""
import json
import time
from typing import Dict, Any, Optional, Union

from .reporter import ResultReporter


class HTMLReporter(ResultReporter):
    """Reporter that generates HTML reports with visualizations."""
    
    def __init__(self):
        """Initialize HTML reporter."""
        super().__init__()
    
    def report(self, output_file: Optional[str] = None) -> Union[str, None]:
        """
        Generate HTML report with charts and tables.
        
        Args:
            output_file: Optional file path to write report to
            
        Returns:
            HTML string if no output file, None if written to file
        """
        html_content = self._generate_html()
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(html_content)
            return None
        else:
            return html_content
    
    def _generate_html(self) -> str:
        """Generate the complete HTML document."""
        title = self.metadata.get("title", "Benchmark Results")
        
        html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        {self._get_css()}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <p class="timestamp">Generated: {time.strftime("%Y-%m-%d %H:%M:%S", time.localtime())}</p>
        
        {self._generate_summary_section()}
        {self._generate_charts_section()}
        {self._generate_results_table()}
        {self._generate_details_section()}
    </div>
    
    <script>
        {self._get_javascript()}
    </script>
</body>
</html>"""
        return html
    
    def _get_css(self) -> str:
        """Get CSS styles for the report."""
        return """
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 
                         'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            margin: 0;
            padding: 0;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: white;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }
        
        h1 {
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
        }
        
        h2 {
            color: #34495e;
            margin-top: 30px;
        }
        
        .timestamp {
            color: #7f8c8d;
            font-style: italic;
        }
        
        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }
        
        .summary-card {
            background: #ecf0f1;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }
        
        .summary-card h3 {
            margin: 0 0 10px 0;
            color: #2c3e50;
        }
        
        .summary-card .value {
            font-size: 2em;
            font-weight: bold;
            color: #3498db;
        }
        
        .chart-container {
            position: relative;
            height: 400px;
            margin: 20px 0;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }
        
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        
        th {
            background: #34495e;
            color: white;
            font-weight: bold;
        }
        
        tr:nth-child(even) {
            background: #f9f9f9;
        }
        
        tr:hover {
            background: #e8f4f8;
        }
        
        .details {
            margin: 20px 0;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
        }
        
        .metrics {
            margin: 10px 0;
            padding: 10px;
            background: #e9ecef;
            border-radius: 4px;
            font-family: monospace;
            font-size: 0.9em;
        }
        """
    
    def _generate_summary_section(self) -> str:
        """Generate summary cards section."""
        total_benchmarks = len(self.results)
        total_time = sum(
            sum(r["execution_times"]) 
            for r in self.results 
            if "execution_times" in r
        )
        total_iterations = sum(
            r.get("iterations", len(r.get("execution_times", []))) 
            for r in self.results
        )
        
        return f"""
        <h2>Summary</h2>
        <div class="summary-grid">
            <div class="summary-card">
                <h3>Total Benchmarks</h3>
                <div class="value">{total_benchmarks}</div>
            </div>
            <div class="summary-card">
                <h3>Total Iterations</h3>
                <div class="value">{total_iterations}</div>
            </div>
            <div class="summary-card">
                <h3>Total Time</h3>
                <div class="value">{self.format_time(total_time)}</div>
            </div>
        </div>
        """
    
    def _generate_charts_section(self) -> str:
        """Generate charts section."""
        return """
        <h2>Performance Comparison</h2>
        <div class="chart-container">
            <canvas id="performanceChart"></canvas>
        </div>
        
        <h2>Execution Time Distribution</h2>
        <div class="chart-container">
            <canvas id="distributionChart"></canvas>
        </div>
        """
    
    def _generate_results_table(self) -> str:
        """Generate results table."""
        rows = []
        
        for result in self.results:
            if "execution_times" in result:
                stats = self.calculate_statistics(result["execution_times"])
                
                rows.append(f"""
                <tr>
                    <td>{result.get('name', 'Unknown')}</td>
                    <td>{result.get('iterations', len(result['execution_times']))}</td>
                    <td>{self.format_time(stats['mean'])}</td>
                    <td>{self.format_time(stats['min'])}</td>
                    <td>{self.format_time(stats['max'])}</td>
                    <td>{self.format_time(stats['std_dev'])}</td>
                </tr>
                """)
        
        return f"""
        <h2>Benchmark Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Benchmark</th>
                    <th>Iterations</th>
                    <th>Mean Time</th>
                    <th>Min Time</th>
                    <th>Max Time</th>
                    <th>Std Dev</th>
                </tr>
            </thead>
            <tbody>
                {''.join(rows)}
            </tbody>
        </table>
        """
    
    def _generate_details_section(self) -> str:
        """Generate detailed results section."""
        details = []
        
        for result in self.results:
            metrics_html = ""
            if "metrics" in result and self.metadata.get("show_metrics", True):
                metrics_html = f"""
                <div class="metrics">
                    <strong>Metrics:</strong>
                    <pre>{json.dumps(result['metrics'], indent=2)}</pre>
                </div>
                """
            
            details.append(f"""
            <div class="details">
                <h3>{result.get('name', 'Unknown')}</h3>
                {metrics_html}
            </div>
            """)
        
        return f"""
        <h2>Detailed Results</h2>
        {''.join(details)}
        """
    
    def _get_javascript(self) -> str:
        """Get JavaScript for charts."""
        # Prepare data for charts
        labels = []
        mean_times = []
        all_times = []
        
        for result in self.results:
            if "execution_times" in result:
                labels.append(result.get("name", "Unknown"))
                times = result["execution_times"]
                mean_times.append(sum(times) / len(times) * 1000)  # Convert to ms
                all_times.extend(times)
        
        return f"""
        // Performance comparison chart
        const perfCtx = document.getElementById('performanceChart').getContext('2d');
        new Chart(perfCtx, {{
            type: 'bar',
            data: {{
                labels: {json.dumps(labels)},
                datasets: [{{
                    label: 'Mean Execution Time (ms)',
                    data: {json.dumps(mean_times)},
                    backgroundColor: 'rgba(52, 152, 219, 0.8)',
                    borderColor: 'rgba(52, 152, 219, 1)',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        title: {{
                            display: true,
                            text: 'Time (ms)'
                        }}
                    }}
                }}
            }}
        }});
        
        // Distribution chart
        const distCtx = document.getElementById('distributionChart').getContext('2d');
        const allTimes = {json.dumps([t * 1000 for t in all_times[:1000]])};  // Limit to 1000 points
        
        new Chart(distCtx, {{
            type: 'scatter',
            data: {{
                datasets: [{{
                    label: 'Execution Times',
                    data: allTimes.map((time, index) => ({{x: index, y: time}})),
                    backgroundColor: 'rgba(231, 76, 60, 0.6)',
                    pointRadius: 3
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                scales: {{
                    x: {{
                        title: {{
                            display: true,
                            text: 'Execution Index'
                        }}
                    }},
                    y: {{
                        title: {{
                            display: true,
                            text: 'Time (ms)'
                        }}
                    }}
                }}
            }}
        }});
        """