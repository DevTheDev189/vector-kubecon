use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use time::Duration;

type HmacSha256 = Hmac<Sha256>;

pub const COOKIE_NAME: &str = "vector_sid";
pub const OAUTH_STATE_COOKIE: &str = "vector_oauth_state";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionToken {
    pub username: String,
    pub github_id: u64,
    pub iat: i64,
    pub exp: i64,
}

impl SessionToken {
    pub fn new(username: String, github_id: u64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            username,
            github_id,
            iat: now,
            exp: now + 86400 * 7, // 7 days
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

pub fn sign_session_cookie<'a>(token: &SessionToken, secret: &[u8], is_secure: bool) -> Cookie<'a> {
    let payload_bytes = serde_json::to_vec(token).unwrap_or_default();
    let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_bytes);

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(payload_b64.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);

    let cookie_val = format!("{}.{}", payload_b64, sig_b64);

    let mut builder = Cookie::build((COOKIE_NAME, cookie_val))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(7));

    if is_secure {
        builder = builder.secure(true);
    }

    builder.build()
}

pub fn verify_session_cookie(cookie_value: &str, secret: &[u8]) -> Option<SessionToken> {
    let (payload_b64, sig_b64) = cookie_value.split_once('.')?;

    let expected_sig = URL_SAFE_NO_PAD.decode(sig_b64).ok()?;

    let mut mac = HmacSha256::new_from_slice(secret).ok()?;
    mac.update(payload_b64.as_bytes());

    let actual_sig = mac.finalize().into_bytes();
    if actual_sig.ct_eq(&expected_sig).unwrap_u8() != 1 {
        return None;
    }

    let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let token: SessionToken = serde_json::from_slice(&payload_bytes).ok()?;

    if token.is_expired() {
        return None;
    }

    Some(token)
}

pub fn clear_session_cookie<'a>(is_secure: bool) -> Cookie<'a> {
    let mut builder = Cookie::build((COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0));

    if is_secure {
        builder = builder.secure(true);
    }

    builder.build()
}
