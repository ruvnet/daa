"""
CPU metric collector for benchmarks.
Tracks CPU usage during benchmark execution.
"""
import psutil
import time
from typing import Dict, Any


class CPUMetric:
    """Collects CPU usage metrics."""
    
    def __init__(self):
        """Initialize CPU metric collector."""
        self.process = psutil.Process()
        self.last_cpu_times = None
        self.last_time = None
    
    def collect(self) -> Dict[str, Any]:
        """
        Collect current CPU metrics.
        
        Returns:
            Dictionary containing CPU metrics:
            - percent: CPU usage percentage
            - user_time: User mode CPU time
            - system_time: System mode CPU time
            - threads: Number of threads
            - cpu_num: CPU core number (if available)
        """
        try:
            # Get CPU percentage (this might block briefly on first call)
            cpu_percent = self.process.cpu_percent(interval=0.01)
            
            # Get CPU times
            cpu_times = self.process.cpu_times()
            
            # Get thread count
            num_threads = self.process.num_threads()
            
            # Try to get CPU affinity (which CPU cores the process can run on)
            try:
                cpu_affinity = len(self.process.cpu_affinity())
            except:
                cpu_affinity = psutil.cpu_count()
            
            result = {
                "percent": cpu_percent,
                "user_time": cpu_times.user,
                "system_time": cpu_times.system,
                "threads": num_threads,
                "cpu_count": cpu_affinity
            }
            
            # Calculate incremental CPU usage if we have previous data
            current_time = time.time()
            if self.last_cpu_times and self.last_time:
                time_delta = current_time - self.last_time
                if time_delta > 0:
                    user_delta = cpu_times.user - self.last_cpu_times.user
                    system_delta = cpu_times.system - self.last_cpu_times.system
                    
                    result["user_percent"] = (user_delta / time_delta) * 100
                    result["system_percent"] = (system_delta / time_delta) * 100
            
            # Update last values
            self.last_cpu_times = cpu_times
            self.last_time = current_time
            
            return result
            
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            # Process might have ended or we lack permissions
            return {}
    
    def reset(self):
        """Reset metric collector state."""
        self.last_cpu_times = None
        self.last_time = None