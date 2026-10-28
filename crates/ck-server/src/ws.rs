//! `GET /ws`: push the snapshot on connect, then every bus event. Inbound messages are ignored;
//! clients change the project through the HTTP API so every write passes the same gates.

use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::{IntoResponse, Response};
use ck_core::{CkError, CkResult, ErrorCode};
use tokio::sync::broadcast::error::RecvError;

use crate::event::Event;
use crate::http_error::ApiError;
use crate::session::Session;

/// Inbound messages are ignored, so keep them tiny.
const MAX_INBOUND: usize = 16 * 1024;

type Subscription = (Event, tokio::sync::broadcast::Receiver<Event>);

pub async fn upgrade(State(s): State<Arc<Session>>, ws: WebSocketUpgrade) -> Response {
    match subscribe(s.clone()).await {
        Ok((first, rx)) => ws
            .max_message_size(MAX_INBOUND)
            .max_frame_size(MAX_INBOUND)
            .on_upgrade(move |socket| stream(socket, s, first, rx)),
        Err(e) => ApiError(e).into_response(),
    }
}

/// The session mutex can be held across an fsync, so take it on the blocking pool.
async fn subscribe(s: Arc<Session>) -> CkResult<Subscription> {
    tokio::task::spawn_blocking(move || s.subscribe())
        .await
        .map_err(|e| CkError::new(ErrorCode::Server, format!("task failed: {e}")))?
}

async fn stream(
    mut socket: WebSocket,
    session: Arc<Session>,
    first: Event,
    mut rx: tokio::sync::broadcast::Receiver<Event>,
) {
    if send(&mut socket, &first).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            event = rx.recv() => {
                let event = match event {
                    Ok(e) => e,
                    // Too slow to keep up: skip the backlog and resend the current state.
                    Err(RecvError::Lagged(_)) => match subscribe(session.clone()).await {
                        Ok((snap, fresh)) => { rx = fresh; snap }
                        Err(_) => return,
                    },
                    Err(RecvError::Closed) => return,
                };
                if send(&mut socket, &event).await.is_err() {
                    return;
                }
            }
            inbound = socket.recv() => match inbound {
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return,
                Some(Ok(_)) => {}
            },
        }
    }
}

async fn send(socket: &mut WebSocket, event: &Event) -> Result<(), ()> {
    let text = serde_json::to_string(event).map_err(|_| ())?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|_| ())
}
