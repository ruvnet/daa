#!/usr/bin/env python3
"""
QuDAG Protocol Compatibility Matrix Testing

This script creates and executes a comprehensive test matrix for compatibility
testing across different Rust versions, target platforms, and feature combinations.
"""

import subprocess
import sys
import json
import toml
import itertools
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from concurrent.futures import ThreadPoolExecutor, as_completed
import time

@dataclass
class TestConfig:
    rust_version: str
    target_platform: str
    features: str
    crate: str
    
@dataclass
class TestResult:
    config: TestConfig
    success: bool
    duration: float
    error_message: Optional[str] = None

class CompatibilityTester:
    def __init__(self, config_path: Path):
        self.config_path = config_path
        self.config = self._load_config()
        self.results: List[TestResult] = []
        
    def _load_config(self) -> Dict:
        """Load the compatibility configuration file."""
        try:
            with open(self.config_path, 'r') as f:
                return toml.load(f)
        except Exception as e:
            print(f"Failed to load config: {e}")
            sys.exit(1)
    
    def _run_command(self, cmd: List[str], timeout: int = 300) -> Tuple[bool, str]:
        """Run a command and return success status and output."""
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            return result.returncode == 0, result.stderr
        except subprocess.TimeoutExpired:
            return False, "Command timed out"
        except Exception as e:
            return False, str(e)
    
    def _install_rust_version(self, version: str) -> bool:
        """Install a Rust version using rustup."""
        if version in ["stable", "beta", "nightly"]:
            cmd = ["rustup", "toolchain", "install", version]
        else:
            cmd = ["rustup", "toolchain", "install", version]
        
        success, error = self._run_command(cmd)
        if not success:
            print(f"Failed to install Rust {version}: {error}")
        return success
    
    def _install_target(self, target: str) -> bool:
        """Install a target platform."""
        cmd = ["rustup", "target", "add", target]
        success, error = self._run_command(cmd)
        return success  # Don't fail if target already exists
    
    def _set_rust_version(self, version: str) -> bool:
        """Set the active Rust version."""
        cmd = ["rustup", "default", version]
        success, error = self._run_command(cmd)
        if not success:
            print(f"Failed to set Rust version {version}: {error}")
        return success
    
    def _test_configuration(self, test_config: TestConfig) -> TestResult:
        """Test a specific configuration."""
        start_time = time.time()
        
        # Set Rust version
        if not self._set_rust_version(test_config.rust_version):
            return TestResult(
                test_config, 
                False, 
                time.time() - start_time,
                f"Failed to set Rust version {test_config.rust_version}"
            )
        
        # Install target if needed
        if test_config.target_platform != "native":
            self._install_target(test_config.target_platform)
        
        # Build test command
        cmd = ["cargo", "test", "-p", test_config.crate]
        
        if test_config.features:
            cmd.extend(test_config.features.split())
        
        if test_config.target_platform != "native":
            cmd.extend(["--target", test_config.target_platform])
        
        cmd.append("--quiet")
        
        # Run test
        success, error = self._run_command(cmd)
        duration = time.time() - start_time
        
        return TestResult(
            test_config,
            success,
            duration,
            error if not success else None
        )
    
    def _generate_test_matrix(self) -> List[TestConfig]:
        """Generate the test matrix from configuration."""
        matrix = []
        
        # Get all combinations from config
        rust_versions = self.config.get("rust_versions", {}).get("supported", ["stable"])
        target_platforms = []
        
        # Collect all target platforms
        for tier in ["tier1", "tier2", "experimental"]:
            targets = self.config.get("target_platforms", {}).get(tier, [])
            target_platforms.extend(targets)
        
        if not target_platforms:
            target_platforms = ["native"]
        
        workspace_members = self.config.get("workspace", {}).get("members", ["core/crypto"])
        feature_combinations = self.config.get("features", {}).get("combinations", [
            {"name": "default", "flags": []}
        ])
        
        # Generate all combinations
        for rust_version in rust_versions:
            for target in target_platforms:
                for member in workspace_members:
                    for feature_combo in feature_combinations:
                        # Skip incompatible combinations
                        if self._should_skip_combination(rust_version, target, member, feature_combo):
                            continue
                            
                        crate_name = Path(member).name
                        if crate_name.startswith("qudag-"):
                            crate_name = crate_name
                        else:
                            crate_name = f"qudag-{crate_name}"
                        
                        features = " ".join(feature_combo.get("flags", []))
                        
                        matrix.append(TestConfig(
                            rust_version=rust_version,
                            target_platform=target,
                            features=features,
                            crate=crate_name
                        ))
        
        return matrix
    
    def _should_skip_combination(self, rust_version: str, target: str, member: str, feature_combo: Dict) -> bool:
        """Determine if a combination should be skipped."""
        # Skip WASM for CLI tools
        if target == "wasm32-unknown-unknown" and "cli" in member:
            return True
        
        # Skip nightly for critical combinations unless explicitly testing nightly features
        if rust_version == "nightly" and "nightly" not in feature_combo.get("name", ""):
            return True
        
        # Skip experimental targets for older Rust versions
        experimental_targets = self.config.get("target_platforms", {}).get("experimental", [])
        if target in experimental_targets and rust_version not in ["stable", "beta", "nightly"]:
            return True
        
        return False
    
    def run_tests(self, max_workers: int = 4) -> None:
        """Run all tests in the matrix."""
        matrix = self._generate_test_matrix()
        
        print(f"Generated test matrix with {len(matrix)} configurations")
        print("Starting compatibility tests...\n")
        
        # Run tests with thread pool
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            future_to_config = {
                executor.submit(self._test_configuration, config): config 
                for config in matrix
            }
            
            completed = 0
            for future in as_completed(future_to_config):
                result = future.result()
                self.results.append(result)
                completed += 1
                
                status = "✓" if result.success else "✗"
                print(f"[{completed}/{len(matrix)}] {status} {result.config.rust_version} | "
                      f"{result.config.target_platform} | {result.config.crate} | "
                      f"{result.config.features or 'default'} | {result.duration:.2f}s")
                
                if not result.success and result.error_message:
                    print(f"    Error: {result.error_message}")
    
    def generate_report(self) -> None:
        """Generate a comprehensive test report."""
        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.success)
        failed_tests = total_tests - passed_tests
        
        print("\n" + "="*80)
        print("COMPATIBILITY TEST REPORT")
        print("="*80)
        print(f"Total tests: {total_tests}")
        print(f"Passed: {passed_tests} ({passed_tests/total_tests*100:.1f}%)")
        print(f"Failed: {failed_tests} ({failed_tests/total_tests*100:.1f}%)")
        
        if failed_tests > 0:
            print(f"\nFAILED TESTS:")
            print("-" * 40)
            for result in self.results:
                if not result.success:
                    print(f"• {result.config.rust_version} + {result.config.target_platform} + "
                          f"{result.config.crate} + {result.config.features or 'default'}")
                    if result.error_message:
                        print(f"  Error: {result.error_message}")
        
        # Group results by category
        self._print_category_results("Rust Version", lambda r: r.config.rust_version)
        self._print_category_results("Target Platform", lambda r: r.config.target_platform)
        self._print_category_results("Crate", lambda r: r.config.crate)
        
        # Save detailed report to file
        self._save_json_report()
    
    def _print_category_results(self, category: str, key_func) -> None:
        """Print results grouped by category."""
        print(f"\nRESULTS BY {category.upper()}:")
        print("-" * 40)
        
        category_results = {}
        for result in self.results:
            key = key_func(result)
            if key not in category_results:
                category_results[key] = {"passed": 0, "failed": 0}
            
            if result.success:
                category_results[key]["passed"] += 1
            else:
                category_results[key]["failed"] += 1
        
        for key, counts in sorted(category_results.items()):
            total = counts["passed"] + counts["failed"]
            success_rate = counts["passed"] / total * 100
            print(f"{key:30} {counts['passed']:3}/{total:3} ({success_rate:5.1f}%)")
    
    def _save_json_report(self) -> None:
        """Save detailed results to JSON file."""
        report_data = {
            "timestamp": time.time(),
            "total_tests": len(self.results),
            "passed_tests": sum(1 for r in self.results if r.success),
            "failed_tests": sum(1 for r in self.results if not r.success),
            "results": [
                {
                    "rust_version": r.config.rust_version,
                    "target_platform": r.config.target_platform,
                    "crate": r.config.crate,
                    "features": r.config.features,
                    "success": r.success,
                    "duration": r.duration,
                    "error_message": r.error_message
                }
                for r in self.results
            ]
        }
        
        report_path = Path("compatibility_report.json")
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2)
        
        print(f"\nDetailed report saved to: {report_path}")

def main():
    if len(sys.argv) > 1 and sys.argv[1] in ["-h", "--help"]:
        print("Usage: python test_matrix.py [max_workers]")
        print("Run comprehensive compatibility testing for QuDAG Protocol")
        sys.exit(0)
    
    max_workers = int(sys.argv[1]) if len(sys.argv) > 1 else 4
    
    config_path = Path(".compatibility.toml")
    if not config_path.exists():
        print(f"Configuration file not found: {config_path}")
        sys.exit(1)
    
    tester = CompatibilityTester(config_path)
    tester.run_tests(max_workers=max_workers)
    tester.generate_report()
    
    # Exit with error code if any tests failed
    failed_tests = sum(1 for r in tester.results if not r.success)
    sys.exit(1 if failed_tests > 0 else 0)

if __name__ == "__main__":
    main()