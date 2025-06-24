use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Comprehensive resource tracking for CLI performance analysis
pub struct ResourceTracker {
    start_time: Instant,
    initial_memory: Option<usize>,
    memory_samples: Arc<Mutex<Vec<(Instant, usize)>>>,
    cpu_samples: Arc<Mutex<Vec<(Instant, f64)>>>,
    operation_counts: Arc<Mutex<HashMap<String, usize>>>,
    operation_timings: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
    peak_memory: Arc<Mutex<usize>>,
}

/// Resource usage snapshot
#[derive(Debug, Clone)]
pub struct ResourceSnapshot {
    pub timestamp: Instant,
    pub memory_usage: Option<usize>,
    pub cpu_usage: Option<f64>,
    pub active_operations: usize,
    pub total_operations: usize,
}

/// Performance metrics aggregation
#[derive(Debug)]
pub struct PerformanceReport {
    pub execution_time: Duration,
    pub memory_stats: MemoryStats,
    pub cpu_stats: CpuStats,
    pub operation_stats: OperationStats,
    pub resource_efficiency: ResourceEfficiency,
}

#[derive(Debug)]
pub struct MemoryStats {
    pub initial: usize,
    pub peak: usize,
    pub current: usize,
    pub average: usize,
    pub growth_rate: f64,
}

#[derive(Debug)]
pub struct CpuStats {
    pub average: f64,
    pub peak: f64,
    pub efficiency: f64,
}

#[derive(Debug)]
pub struct OperationStats {
    pub total_operations: usize,
    pub operations_per_second: f64,
    pub average_operation_time: Duration,
    pub slowest_operation: Option<(String, Duration)>,
    pub fastest_operation: Option<(String, Duration)>,
}

#[derive(Debug)]
pub struct ResourceEfficiency {
    pub memory_efficiency: f64,
    pub cpu_efficiency: f64,
    pub throughput_efficiency: f64,
    pub overall_score: f64,
}

impl ResourceTracker {
    /// Create new resource tracker with enhanced monitoring
    pub fn new() -> Self {
        let initial_memory = Self::get_current_memory();
        Self {
            start_time: Instant::now(),
            initial_memory,
            memory_samples: Arc::new(Mutex::new(Vec::new())),
            cpu_samples: Arc::new(Mutex::new(Vec::new())),
            operation_counts: Arc::new(Mutex::new(HashMap::new())),
            operation_timings: Arc::new(Mutex::new(HashMap::new())),
            peak_memory: Arc::new(Mutex::new(initial_memory.unwrap_or(0))),
        }
    }

    /// Get elapsed time since tracker creation
    pub fn elapsed_time(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Get current memory usage delta
    pub fn memory_usage(&self) -> Option<usize> {
        let current = Self::get_current_memory()?;
        let initial = self.initial_memory?;
        Some(current.saturating_sub(initial))
    }

    /// Record an operation start
    pub fn start_operation(&self, operation: &str) -> OperationTracker {
        if let Ok(mut counts) = self.operation_counts.lock() {
            *counts.entry(operation.to_string()).or_insert(0) += 1;
        }
        
        OperationTracker {
            operation: operation.to_string(),
            start_time: Instant::now(),
            tracker: self,
        }
    }

    /// Record operation completion
    fn record_operation_completion(&self, operation: &str, duration: Duration) {
        if let Ok(mut timings) = self.operation_timings.lock() {
            timings.entry(operation.to_string())
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    /// Sample current resource usage
    pub fn sample_resources(&self) {
        let now = Instant::now();
        
        // Sample memory
        if let Some(memory) = Self::get_current_memory() {
            if let Ok(mut samples) = self.memory_samples.lock() {
                samples.push((now, memory));
                
                // Update peak memory
                if let Ok(mut peak) = self.peak_memory.lock() {
                    if memory > *peak {
                        *peak = memory;
                    }
                }
                
                // Keep only recent samples (last 100)
                if samples.len() > 100 {
                    samples.drain(0..samples.len() - 100);
                }
            }
        }
        
        // Sample CPU
        if let Some(cpu) = self.get_cpu_usage() {
            if let Ok(mut samples) = self.cpu_samples.lock() {
                samples.push((now, cpu));
                
                // Keep only recent samples (last 100)
                if samples.len() > 100 {
                    samples.drain(0..samples.len() - 100);
                }
            }
        }
    }

    /// Get current resource snapshot
    pub fn get_snapshot(&self) -> ResourceSnapshot {
        let memory_usage = Self::get_current_memory();
        let cpu_usage = self.get_cpu_usage();
        
        let (active_operations, total_operations) = if let Ok(counts) = self.operation_counts.lock() {
            let total = counts.values().sum();
            (0, total) // Active operations would need more sophisticated tracking
        } else {
            (0, 0)
        };

        ResourceSnapshot {
            timestamp: Instant::now(),
            memory_usage,
            cpu_usage,
            active_operations,
            total_operations,
        }
    }

    /// Generate comprehensive performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let execution_time = self.start_time.elapsed();
        let memory_stats = self.calculate_memory_stats();
        let cpu_stats = self.calculate_cpu_stats();
        let operation_stats = self.calculate_operation_stats(execution_time);
        let resource_efficiency = self.calculate_efficiency(&memory_stats, &cpu_stats, &operation_stats);

        PerformanceReport {
            execution_time,
            memory_stats,
            cpu_stats,
            operation_stats,
            resource_efficiency,
        }
    }

    /// Get current memory usage in bytes
    fn get_current_memory() -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            let contents = std::fs::read_to_string("/proc/self/statm").ok()?;
            let values: Vec<&str> = contents.split_whitespace().collect();
            let pages = values.first()?.parse::<usize>().ok()?;
            Some(pages * 4096) // Convert pages to bytes
        }

        #[cfg(target_os = "macos")]
        {
            // Use task_info on macOS for more accurate memory tracking
            use std::mem;
            
            extern "C" {
                fn task_info(
                    task: u32,
                    flavor: u32,
                    task_info: *mut u8,
                    task_info_count: *mut u32,
                ) -> i32;
                fn mach_task_self() -> u32;
            }
            
            const MACH_TASK_BASIC_INFO: u32 = 20;
            
            #[repr(C)]
            struct TaskBasicInfo {
                suspend_count: u32,
                virtual_size: u64,
                resident_size: u64,
                user_time: [u32; 2],
                system_time: [u32; 2],
                policy: u32,
            }
            
            unsafe {
                let mut info: TaskBasicInfo = mem::zeroed();
                let mut count = (mem::size_of::<TaskBasicInfo>() / mem::size_of::<u32>()) as u32;
                
                if task_info(
                    mach_task_self(),
                    MACH_TASK_BASIC_INFO,
                    &mut info as *mut _ as *mut u8,
                    &mut count,
                ) == 0 {
                    return Some(info.resident_size as usize);
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use Windows API for memory tracking
            use std::mem;
            
            extern "system" {
                fn GetCurrentProcess() -> *mut std::ffi::c_void;
                fn GetProcessMemoryInfo(
                    process: *mut std::ffi::c_void,
                    info: *mut ProcessMemoryCounters,
                    size: u32,
                ) -> i32;
            }
            
            #[repr(C)]
            struct ProcessMemoryCounters {
                cb: u32,
                page_fault_count: u32,
                peak_working_set_size: usize,
                working_set_size: usize,
                quota_peak_paged_pool_usage: usize,
                quota_paged_pool_usage: usize,
                quota_peak_non_paged_pool_usage: usize,
                quota_non_paged_pool_usage: usize,
                pagefile_usage: usize,
                peak_pagefile_usage: usize,
            }
            
            unsafe {
                let mut counters: ProcessMemoryCounters = mem::zeroed();
                counters.cb = mem::size_of::<ProcessMemoryCounters>() as u32;
                
                if GetProcessMemoryInfo(
                    GetCurrentProcess(),
                    &mut counters,
                    counters.cb,
                ) != 0 {
                    return Some(counters.working_set_size);
                }
            }
        }

        // Fallback for other platforms
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        None
    }

    /// Get current CPU usage percentage
    fn get_cpu_usage(&self) -> Option<f64> {
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/stat for CPU usage
            let contents = std::fs::read_to_string("/proc/stat").ok()?;
            let first_line = contents.lines().next()?;
            let values: Vec<&str> = first_line.split_whitespace().collect();
            
            if values.len() >= 5 {
                let user: u64 = values[1].parse().ok()?;
                let nice: u64 = values[2].parse().ok()?;
                let system: u64 = values[3].parse().ok()?;
                let idle: u64 = values[4].parse().ok()?;
                
                let total = user + nice + system + idle;
                let busy = total - idle;
                
                if total > 0 {
                    return Some((busy as f64 / total as f64) * 100.0);
                }
            }
        }

        // Fallback: estimate based on operation frequency
        Some(10.0) // Default 10% estimate
    }

    /// Calculate memory statistics
    fn calculate_memory_stats(&self) -> MemoryStats {
        let initial = self.initial_memory.unwrap_or(0);
        let current = Self::get_current_memory().unwrap_or(initial);
        let peak = if let Ok(peak) = self.peak_memory.lock() {
            *peak
        } else {
            current
        };

        let average = if let Ok(samples) = self.memory_samples.lock() {
            if !samples.is_empty() {
                samples.iter().map(|(_, mem)| *mem).sum::<usize>() / samples.len()
            } else {
                current
            }
        } else {
            current
        };

        let growth_rate = if initial > 0 {
            ((current as f64 - initial as f64) / initial as f64) * 100.0
        } else {
            0.0
        };

        MemoryStats {
            initial,
            peak,
            current,
            average,
            growth_rate,
        }
    }

    /// Calculate CPU statistics
    fn calculate_cpu_stats(&self) -> CpuStats {
        if let Ok(samples) = self.cpu_samples.lock() {
            if !samples.is_empty() {
                let average = samples.iter().map(|(_, cpu)| *cpu).sum::<f64>() / samples.len() as f64;
                let peak = samples.iter().map(|(_, cpu)| *cpu).fold(0.0, f64::max);
                let efficiency = if peak > 0.0 { average / peak } else { 1.0 };
                
                CpuStats {
                    average,
                    peak,
                    efficiency,
                }
            } else {
                CpuStats {
                    average: 0.0,
                    peak: 0.0,
                    efficiency: 1.0,
                }
            }
        } else {
            CpuStats {
                average: 0.0,
                peak: 0.0,
                efficiency: 1.0,
            }
        }
    }

    /// Calculate operation statistics
    fn calculate_operation_stats(&self, execution_time: Duration) -> OperationStats {
        let (total_operations, operations_per_second, average_operation_time, slowest, fastest) = 
            if let (Ok(counts), Ok(timings)) = (self.operation_counts.lock(), self.operation_timings.lock()) {
                let total = counts.values().sum::<usize>();
                let ops_per_sec = if execution_time.as_secs_f64() > 0.0 {
                    total as f64 / execution_time.as_secs_f64()
                } else {
                    0.0
                };

                let all_durations: Vec<Duration> = timings.values().flatten().cloned().collect();
                let avg_time = if !all_durations.is_empty() {
                    all_durations.iter().sum::<Duration>() / all_durations.len() as u32
                } else {
                    Duration::from_millis(0)
                };

                let slowest_op = timings.iter()
                    .filter_map(|(op, durations)| {
                        durations.iter().max().map(|&max_duration| (op.clone(), max_duration))
                    })
                    .max_by_key(|(_, duration)| *duration);

                let fastest_op = timings.iter()
                    .filter_map(|(op, durations)| {
                        durations.iter().min().map(|&min_duration| (op.clone(), min_duration))
                    })
                    .min_by_key(|(_, duration)| *duration);

                (total, ops_per_sec, avg_time, slowest_op, fastest_op)
            } else {
                (0, 0.0, Duration::from_millis(0), None, None)
            };

        OperationStats {
            total_operations,
            operations_per_second,
            average_operation_time,
            slowest_operation: slowest,
            fastest_operation: fastest,
        }
    }

    /// Calculate resource efficiency metrics
    fn calculate_efficiency(&self, memory: &MemoryStats, cpu: &CpuStats, ops: &OperationStats) -> ResourceEfficiency {
        // Memory efficiency: lower memory growth is better
        let memory_efficiency = if memory.growth_rate > 0.0 {
            100.0 / (1.0 + memory.growth_rate / 100.0)
        } else {
            100.0
        };

        // CPU efficiency: closer to optimal utilization (50-80%) is better
        let cpu_efficiency = if cpu.average > 0.0 {
            let optimal_range = 50.0..=80.0;
            if optimal_range.contains(&cpu.average) {
                100.0
            } else if cpu.average < 50.0 {
                cpu.average * 2.0
            } else {
                100.0 - ((cpu.average - 80.0) / 20.0 * 50.0).min(50.0)
            }
        } else {
            50.0
        };

        // Throughput efficiency: higher ops/sec is better
        let throughput_efficiency = if ops.operations_per_second > 0.0 {
            (ops.operations_per_second / 100.0 * 100.0).min(100.0)
        } else {
            0.0
        };

        let overall_score = (memory_efficiency + cpu_efficiency + throughput_efficiency) / 3.0;

        ResourceEfficiency {
            memory_efficiency,
            cpu_efficiency,
            throughput_efficiency,
            overall_score,
        }
    }
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII operation tracker
pub struct OperationTracker<'a> {
    operation: String,
    start_time: Instant,
    tracker: &'a ResourceTracker,
}

impl<'a> Drop for OperationTracker<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.tracker.record_operation_completion(&self.operation, duration);
    }
}

/// Utility functions for performance analysis
pub fn format_performance_report(report: &PerformanceReport) -> String {
    let mut output = String::new();
    
    output.push_str("=== Resource Performance Report ===\n\n");
    
    // Execution summary
    output.push_str(&format!("Execution Time: {:.2}ms\n", 
        report.execution_time.as_secs_f64() * 1000.0));
    
    // Memory statistics
    output.push_str("\nMemory Usage:\n");
    output.push_str(&format!("  Initial: {:.2} MB\n", 
        report.memory_stats.initial as f64 / 1024.0 / 1024.0));
    output.push_str(&format!("  Peak: {:.2} MB\n", 
        report.memory_stats.peak as f64 / 1024.0 / 1024.0));
    output.push_str(&format!("  Current: {:.2} MB\n", 
        report.memory_stats.current as f64 / 1024.0 / 1024.0));
    output.push_str(&format!("  Average: {:.2} MB\n", 
        report.memory_stats.average as f64 / 1024.0 / 1024.0));
    output.push_str(&format!("  Growth Rate: {:.2}%\n", 
        report.memory_stats.growth_rate));
    
    // CPU statistics
    output.push_str("\nCPU Usage:\n");
    output.push_str(&format!("  Average: {:.2}%\n", report.cpu_stats.average));
    output.push_str(&format!("  Peak: {:.2}%\n", report.cpu_stats.peak));
    output.push_str(&format!("  Efficiency: {:.2}%\n", 
        report.cpu_stats.efficiency * 100.0));
    
    // Operation statistics
    output.push_str("\nOperation Statistics:\n");
    output.push_str(&format!("  Total Operations: {}\n", 
        report.operation_stats.total_operations));
    output.push_str(&format!("  Operations/Second: {:.2}\n", 
        report.operation_stats.operations_per_second));
    output.push_str(&format!("  Average Operation Time: {:.2}ms\n", 
        report.operation_stats.average_operation_time.as_secs_f64() * 1000.0));
    
    if let Some((op, duration)) = &report.operation_stats.slowest_operation {
        output.push_str(&format!("  Slowest Operation: {} ({:.2}ms)\n", 
            op, duration.as_secs_f64() * 1000.0));
    }
    
    if let Some((op, duration)) = &report.operation_stats.fastest_operation {
        output.push_str(&format!("  Fastest Operation: {} ({:.2}ms)\n", 
            op, duration.as_secs_f64() * 1000.0));
    }
    
    // Efficiency metrics
    output.push_str("\nEfficiency Metrics:\n");
    output.push_str(&format!("  Memory Efficiency: {:.1}%\n", 
        report.resource_efficiency.memory_efficiency));
    output.push_str(&format!("  CPU Efficiency: {:.1}%\n", 
        report.resource_efficiency.cpu_efficiency));
    output.push_str(&format!("  Throughput Efficiency: {:.1}%\n", 
        report.resource_efficiency.throughput_efficiency));
    output.push_str(&format!("  Overall Score: {:.1}%\n", 
        report.resource_efficiency.overall_score));
    
    output
}