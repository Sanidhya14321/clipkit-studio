//! A real server on a free loopback port: server.json, WebSocket push and cleanup.

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use super::fixture::Fixture;
use crate::info::ServerInfo;
use crate::serve::Server;

async fn next_json(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Value {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(t) = msg {
            return serde_json::from_str(&t).unwrap();
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serves_ws_events_and_cleans_up() {
    let f = Fixture::new();
    let server = Server::bind(f.dir.path(), f.session.clone(), 0)
        .await
        .unwrap();
    let info = server.info().clone();
    assert_ne!(info.port, 0);
    assert_eq!(info.token.len(), 64);
    assert_eq!(ServerInfo::read(f.dir.path()).unwrap(), info);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(ServerInfo::path(f.dir.path()))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "server.json holds the token");
    }

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(server.run(async {
        let _ = stop_rx.await;
    }));

    // Without the token the handshake is refused.
    let bare = format!("ws://127.0.0.1:{}/ws", info.port);
    assert!(tokio_tungstenite::connect_async(&bare).await.is_err());

    let url = format!("ws://127.0.0.1:{}/ws?token={}", info.port, info.token);
    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let first = next_json(&mut ws).await;
    assert_eq!(first["type"], "snapshot");
    assert_eq!(first["project"]["rev"], 0);

    let started = Instant::now();
    let req = serde_json::from_value(super::fixture::rename(0, "human", "Live")).unwrap();
    let session = f.session.clone();
    tokio::task::spawn_blocking(move || session.patch(&req))
        .await
        .unwrap()
        .unwrap();
    let rev = next_json(&mut ws).await;
    let snap = next_json(&mut ws).await;
    let elapsed = started.elapsed();
    assert_eq!(rev["type"], "revision");
    assert_eq!(rev["touched"][0], "title");
    assert_eq!(snap["project"]["tracks"][0]["items"][0]["name"], "Live");
    assert!(elapsed < Duration::from_millis(300), "took {elapsed:?}");

    stop_tx.send(()).unwrap();
    drop(ws);
    handle.await.unwrap().unwrap();
    assert!(
        !ServerInfo::path(f.dir.path()).exists(),
        "server.json is removed on shutdown"
    );
}
