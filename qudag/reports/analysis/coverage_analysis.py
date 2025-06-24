#!/usr/bin/env python3
"""
QuDAG Test Coverage Analysis Tool
Analyzes test coverage across the QuDAG workspace and identifies uncovered code paths.
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple
from dataclasses import dataclass, asdict
import subprocess
import sys

@dataclass
class CoverageStats:
    total_lines: int
    test_lines: int
    covered_lines: int
    coverage_percentage: float
    uncovered_functions: List[str]
    uncovered_modules: List[str]

@dataclass 
class ModuleCoverage:
    module_name: str
    source_files: List[str]
    test_files: List[str]
    functions: List[str]
    tested_functions: List[str]
    coverage_stats: CoverageStats

class CoverageAnalyzer:
    def __init__(self, workspace_root: str):
        self.workspace_root = Path(workspace_root)
        self.modules = {
            'crypto': self.workspace_root / 'core' / 'crypto',
            'dag': self.workspace_root / 'core' / 'dag', 
            'network': self.workspace_root / 'core' / 'network',
            'protocol': self.workspace_root / 'core' / 'protocol',
            'cli': self.workspace_root / 'tools' / 'cli',
            'simulator': self.workspace_root / 'tools' / 'simulator',
        }
        
    def analyze_workspace(self) -> Dict[str, ModuleCoverage]:
        """Analyze coverage for all modules in the workspace."""
        results = {}
        
        for module_name, module_path in self.modules.items():
            if module_path.exists():
                print(f"Analyzing {module_name} module...")
                results[module_name] = self.analyze_module(module_name, module_path)
                
        return results
    
    def analyze_module(self, module_name: str, module_path: Path) -> ModuleCoverage:
        """Analyze coverage for a specific module."""
        
        # Find source files
        source_files = list(self.find_rust_files(module_path / 'src'))
        test_files = list(self.find_rust_files(module_path / 'tests')) if (module_path / 'tests').exists() else []
        
        # Also check for inline tests in src files
        for src_file in source_files:
            if self.has_inline_tests(src_file):
                test_files.append(str(src_file))
        
        # Extract functions from source files
        functions = []
        for src_file in source_files:
            functions.extend(self.extract_functions(src_file))
        
        # Extract tested functions from test files
        tested_functions = []
        for test_file in test_files:
            tested_functions.extend(self.extract_tested_functions(test_file))
        
        # Calculate coverage stats
        coverage_stats = self.calculate_coverage_stats(
            source_files, test_files, functions, tested_functions
        )
        
        return ModuleCoverage(
            module_name=module_name,
            source_files=[str(f) for f in source_files],
            test_files=[str(f) for f in test_files],
            functions=functions,
            tested_functions=tested_functions,
            coverage_stats=coverage_stats
        )
    
    def find_rust_files(self, directory: Path) -> List[Path]:
        """Find all Rust source files in a directory."""
        if not directory.exists():
            return []
        
        rust_files = []
        for file_path in directory.rglob('*.rs'):
            if file_path.is_file():
                rust_files.append(file_path)
        
        return rust_files
    
    def has_inline_tests(self, file_path: Path) -> bool:
        """Check if a source file contains inline tests."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                return '#[test]' in content or '#[cfg(test)]' in content
        except Exception:
            return False
    
    def extract_functions(self, file_path: Path) -> List[str]:
        """Extract function names from a Rust source file."""
        functions = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
                # Match function definitions
                function_pattern = r'(?:pub\s+)?(?:async\s+)?fn\s+(\w+)'
                matches = re.findall(function_pattern, content)
                functions.extend(matches)
                
                # Match method definitions in impl blocks
                impl_method_pattern = r'impl[^{]*\{[^}]*?(?:pub\s+)?(?:async\s+)?fn\s+(\w+)'
                impl_matches = re.findall(impl_method_pattern, content, re.DOTALL)
                functions.extend(impl_matches)
                
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
        
        return list(set(functions))  # Remove duplicates
    
    def extract_tested_functions(self, file_path: Path) -> List[str]:
        """Extract function names that are being tested."""
        tested_functions = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
                # Look for function calls in test functions
                test_function_pattern = r'#\[test\][^}]*?fn\s+\w+[^}]*?\{([^}]*)\}'
                test_matches = re.findall(test_function_pattern, content, re.DOTALL)
                
                for test_body in test_matches:
                    # Extract function calls
                    function_call_pattern = r'(\w+)\s*\('
                    calls = re.findall(function_call_pattern, test_body)
                    tested_functions.extend(calls)
                
                # Also look for direct function name mentions in test names
                test_name_pattern = r'#\[test\][^}]*?fn\s+test_(\w+)'
                name_matches = re.findall(test_name_pattern, content)
                tested_functions.extend(name_matches)
                
        except Exception as e:
            print(f"Error reading test file {file_path}: {e}")
        
        return list(set(tested_functions))  # Remove duplicates
    
    def calculate_coverage_stats(self, source_files: List[Path], test_files: List[Path], 
                               functions: List[str], tested_functions: List[str]) -> CoverageStats:
        """Calculate coverage statistics."""
        
        total_lines = 0
        test_lines = 0
        
        # Count lines in source files
        for src_file in source_files:
            try:
                with open(src_file, 'r', encoding='utf-8') as f:
                    lines = len([line for line in f if line.strip() and not line.strip().startswith('//')])
                    total_lines += lines
            except Exception:
                pass
        
        # Count lines in test files
        for test_file in test_files:
            try:
                with open(test_file, 'r', encoding='utf-8') as f:
                    lines = len([line for line in f if line.strip() and not line.strip().startswith('//')])
                    test_lines += lines
            except Exception:
                pass
        
        # Calculate function coverage
        tested_function_set = set(tested_functions)
        function_set = set(functions)
        
        covered_functions = function_set.intersection(tested_function_set)
        uncovered_functions = list(function_set - tested_function_set)
        
        # Estimate covered lines (rough approximation)
        if len(functions) > 0:
            function_coverage_ratio = len(covered_functions) / len(functions)
            covered_lines = int(total_lines * function_coverage_ratio)
        else:
            covered_lines = 0
        
        coverage_percentage = (covered_lines / total_lines * 100) if total_lines > 0 else 0
        
        return CoverageStats(
            total_lines=total_lines,
            test_lines=test_lines,
            covered_lines=covered_lines,
            coverage_percentage=coverage_percentage,
            uncovered_functions=uncovered_functions,
            uncovered_modules=[]
        )
    
    def generate_report(self, coverage_results: Dict[str, ModuleCoverage]) -> str:
        """Generate a comprehensive coverage report."""
        
        report = []
        report.append("# QuDAG Test Coverage Analysis Report")
        report.append("=" * 50)
        report.append("")
        
        # Overall summary
        total_lines = sum(result.coverage_stats.total_lines for result in coverage_results.values())
        total_covered = sum(result.coverage_stats.covered_lines for result in coverage_results.values())
        overall_coverage = (total_covered / total_lines * 100) if total_lines > 0 else 0
        
        report.append("## Overall Coverage Summary")
        report.append(f"- Total Source Lines: {total_lines:,}")
        report.append(f"- Covered Lines: {total_covered:,}")
        report.append(f"- Overall Coverage: {overall_coverage:.2f}%")
        report.append("")
        
        # Per-module analysis
        report.append("## Module Coverage Analysis")
        report.append("")
        
        for module_name, module_coverage in coverage_results.items():
            stats = module_coverage.coverage_stats
            
            report.append(f"### {module_name.upper()} Module")
            report.append(f"- Source Files: {len(module_coverage.source_files)}")
            report.append(f"- Test Files: {len(module_coverage.test_files)}")
            report.append(f"- Total Functions: {len(module_coverage.functions)}")
            report.append(f"- Tested Functions: {len(module_coverage.tested_functions)}")
            report.append(f"- Coverage: {stats.coverage_percentage:.2f}%")
            
            if stats.uncovered_functions:
                report.append(f"- Uncovered Functions ({len(stats.uncovered_functions)}): ")
                for func in sorted(stats.uncovered_functions)[:10]:  # Show first 10
                    report.append(f"  - {func}")
                if len(stats.uncovered_functions) > 10:
                    report.append(f"  ... and {len(stats.uncovered_functions) - 10} more")
            
            report.append("")
        
        # Priority recommendations
        report.append("## Coverage Improvement Priorities")
        report.append("")
        
        # Sort modules by coverage percentage
        sorted_modules = sorted(coverage_results.items(), 
                              key=lambda x: x[1].coverage_stats.coverage_percentage)
        
        for i, (module_name, module_coverage) in enumerate(sorted_modules[:3]):
            priority = ["HIGH", "MEDIUM", "LOW"][i] if i < 3 else "LOW"
            stats = module_coverage.coverage_stats
            
            report.append(f"### {priority} Priority: {module_name.upper()} Module")
            report.append(f"- Current Coverage: {stats.coverage_percentage:.2f}%")
            report.append(f"- Uncovered Functions: {len(stats.uncovered_functions)}")
            
            if stats.uncovered_functions:
                report.append("- Immediate Actions Needed:")
                for func in sorted(stats.uncovered_functions)[:5]:
                    report.append(f"  - Add tests for `{func}()` function")
            
            report.append("")
        
        # Path to 100% coverage
        report.append("## Path to 100% Coverage Achievement")
        report.append("")
        
        total_uncovered = sum(len(result.coverage_stats.uncovered_functions) 
                            for result in coverage_results.values())
        
        report.append(f"### Total Uncovered Functions: {total_uncovered}")
        report.append("")
        
        phase_1_target = 70
        phase_2_target = 85
        phase_3_target = 95
        
        report.append("### Phase 1: Foundation (Target: 70% Coverage)")
        report.append("- Focus on core crypto and DAG consensus functions")
        report.append("- Implement unit tests for all public APIs")
        report.append("- Add integration tests for critical paths")
        report.append("")
        
        report.append("### Phase 2: Integration (Target: 85% Coverage)")
        report.append("- Add comprehensive network and protocol tests")
        report.append("- Implement property-based testing")
        report.append("- Add security and adversarial tests")
        report.append("")
        
        report.append("### Phase 3: Completion (Target: 95%+ Coverage)")
        report.append("- Add edge case and error handling tests")
        report.append("- Implement fuzzing-based test generation")
        report.append("- Add performance regression tests")
        report.append("")
        
        report.append("### Phase 4: Excellence (Target: 100% Coverage)")
        report.append("- Cover all remaining edge cases")
        report.append("- Add comprehensive documentation tests")
        report.append("- Implement mutation testing for test quality")
        report.append("")
        
        return "\n".join(report)
    
    def save_json_report(self, coverage_results: Dict[str, ModuleCoverage], 
                        output_path: str):
        """Save coverage results as JSON for further analysis."""
        
        json_data = {}
        for module_name, module_coverage in coverage_results.items():
            json_data[module_name] = asdict(module_coverage)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(json_data, f, indent=2)

def main():
    """Main function to run coverage analysis."""
    
    workspace_root = '/workspaces/QuDAG'
    
    print("QuDAG Test Coverage Analysis")
    print("=" * 40)
    print(f"Analyzing workspace: {workspace_root}")
    print()
    
    analyzer = CoverageAnalyzer(workspace_root)
    
    # Run analysis
    coverage_results = analyzer.analyze_workspace()
    
    # Generate reports
    report_text = analyzer.generate_report(coverage_results)
    
    # Save reports
    with open(f'{workspace_root}/COVERAGE_ANALYSIS_REPORT.md', 'w', encoding='utf-8') as f:
        f.write(report_text)
    
    analyzer.save_json_report(coverage_results, f'{workspace_root}/coverage_analysis.json')
    
    print(report_text)
    print()
    print("Reports saved:")
    print(f"- Text report: {workspace_root}/COVERAGE_ANALYSIS_REPORT.md")
    print(f"- JSON data: {workspace_root}/coverage_analysis.json")

if __name__ == '__main__':
    main()