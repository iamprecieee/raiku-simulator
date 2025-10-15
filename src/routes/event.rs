use std::convert::Infallible;

use axum::{extract::State, response::Sse};
use futures_util::{Stream, stream};

use crate::app::api::AppContext;

#[utoipa::path(
    get,
    path = "/events",
    tag = "SSE",
    responses(
        (status = 200, description = "Event stream", content_type = "text/event-stream"),
    )
)]
pub async fn sse_handler(
    State(context): State<AppContext>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let receiver = context.state.events.subscribe();

    let stream = stream::unfold(receiver, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => {
                let event_data = serde_json::to_string(&event).unwrap_or_default();
                let sse_event = axum::response::sse::Event::default().data(event_data);
                Some((Ok(sse_event), rx))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("keep-alive"),
    )
}
