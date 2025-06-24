#!/usr/bin/env python3
"""
QuDAG Benchmark Comparison Tool

Compare benchmark results between runs to track performance changes.
"""

import json
import sys
import argparse
from typing import Dict, Any, List, Tuple
from datetime import datetime

class BenchmarkComparator:
    """Compare benchmark results and identify performance changes"""
    
    def __init__(self, baseline_path: str, current_path: str):
        self.baseline = self._load_results(baseline_path)
        self.current = self._load_results(current_path)
        self.threshold = 0.1  # 10% change threshold
        
    def _load_results(self, path: str) -> Dict[str, Any]:
        """Load benchmark results from JSON file"""
        with open(path, 'r') as f:
            return json.load(f)
            
    def _calculate_change(self, baseline: float, current: float) -> float:
        """Calculate percentage change"""
        if baseline == 0:
            return 0
        return (current - baseline) / baseline
        
    def _format_change(self, change: float) -> str:
        """Format change percentage with color codes"""
        percentage = change * 100
        if abs(percentage) < self.threshold * 100:
            return f"{percentage:+.1f}%"
        elif percentage < 0:
            return f"\033[92m{percentage:+.1f}%\033[0m"  # Green for improvement
        else:
            return f"\033[91m{percentage:+.1f}%\033[0m"  # Red for regression
            
    def compare_metrics(self) -> List[Dict[str, Any]]:
        """Compare all metrics between baseline and current"""
        comparisons = []
        
        # Navigate through benchmark structure
        for category in ['cli', 'network', 'dag', 'swarm']:
            if category not in self.baseline.get('benchmarks', {}):
                continue
                
            baseline_cat = self.baseline['benchmarks'][category]
            current_cat = self.current['benchmarks'].get(category, {})
            
            if 'benchmarks' not in baseline_cat:
                continue
                
            for bench_type, bench_data in baseline_cat['benchmarks'].items():
                if not isinstance(bench_data, dict):
                    continue
                    
                current_bench = current_cat.get('benchmarks', {}).get(bench_type, {})
                
                for metric_name, metric_data in bench_data.items():
                    if not isinstance(metric_data, dict) or 'avg_time' not in metric_data:
                        continue
                        
                    current_metric = current_bench.get(metric_name, {})
                    if 'avg_time' not in current_metric:
                        continue
                        
                    baseline_time = metric_data['avg_time']
                    current_time = current_metric['avg_time']
                    change = self._calculate_change(baseline_time, current_time)
                    
                    comparisons.append({
                        'category': category,
                        'benchmark': bench_type,
                        'metric': metric_name,
                        'baseline': baseline_time,
                        'current': current_time,
                        'change': change,
                        'regression': change > self.threshold
                    })
                    
        return comparisons
        
    def generate_report(self, comparisons: List[Dict[str, Any]]) -> str:
        """Generate comparison report"""
        report = [
            "# QuDAG Benchmark Comparison Report",
            f"Baseline: {self.baseline.get('timestamp', 'Unknown')}",
            f"Current: {self.current.get('timestamp', 'Unknown')}",
            f"Change threshold: ±{self.threshold*100:.0f}%",
            "",
            "## Summary",
            ""
        ]
        
        # Count regressions and improvements
        regressions = [c for c in comparisons if c['change'] > self.threshold]
        improvements = [c for c in comparisons if c['change'] < -self.threshold]
        
        report.extend([
            f"- Total metrics compared: {len(comparisons)}",
            f"- Regressions detected: {len(regressions)}",
            f"- Improvements detected: {len(improvements)}",
            f"- Unchanged (within threshold): {len(comparisons) - len(regressions) - len(improvements)}",
            ""
        ])
        
        # Show regressions
        if regressions:
            report.extend([
                "## ⚠️ Performance Regressions",
                ""
            ])
            for reg in sorted(regressions, key=lambda x: x['change'], reverse=True):
                report.append(
                    f"- **{reg['category']}/{reg['benchmark']}/{reg['metric']}**: "
                    f"{reg['baseline']*1000:.2f}ms → {reg['current']*1000:.2f}ms "
                    f"({self._format_change(reg['change'])})"
                )
                
        # Show improvements
        if improvements:
            report.extend([
                "",
                "## ✅ Performance Improvements",
                ""
            ])
            for imp in sorted(improvements, key=lambda x: x['change']):
                report.append(
                    f"- **{imp['category']}/{imp['benchmark']}/{imp['metric']}**: "
                    f"{imp['baseline']*1000:.2f}ms → {imp['current']*1000:.2f}ms "
                    f"({self._format_change(imp['change'])})"
                )
                
        # Detailed comparison table
        report.extend([
            "",
            "## Detailed Comparison",
            "",
            "| Category | Benchmark | Metric | Baseline (ms) | Current (ms) | Change |",
            "|----------|-----------|--------|---------------|--------------|--------|"
        ])
        
        for comp in sorted(comparisons, key=lambda x: (x['category'], x['benchmark'], x['metric'])):
            report.append(
                f"| {comp['category']} | {comp['benchmark']} | {comp['metric']} | "
                f"{comp['baseline']*1000:.2f} | {comp['current']*1000:.2f} | "
                f"{self._format_change(comp['change'])} |"
            )
            
        return "\n".join(report)
        
    def check_ci_pass(self, comparisons: List[Dict[str, Any]], 
                     regression_threshold: float = 0.2) -> bool:
        """Check if CI should pass based on regression threshold"""
        major_regressions = [
            c for c in comparisons 
            if c['change'] > regression_threshold
        ]
        return len(major_regressions) == 0


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="Compare QuDAG benchmark results"
    )
    
    parser.add_argument(
        'baseline',
        help='Path to baseline benchmark results JSON'
    )
    
    parser.add_argument(
        'current',
        help='Path to current benchmark results JSON'
    )
    
    parser.add_argument(
        '--threshold',
        type=float,
        default=0.1,
        help='Performance change threshold (default: 0.1 = 10%%)'
    )
    
    parser.add_argument(
        '--ci-mode',
        action='store_true',
        help='CI mode - exit with error if major regressions detected'
    )
    
    parser.add_argument(
        '--regression-threshold',
        type=float,
        default=0.2,
        help='Major regression threshold for CI mode (default: 0.2 = 20%%)'
    )
    
    parser.add_argument(
        '--output',
        help='Output file for comparison report'
    )
    
    args = parser.parse_args()
    
    # Create comparator
    comparator = BenchmarkComparator(args.baseline, args.current)
    comparator.threshold = args.threshold
    
    # Run comparison
    comparisons = comparator.compare_metrics()
    
    # Generate report
    report = comparator.generate_report(comparisons)
    print(report)
    
    # Save report if requested
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
        print(f"\nReport saved to: {args.output}")
        
    # CI mode check
    if args.ci_mode:
        if not comparator.check_ci_pass(comparisons, args.regression_threshold):
            print(f"\n❌ CI FAILED: Major performance regressions detected!")
            sys.exit(1)
        else:
            print(f"\n✅ CI PASSED: No major regressions detected")
            sys.exit(0)


if __name__ == "__main__":
    main()