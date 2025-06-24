use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Performance metrics for CLI operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub startup_time: Duration,
    pub command_execution_times: HashMap<String, Duration>,
    pub memory_usage: MemoryUsage,
    pub async_task_metrics: AsyncTaskMetrics,
    pub error_counts: HashMap<String, usize>,
}

/// Memory usage tracking
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub initial_memory: usize,
    pub peak_memory: usize,
    pub current_memory: usize,
    pub allocations: usize,
    pub deallocations: usize,
}

/// Async task performance metrics
#[derive(Debug, Clone)]
pub struct AsyncTaskMetrics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_task_duration: Duration,
    pub max_task_duration: Duration,
    pub min_task_duration: Duration,
    pub pending_tasks: usize,
}

/// Performance tracker for CLI operations
pub struct PerformanceTracker {
    start_time: Instant,
    command_times: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
    memory_tracker: Arc<Mutex<MemoryUsage>>,
    async_tracker: Arc<Mutex<AsyncTaskMetrics>>,
    error_tracker: Arc<Mutex<HashMap<String, usize>>>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new() -> Self {
        let initial_memory = Self::get_memory_usage();

        Self {
            start_time: Instant::now(),
            command_times: Arc::new(Mutex::new(HashMap::new())),
            memory_tracker: Arc::new(Mutex::new(MemoryUsage {
                initial_memory,
                peak_memory: initial_memory,
                current_memory: initial_memory,
                allocations: 0,
                deallocations: 0,
            })),
            async_tracker: Arc::new(Mutex::new(AsyncTaskMetrics {
                total_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_task_duration: Duration::from_millis(0),
                max_task_duration: Duration::from_millis(0),
                min_task_duration: Duration::from_secs(u64::MAX),
                pending_tasks: 0,
            })),
            error_tracker: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start tracking a command execution
    pub fn start_command(self: &Arc<Self>, command: &str) -> CommandTracker {
        let tracker = Arc::clone(self);

        tokio::spawn({
            let async_tracker = Arc::clone(&self.async_tracker);
            async move {
                let mut tracker = async_tracker.lock().await;
                tracker.total_tasks += 1;
                tracker.pending_tasks += 1;
            }
        });

        CommandTracker {
            command: command.to_string(),
            start_time: Instant::now(),
            tracker,
        }
    }

    /// Record command completion
    pub async fn record_command_completion(
        &self,
        command: &str,
        duration: Duration,
        success: bool,
    ) {
        // Record command timing
        let mut command_times = self.command_times.lock().await;
        command_times
            .entry(command.to_string())
            .or_insert_with(Vec::new)
            .push(duration);

        // Update async metrics
        let mut async_tracker = self.async_tracker.lock().await;
        async_tracker.pending_tasks = async_tracker.pending_tasks.saturating_sub(1);

        if success {
            async_tracker.completed_tasks += 1;
        } else {
            async_tracker.failed_tasks += 1;
        }

        // Update duration metrics
        if duration > async_tracker.max_task_duration {
            async_tracker.max_task_duration = duration;
        }
        if duration < async_tracker.min_task_duration {
            async_tracker.min_task_duration = duration;
        }

        // Recalculate average
        let total_completed = async_tracker.completed_tasks + async_tracker.failed_tasks;
        if total_completed > 0 {
            let total_duration = command_times.values().flatten().sum::<Duration>();
            async_tracker.average_task_duration = total_duration / total_completed as u32;
        }
    }

    /// Record an error
    pub async fn record_error(&self, error_type: &str) {
        let mut error_tracker = self.error_tracker.lock().await;
        *error_tracker.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self) {
        let current_memory = Self::get_memory_usage();
        let mut memory_tracker = self.memory_tracker.lock().await;

        memory_tracker.current_memory = current_memory;
        if current_memory > memory_tracker.peak_memory {
            memory_tracker.peak_memory = current_memory;
        }
        memory_tracker.allocations += 1;
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let command_times = self.command_times.lock().await;
        let memory_usage = self.memory_tracker.lock().await;
        let async_metrics = self.async_tracker.lock().await;
        let error_counts = self.error_tracker.lock().await;

        // Calculate average execution times per command
        let mut command_execution_times = HashMap::new();
        for (command, times) in command_times.iter() {
            let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
            command_execution_times.insert(command.clone(), avg_time);
        }

        PerformanceMetrics {
            startup_time: self.start_time.elapsed(),
            command_execution_times,
            memory_usage: memory_usage.clone(),
            async_task_metrics: async_metrics.clone(),
            error_counts: error_counts.clone(),
        }
    }

    /// Get memory usage in bytes
    fn get_memory_usage() -> usize {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
                if let Some(first) = contents.split_whitespace().next() {
                    if let Ok(pages) = first.parse::<usize>() {
                        return pages * 4096; // Convert pages to bytes
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Use task_info on macOS
            use std::mem;
            use std::ptr;

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
                ) == 0
                {
                    return info.resident_size as usize;
                }
            }
        }

        // Fallback: estimate based on allocator stats (rough estimate)
        8 * 1024 * 1024 // 8MB default estimate
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> String {
        let metrics = self.get_metrics().await;
        let mut report = String::new();

        report.push_str("=== CLI Performance Report ===\n\n");

        // Startup performance
        report.push_str(&format!(
            "Startup Time: {:.2}ms\n",
            metrics.startup_time.as_secs_f64() * 1000.0
        ));

        // Command execution times
        report.push_str("\nCommand Execution Times:\n");
        for (command, duration) in &metrics.command_execution_times {
            report.push_str(&format!(
                "  {}: {:.2}ms\n",
                command,
                duration.as_secs_f64() * 1000.0
            ));
        }

        // Memory usage
        report.push_str("\nMemory Usage:\n");
        report.push_str(&format!(
            "  Initial: {:.2} MB\n",
            metrics.memory_usage.initial_memory as f64 / 1024.0 / 1024.0
        ));
        report.push_str(&format!(
            "  Peak: {:.2} MB\n",
            metrics.memory_usage.peak_memory as f64 / 1024.0 / 1024.0
        ));
        report.push_str(&format!(
            "  Current: {:.2} MB\n",
            metrics.memory_usage.current_memory as f64 / 1024.0 / 1024.0
        ));

        // Async task metrics
        report.push_str("\nAsync Task Performance:\n");
        report.push_str(&format!(
            "  Total Tasks: {}\n",
            metrics.async_task_metrics.total_tasks
        ));
        report.push_str(&format!(
            "  Completed: {}\n",
            metrics.async_task_metrics.completed_tasks
        ));
        report.push_str(&format!(
            "  Failed: {}\n",
            metrics.async_task_metrics.failed_tasks
        ));
        report.push_str(&format!(
            "  Pending: {}\n",
            metrics.async_task_metrics.pending_tasks
        ));
        report.push_str(&format!(
            "  Average Duration: {:.2}ms\n",
            metrics
                .async_task_metrics
                .average_task_duration
                .as_secs_f64()
                * 1000.0
        ));
        report.push_str(&format!(
            "  Max Duration: {:.2}ms\n",
            metrics.async_task_metrics.max_task_duration.as_secs_f64() * 1000.0
        ));
        report.push_str(&format!(
            "  Min Duration: {:.2}ms\n",
            metrics.async_task_metrics.min_task_duration.as_secs_f64() * 1000.0
        ));

        // Error statistics
        if !metrics.error_counts.is_empty() {
            report.push_str("\nError Statistics:\n");
            for (error_type, count) in &metrics.error_counts {
                report.push_str(&format!("  {}: {}\n", error_type, count));
            }
        }

        // Performance recommendations
        report.push_str("\nPerformance Recommendations:\n");

        if metrics.startup_time > Duration::from_millis(500) {
            report.push_str("  - Startup time is high (>500ms). Consider lazy loading or reducing dependencies.\n");
        }

        if let Some(max_cmd_time) = metrics.command_execution_times.values().max() {
            if *max_cmd_time > Duration::from_millis(100) {
                report.push_str("  - Some commands take >100ms. Consider async optimization.\n");
            }
        }

        let memory_growth = metrics.memory_usage.peak_memory - metrics.memory_usage.initial_memory;
        if memory_growth > 10 * 1024 * 1024 {
            // 10MB
            report.push_str("  - High memory growth detected. Check for memory leaks.\n");
        }

        if metrics.async_task_metrics.failed_tasks > 0 {
            let failure_rate = metrics.async_task_metrics.failed_tasks as f64
                / metrics.async_task_metrics.total_tasks as f64;
            if failure_rate > 0.1 {
                // 10% failure rate
                report.push_str("  - High async task failure rate. Improve error handling.\n");
            }
        }

        report
    }
}

impl Clone for PerformanceTracker {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            command_times: Arc::clone(&self.command_times),
            memory_tracker: Arc::clone(&self.memory_tracker),
            async_tracker: Arc::clone(&self.async_tracker),
            error_tracker: Arc::clone(&self.error_tracker),
        }
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Command execution tracker
pub struct CommandTracker {
    command: String,
    start_time: Instant,
    tracker: Arc<PerformanceTracker>,
}

impl CommandTracker {
    /// Complete the command tracking
    pub async fn complete(self, success: bool) {
        let duration = self.start_time.elapsed();

        self.tracker
            .record_command_completion(&self.command, duration, success)
            .await;
        self.tracker.update_memory_usage().await;

        // Log performance for monitoring
        if duration > Duration::from_millis(100) {
            warn!(
                "Command '{}' took {:.2}ms",
                self.command,
                duration.as_secs_f64() * 1000.0
            );
        } else {
            info!(
                "Command '{}' completed in {:.2}ms",
                self.command,
                duration.as_secs_f64() * 1000.0
            );
        }
    }

    /// Complete with error
    pub async fn complete_with_error(self, error_type: &str) {
        self.tracker.record_error(error_type).await;
        self.complete(false).await;
    }
}

/// Async task optimization utilities
pub struct AsyncOptimizer;

impl AsyncOptimizer {
    /// Optimize async task execution with batching
    pub async fn batch_execute<F, T>(
        mut tasks: Vec<F>,
    ) -> Vec<Result<T, Box<dyn std::error::Error + Send + Sync>>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
        T: Send + 'static,
    {
        let batch_size = std::cmp::min(tasks.len(), 10); // Limit concurrent tasks
        let mut results = Vec::with_capacity(tasks.len());

        while !tasks.is_empty() {
            let chunk_len = batch_size.min(tasks.len());
            let chunk: Vec<_> = tasks.drain(..chunk_len).collect();
            let chunk_results = futures::future::join_all(chunk).await;
            results.extend(chunk_results);
        }

        results
    }

    /// Execute with timeout and retry logic
    pub async fn execute_with_retry<F, T>(
        task: F,
        max_retries: usize,
        timeout: Duration,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = Result<T, Box<dyn std::error::Error + Send + Sync>>,
                        > + Send,
                >,
            > + Send
            + Sync,
    {
        for attempt in 0..=max_retries {
            match tokio::time::timeout(timeout, task()).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => {
                    if attempt == max_retries {
                        return Err(e);
                    }
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * (1 << attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(_) => {
                    if attempt == max_retries {
                        return Err("Task timed out".into());
                    }
                }
            }
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_tracker() {
        let tracker = Arc::new(PerformanceTracker::new());

        // Test command tracking
        let cmd_tracker = tracker.start_command("test_command");
        tokio::time::sleep(Duration::from_millis(10)).await;
        cmd_tracker.complete(true).await;

        let metrics = tracker.get_metrics().await;
        assert!(metrics.command_execution_times.contains_key("test_command"));
        assert_eq!(metrics.async_task_metrics.completed_tasks, 1);
    }

    #[tokio::test]
    async fn test_async_optimizer() {
        use std::pin::Pin;
        let tasks: Vec<
            Pin<
                Box<
                    dyn std::future::Future<
                            Output = Result<i32, Box<dyn std::error::Error + Send + Sync>>,
                        > + Send,
                >,
            >,
        > = vec![
            Box::pin(async { Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(1) }),
            Box::pin(async { Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(2) }),
            Box::pin(async { Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(3) }),
        ];

        let results = AsyncOptimizer::batch_execute(tasks).await;
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
