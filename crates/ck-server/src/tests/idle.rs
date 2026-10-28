//! Idle shutdown: the limit counts from the last activity, open connections hold it off, and
//! only authorized calls count.

use std::sync::Arc;
use std::time::Duration;

use axum::http::Request;
use tokio::time::{Instant, sleep, timeout};

use super::fixture::{Fixture, HOST, body_of};
use crate::idle::Activity;

const LIMIT: Duration = Duration::from_secs(60);

#[tokio::test(start_paused = true)]
async fn stops_after_the_limit_from_start() {
    let t0 = Instant::now();
    Arc::new(Activity::default()).until_idle(LIMIT).await;
    assert_eq!(t0.elapsed(), LIMIT);
}

#[tokio::test(start_paused = true)]
async fn activity_restarts_the_count() {
    let activity = Arc::new(Activity::default());
    let t0 = Instant::now();
    let a = activity.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(30)).await;
        a.touch();
    });
    activity.until_idle(LIMIT).await;
    assert_eq!(t0.elapsed(), Duration::from_secs(90));
}

#[tokio::test(start_paused = true)]
async fn an_open_connection_holds_it_off_until_it_closes() {
    let activity = Arc::new(Activity::default());
    let t0 = Instant::now();
    let open = activity.connection();
    tokio::spawn(async move {
        sleep(Duration::from_secs(100)).await;
        drop(open);
    });
    assert_eq!(activity.idle_for(), None);
    activity.until_idle(LIMIT).await;
    assert_eq!(t0.elapsed(), Duration::from_secs(160));
}

#[tokio::test(start_paused = true)]
async fn zero_never_stops() {
    let never = Arc::new(Activity::default()).until_idle(Duration::ZERO);
    assert!(
        timeout(Duration::from_secs(7 * 24 * 3600), never)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn only_authorized_calls_count() {
    let f = Fixture::new();
    let idle = || f.session.activity().idle_for().unwrap();
    sleep(Duration::from_millis(40)).await;

    let no_token = Request::builder()
        .uri("/api/status")
        .header("host", HOST)
        .body(body_of(None))
        .unwrap();
    f.send(no_token).await;
    f.send(
        Request::builder()
            .uri("/")
            .header("host", HOST)
            .body(body_of(None))
            .unwrap(),
    )
    .await;
    assert!(idle() >= Duration::from_millis(40), "refused calls count");

    f.call("GET", "/api/status", None).await;
    assert!(
        idle() < Duration::from_millis(40),
        "authorized call ignored"
    );
}
