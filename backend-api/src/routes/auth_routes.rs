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
    models::{BuilderProfile, GeoPoint, SensoryPreferences},
    state::AppState,
};

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/status"))]
pub async fn auth_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(
        json!({ "oauth_configured": state.config().oauth_configured(), "timestamp_utc": Utc::now() }),
    )
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/github"))]
pub async fn github_login_handler(State(state): State<AppState>) -> Result<Response, ApiError> {
    let (url, cookies) = build_github_authorize_url(state.config())?;
    let jar = cookies
        .into_iter()
        .fold(CookieJar::new(), |jar, cookie| jar.add(cookie));
    Ok((jar, Redirect::temporary(&url)).into_response())
}

#[tracing::instrument(skip_all, fields(http.route = "/api/auth/github/callback"))]
pub async fn github_callback_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<OAuthCallbackQuery>,
) -> Response {
    use crate::auth::{OAUTH_STATE_COOKIE, OAUTH_VERIFIER_COOKIE};
    use subtle::ConstantTimeEq;
    let result = async {
        let expected = jar.get(OAUTH_STATE_COOKIE).ok_or_else(|| {
            ApiError::Unauthorized("OAuth session expired. Please sign in again.".into())
        })?;
        let actual = query
            .state
            .as_deref()
            .ok_or_else(|| ApiError::Unauthorized("Missing OAuth state".into()))?;
        if actual.is_empty()
            || expected
                .value()
                .as_bytes()
                .ct_eq(actual.as_bytes())
                .unwrap_u8()
                != 1
        {
            return Err(ApiError::Unauthorized("OAuth state did not match".into()));
        }
        let verifier = jar
            .get(OAUTH_VERIFIER_COOKIE)
            .filter(|c| c.value().len() >= 43)
            .ok_or_else(|| ApiError::Unauthorized("Missing OAuth verification cookie".into()))?;
        let code = query
            .code
            .as_deref()
            .filter(|c| !c.is_empty())
            .ok_or_else(|| ApiError::BadRequest("GitHub authorization was not completed".into()))?;
        let token =
            exchange_code_for_token(state.config(), state.http(), code, verifier.value()).await?;
        let (username, github_id) = state.github().authenticated_identity(&token).await?;
        let profile = sync_and_index_profile(&state, &username, None, Some(&token)).await?;
        if profile.github_id != github_id {
            return Err(ApiError::Unauthorized(
                "GitHub account identity changed during sign-in".into(),
            ));
        }
        Ok(sign_session_cookie(
            &SessionToken::new(username, github_id),
            &state.config().session_secret,
            state.config().is_secure(),
        ))
    }
    .await;
    let mut cleared = jar;
    for name in [OAUTH_STATE_COOKIE, OAUTH_VERIFIER_COOKIE] {
        let mut cookie = crate::auth::oauth_cookie(name, String::new(), state.config());
        cookie.make_removal();
        cleared = cleared.add(cookie);
    }
    match result {
        Ok(cookie) => (cleared.add(cookie), Redirect::temporary("/")).into_response(),
        Err(error) => (cleared, error).into_response(),
    }
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
    state
        .http()
        .put(&doc_url)
        .json(&profile)
        .send()
        .await?
        .error_for_status()?;
    state.elastic().refresh_index("builders").await?;

    Ok(profile)
}
