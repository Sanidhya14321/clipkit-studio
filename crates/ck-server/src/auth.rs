//! Request guards: Host and Origin checks against DNS rebinding and cross-site requests, and the
//! session token for every API and WebSocket call.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use ck_core::{CkError, ErrorCode};

use crate::http_error::ApiError;

/// Header carrying the session token (the WebSocket uses the `token` query parameter instead,
/// because browsers cannot set headers on a WebSocket handshake).
pub const TOKEN_HEADER: &str = "x-ck-token";

#[derive(Debug, Clone)]
pub struct Guard {
    port: u16,
    token: Arc<str>,
}

impl Guard {
    pub fn new(port: u16, token: &str) -> Self {
        Self {
            port,
            token: Arc::from(token),
        }
    }

    fn host_ok(&self, host: &str) -> bool {
        host == format!("127.0.0.1:{}", self.port) || host == format!("localhost:{}", self.port)
    }

    fn origin_ok(&self, origin: &str) -> bool {
        origin == format!("http://127.0.0.1:{}", self.port)
            || origin == format!("http://localhost:{}", self.port)
    }

    /// Host must name this server on loopback; Origin, when sent, must be this server.
    pub fn check_site(&self, headers: &HeaderMap) -> Result<(), CkError> {
        let host = headers
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        if !self.host_ok(host) {
            return Err(denied("Host header is not this local server"));
        }
        if let Some(origin) = headers.get(header::ORIGIN)
            && !origin.to_str().is_ok_and(|o| self.origin_ok(o))
        {
            return Err(denied("Origin is not this local server"));
        }
        Ok(())
    }

    pub fn check_token(&self, req: &Request) -> Result<(), CkError> {
        let from_header = req
            .headers()
            .get(TOKEN_HEADER)
            .and_then(|h| h.to_str().ok());
        // Only the WebSocket and media (a `<video>` src) may carry the token in the URL; API
        // calls keep it out of URLs.
        let path = req.uri().path();
        let from_query = (path == "/ws" || path.starts_with("/media/"))
            .then(|| req.uri().query())
            .flatten()
            .and_then(|q| q.split('&').find_map(|pair| pair.strip_prefix("token=")));
        match from_header.or(from_query) {
            Some(t) if constant_time_eq(t.as_bytes(), self.token.as_bytes()) => Ok(()),
            Some(_) => Err(denied("session token is wrong")),
            None => Err(denied("session token is missing")),
        }
    }
}

fn denied(message: &str) -> CkError {
    CkError::new(ErrorCode::Auth, message)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Every route: Host and Origin.
pub async fn site_guard(State(guard): State<Guard>, req: Request, next: Next) -> Response {
    match guard.check_site(req.headers()) {
        Ok(()) => next.run(req).await,
        Err(e) => ApiError(e).into_response(),
    }
}

/// API and WebSocket routes: the session token.
pub async fn token_guard(State(guard): State<Guard>, req: Request, next: Next) -> Response {
    match guard.check_token(&req) {
        Ok(()) => next.run(req).await,
        Err(e) => ApiError(e).into_response(),
    }
}
