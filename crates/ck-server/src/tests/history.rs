//! Studio undo, redo and split (push-to-talk and keyboard commands).

use ck_core::ErrorCode;
use ck_core::model::{Author, Holder, ItemBody};

use super::fixture::Fixture;
use crate::session::LockRequest;
use crate::session::history::{HistoryRequest, SplitRequest};

fn split(at: f64, author: Option<Author>) -> SplitRequest {
    SplitRequest {
        layer_id: "title".into(),
        at,
        author,
    }
}

#[test]
fn split_then_undo_and_redo_by_voice() {
    let f = Fixture::new();
    let r = f.session.split(&split(0.5, Some(Author::Voice))).unwrap();
    assert_eq!(r.touched, ["title", "title-b"]);
    let p = f.session.project().unwrap();
    let dur = |id: &str| match &p.item(id).unwrap().body {
        ItemBody::Clip { at, dur, .. } => (*at, *dur),
        _ => (-1.0, -1.0),
    };
    assert_eq!((dur("title"), dur("title-b")), ((0.0, 0.5), (0.5, 1.5)));

    let voice = HistoryRequest {
        author: Some(Author::Voice),
    };
    f.session.undo(&voice).unwrap();
    assert!(f.session.project().unwrap().item("title-b").is_none());
    f.session.redo(&voice).unwrap();
    assert!(f.session.project().unwrap().item("title-b").is_some());
}

#[test]
fn bad_splits_and_agent_locks_are_refused() {
    let f = Fixture::new();
    for at in [0.0, 2.0, 5.0] {
        assert_eq!(
            f.session.split(&split(at, None)).unwrap_err().code,
            ErrorCode::Request,
            "{at}"
        );
    }
    let err = f
        .session
        .split(&split(1.0, Some(Author::Agent)))
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Request);
    f.session
        .lock(&LockRequest {
            holder: Holder::Agent,
            layer_ids: vec!["title".into()],
        })
        .unwrap();
    let err = f.session.split(&split(1.0, None)).unwrap_err();
    assert_eq!(err.code, ErrorCode::LockHeld);
    let err = f.session.undo(&HistoryRequest::default()).unwrap_err();
    assert_eq!(err.code, ErrorCode::HistoryEmpty);
}
