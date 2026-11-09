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
        if secret.len() < 32 {
            return Ok(OptionalAuthUser(None));
        }

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
                "Authentication required. Please sign in with GitHub.".to_string(),
            )),
        }
    }
}

pub async fn require_session(
    axum::extract::State(state): axum::extract::State<AppState>,
    RequireAuthUser(_user): RequireAuthUser,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, ApiError> {
    if !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        if let Some(origin) = request.headers().get("origin") {
            let expected = reqwest::Url::parse(&state.config().public_base_url)
                .map_err(|_| ApiError::InternalError("Invalid public URL".into()))?
                .origin()
                .ascii_serialization();
            if origin.to_str().ok() != Some(expected.as_str()) {
                return Err(ApiError::Forbidden(
                    "Request origin does not match this app".into(),
                ));
            }
        }
    }
    Ok(next.run(request).await)
}

pub async fn require_admin(
    axum::extract::State(state): axum::extract::State<AppState>,
    RequireAuthUser(user): RequireAuthUser,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, ApiError> {
    if !state.config().is_admin(&user.username) {
        return Err(ApiError::Forbidden(
            "Demo controls require an administrator account".into(),
        ));
    }
    Ok(next.run(request).await)
}
