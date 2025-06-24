pub mod metrics_collector;
pub mod exporter;
pub mod logger;
pub mod integration;

pub use metrics_collector::{MetricsCollector, MetricTimer};
pub use exporter::PrometheusExporter;
pub use logger::{StructuredLogger, LogContext};
pub use integration::{
    MonitoredComponent, MonitoredChunkedProcessor, MonitoredConnectionPool,
    MonitoredValidationCache, MonitoredSwarmCoordinator, SystemMonitor
};

use std::sync::Arc;
use prometheus::Registry;

pub struct MonitoringSystem {
    pub metrics: Arc<MetricsCollector>,
    pub exporter: PrometheusExporter,
    pub logger: StructuredLogger,
}

impl MonitoringSystem {
    pub fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        let metrics = Arc::new(MetricsCollector::new(&registry)?);
        let exporter = PrometheusExporter::new(registry, port);
        let logger = StructuredLogger::new();
        
        Ok(Self {
            metrics,
            exporter,
            logger,
        })
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.exporter.start().await?;
        self.logger.info("Monitoring system started", LogContext::new());
        Ok(())
    }
}