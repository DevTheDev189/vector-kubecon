use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use opentelemetry::trace::TraceContextExt;
use serde::Serialize;
use std::fmt;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    ElasticError(String),
    OpenAiError(String),
    GitHubError(String),
    InternalError(String),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Self::ElasticError(msg) => write!(f, "Elasticsearch Error: {}", msg),
            Self::OpenAiError(msg) => write!(f, "OpenAI Error: {}", msg),
            Self::GitHubError(msg) => write!(f, "GitHub Error: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    code: &'static str,
    message: String,
    trace_id: String,
    timestamp_utc: String,
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::ElasticError(_) => StatusCode::BAD_GATEWAY,
            Self::OpenAiError(_) => StatusCode::BAD_GATEWAY,
            Self::GitHubError(_) => StatusCode::BAD_GATEWAY,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::NotFound(_) => "NOT_FOUND",
            Self::ElasticError(_) => "ELASTICSEARCH_ERROR",
            Self::OpenAiError(_) => "OPENAI_ERROR",
            Self::GitHubError(_) => "GITHUB_ERROR",
            Self::InternalError(_) => "INTERNAL_SERVER_ERROR",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::NotFound(msg)
            | Self::ElasticError(msg)
            | Self::OpenAiError(msg)
            | Self::GitHubError(msg)
            | Self::InternalError(msg) => msg.clone(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let message = self.message();

        // Extract trace_id from the current tracing span
        let trace_id = {
            let span = tracing::Span::current();
            let context = span.context();
            let span_ref = context.span();
            let sc = span_ref.span_context();
            if sc.is_valid() {
                sc.trace_id().to_string()
            } else {
                uuid::Uuid::new_v4().simple().to_string()
            }
        };

        let now_utc = Utc::now().to_rfc3339();

        tracing::error!(
            error.code = code,
            error.message = %message,
            error.status = status.as_u16(),
            trace_id = %trace_id,
            "API request failed"
        );

        let body = Json(ErrorEnvelope {
            error: ErrorDetails {
                code,
                message,
                trace_id,
                timestamp_utc: now_utc,
            },
        });

        (status, body).into_response()
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        Self::InternalError(format!("HTTP client error: {}", err))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadRequest(format!("JSON serialization/deserialization error: {}", err))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::InternalError(format!("I/O error: {}", err))
    }
}
