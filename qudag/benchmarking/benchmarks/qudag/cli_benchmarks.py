#!/usr/bin/env python3
"""
QuDAG CLI Performance Benchmarks

This module provides comprehensive benchmarks for QuDAG CLI operations including:
- Command execution performance
- Argument parsing efficiency
- Output formatting overhead
- Memory/swarm coordination commands
"""

import subprocess
import time
import json
import statistics
import sys
import os
from typing import Dict, List, Any, Tuple
from datetime import datetime
import concurrent.futures
import psutil

class CLIBenchmark:
    """Benchmarking suite for QuDAG CLI operations"""
    
    def __init__(self, cli_path: str = "./claude-flow"):
        self.cli_path = cli_path
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "cli_path": cli_path,
            "benchmarks": {}
        }
        
    def _measure_command(self, command: List[str], iterations: int = 10) -> Dict[str, Any]:
        """Measure the performance of a CLI command"""
        timings = []
        memory_usage = []
        process = psutil.Process()
        
        for _ in range(iterations):
            start_time = time.perf_counter()
            start_memory = process.memory_info().rss
            
            try:
                result = subprocess.run(
                    [self.cli_path] + command,
                    capture_output=True,
                    text=True,
                    check=False
                )
                
                end_time = time.perf_counter()
                end_memory = process.memory_info().rss
                
                timings.append(end_time - start_time)
                memory_usage.append(end_memory - start_memory)
                
            except Exception as e:
                print(f"Error running command {command}: {e}")
                continue
                
        return {
            "command": " ".join(command),
            "iterations": iterations,
            "avg_time": statistics.mean(timings) if timings else 0,
            "min_time": min(timings) if timings else 0,
            "max_time": max(timings) if timings else 0,
            "std_dev": statistics.stdev(timings) if len(timings) > 1 else 0,
            "avg_memory": statistics.mean(memory_usage) if memory_usage else 0,
            "timings": timings
        }
    
    def benchmark_basic_commands(self):
        """Benchmark basic CLI commands"""
        commands = [
            (["--help"], "Help command"),
            (["status"], "Status check"),
            (["config", "show"], "Configuration display"),
            (["memory", "list"], "Memory listing"),
            (["agent", "list"], "Agent listing"),
        ]
        
        results = {}
        for cmd, description in commands:
            print(f"Benchmarking: {description}")
            results[description] = self._measure_command(cmd)
            
        self.results["benchmarks"]["basic_commands"] = results
        
    def benchmark_memory_operations(self):
        """Benchmark memory-intensive operations"""
        test_data = {
            "small": "x" * 100,
            "medium": "x" * 10000,
            "large": "x" * 100000
        }
        
        results = {}
        for size, data in test_data.items():
            # Store operation
            store_cmd = ["memory", "store", f"bench_test_{size}", data]
            results[f"store_{size}"] = self._measure_command(store_cmd, iterations=5)
            
            # Get operation
            get_cmd = ["memory", "get", f"bench_test_{size}"]
            results[f"get_{size}"] = self._measure_command(get_cmd, iterations=5)
            
        self.results["benchmarks"]["memory_operations"] = results
        
    def benchmark_agent_spawning(self):
        """Benchmark agent spawning performance"""
        agent_types = ["researcher", "coder", "analyst"]
        results = {}
        
        for agent_type in agent_types:
            cmd = ["agent", "spawn", agent_type, "--name", f"bench_{agent_type}"]
            results[f"spawn_{agent_type}"] = self._measure_command(cmd, iterations=3)
            
        self.results["benchmarks"]["agent_spawning"] = results
        
    def benchmark_parallel_operations(self):
        """Benchmark parallel command execution"""
        commands = [
            ["status"],
            ["memory", "list"],
            ["agent", "list"],
            ["config", "show"]
        ]
        
        def run_parallel_commands(num_parallel: int):
            start_time = time.perf_counter()
            
            with concurrent.futures.ProcessPoolExecutor(max_workers=num_parallel) as executor:
                futures = []
                for cmd in commands * num_parallel:
                    future = executor.submit(subprocess.run, 
                                           [self.cli_path] + cmd,
                                           capture_output=True)
                    futures.append(future)
                
                # Wait for all commands to complete
                concurrent.futures.wait(futures)
                
            return time.perf_counter() - start_time
            
        results = {}
        for parallel_count in [1, 2, 4, 8]:
            print(f"Testing {parallel_count} parallel operations")
            results[f"parallel_{parallel_count}"] = {
                "count": parallel_count,
                "total_time": run_parallel_commands(parallel_count),
                "commands_per_second": (len(commands) * parallel_count) / run_parallel_commands(parallel_count)
            }
            
        self.results["benchmarks"]["parallel_operations"] = results
        
    def benchmark_swarm_coordination(self):
        """Benchmark swarm coordination commands"""
        swarm_configs = [
            {
                "objective": "Test swarm operation",
                "strategy": "research",
                "mode": "centralized",
                "max_agents": 3
            },
            {
                "objective": "Test distributed swarm",
                "strategy": "development",
                "mode": "distributed",
                "max_agents": 5
            }
        ]
        
        results = {}
        for i, config in enumerate(swarm_configs):
            cmd = [
                "swarm",
                config["objective"],
                "--strategy", config["strategy"],
                "--mode", config["mode"],
                "--max-agents", str(config["max_agents"]),
                "--dry-run"  # Don't actually spawn agents
            ]
            
            results[f"swarm_config_{i}"] = {
                "config": config,
                "performance": self._measure_command(cmd, iterations=3)
            }
            
        self.results["benchmarks"]["swarm_coordination"] = results
        
    def generate_report(self) -> str:
        """Generate a comprehensive benchmark report"""
        report = [
            "# QuDAG CLI Benchmark Report",
            f"Generated: {self.results['timestamp']}",
            f"CLI Path: {self.results['cli_path']}",
            "\n## Summary",
            ""
        ]
        
        # Analyze results and provide insights
        for category, benchmarks in self.results["benchmarks"].items():
            report.append(f"\n### {category.replace('_', ' ').title()}")
            
            if isinstance(benchmarks, dict):
                for name, data in benchmarks.items():
                    if isinstance(data, dict) and "avg_time" in data:
                        report.append(f"- **{name}**: {data['avg_time']:.3f}s (±{data['std_dev']:.3f}s)")
                    else:
                        report.append(f"- **{name}**: {json.dumps(data, indent=2)}")
                        
        return "\n".join(report)
        
    def save_results(self, output_path: str = "cli_benchmark_results.json"):
        """Save benchmark results to a file"""
        with open(output_path, "w") as f:
            json.dump(self.results, f, indent=2)
            
    def run_all_benchmarks(self):
        """Run all benchmark suites"""
        print("Starting QuDAG CLI benchmarks...")
        
        benchmarks = [
            ("Basic Commands", self.benchmark_basic_commands),
            ("Memory Operations", self.benchmark_memory_operations),
            ("Agent Spawning", self.benchmark_agent_spawning),
            ("Parallel Operations", self.benchmark_parallel_operations),
            ("Swarm Coordination", self.benchmark_swarm_coordination)
        ]
        
        for name, benchmark_func in benchmarks:
            print(f"\n[{name}]")
            try:
                benchmark_func()
            except Exception as e:
                print(f"Error in {name}: {e}")
                self.results["benchmarks"][name.lower().replace(" ", "_")] = {
                    "error": str(e)
                }
                
        print("\nBenchmarks completed!")


def main():
    """Main entry point for CLI benchmarking"""
    # Check if QuDAG CLI exists
    cli_path = "./claude-flow"
    if not os.path.exists(cli_path):
        print(f"Error: QuDAG CLI not found at {cli_path}")
        sys.exit(1)
        
    # Create and run benchmarks
    benchmark = CLIBenchmark(cli_path)
    benchmark.run_all_benchmarks()
    
    # Generate and save report
    report = benchmark.generate_report()
    print("\n" + report)
    
    # Save detailed results
    benchmark.save_results()
    print(f"\nDetailed results saved to: cli_benchmark_results.json")


if __name__ == "__main__":
    main()