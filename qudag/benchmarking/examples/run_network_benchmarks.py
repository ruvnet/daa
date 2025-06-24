#!/usr/bin/env python3
"""
Example: Run specific network benchmarks and analyze results
"""

import sys
import os
import json

# Add parent directory to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from benchmarks.qudag.network_benchmarks import NetworkBenchmark

def main():
    """Run network benchmarks with custom configuration"""
    
    print("Running QuDAG Network Benchmarks")
    print("=" * 40)
    
    # Create benchmark instance
    benchmark = NetworkBenchmark()
    
    # Run specific benchmarks
    print("\n1. Connection Establishment")
    benchmark.benchmark_connection_establishment()
    
    print("\n2. Message Routing")
    benchmark.benchmark_message_routing()
    
    print("\n3. Onion Routing")
    benchmark.benchmark_onion_routing()
    
    print("\n4. Dark Addressing") 
    benchmark.benchmark_dark_addressing()
    
    # Generate report
    report = benchmark.generate_report()
    print("\n" + report)
    
    # Save results
    benchmark.save_results("network_benchmark_custom.json")
    
    # Analyze specific metrics
    print("\n" + "="*40)
    print("Performance Analysis")
    print("="*40)
    
    results = benchmark.results["benchmarks"]
    
    # Connection establishment analysis
    if "connection_establishment" in results:
        conn_results = results["connection_establishment"]
        if "single_connection" in conn_results:
            single = conn_results["single_connection"]
            print(f"\nSingle connection establishment:")
            print(f"  Average: {single['avg_time']*1000:.2f}ms")
            print(f"  P95: {single.get('p95_time', 0)*1000:.2f}ms")
            
    # Message routing analysis
    if "message_routing" in results:
        routing = results["message_routing"]
        print(f"\nMessage routing performance:")
        for size in ["small", "medium", "large"]:
            key = f"route_{size}_message"
            if key in routing:
                data = routing[key]
                print(f"  {size}: {data['avg_time']*1000:.2f}ms avg")
                
    # Dark addressing analysis
    if "dark_addressing" in results:
        dark = results["dark_addressing"]
        if "resolve_dark_address" in dark:
            resolve = dark["resolve_dark_address"]
            print(f"\nDark address resolution:")
            print(f"  Average: {resolve['avg_time']*1000:.2f}ms")
            print(f"  Success rate: {resolve['successful']/resolve['iterations']*100:.1f}%")


if __name__ == "__main__":
    main()