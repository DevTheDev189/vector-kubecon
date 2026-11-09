use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::{
        build_github_authorize_url, clear_session_cookie, exchange_code_for_token,
        sign_session_cookie, SessionToken,
    },
    error::ApiError,
    models::{BuilderProfile, GeoPoint, SensoryPreferences, SseEvent},
    state::AppState,
};

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[derive(Deserialize)]
pub struct ImpersonateRequest {
    pub username: String,
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/github"))]
pub async fn github_login_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (auth_url, state_cookie) = build_github_authorize_url(state.config());
    let jar = CookieJar::new().add(state_cookie);
    (jar, Redirect::temporary(&auth_url))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/github/callback"))]
pub async fn github_callback_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Response, ApiError> {
    let code = query.code.ok_or_else(|| {
        ApiError::BadRequest("Missing authorization code in GitHub OAuth callback".to_string())
    })?;

    let token = exchange_code_for_token(state.config(), state.http(), &code).await?;
    let scraped = state
        .github()
        .scrape_github_user("me", Some(&token))
        .await?;

    let profile = sync_and_index_profile(&state, &scraped.username, None, Some(&token)).await?;

    let session = SessionToken::new(profile.username.clone(), profile.github_id);
    let session_cookie = sign_session_cookie(
        &session,
        &state.config().session_secret,
        state.config().is_secure(),
    );

    let updated_jar = jar.add(session_cookie);
    Ok((updated_jar, Redirect::temporary("/")).into_response())
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/impersonate"))]
pub async fn impersonate_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<ImpersonateRequest>,
) -> Result<(CookieJar, Json<BuilderProfile>), ApiError> {
    let username = payload.username.trim().to_string();

    state.broadcast_sse(SseEvent::DnaSyncProgress {
        username: username.clone(),
        step: "Fetching GitHub Repositories".to_string(),
        progress_pct: 20,
        detail: format!("Connecting to GitHub API for @{}...", username),
        timestamp_utc: Utc::now(),
    });

    state.broadcast_sse(SseEvent::DnaSyncProgress {
        username: username.clone(),
        step: "Extracting Builder DNA via AI Engine".to_string(),
        progress_pct: 50,
        detail: "Analyzing tech stack, languages, and repo READMEs...".to_string(),
        timestamp_utc: Utc::now(),
    });

    let profile = sync_and_index_profile(&state, &username, None, None).await?;

    state.broadcast_sse(SseEvent::DnaSyncProgress {
        username: username.clone(),
        step: "Generating Dense 1536-dim Embedding".to_string(),
        progress_pct: 80,
        detail: "Indexing cosine vector into Elasticsearch 8.15...".to_string(),
        timestamp_utc: Utc::now(),
    });

    state.broadcast_sse(SseEvent::DnaSyncProgress {
        username: username.clone(),
        step: "Hydration Complete".to_string(),
        progress_pct: 100,
        detail: format!("Ready! Logged in as {}", profile.display_name),
        timestamp_utc: Utc::now(),
    });

    let session = SessionToken::new(profile.username.clone(), profile.github_id);
    let session_cookie = sign_session_cookie(
        &session,
        &state.config().session_secret,
        state.config().is_secure(),
    );

    let updated_jar = jar.add(session_cookie);
    Ok((updated_jar, Json(profile)))
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/logout"))]
pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Json<serde_json::Value>) {
    let clear_cookie = clear_session_cookie(state.config().is_secure());
    let updated_jar = jar.add(clear_cookie);
    (updated_jar, Json(json!({ "status": "logged_out" })))
}

/// Helper function to run the full Scrape -> AI Extract -> Embedding -> ES Index pipeline
pub async fn sync_and_index_profile(
    state: &AppState,
    username: &str,
    linkedin_url: Option<String>,
    oauth_token: Option<&str>,
) -> Result<BuilderProfile, ApiError> {
    let scraped = state
        .github()
        .scrape_github_user(username, oauth_token)
        .await?;
    let extracted_dna = state.openai().extract_builder_dna(&scraped).await?;

    let embedding_text = format!(
        "{} {} {} {}",
        scraped.display_name,
        extracted_dna.persona_title,
        extracted_dna.dna_tags.join(" "),
        extracted_dna.ideal_talk_query
    );
    let embedding = state.openai().generate_embedding(&embedding_text).await?;

    let profile = BuilderProfile {
        username: scraped.username.clone(),
        github_id: scraped.github_id,
        display_name: scraped.display_name,
        avatar_url: scraped.avatar_url,
        bio: scraped.bio,
        company: scraped.company,
        github_url: scraped.github_url,
        linkedin_url,
        email: scraped.email,
        persona_title: extracted_dna.persona_title,
        dna_tags: extracted_dna.dna_tags,
        top_languages: scraped.languages,
        hallway_icebreaker: extracted_dna.hallway_icebreaker,
        ideal_talk_query: extracted_dna.ideal_talk_query,
        top_repos: scraped.top_repos,
        current_room_id: "ballroom-b".to_string(),
        coordinates: GeoPoint {
            lat: 40.76630,
            lon: -111.89670,
        },
        sensory_prefs: SensoryPreferences::default(),
        bookmarked_talk_ids: vec!["talk-ebpf-rust-mesh".to_string()],
        bumped_from_talk_id: None,
        pin_code: "4829".to_string(),
        embedding,
        updated_at: Utc::now(),
    };

    // Index into Elasticsearch
    let doc_url = format!(
        "{}/builders/_doc/{}",
        state.elastic().base_url,
        profile.username
    );
    let _ = state.http().put(&doc_url).json(&profile).send().await;
    let _ = state.elastic().refresh_index("builders").await;

    Ok(profile)
}
