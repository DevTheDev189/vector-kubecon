use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub public_base_url: String,
    pub session_secret: Vec<u8>,
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_model: String,
    pub openai_embedding_model: String,
    pub elasticsearch_url: String,
    pub dash0_otlp_endpoint: Option<String>,
    pub dash0_auth_token: Option<String>,
    pub dash0_dataset: String,
    pub rust_log: String,
    pub static_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Load .env if present, ignore if missing
        dotenvy_minimal();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        let public_base_url =
            env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| format!("http://127.0.0.1:{}", port));

        let secret_raw = env::var("SESSION_SECRET").unwrap_or_else(|_| {
            "vector-production-grade-hmac-sha256-secret-seed-key-32b".to_string()
        });
        let session_secret = secret_raw.into_bytes();

        let github_client_id = env::var("GITHUB_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let openai_api_key = env::var("OPENAI_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let openai_model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-6.1-sol".to_string());

        let openai_embedding_model = env::var("OPENAI_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "text-embedding-3-small".to_string());

        let elasticsearch_url =
            env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:9200".to_string());

        let dash0_otlp_endpoint = env::var("DASH0_OTLP_ENDPOINT")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let dash0_auth_token = env::var("DASH0_AUTH_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let dash0_dataset = env::var("DASH0_DATASET").unwrap_or_else(|_| "default".to_string());

        let rust_log =
            env::var("RUST_LOG").unwrap_or_else(|_| "info,backend_api=debug".to_string());

        let static_dir =
            env::var("STATIC_DIR").unwrap_or_else(|_| "../frontend-pwa/dist".to_string());

        Self {
            port,
            public_base_url,
            session_secret,
            github_client_id,
            github_client_secret,
            openai_api_key,
            openai_model,
            openai_embedding_model,
            elasticsearch_url,
            dash0_otlp_endpoint,
            dash0_auth_token,
            dash0_dataset,
            rust_log,
            static_dir,
        }
    }

    pub fn is_secure(&self) -> bool {
        self.public_base_url.starts_with("https://")
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Simple helper to parse .env file if it exists without extra external dependency
fn dotenvy_minimal() {
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let mut val = v.trim();
                if (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''))
                {
                    val = &val[1..val.len() - 1];
                }
                if env::var(key).is_err() {
                    env::set_var(key, val);
                }
            }
        }
    }
}
