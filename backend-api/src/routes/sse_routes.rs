use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::{error::ApiError, state::AppState};

#[tracing::instrument(skip_all, fields(http.route = "/api/events/stream"))]
pub async fn sse_events_stream_handler(
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let rx = state.sse_sender().subscribe();
    let broadcast_stream = BroadcastStream::new(rx);

    // Track active SSE connection count in Dash0 metrics
    state.metrics().active_sse_clients.add(1, &[]);

    let stream = broadcast_stream.filter_map(move |item| match item {
        Ok(event) => {
            let json_str = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(json_str)))
        }
        Err(_) => None,
    });

    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    );

    Ok(sse)
}
