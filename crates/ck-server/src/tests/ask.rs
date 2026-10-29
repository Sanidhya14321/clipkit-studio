//! `ask` (spec 6.4, Lane C): a question card in the studio, the answer as the tool result.

use axum::http::StatusCode;
use ck_core::ErrorCode;
use serde_json::json;

use super::fixture::Fixture;
use crate::session::ask::{AnswerRequest, AskRequest};

fn question(q: &str, options: &[&str]) -> AskRequest {
    AskRequest {
        question: Some(q.into()),
        options: options.iter().map(|o| (*o).to_owned()).collect(),
        id: None,
        timeout_secs: Some(0),
    }
}

#[test]
fn a_question_is_answered_once_and_then_forgotten() {
    let f = Fixture::new();
    let id = f
        .session
        .ask_open(&question("Which logo?", &["Old", "New"]))
        .unwrap();
    assert_eq!(id, "q1");
    assert_eq!(f.session.asks_open().unwrap()[0].options, ["Old", "New"]);
    assert!(!f.session.ask_check(&id).unwrap().answered);

    let left = f
        .session
        .ask_answer(&AnswerRequest {
            id: id.clone(),
            answer: "New".into(),
        })
        .unwrap();
    assert!(left.is_empty());
    let got = f.session.ask_check(&id).unwrap();
    assert_eq!((got.answered, got.answer.as_deref()), (true, Some("New")));
    assert!(
        !f.session.ask_check(&id).unwrap().answered,
        "delivered once"
    );
}

#[test]
fn bad_questions_and_answers_are_refused() {
    let f = Fixture::new();
    let seven = ["a", "b", "c", "d", "e", "f", "g"];
    for req in [
        question(" ", &[]),
        question("Ok?", &seven),
        question("Ok?", &[""]),
        AskRequest::default(),
    ] {
        assert_eq!(
            f.session.ask_open(&req).unwrap_err().code,
            ErrorCode::Request
        );
    }
    let resume = AskRequest {
        id: Some("q9".into()),
        ..AskRequest::default()
    };
    assert_eq!(
        f.session.ask_open(&resume).unwrap_err().code,
        ErrorCode::NotFound
    );
    let err = f
        .session
        .ask_answer(&AnswerRequest {
            id: "q9".into(),
            answer: "x".into(),
        })
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[tokio::test]
async fn the_ask_call_blocks_until_the_studio_answers() {
    let f = Fixture::new();
    let session = f.session.clone();
    let answer = tokio::spawn(async move {
        loop {
            let open = session.asks_open().unwrap();
            if let Some(q) = open.first() {
                session
                    .ask_answer(&AnswerRequest {
                        id: q.id.clone(),
                        answer: "Blue".into(),
                    })
                    .unwrap();
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    });
    let (status, body) = f
        .call(
            "POST",
            "/api/ask",
            Some(
                json!({ "question": "Which colour?", "options": ["Blue", "Red"],
                         "timeoutSecs": 5 }),
            ),
        )
        .await;
    answer.await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["answered"], true);
    assert_eq!(body["answer"], "Blue");

    // Unanswered: the id comes back to keep waiting with.
    let (_, body) = f
        .call(
            "POST",
            "/api/ask",
            Some(json!({ "question": "Still there?", "timeoutSecs": 0 })),
        )
        .await;
    assert_eq!(
        (body["answered"].clone(), body["id"].clone()),
        (json!(false), json!("q2"))
    );
}
