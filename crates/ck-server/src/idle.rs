//! Idle shutdown (spec 5.1): the server stops after a configurable time with no authorized API
//! call and no open studio WebSocket. Time is tokio's monotonic clock, so tests can pause it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use tokio::time::Instant;

use crate::session::Session;

/// Default idle limit for `ck studio`.
pub const DEFAULT_IDLE_SECS: u64 = 30 * 60;

#[derive(Debug)]
pub struct Activity {
    start: Instant,
    /// Milliseconds after `start` of the last activity.
    last_ms: AtomicU64,
    /// Open WebSocket connections; the server is never idle while one is open.
    open: AtomicUsize,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            last_ms: AtomicU64::new(0),
            open: AtomicUsize::new(0),
        }
    }
}

impl Activity {
    pub fn touch(&self) {
        let now = u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.last_ms.fetch_max(now, Ordering::SeqCst);
    }

    /// Count an open connection until the guard drops; closing it counts as activity.
    pub fn connection(self: &Arc<Self>) -> Connection {
        self.open.fetch_add(1, Ordering::SeqCst);
        self.touch();
        Connection(self.clone())
    }

    /// How long nothing has happened, or `None` while a connection is open.
    pub fn idle_for(&self) -> Option<Duration> {
        if self.open.load(Ordering::SeqCst) > 0 {
            return None;
        }
        let last = Duration::from_millis(self.last_ms.load(Ordering::SeqCst));
        Some(self.start.elapsed().saturating_sub(last))
    }

    /// Resolve once the server has been idle for `limit`; never when `limit` is zero.
    pub async fn until_idle(self: Arc<Self>, limit: Duration) {
        if limit.is_zero() {
            return std::future::pending().await;
        }
        loop {
            let wait = match self.idle_for() {
                Some(idle) if idle >= limit => return,
                Some(idle) => limit - idle,
                None => limit,
            };
            tokio::time::sleep(wait).await;
        }
    }
}

pub struct Connection(Arc<Activity>);

impl Drop for Connection {
    fn drop(&mut self) {
        self.0.open.fetch_sub(1, Ordering::SeqCst);
        self.0.touch();
    }
}

/// Middleware on the token-guarded routes: every authorized call counts as activity.
pub async fn touch(State(s): State<Arc<Session>>, req: Request, next: Next) -> Response {
    s.activity().touch();
    next.run(req).await
}
