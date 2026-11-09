use reqwest::Client;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::config::AppConfig;
use crate::telemetry::TelemetryMetrics;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<InnerState>,
}

pub struct InnerState {
    pub config: AppConfig,
    pub http: Client,
    pub metrics: TelemetryMetrics,
    pub sse_tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let metrics = TelemetryMetrics::new();
        let (sse_tx, _) = broadcast::channel(1024);

        let inner = InnerState {
            config,
            http,
            metrics,
            sse_tx,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }

    pub fn http(&self) -> &Client {
        &self.inner.http
    }

    pub fn metrics(&self) -> &TelemetryMetrics {
        &self.inner.metrics
    }

    pub fn sse_sender(&self) -> &broadcast::Sender<String> {
        &self.inner.sse_tx
    }

    pub fn broadcast_sse_raw(&self, payload: String) {
        let _ = self.inner.sse_tx.send(payload);
    }
}
