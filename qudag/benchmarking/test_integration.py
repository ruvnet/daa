#!/usr/bin/env python3
"""
QuDAG Benchmarking Integration Test

Quick test to verify the benchmarking tool is properly integrated.
"""

import sys
import os

# Add benchmarks to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'benchmarks/qudag'))

def test_imports():
    """Test that all benchmark modules can be imported"""
    print("Testing benchmark module imports...")
    
    modules = [
        'cli_benchmarks',
        'network_benchmarks', 
        'dag_benchmarks',
        'swarm_benchmarks'
    ]
    
    for module in modules:
        try:
            exec(f"import {module}")
            print(f"✓ {module} imported successfully")
        except ImportError as e:
            print(f"✗ Failed to import {module}: {e}")
            return False
            
    return True

def test_basic_benchmark():
    """Run a simple benchmark test"""
    print("\nRunning basic DAG benchmark test...")
    
    try:
        from dag_benchmarks import DAGBenchmark
        
        # Create benchmark instance
        benchmark = DAGBenchmark()
        
        # Run vertex operations benchmark
        benchmark.benchmark_vertex_operations()
        
        # Check results
        if "vertex_operations" in benchmark.results["benchmarks"]:
            results = benchmark.results["benchmarks"]["vertex_operations"]
            if "vertex_creation" in results:
                avg_time = results["vertex_creation"]["avg_time"]
                ops_per_sec = results["vertex_creation"]["ops_per_second"]
                print(f"✓ Vertex creation: {avg_time*1000:.3f}ms avg, {ops_per_sec:,.0f} ops/sec")
                return True
                
    except Exception as e:
        print(f"✗ Benchmark test failed: {e}")
        return False
        
    return False

def test_cli_integration():
    """Test CLI integration if available"""
    print("\nTesting QuDAG CLI integration...")
    
    if not os.path.exists("./claude-flow"):
        print("⚠ QuDAG CLI not found - skipping CLI tests")
        return True
        
    try:
        import subprocess
        
        # Test memory list command
        result = subprocess.run(
            ["./claude-flow", "memory", "list"],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        if result.returncode == 0:
            print("✓ CLI integration working")
            return True
        else:
            print(f"✗ CLI returned error: {result.stderr}")
            return False
            
    except Exception as e:
        print(f"✗ CLI test failed: {e}")
        return False

def main():
    """Run all integration tests"""
    print("QuDAG Benchmarking Integration Test")
    print("=" * 40)
    
    tests = [
        ("Module imports", test_imports),
        ("Basic benchmark", test_basic_benchmark),
        ("CLI integration", test_cli_integration)
    ]
    
    passed = 0
    for test_name, test_func in tests:
        print(f"\n[{test_name}]")
        if test_func():
            passed += 1
            
    print("\n" + "=" * 40)
    print(f"Tests passed: {passed}/{len(tests)}")
    
    if passed == len(tests):
        print("✅ All tests passed! Integration successful.")
        return 0
    else:
        print("❌ Some tests failed. Check the output above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())