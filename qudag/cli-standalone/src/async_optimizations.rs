use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, warn};

/// Async operation optimizer with timeouts and retries
pub struct AsyncOptimizer {
    max_concurrent: Arc<Semaphore>,
    default_timeout: Duration,
    retry_config: RetryConfig,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

impl AsyncOptimizer {
    /// Create new async optimizer
    pub fn new(max_concurrent: usize, default_timeout: Duration) -> Self {
        Self {
            max_concurrent: Arc::new(Semaphore::new(max_concurrent)),
            default_timeout,
            retry_config: RetryConfig::default(),
        }
    }

    /// Execute future with optimization
    pub async fn execute<F, T>(&self, future: F) -> Result<T, AsyncError>
    where
        F: Future<Output = Result<T, AsyncError>>,
    {
        self.execute_with_timeout(future, self.default_timeout)
            .await
    }

    /// Execute future with custom timeout
    pub async fn execute_with_timeout<F, T>(
        &self,
        future: F,
        timeout_duration: Duration,
    ) -> Result<T, AsyncError>
    where
        F: Future<Output = Result<T, AsyncError>>,
    {
        let _permit = self
            .max_concurrent
            .acquire()
            .await
            .map_err(|_| AsyncError::ResourceExhausted)?;

        let start_time = Instant::now();

        match timeout(timeout_duration, future).await {
            Ok(result) => {
                let duration = start_time.elapsed();
                if duration > Duration::from_millis(100) {
                    debug!(
                        "Async operation took {:.2}ms",
                        duration.as_secs_f64() * 1000.0
                    );
                }
                result
            }
            Err(_) => {
                warn!(
                    "Async operation timed out after {:.2}s",
                    timeout_duration.as_secs_f64()
                );
                Err(AsyncError::Timeout)
            }
        }
    }

    /// Execute with retry logic
    pub async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T, AsyncError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, AsyncError>>,
    {
        let mut attempt = 0;
        let mut delay = self.retry_config.base_delay;

        loop {
            attempt += 1;

            match self.execute(operation()).await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.retry_config.max_attempts {
                        error!("Operation failed after {} attempts: {:?}", attempt, e);
                        return Err(e);
                    }

                    if !e.is_retryable() {
                        error!("Non-retryable error: {:?}", e);
                        return Err(e);
                    }

                    warn!(
                        "Attempt {} failed, retrying in {:.2}ms: {:?}",
                        attempt,
                        delay.as_secs_f64() * 1000.0,
                        e
                    );

                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * self.retry_config.backoff_multiplier) as u64,
                    )
                    .min(self.retry_config.max_delay);
                }
            }
        }
    }

    /// Batch execute multiple operations
    pub async fn batch_execute<F, T>(&self, mut operations: Vec<F>) -> Vec<Result<T, AsyncError>>
    where
        F: Future<Output = Result<T, AsyncError>>,
    {
        let chunk_size = self
            .max_concurrent
            .available_permits()
            .min(operations.len());
        let mut results = Vec::with_capacity(operations.len());

        while !operations.is_empty() {
            let chunk_len = chunk_size.min(operations.len());
            let chunk: Vec<_> = operations.drain(..chunk_len).collect();

            let chunk_futures: Vec<_> = chunk.into_iter().map(|op| self.execute(op)).collect();

            let chunk_results = futures::future::join_all(chunk_futures).await;
            results.extend(chunk_results);
        }

        results
    }
}

/// Type alias for stream processing function
type ProcessorFn<T> = Arc<
    dyn Fn(Vec<T>) -> Pin<Box<dyn Future<Output = Result<(), AsyncError>> + Send>> + Send + Sync,
>;

/// Optimized stream processing
pub struct StreamProcessor<T> {
    buffer_size: usize,
    batch_timeout: Duration,
    processor: ProcessorFn<T>,
}

impl<T: Send + 'static> StreamProcessor<T> {
    /// Create new stream processor
    pub fn new<F, Fut>(buffer_size: usize, batch_timeout: Duration, processor: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), AsyncError>> + Send + 'static,
    {
        Self {
            buffer_size,
            batch_timeout,
            processor: Arc::new(move |batch| Box::pin(processor(batch))),
        }
    }

    /// Process stream with batching
    pub async fn process_stream(
        &self,
        mut receiver: tokio::sync::mpsc::Receiver<T>,
    ) -> Result<(), AsyncError> {
        let mut buffer = Vec::with_capacity(self.buffer_size);
        let mut last_flush = Instant::now();

        while let Some(item) = receiver.recv().await {
            buffer.push(item);

            // Flush if buffer is full or timeout reached
            let should_flush =
                buffer.len() >= self.buffer_size || last_flush.elapsed() >= self.batch_timeout;

            if should_flush {
                self.flush_buffer(&mut buffer).await?;
                last_flush = Instant::now();
            }
        }

        // Flush remaining items
        if !buffer.is_empty() {
            self.flush_buffer(&mut buffer).await?;
        }

        Ok(())
    }

    /// Flush buffer contents
    async fn flush_buffer(&self, buffer: &mut Vec<T>) -> Result<(), AsyncError> {
        if buffer.is_empty() {
            return Ok(());
        }

        let batch = std::mem::take(buffer);
        let batch_size = batch.len();

        debug!("Processing batch of {} items", batch_size);

        let start = Instant::now();
        (self.processor)(batch).await?;
        let duration = start.elapsed();

        debug!(
            "Batch processed in {:.2}ms",
            duration.as_secs_f64() * 1000.0
        );

        Ok(())
    }
}

/// Task pool for efficient task management
pub struct TaskPool {
    pool: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    max_tasks: usize,
}

impl TaskPool {
    /// Create new task pool
    pub fn new(max_tasks: usize) -> Self {
        Self {
            pool: Arc::new(RwLock::new(Vec::with_capacity(max_tasks))),
            max_tasks,
        }
    }

    /// Spawn task in pool
    pub async fn spawn<F>(&self, future: F) -> Result<(), AsyncError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut pool = self.pool.write().await;

        // Clean up completed tasks
        pool.retain(|handle| !handle.is_finished());

        if pool.len() >= self.max_tasks {
            return Err(AsyncError::ResourceExhausted);
        }

        let handle = tokio::spawn(future);
        pool.push(handle);

        Ok(())
    }

    /// Wait for all tasks to complete
    pub async fn join_all(&self) -> Result<(), AsyncError> {
        let mut pool = self.pool.write().await;
        let handles = std::mem::take(&mut *pool);

        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task failed: {:?}", e);
            }
        }

        Ok(())
    }

    /// Shutdown pool gracefully
    pub async fn shutdown(&self) {
        let mut pool = self.pool.write().await;

        for handle in pool.drain(..) {
            handle.abort();
        }
    }
}

/// Async error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum AsyncError {
    #[error("Operation timed out")]
    Timeout,

    #[error("Resource exhausted")]
    ResourceExhausted,

    #[error("Task cancelled")]
    Cancelled,

    #[error("IO error: {0}")]
    Io(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AsyncError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            AsyncError::Timeout => true,
            AsyncError::ResourceExhausted => true,
            AsyncError::Network(_) => true,
            AsyncError::Io(_) => true,
            AsyncError::Cancelled => false,
            AsyncError::Internal(_) => false,
        }
    }
}

/// Type alias for error handler function
type ErrorHandlerFn = Box<dyn Fn(&AsyncError) + Send + Sync>;

/// Future wrapper for error propagation optimization
#[pin_project]
pub struct ErrorPropagationFuture<F> {
    #[pin]
    inner: F,
    error_handler: Option<ErrorHandlerFn>,
}

impl<F> ErrorPropagationFuture<F> {
    /// Create new error propagation future
    pub fn new(future: F) -> Self {
        Self {
            inner: future,
            error_handler: None,
        }
    }

    /// Set error handler
    pub fn with_error_handler<H>(mut self, handler: H) -> Self
    where
        H: Fn(&AsyncError) + Send + Sync + 'static,
    {
        self.error_handler = Some(Box::new(handler));
        self
    }
}

impl<F, T> Future for ErrorPropagationFuture<F>
where
    F: Future<Output = Result<T, AsyncError>>,
{
    type Output = Result<T, AsyncError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(Err(e)) => {
                if let Some(ref handler) = this.error_handler {
                    handler(&e);
                }
                Poll::Ready(Err(e))
            }
            poll => poll,
        }
    }
}

/// Resource limiter for preventing resource exhaustion
pub struct ResourceLimiter {
    memory_limit: usize,
    task_limit: usize,
    current_memory: Arc<RwLock<usize>>,
    current_tasks: Arc<RwLock<usize>>,
}

impl ResourceLimiter {
    /// Create new resource limiter
    pub fn new(memory_limit: usize, task_limit: usize) -> Self {
        Self {
            memory_limit,
            task_limit,
            current_memory: Arc::new(RwLock::new(0)),
            current_tasks: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if resources are available
    pub async fn check_resources(
        &self,
        memory_needed: usize,
        tasks_needed: usize,
    ) -> Result<(), AsyncError> {
        let current_memory = *self.current_memory.read().await;
        let current_tasks = *self.current_tasks.read().await;

        if current_memory + memory_needed > self.memory_limit {
            return Err(AsyncError::ResourceExhausted);
        }

        if current_tasks + tasks_needed > self.task_limit {
            return Err(AsyncError::ResourceExhausted);
        }

        Ok(())
    }

    /// Acquire resources
    pub async fn acquire_resources(
        self: &Arc<Self>,
        memory: usize,
        tasks: usize,
    ) -> Result<ResourceGuard, AsyncError> {
        self.check_resources(memory, tasks).await?;

        {
            let mut current_memory = self.current_memory.write().await;
            let mut current_tasks = self.current_tasks.write().await;

            *current_memory += memory;
            *current_tasks += tasks;
        }

        Ok(ResourceGuard {
            limiter: Arc::clone(self),
            memory,
            tasks,
        })
    }

    /// Release resources
    async fn release_resources(&self, memory: usize, tasks: usize) {
        let mut current_memory = self.current_memory.write().await;
        let mut current_tasks = self.current_tasks.write().await;

        *current_memory = current_memory.saturating_sub(memory);
        *current_tasks = current_tasks.saturating_sub(tasks);
    }
}

/// RAII guard for resources
pub struct ResourceGuard {
    limiter: Arc<ResourceLimiter>,
    memory: usize,
    tasks: usize,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        // Spawn a task to release resources asynchronously
        // This is safe because we don't need to wait for completion
        let limiter = Arc::clone(&self.limiter);
        let memory = self.memory;
        let tasks = self.tasks;

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                limiter.release_resources(memory, tasks).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_optimizer() {
        let optimizer = AsyncOptimizer::new(2, Duration::from_secs(1));

        let result = optimizer
            .execute(async { Ok::<i32, AsyncError>(42) })
            .await
            .unwrap();

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let optimizer = AsyncOptimizer::new(2, Duration::from_secs(1));
        let mut attempt = 0;

        let result = optimizer
            .execute_with_retry(|| {
                attempt += 1;
                async move {
                    if attempt < 3 {
                        Err(AsyncError::Network("temporary failure".to_string()))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await
            .unwrap();

        assert_eq!(result, 42);
        assert_eq!(attempt, 3);
    }

    #[tokio::test]
    async fn test_resource_limiter() {
        let limiter = Arc::new(ResourceLimiter::new(1000, 10));

        // Should succeed
        let _guard = limiter.acquire_resources(500, 5).await.unwrap();

        // Should fail due to memory limit
        assert!(limiter.acquire_resources(600, 3).await.is_err());
    }

    #[tokio::test]
    async fn test_task_pool() {
        let pool = TaskPool::new(2);

        pool.spawn(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
        })
        .await
        .unwrap();

        pool.spawn(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
        })
        .await
        .unwrap();

        // Should fail as pool is full
        assert!(pool.spawn(async {}).await.is_err());

        pool.join_all().await.unwrap();
    }
}
