//! Bind to 127.0.0.1, publish `.clipkit/server.json`, serve until shutdown, then clean up.

use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Router;
use ck_core::{CkError, CkResult, ErrorCode};
use tokio::net::TcpListener;

use crate::api;
use crate::auth::Guard;
use crate::info::{ServerInfo, new_token};
use crate::session::Session;

pub struct Server {
    listener: TcpListener,
    router: Router,
    info: ServerInfo,
    root: PathBuf,
}

impl Server {
    /// Bind loopback on `port` (0 = a free port) and write `server.json` under `root`.
    pub async fn bind(root: &Path, session: Arc<Session>, port: u16) -> CkResult<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .await
            .map_err(|e| server_err(format!("cannot bind 127.0.0.1:{port}: {e}")))?;
        let port = listener
            .local_addr()
            .map_err(|e| server_err(e.to_string()))?
            .port();
        let info = ServerInfo::new(port, new_token()?);
        info.write(root)?;
        let router = api::router(session, Guard::new(port, &info.token));
        tracing::info!(port, "server listening");
        Ok(Self {
            listener,
            router,
            info,
            root: root.to_path_buf(),
        })
    }

    pub fn info(&self) -> &ServerInfo {
        &self.info
    }

    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::LOCALHOST, self.info.port))
    }

    /// Serve until `shutdown` resolves, then remove `server.json` if it is still ours.
    pub async fn run(self, shutdown: impl Future<Output = ()> + Send + 'static) -> CkResult<()> {
        let result = axum::serve(self.listener, self.router)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| server_err(format!("server stopped: {e}")));
        self.info.remove_if_ours(&self.root);
        result
    }
}

fn server_err(message: String) -> CkError {
    CkError::new(ErrorCode::Server, message)
}
