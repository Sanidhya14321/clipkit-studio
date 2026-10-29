//! Comments (spec 8.4) and the scope gate (spec 4) through the session.

use ck_core::ErrorCode;
use ck_core::model::{Anchor, Author, CommentStatus, Holder};
use ck_core::patch::{PROJECT_LAYER, PatchRequest};
use serde_json::json;

use super::fixture::Fixture;
use crate::session::LockRequest;
use crate::session::comments::{
    CommentAddRequest, CommentListRequest, CommentReplyRequest, CommentResolveRequest,
};

fn on_layer(text: &str, layer: &str) -> CommentAddRequest {
    CommentAddRequest {
        text: text.into(),
        anchor: Anchor {
            layer_id: Some(layer.into()),
            ..Anchor::default()
        },
        author: Some(Author::Human),
    }
}

fn rename(f: &Fixture, name: &str) -> Result<crate::session::PatchResult, ck_core::CkError> {
    let rev = f.session.project().unwrap().rev;
    f.session.patch(&PatchRequest {
        base_rev: rev,
        layer_revs: Default::default(),
        ops: serde_json::from_value(
            json!([{ "op": "replace", "path": "/tracks/0/items/0/name", "value": name }]),
        )
        .unwrap(),
        author: Author::Agent,
        label: "rename".into(),
    })
}

#[test]
fn the_user_comments_while_the_agent_holds_the_project() {
    let f = Fixture::new();
    f.session
        .lock(&LockRequest {
            holder: Holder::Agent,
            layer_ids: vec![PROJECT_LAYER.into()],
        })
        .unwrap();
    let added = f
        .session
        .comments_add(&on_layer("Bigger", "title"))
        .unwrap();
    assert_eq!(
        (added.id.as_str(), added.status),
        ("c1", CommentStatus::Open)
    );

    let bundle = f
        .session
        .comments_list(&CommentListRequest::default())
        .unwrap();
    assert_eq!(bundle.comments.len(), 1);
    assert_eq!(bundle.layers[0].id, "title");

    let bad = f.session.comments_add(&on_layer("x", "nope")).unwrap_err();
    assert_eq!(bad.code, ErrorCode::NotFound);
    let empty = f.session.comments_add(&on_layer(" ", "title")).unwrap_err();
    assert_eq!(empty.code, ErrorCode::Request);
}

#[test]
fn an_agent_reply_starts_scoped_work_and_resolve_reports_the_change() {
    let f = Fixture::new();
    let id = f
        .session
        .comments_add(&on_layer("Rename it", "title"))
        .unwrap()
        .id;
    let replied = f
        .session
        .comments_reply(&CommentReplyRequest {
            id: id.clone(),
            text: "On it".into(),
            scope: Some(vec![]),
            author: None,
        })
        .unwrap();
    assert_eq!(replied.status, CommentStatus::Working);

    // Inside the scope (the anchored layer) is fine; the project fields are not.
    f.session
        .lock(&LockRequest {
            holder: Holder::Agent,
            layer_ids: vec!["title".into(), PROJECT_LAYER.into()],
        })
        .unwrap();
    rename(&f, "Hook").unwrap();
    let rev = f.session.project().unwrap().rev;
    let outside = f
        .session
        .patch(&PatchRequest {
            base_rev: rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(
                json!([{ "op": "replace", "path": "/name", "value": "Other" }]),
            )
            .unwrap(),
            author: Author::Agent,
            label: "rename project".into(),
        })
        .unwrap_err();
    assert_eq!(outside.code, ErrorCode::Scope);
    assert_eq!(outside.layer_ids, [PROJECT_LAYER]);

    let resolved = f
        .session
        .comments_resolve(&CommentResolveRequest {
            id: id.clone(),
            text: Some("Renamed".into()),
            author: None,
        })
        .unwrap();
    assert_eq!(resolved.status, CommentStatus::Resolved);
    assert_eq!(resolved.changed, ["title"]);
    let c = &f.session.project().unwrap().comments[0];
    assert_eq!(c.resolved_rev, Some(resolved.rev));
    assert_eq!(c.replies.len(), 2);

    // Resolved comments leave the bundle unless asked for; the gate lifts.
    let open = f
        .session
        .comments_list(&CommentListRequest::default())
        .unwrap();
    assert!(open.comments.is_empty());
    let all = f
        .session
        .comments_list(&CommentListRequest { all: true })
        .unwrap();
    assert_eq!(all.comments.len(), 1);
    rename(&f, "Free").unwrap();
}

#[test]
fn unknown_comments_and_scopes_are_refused() {
    let f = Fixture::new();
    let reply = |id: &str, scope: Option<Vec<String>>| CommentReplyRequest {
        id: id.into(),
        text: "x".into(),
        scope,
        author: None,
    };
    let err = f.session.comments_reply(&reply("c9", None)).unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
    f.session.comments_add(&on_layer("a", "title")).unwrap();
    let err = f
        .session
        .comments_reply(&reply("c1", Some(vec!["ghost".into()])))
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[test]
fn context_reports_what_the_studio_points_at_plus_open_comments() {
    use crate::session::context::{Hovered, StudioContext};

    let f = Fixture::new();
    let empty = f.session.context().unwrap();
    assert_eq!(
        (empty.studio.selection.clone(), empty.studio.playhead),
        (None, 0.0)
    );

    f.session
        .comments_add(&on_layer(&"long ".repeat(40), "title"))
        .unwrap();
    f.session
        .set_context(StudioContext {
            selection: Some("title".into()),
            hovered: Some(Hovered {
                layer_id: "title".into(),
                path: Some("headline".into()),
            }),
            playhead: 1.25,
            loop_range: Some([1.0, 2.0]),
        })
        .unwrap();
    let ctx = f.session.context().unwrap();
    assert_eq!(ctx.selection_name.as_deref(), Some("Title"));
    assert_eq!(ctx.studio.playhead, 1.25);
    assert_eq!(ctx.open_comments.len(), 1);
    assert_eq!(ctx.open_comments[0].text.chars().count(), 80);
    // About 300 tokens.
    assert!(serde_json::to_vec(&ctx).unwrap().len() < 1200);

    let bad = f
        .session
        .set_context(StudioContext {
            playhead: -1.0,
            ..StudioContext::default()
        })
        .unwrap_err();
    assert_eq!(bad.code, ErrorCode::Request);
}
