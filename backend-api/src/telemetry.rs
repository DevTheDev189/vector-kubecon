use opentelemetry::metrics::{Counter, Gauge, Histogram, UpDownCounter};
use opentelemetry::trace::TraceContextExt;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use std::collections::HashMap;
use std::time::Duration;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct TelemetryMetrics {
    pub room_occupancy_ratio: Gauge<f64>,
    pub room_decibels: Gauge<f64>,
    pub surge_reroutes_total: Counter<u64>,
    pub elastic_query_latency_ms: Histogram<f64>,
    pub openai_tokens_total: Counter<u64>,
    pub active_sse_clients: UpDownCounter<i64>,
}

impl TelemetryMetrics {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("vector-api");

        let room_occupancy_ratio = meter
            .f64_gauge("vector.room.occupancy_ratio")
            .with_description("Current room occupancy ratio (0.0 to 1.0)")
            .init();

        let room_decibels = meter
            .f64_gauge("vector.room.decibels")
            .with_description("Live sound level in decibels (dB)")
            .init();

        let surge_reroutes_total = meter
            .u64_counter("vector.surge.reroutes_total")
            .with_description("Total crowd surge reroutes triggered")
            .init();

        let elastic_query_latency_ms = meter
            .f64_histogram("vector.elastic.rrf_latency_ms")
            .with_description("Elasticsearch 3-way RRF query latency in ms")
            .init();

        let openai_tokens_total = meter
            .u64_counter("vector.gen_ai.tokens_total")
            .with_description("Total OpenAI tokens consumed")
            .init();

        let active_sse_clients = meter
            .i64_up_down_counter("vector.sse.active_clients")
            .with_description("Current connected SSE streaming clients")
            .init();

        Self {
            room_occupancy_ratio,
            room_decibels,
            surge_reroutes_total,
            elastic_query_latency_ms,
            openai_tokens_total,
            active_sse_clients,
        }
    }
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to get the active Dash0 OpenTelemetry trace_id as a hex string
pub fn current_trace_id() -> String {
    let span = tracing::Span::current();
    let context = span.context();
    let span_ref = context.span();
    let sc = span_ref.span_context();
    if sc.is_valid() {
        sc.trace_id().to_string()
    } else {
        uuid::Uuid::new_v4().simple().to_string()
    }
}

/// Initialize tracing and OpenTelemetry pipeline for Dash0
pub fn init_telemetry(config: &AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.rust_log));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true);

    let resource = Resource::new(vec![
        KeyValue::new("service.name", "vector-api"),
        KeyValue::new("service.version", "0.1.0"),
    ]);

    if let (Some(endpoint), Some(token)) = (&config.dash0_otlp_endpoint, &config.dash0_auth_token) {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert("dash0-dataset".to_string(), config.dash0_dataset.clone());

        let exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(endpoint)
            .with_headers(headers)
            .with_timeout(Duration::from_secs(5));

        use opentelemetry::trace::TracerProvider as _;

        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(resource))
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        let tracer = provider.tracer("vector-api");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .try_init()?;

        tracing::info!(
            endpoint = %endpoint,
            dataset = %config.dash0_dataset,
            "Dash0 OpenTelemetry exporter initialized"
        );
    } else {
        // Fallback to local stdout logging when Dash0 endpoint is not configured (e.g. tests)
        Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()?;

        tracing::info!(
            "Telemetry initialized in stdout-only fallback mode (Dash0 OTLP endpoint not set)"
        );
    }

    Ok(())
}
