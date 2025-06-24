use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct PrometheusExporter {
    registry: Arc<Registry>,
    port: u16,
}

impl PrometheusExporter {
    pub fn new(registry: Registry, port: u16) -> Self {
        Self {
            registry: Arc::new(registry),
            port,
        }
    }
    
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let registry = Arc::clone(&self.registry);
        
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(registry);
        
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Prometheus exporter listening on {}", addr);
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        
        Ok(())
    }
}

async fn metrics_handler(
    State(registry): State<Arc<Registry>>,
) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    
    let mut buffer = Vec::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => (StatusCode::OK, buffer),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Vec::new())
        }
    }
}