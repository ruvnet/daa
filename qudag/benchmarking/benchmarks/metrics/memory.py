"""
Memory metric collector for benchmarks.
Tracks memory usage during benchmark execution.
"""
import psutil
from typing import Dict, Any


class MemoryMetric:
    """Collects memory usage metrics."""
    
    def __init__(self):
        """Initialize memory metric collector."""
        self.process = psutil.Process()
        self.initial_memory = None
        self.peak_memory = 0
    
    def collect(self) -> Dict[str, Any]:
        """
        Collect current memory metrics.
        
        Returns:
            Dictionary containing memory metrics:
            - rss: Resident Set Size in bytes
            - vms: Virtual Memory Size in bytes  
            - percent: Memory usage percentage
            - available: Available system memory
        """
        try:
            # Get process memory info
            mem_info = self.process.memory_info()
            
            # Get system memory info
            sys_mem = psutil.virtual_memory()
            
            current_rss = mem_info.rss
            
            # Track initial memory if not set
            if self.initial_memory is None:
                self.initial_memory = current_rss
            
            # Track peak memory
            if current_rss > self.peak_memory:
                self.peak_memory = current_rss
            
            return {
                "rss": current_rss,
                "vms": mem_info.vms,
                "percent": self.process.memory_percent(),
                "available": sys_mem.available,
                "initial": self.initial_memory,
                "peak": self.peak_memory
            }
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            # Process might have ended or we lack permissions
            return {}
    
    def reset(self):
        """Reset metric collector state."""
        self.initial_memory = None
        self.peak_memory = 0