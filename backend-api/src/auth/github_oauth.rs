use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::Duration;

use crate::{
    auth::cookies::{OAUTH_STATE_COOKIE, OAUTH_VERIFIER_COOKIE},
    config::AppConfig,
    error::ApiError,
};

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub fn oauth_cookie(name: &'static str, value: String, config: &AppConfig) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(config.is_secure())
        .max_age(Duration::minutes(15))
        .build()
}

pub fn build_github_authorize_url(
    config: &AppConfig,
) -> Result<(String, Vec<Cookie<'static>>), ApiError> {
    let client_id = config
        .github_client_id
        .as_ref()
        .filter(|_| config.oauth_configured())
        .ok_or_else(|| {
            ApiError::ServiceUnavailable("GitHub sign-in is awaiting server configuration".into())
        })?;
    if config.session_secret.len() < 32 {
        return Err(ApiError::ServiceUnavailable(
            "Session signing key is not configured securely".into(),
        ));
    }
    let state = uuid::Uuid::new_v4().simple().to_string();
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let redirect_uri = format!(
        "{}/api/auth/github/callback",
        config.public_base_url.trim_end_matches('/')
    );
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read%3Auser&state={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(client_id), urlencoding::encode(&redirect_uri), state, challenge
    );
    Ok((
        auth_url,
        vec![
            oauth_cookie(OAUTH_STATE_COOKIE, state, config),
            oauth_cookie(OAUTH_VERIFIER_COOKIE, verifier, config),
        ],
    ))
}

pub async fn exchange_code_for_token(
    config: &AppConfig,
    http: &Client,
    code: &str,
    verifier: &str,
) -> Result<String, ApiError> {
    let client_id = config.github_client_id.as_ref().ok_or_else(|| {
        ApiError::GitHubError("GITHUB_CLIENT_ID not configured on server".to_string())
    })?;
    let client_secret = config.github_client_secret.as_ref().ok_or_else(|| {
        ApiError::GitHubError("GITHUB_CLIENT_SECRET not configured on server".to_string())
    })?;

    let redirect_uri = format!(
        "{}/api/auth/github/callback",
        config.public_base_url.trim_end_matches('/')
    );
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("code", code),
        ("code_verifier", verifier),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let resp = http
        .post(&config.github_token_url)
        .header("Accept", "application/json")
        .header("User-Agent", "VECTOR-KubeCon-App/1.0")
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(ApiError::GitHubError(
            "GitHub rejected the authorization code".into(),
        ));
    }

    let token_data: GitHubTokenResponse = resp.json().await?;

    if let Some(token) = token_data.access_token.filter(|t| !t.is_empty()) {
        Ok(token)
    } else {
        let _ = (token_data.error, token_data.error_description);
        Err(ApiError::GitHubError(
            "GitHub sign-in could not be completed. Please try again.".into(),
        ))
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
