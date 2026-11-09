use crate::{
    auth::RequireAuthUser, error::ApiError, models::SseEvent, state::AppState,
    telemetry::TelemetryMetrics,
};
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::{Stream, StreamExt};
use std::{convert::Infallible, time::Duration};
use tokio_stream::wrappers::BroadcastStream;

struct ConnectionGuard(TelemetryMetrics);
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.active_sse_clients.add(-1, &[]);
    }
}

#[tracing::instrument(skip_all, fields(http.route = "/api/events/stream"))]
pub async fn sse_events_stream_handler(
    State(state): State<AppState>,
    RequireAuthUser(user): RequireAuthUser,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let rx = state.sse_sender().subscribe();
    state.metrics().active_sse_clients.add(1, &[]);
    let guard = ConnectionGuard(state.metrics().clone());
    let expires_in = (user.exp - chrono::Utc::now().timestamp()).max(0) as u64;
    let live = BroadcastStream::new(rx).filter_map(move |item| {
        let _keep_guard_alive = &guard;
        let event = item
            .ok()
            .filter(|event| match event {
                SseEvent::HandshakeCompleted { handshake, .. } => {
                    handshake.initiator_username == user.username
                }
                SseEvent::DnaSyncProgress { username, .. } => *username == user.username,
                _ => true,
            })
            .and_then(|event| serde_json::to_string(&event).ok())
            .map(|json| Ok(Event::default().data(json)));
        futures::future::ready(event)
    });
    let stream = futures::stream::once(async { Ok(Event::default().comment("connected")) })
        .chain(live)
        .take_until(tokio::time::sleep(Duration::from_secs(expires_in)));
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    ))
}
