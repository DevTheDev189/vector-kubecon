use axum_extra::extract::cookie::{Cookie, SameSite};
use reqwest::Client;
use serde::Deserialize;
use time::Duration;

use crate::{auth::cookies::OAUTH_STATE_COOKIE, config::AppConfig, error::ApiError};

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub fn build_github_authorize_url(config: &AppConfig) -> (String, Cookie<'static>) {
    let client_id = config
        .github_client_id
        .clone()
        .unwrap_or_else(|| "github_client_id_placeholder".to_string());

    let state = uuid::Uuid::new_v4().to_string();
    let redirect_uri = format!("{}/api/auth/github/callback", config.public_base_url);

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user,repo&state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state)
    );

    let mut cookie_builder = Cookie::build((OAUTH_STATE_COOKIE, state))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(15));

    if config.is_secure() {
        cookie_builder = cookie_builder.secure(true);
    }

    (auth_url, cookie_builder.build())
}

pub async fn exchange_code_for_token(
    config: &AppConfig,
    http: &Client,
    code: &str,
) -> Result<String, ApiError> {
    let client_id = config.github_client_id.as_ref().ok_or_else(|| {
        ApiError::GitHubError("GITHUB_CLIENT_ID not configured on server".to_string())
    })?;
    let client_secret = config.github_client_secret.as_ref().ok_or_else(|| {
        ApiError::GitHubError("GITHUB_CLIENT_SECRET not configured on server".to_string())
    })?;

    let token_url = "https://github.com/login/oauth/access_token";
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code),
    ];

    let resp = http
        .post(token_url)
        .header("Accept", "application/json")
        .header("User-Agent", "VECTOR-KubeCon-App/1.0")
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let err = resp.text().await.unwrap_or_default();
        return Err(ApiError::GitHubError(format!(
            "OAuth token exchange failed: {}",
            err
        )));
    }

    let token_data: GitHubTokenResponse = resp.json().await?;

    if let Some(token) = token_data.access_token {
        Ok(token)
    } else {
        let desc = token_data
            .error_description
            .unwrap_or_else(|| token_data.error.unwrap_or_default());
        Err(ApiError::GitHubError(format!(
            "GitHub token exchange error: {}",
            desc
        )))
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::with_capacity(s.len() * 3);
        for c in s.chars() {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                result.push(c);
            } else {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
        result
    }
}
