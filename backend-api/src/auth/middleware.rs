use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{
    auth::cookies::{verify_session_cookie, SessionToken, COOKIE_NAME},
    error::ApiError,
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct OptionalAuthUser(pub Option<SessionToken>);

#[derive(Clone, Debug)]
pub struct RequireAuthUser(pub SessionToken);

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let secret = &state.config().session_secret;

        // 1. Check CookieJar
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .unwrap_or_default();

        if let Some(cookie) = jar.get(COOKIE_NAME) {
            if let Some(token) = verify_session_cookie(cookie.value(), secret) {
                tracing::Span::current().record("enduser.id", &token.username);
                return Ok(OptionalAuthUser(Some(token)));
            }
        }

        // 2. Fallback header for testing / CLI (X-Vector-User)
        if let Some(user_hdr) = parts.headers.get("x-vector-user") {
            if let Ok(username) = user_hdr.to_str() {
                if !username.trim().is_empty() {
                    let token = SessionToken::new(username.trim().to_string(), 999999);
                    tracing::Span::current().record("enduser.id", username);
                    return Ok(OptionalAuthUser(Some(token)));
                }
            }
        }

        Ok(OptionalAuthUser(None))
    }
}

#[async_trait]
impl FromRequestParts<AppState> for RequireAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let opt = OptionalAuthUser::from_request_parts(parts, state).await?;
        match opt.0 {
            Some(token) => Ok(RequireAuthUser(token)),
            None => Err(ApiError::Unauthorized(
                "Authentication required. Please sign in with GitHub or choose a Judge persona."
                    .to_string(),
            )),
        }
    }
}
