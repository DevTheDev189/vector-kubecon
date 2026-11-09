use reqwest::Client;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::config::AppConfig;
use crate::models::SseEvent;
use crate::services::{ElasticClient, GitHubScraper, OpenAiEngine};
use crate::telemetry::TelemetryMetrics;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<InnerState>,
}

pub struct InnerState {
    pub room_updates: tokio::sync::Mutex<()>,
    pub config: AppConfig,
    pub http: Client,
    pub elastic: ElasticClient,
    pub openai: OpenAiEngine,
    pub github: GitHubScraper,
    pub metrics: TelemetryMetrics,
    pub sse_tx: broadcast::Sender<SseEvent>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let elastic = ElasticClient::new(&config.elasticsearch_url, http.clone());
        let openai = OpenAiEngine::new(config.clone(), http.clone());
        let github = GitHubScraper::new(http.clone());
        let metrics = TelemetryMetrics::new();
        let (sse_tx, _) = broadcast::channel(1024);

        let inner = InnerState {
            room_updates: tokio::sync::Mutex::new(()),
            config,
            http,
            elastic,
            openai,
            github,
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

    pub fn elastic(&self) -> &ElasticClient {
        &self.inner.elastic
    }

    pub fn openai(&self) -> &OpenAiEngine {
        &self.inner.openai
    }

    pub fn github(&self) -> &GitHubScraper {
        &self.inner.github
    }

    pub fn metrics(&self) -> &TelemetryMetrics {
        &self.inner.metrics
    }

    pub fn sse_sender(&self) -> &broadcast::Sender<SseEvent> {
        &self.inner.sse_tx
    }

    pub fn broadcast_sse(&self, event: SseEvent) {
        let _ = self.inner.sse_tx.send(event);
    }
}
