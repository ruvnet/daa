#!/usr/bin/env python3
"""
Detailed QuDAG Test Coverage Analysis
Provides deep analysis of code paths, uncovered branches, and specific test scenarios.
"""

import os
import re
import ast
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass
import json

@dataclass
class CodePath:
    file_path: str
    function_name: str
    line_start: int
    line_end: int
    complexity: int
    is_public: bool
    has_tests: bool
    test_scenarios: List[str]
    missing_scenarios: List[str]

@dataclass
class CriticalPath:
    path_name: str
    functions: List[str]
    security_critical: bool
    performance_critical: bool
    coverage_percentage: float
    risk_level: str

class DetailedCoverageAnalyzer:
    def __init__(self, workspace_root: str):
        self.workspace_root = Path(workspace_root)
        self.critical_patterns = {
            'crypto': ['encrypt', 'decrypt', 'sign', 'verify', 'keygen', 'hash'],
            'security': ['validate', 'authenticate', 'authorize', 'sanitize'],
            'network': ['send', 'receive', 'route', 'connect', 'disconnect'],
            'consensus': ['propose', 'vote', 'finalize', 'commit', 'rollback'],
            'error': ['error', 'panic', 'unwrap', 'expect', 'result']
        }
        
    def analyze_code_paths(self) -> Dict[str, List[CodePath]]:
        """Analyze individual code paths and their test coverage."""
        results = {}
        
        for module_name in ['crypto', 'dag', 'network', 'protocol']:
            module_path = self.workspace_root / 'core' / module_name
            if module_path.exists():
                print(f"Analyzing code paths in {module_name}...")
                results[module_name] = self.analyze_module_paths(module_path)
        
        return results
    
    def analyze_module_paths(self, module_path: Path) -> List[CodePath]:
        """Analyze code paths in a specific module."""
        code_paths = []
        
        src_path = module_path / 'src'
        if not src_path.exists():
            return code_paths
        
        # Find all Rust source files
        for rs_file in src_path.rglob('*.rs'):
            paths = self.extract_code_paths(rs_file, module_path)
            code_paths.extend(paths)
        
        return code_paths
    
    def extract_code_paths(self, file_path: Path, module_path: Path) -> List[CodePath]:
        """Extract code paths from a Rust source file."""
        code_paths = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Find all function definitions
            functions = self.find_rust_functions(content)
            
            # Find corresponding tests
            test_coverage = self.find_test_coverage(file_path, module_path)
            
            for func_info in functions:
                func_name = func_info['name']
                
                # Determine test coverage for this function
                has_tests = func_name in test_coverage
                test_scenarios = test_coverage.get(func_name, [])
                
                # Identify missing test scenarios
                missing_scenarios = self.identify_missing_scenarios(
                    func_info, content, has_tests
                )
                
                code_path = CodePath(
                    file_path=str(file_path.relative_to(self.workspace_root)),
                    function_name=func_name,
                    line_start=func_info['line_start'],
                    line_end=func_info['line_end'],
                    complexity=self.calculate_complexity(func_info['body']),
                    is_public=func_info['is_public'],
                    has_tests=has_tests,
                    test_scenarios=test_scenarios,
                    missing_scenarios=missing_scenarios
                )
                
                code_paths.append(code_path)
        
        except Exception as e:
            print(f"Error analyzing {file_path}: {e}")
        
        return code_paths
    
    def find_rust_functions(self, content: str) -> List[Dict]:
        """Find all function definitions in Rust code."""
        functions = []
        
        # Match function definitions with their bodies
        pattern = r'(pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)(?:\s*->\s*[^{]+)?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}'
        
        for match in re.finditer(pattern, content, re.DOTALL):
            is_public = match.group(1) is not None
            func_name = match.group(2)
            func_body = match.group(3)
            
            # Calculate line numbers
            start_pos = match.start()
            line_start = content[:start_pos].count('\n') + 1
            end_pos = match.end()
            line_end = content[:end_pos].count('\n') + 1
            
            functions.append({
                'name': func_name,
                'is_public': is_public,
                'body': func_body,
                'line_start': line_start,
                'line_end': line_end
            })
        
        return functions
    
    def find_test_coverage(self, source_file: Path, module_path: Path) -> Dict[str, List[str]]:
        """Find test coverage for functions in the source file."""
        test_coverage = {}
        
        # Check for inline tests in the same file
        self.check_inline_tests(source_file, test_coverage)
        
        # Check for external test files
        tests_dir = module_path / 'tests'
        if tests_dir.exists():
            for test_file in tests_dir.rglob('*.rs'):
                self.check_external_tests(test_file, source_file, test_coverage)
        
        return test_coverage
    
    def check_inline_tests(self, source_file: Path, test_coverage: Dict[str, List[str]]):
        """Check for inline tests in the source file."""
        try:
            with open(source_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Find test functions
            test_pattern = r'#\[test\]\s*(?:async\s+)?fn\s+(\w+)'
            test_matches = re.findall(test_pattern, content)
            
            for test_name in test_matches:
                # Try to infer what function is being tested
                if 'test_' in test_name:
                    tested_func = test_name.replace('test_', '').replace('_', '')
                    if tested_func not in test_coverage:
                        test_coverage[tested_func] = []
                    test_coverage[tested_func].append(f"inline:{test_name}")
        
        except Exception as e:
            print(f"Error checking inline tests in {source_file}: {e}")
    
    def check_external_tests(self, test_file: Path, source_file: Path, 
                           test_coverage: Dict[str, List[str]]):
        """Check external test files for coverage."""
        try:
            with open(test_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Find test functions that might test functions from source_file
            test_pattern = r'#\[test\]\s*(?:async\s+)?fn\s+(\w+)'
            test_matches = re.findall(test_pattern, content)
            
            for test_name in test_matches:
                # Look for function calls in the test
                test_func_pattern = rf'fn\s+{re.escape(test_name)}\s*\([^)]*\)\s*\{{([^}}]*(?:\{{[^}}]*\}}[^}}]*)*)\}}'
                test_match = re.search(test_func_pattern, content, re.DOTALL)
                
                if test_match:
                    test_body = test_match.group(1)
                    
                    # Find function calls in test body
                    call_pattern = r'(\w+)\s*\('
                    calls = re.findall(call_pattern, test_body)
                    
                    for call in calls:
                        if call not in test_coverage:
                            test_coverage[call] = []
                        test_coverage[call].append(f"external:{test_name}")
        
        except Exception as e:
            print(f"Error checking external tests in {test_file}: {e}")
    
    def identify_missing_scenarios(self, func_info: Dict, full_content: str, 
                                 has_tests: bool) -> List[str]:
        """Identify missing test scenarios for a function."""
        missing_scenarios = []
        func_body = func_info['body']
        func_name = func_info['name']
        
        # Check for error handling paths
        if 'Result<' in full_content and ('Err(' in func_body or '?' in func_body):
            missing_scenarios.append("Error handling scenarios")
        
        # Check for different input validation paths
        if 'if ' in func_body and not has_tests:
            branch_count = func_body.count('if ')
            missing_scenarios.append(f"Branch coverage ({branch_count} branches)")
        
        # Check for async/await patterns
        if 'await' in func_body:
            missing_scenarios.append("Async execution scenarios")
        
        # Check for security-critical patterns
        for category, patterns in self.critical_patterns.items():
            if any(pattern in func_name.lower() for pattern in patterns):
                missing_scenarios.append(f"{category.title()} security scenarios")
                break
        
        # Check for loop scenarios
        if any(keyword in func_body for keyword in ['for ', 'while ', 'loop ']):
            missing_scenarios.append("Loop iteration scenarios")
        
        # Check for memory management
        if any(keyword in func_body for keyword in ['Box::', 'Vec::', 'HashMap::']):
            missing_scenarios.append("Memory management scenarios")
        
        return missing_scenarios
    
    def calculate_complexity(self, func_body: str) -> int:
        """Calculate cyclomatic complexity of a function."""
        complexity = 1  # Base complexity
        
        # Count decision points
        decision_keywords = ['if ', 'else if ', 'match ', 'while ', 'for ', 'loop ']
        for keyword in decision_keywords:
            complexity += func_body.count(keyword)
        
        # Count match arms (rough approximation)
        complexity += func_body.count('=>')
        
        return complexity
    
    def identify_critical_paths(self, code_paths: Dict[str, List[CodePath]]) -> List[CriticalPath]:
        """Identify critical code paths that need priority testing."""
        critical_paths = []
        
        # Security-critical paths
        security_funcs = []
        for module_name, paths in code_paths.items():
            for path in paths:
                if any(pattern in path.function_name.lower() 
                      for pattern in self.critical_patterns['crypto'] + self.critical_patterns['security']):
                    security_funcs.append(path.function_name)
        
        if security_funcs:
            security_coverage = sum(1 for module_paths in code_paths.values() 
                                  for path in module_paths 
                                  if path.function_name in security_funcs and path.has_tests) / len(security_funcs) * 100
            
            critical_paths.append(CriticalPath(
                path_name="Security-Critical Functions",
                functions=security_funcs,
                security_critical=True,
                performance_critical=False,
                coverage_percentage=security_coverage,
                risk_level="HIGH" if security_coverage < 80 else "MEDIUM"
            ))
        
        # Network-critical paths
        network_funcs = []
        for module_name, paths in code_paths.items():
            if module_name == 'network':
                for path in paths:
                    if path.is_public and path.complexity > 5:
                        network_funcs.append(path.function_name)
        
        if network_funcs:
            network_coverage = sum(1 for module_paths in code_paths.values() 
                                 for path in module_paths 
                                 if path.function_name in network_funcs and path.has_tests) / len(network_funcs) * 100
            
            critical_paths.append(CriticalPath(
                path_name="Network Communication Paths",
                functions=network_funcs,
                security_critical=False,
                performance_critical=True,
                coverage_percentage=network_coverage,
                risk_level="HIGH" if network_coverage < 70 else "MEDIUM"
            ))
        
        # Consensus-critical paths
        consensus_funcs = []
        for module_name, paths in code_paths.items():
            if module_name == 'dag':
                for path in paths:
                    if any(pattern in path.function_name.lower() 
                          for pattern in self.critical_patterns['consensus']):
                        consensus_funcs.append(path.function_name)
        
        if consensus_funcs:
            consensus_coverage = sum(1 for module_paths in code_paths.values() 
                                   for path in module_paths 
                                   if path.function_name in consensus_funcs and path.has_tests) / len(consensus_funcs) * 100
            
            critical_paths.append(CriticalPath(
                path_name="Consensus Algorithm Paths",
                functions=consensus_funcs,
                security_critical=True,
                performance_critical=True,  
                coverage_percentage=consensus_coverage,
                risk_level="CRITICAL" if consensus_coverage < 90 else "HIGH"
            ))
        
        return critical_paths
    
    def generate_detailed_report(self, code_paths: Dict[str, List[CodePath]], 
                               critical_paths: List[CriticalPath]) -> str:
        """Generate a detailed coverage report."""
        report = []
        report.append("# QuDAG Detailed Test Coverage Analysis Report")
        report.append("=" * 60)
        report.append("")
        
        # Critical path analysis
        report.append("## Critical Path Coverage Analysis")
        report.append("")
        
        for crit_path in sorted(critical_paths, key=lambda x: x.coverage_percentage):
            report.append(f"### {crit_path.path_name}")
            report.append(f"- **Risk Level**: {crit_path.risk_level}")
            report.append(f"- **Coverage**: {crit_path.coverage_percentage:.1f}%")
            report.append(f"- **Security Critical**: {'Yes' if crit_path.security_critical else 'No'}")
            report.append(f"- **Performance Critical**: {'Yes' if crit_path.performance_critical else 'No'}")
            report.append(f"- **Functions**: {len(crit_path.functions)}")
            
            uncovered = [func for func in crit_path.functions 
                        if not any(path.function_name == func and path.has_tests 
                                 for module_paths in code_paths.values() 
                                 for path in module_paths)]
            
            if uncovered:
                report.append(f"- **Uncovered Functions** ({len(uncovered)}):")
                for func in sorted(uncovered)[:10]:
                    report.append(f"  - `{func}()`")
                if len(uncovered) > 10:
                    report.append(f"  - ... and {len(uncovered) - 10} more")
            
            report.append("")
        
        # Detailed module analysis
        report.append("## Detailed Module Analysis")
        report.append("")
        
        for module_name, paths in code_paths.items():
            report.append(f"### {module_name.upper()} Module Detailed Analysis")
            
            # Calculate module statistics
            total_paths = len(paths)
            tested_paths = len([p for p in paths if p.has_tests])
            public_paths = len([p for p in paths if p.is_public])
            high_complexity = len([p for p in paths if p.complexity > 10])
            
            report.append(f"- **Total Code Paths**: {total_paths}")
            report.append(f"- **Tested Paths**: {tested_paths} ({tested_paths/total_paths*100:.1f}%)")
            report.append(f"- **Public API Paths**: {public_paths}")
            report.append(f"- **High Complexity Paths**: {high_complexity}")
            report.append("")
            
            # Most critical untested paths
            untested_critical = [p for p in paths 
                               if not p.has_tests and (p.is_public or p.complexity > 5)]
            
            if untested_critical:
                report.append(f"#### Most Critical Untested Paths ({len(untested_critical)})")
                for path in sorted(untested_critical, key=lambda x: x.complexity, reverse=True)[:10]:
                    report.append(f"- **`{path.function_name}()`** (Complexity: {path.complexity})")
                    report.append(f"  - File: `{path.file_path}`")
                    report.append(f"  - Lines: {path.line_start}-{path.line_end}")
                    if path.missing_scenarios:
                        report.append(f"  - Missing scenarios: {', '.join(path.missing_scenarios[:3])}")
                    report.append("")
            
            report.append("")
        
        # Specific testing recommendations
        report.append("## Specific Testing Recommendations")
        report.append("")
        
        # Group recommendations by priority
        high_priority = []
        medium_priority = []
        low_priority = []
        
        for module_name, paths in code_paths.items():
            for path in paths:
                if not path.has_tests:
                    priority_score = 0
                    
                    # Security critical functions get highest priority
                    if any(pattern in path.function_name.lower() 
                          for pattern in self.critical_patterns['crypto'] + self.critical_patterns['security']):
                        priority_score += 10
                    
                    # Public APIs get high priority
                    if path.is_public:
                        priority_score += 5
                    
                    # High complexity gets medium priority
                    if path.complexity > 10:
                        priority_score += 3
                    elif path.complexity > 5:
                        priority_score += 2
                    
                    recommendation = {
                        'module': module_name,
                        'function': path.function_name,
                        'file': path.file_path,
                        'complexity': path.complexity,
                        'scenarios': path.missing_scenarios,
                        'priority_score': priority_score
                    }
                    
                    if priority_score >= 10:
                        high_priority.append(recommendation)
                    elif priority_score >= 5:
                        medium_priority.append(recommendation)
                    else:
                        low_priority.append(recommendation)
        
        # High priority recommendations
        if high_priority:
            report.append("### HIGH Priority Testing Needs")
            report.append("*These require immediate attention due to security or API criticality*")
            report.append("")
            
            for rec in sorted(high_priority, key=lambda x: x['priority_score'], reverse=True)[:15]:
                report.append(f"#### `{rec['function']}()` in {rec['module']} module")
                report.append(f"- **File**: `{rec['file']}`")
                report.append(f"- **Complexity**: {rec['complexity']}")
                if rec['scenarios']:
                    report.append(f"- **Required test scenarios**:")
                    for scenario in rec['scenarios']:
                        report.append(f"  - {scenario}")
                report.append("")
        
        # Medium priority recommendations  
        if medium_priority:
            report.append("### MEDIUM Priority Testing Needs")
            report.append("*Public APIs and complex functions that need coverage*")
            report.append("")
            
            for rec in sorted(medium_priority, key=lambda x: x['priority_score'], reverse=True)[:10]:
                report.append(f"- `{rec['function']}()` in {rec['module']} (Complexity: {rec['complexity']})")
        
        report.append("")
        
        # Implementation timeline
        report.append("## Implementation Timeline for 100% Coverage")
        report.append("")
        
        total_uncovered = sum(len([p for p in paths if not p.has_tests]) 
                            for paths in code_paths.values())
        
        report.append(f"### Current Status")
        report.append(f"- Total uncovered functions: {total_uncovered}")
        report.append(f"- High priority functions: {len(high_priority)}")
        report.append(f"- Medium priority functions: {len(medium_priority)}")
        report.append(f"- Low priority functions: {len(low_priority)}")
        report.append("")
        
        report.append("### Week 1-2: Critical Security Functions")
        report.append("- Focus on crypto and security-critical functions")
        report.append(f"- Target: {len(high_priority)} functions")
        report.append("- Expected coverage gain: 15-20%")
        report.append("")
        
        report.append("### Week 3-4: Public APIs and Integration")
        report.append("- Cover all public API functions")
        report.append(f"- Target: {len(medium_priority)} functions")
        report.append("- Expected coverage gain: 20-25%")
        report.append("")
        
        report.append("### Week 5-6: Complex Internal Functions")
        report.append("- High complexity internal functions")
        report.append("- Property-based testing implementation")
        report.append("- Expected coverage gain: 15-20%")
        report.append("")
        
        report.append("### Week 7-8: Edge Cases and Error Handling")
        report.append("- Error path coverage")
        report.append("- Edge case scenarios")
        report.append("- Expected coverage gain: 10-15%")
        report.append("")
        
        report.append("### Week 9-10: Final Coverage Push")
        report.append("- Remaining low-complexity functions")
        report.append("- Documentation tests")
        report.append("- Expected coverage gain: 10-15%")
        report.append("- **Target: 95%+ total coverage**")
        report.append("")
        
        return "\n".join(report)

def main():
    """Main function for detailed coverage analysis."""
    workspace_root = '/workspaces/QuDAG'
    
    print("QuDAG Detailed Test Coverage Analysis")
    print("=" * 50)
    print()
    
    analyzer = DetailedCoverageAnalyzer(workspace_root)
    
    # Analyze code paths
    print("Analyzing code paths...")
    code_paths = analyzer.analyze_code_paths()
    
    # Identify critical paths
    print("Identifying critical paths...")
    critical_paths = analyzer.identify_critical_paths(code_paths)
    
    # Generate detailed report
    print("Generating detailed report...")
    report = analyzer.generate_detailed_report(code_paths, critical_paths)
    
    # Save report
    with open(f'{workspace_root}/DETAILED_COVERAGE_ANALYSIS.md', 'w', encoding='utf-8') as f:
        f.write(report)
    
    # Save JSON data
    json_data = {
        'code_paths': {module: [path.__dict__ for path in paths] 
                      for module, paths in code_paths.items()},
        'critical_paths': [path.__dict__ for path in critical_paths]
    }
    
    with open(f'{workspace_root}/detailed_coverage_data.json', 'w', encoding='utf-8') as f:
        json.dump(json_data, f, indent=2)
    
    print(report)
    print()
    print("Detailed analysis completed!")
    print(f"- Report saved: {workspace_root}/DETAILED_COVERAGE_ANALYSIS.md")
    print(f"- Data saved: {workspace_root}/detailed_coverage_data.json")

if __name__ == '__main__':
    main()