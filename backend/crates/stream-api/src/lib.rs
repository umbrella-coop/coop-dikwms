//! SSE transport for the live knowledge stream (SPEC-004 AC-4).
//!
//! `GET /events?cursor=<commit>` streams `DomainEvent`s as commits land,
//! polling the commit log on a fixed interval. Each SSE event carries the
//! originating commit as `id:` so clients can reconnect without loss.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use futures::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use terminusdb_repository::Repository;
use terminusdb_repository::stream::{CommitCursor, DomainEvent, poll_events};

#[derive(Clone)]
pub struct StreamState {
    repo: Arc<Repository>,
    poll_interval: Duration,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    cursor: Option<String>,
}

/// Build the router with a repository-backed stream state.
pub fn router(repo: Repository, poll_interval: Duration) -> Router {
    Router::new()
        .route("/events", get(events_handler))
        .with_state(StreamState {
            repo: Arc::new(repo),
            poll_interval,
        })
}

async fn events_handler(
    State(state): State<StreamState>,
    Query(q): Query<EventsQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let cursor = CommitCursor(q.cursor.unwrap_or_default());
    let mut interval = tokio::time::interval(state.poll_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let repo = state.repo.clone();

    let stream = stream::unfold(
        (repo, cursor, interval),
        move |(repo, mut cursor, mut interval)| async move {
            interval.tick().await;
            match poll_events(&repo, &cursor).await {
                Ok((events, new_cursor)) => {
                    cursor = new_cursor;
                    let batch: Vec<Result<Event, Infallible>> = events
                        .iter()
                        .map(|ev: &DomainEvent| {
                            let id = ev.commit().to_string();
                            let data = serde_json::to_string(ev).unwrap_or_default();
                            Ok::<Event, Infallible>(Event::default().id(id).data(data))
                        })
                        .collect();
                    Some((stream::iter(batch), (repo, cursor, interval)))
                }
                Err(e) => {
                    tracing::warn!("poll_events failed: {e}");
                    Some((stream::iter(Vec::new()), (repo, cursor, interval)))
                }
            }
        },
    )
    .flatten();

    Sse::new(stream).keep_alive(KeepAlive::default())
}
