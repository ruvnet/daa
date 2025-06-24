#!/usr/bin/env python3
"""Test runner for QuDAG benchmarking framework."""

import argparse
import subprocess
import sys
from pathlib import Path

def run_tests(args):
    """Run the test suite with specified options."""
    
    # Base pytest command
    cmd = ["pytest"]
    
    # Add test directory
    test_dir = Path(__file__).parent / "tests"
    if args.category:
        test_dir = test_dir / args.category
    cmd.append(str(test_dir))
    
    # Verbosity
    if args.verbose:
        cmd.append("-vv")
    else:
        cmd.append("-v")
    
    # Coverage
    if args.coverage:
        cmd.extend([
            "--cov=benchmarking",
            "--cov-report=html:reports/coverage",
            "--cov-report=term-missing"
        ])
    
    # Benchmarking
    if args.benchmark:
        cmd.append("--benchmark-only")
        cmd.append("--benchmark-autosave")
    
    # Parallel execution
    if args.parallel:
        cmd.extend(["-n", "auto"])
    
    # Markers
    if args.markers:
        for marker in args.markers:
            cmd.extend(["-m", marker])
    
    # Output format
    if args.junit:
        cmd.append(f"--junit-xml=reports/junit-{args.category or 'all'}.xml")
    
    if args.html:
        cmd.append(f"--html=reports/test-report-{args.category or 'all'}.html")
    
    # Debugging
    if args.debug:
        cmd.extend(["-s", "--log-cli-level=DEBUG"])
    
    # Fail fast
    if args.fail_fast:
        cmd.append("-x")
    
    # Run specific test
    if args.test:
        cmd.extend(["-k", args.test])
    
    print(f"Running command: {' '.join(cmd)}")
    
    # Execute tests
    result = subprocess.run(cmd, cwd=Path(__file__).parent.parent)
    return result.returncode

def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Run QuDAG benchmarking tests",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run all tests with coverage
  python run_tests.py --coverage
  
  # Run only unit tests in parallel
  python run_tests.py -c unit --parallel
  
  # Run integration tests with benchmarking
  python run_tests.py -c integration --benchmark
  
  # Run specific test with debug output
  python run_tests.py -t test_client_connection --debug
  
  # Run tests marked as 'critical'
  python run_tests.py -m critical
"""
    )
    
    parser.add_argument(
        "-c", "--category",
        choices=["unit", "integration", "performance"],
        help="Test category to run"
    )
    
    parser.add_argument(
        "-t", "--test",
        help="Run specific test by name pattern"
    )
    
    parser.add_argument(
        "-m", "--markers",
        nargs="+",
        help="Run tests with specific markers"
    )
    
    parser.add_argument(
        "--coverage",
        action="store_true",
        help="Run with coverage analysis"
    )
    
    parser.add_argument(
        "--benchmark",
        action="store_true",
        help="Run benchmark tests only"
    )
    
    parser.add_argument(
        "--parallel",
        action="store_true",
        help="Run tests in parallel"
    )
    
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose output"
    )
    
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug output"
    )
    
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="Stop on first failure"
    )
    
    parser.add_argument(
        "--junit",
        action="store_true",
        help="Generate JUnit XML report"
    )
    
    parser.add_argument(
        "--html",
        action="store_true",
        help="Generate HTML report"
    )
    
    args = parser.parse_args()
    
    # Ensure reports directory exists
    reports_dir = Path(__file__).parent / "reports"
    reports_dir.mkdir(exist_ok=True)
    
    # Run tests
    exit_code = run_tests(args)
    
    if exit_code == 0:
        print("\n✅ All tests passed!")
    else:
        print(f"\n❌ Tests failed with exit code: {exit_code}")
    
    sys.exit(exit_code)

if __name__ == "__main__":
    main()