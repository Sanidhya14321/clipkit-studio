//! `ask` (spec 6.4, Lane C): the agent asks the user a question with options; the studio shows
//! it as a card with the options and an open microphone, and the answer returns as the tool
//! result. Questions live in memory (UI state, not a revision); an unanswered `ask` returns
//! its id so the agent can keep waiting with `ask --id`.

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Session;
use crate::event::Event;

/// Most options on one question.
pub const MAX_OPTIONS: usize = 6;
/// Longest question, option or answer, in bytes.
pub const MAX_ASK_TEXT: usize = 500;
/// Most open questions at once.
pub const MAX_OPEN: usize = 8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Ask {
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AskRequest {
    /// The question; omit it with `id` to keep waiting on an earlier one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    /// Keep waiting for the answer to this earlier question.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Seconds to block (default 20, at most 25); 0 checks once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AskResult {
    pub id: String,
    pub answered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnswerRequest {
    pub id: String,
    pub answer: String,
}

#[derive(Debug, Default)]
pub struct Asks {
    next: u64,
    open: Vec<Ask>,
    answers: Vec<(String, String)>,
}

fn too_long(s: &str) -> bool {
    s.trim().is_empty() || s.len() > MAX_ASK_TEXT
}

fn bad(msg: &str) -> CkError {
    CkError::new(ErrorCode::Request, msg.to_owned())
}

impl Session {
    fn asks(&self) -> CkResult<std::sync::MutexGuard<'_, Asks>> {
        self.asks
            .lock()
            .map_err(|_| CkError::new(ErrorCode::Server, "ask state poisoned"))
    }

    fn publish_asks(&self, asks: &Asks) {
        self.publish(Event::Asks {
            asks: asks.open.clone(),
        });
    }

    /// Open the question (or find the earlier one) and return its id.
    pub fn ask_open(&self, req: &AskRequest) -> CkResult<String> {
        let mut asks = self.asks()?;
        if let Some(id) = &req.id {
            let known =
                asks.open.iter().any(|a| &a.id == id) || asks.answers.iter().any(|(a, _)| a == id);
            return if known {
                Ok(id.clone())
            } else {
                Err(CkError::new(
                    ErrorCode::NotFound,
                    format!("no question {id:?}"),
                ))
            };
        }
        let question = req
            .question
            .as_deref()
            .ok_or_else(|| bad("ask needs a question (or an id to keep waiting)"))?;
        if too_long(question) || req.options.len() > MAX_OPTIONS {
            return Err(bad("a question is 1 to 500 bytes with at most 6 options"));
        }
        if req.options.iter().any(|o| too_long(o)) {
            return Err(bad("each option is 1 to 500 bytes"));
        }
        if asks.open.len() >= MAX_OPEN {
            return Err(bad("too many open questions; wait for answers first"));
        }
        asks.next += 1;
        let id = format!("q{}", asks.next);
        asks.open.push(Ask {
            id: id.clone(),
            question: question.trim().to_owned(),
            options: req.options.iter().map(|o| o.trim().to_owned()).collect(),
        });
        self.publish_asks(&asks);
        Ok(id)
    }

    /// The answer when it has arrived (delivered once, then forgotten).
    pub fn ask_check(&self, id: &str) -> CkResult<AskResult> {
        let mut asks = self.asks()?;
        if let Some(n) = asks.answers.iter().position(|(a, _)| a == id) {
            let (_, answer) = asks.answers.remove(n);
            return Ok(AskResult {
                id: id.to_owned(),
                answered: true,
                answer: Some(answer),
            });
        }
        Ok(AskResult {
            id: id.to_owned(),
            answered: false,
            answer: None,
        })
    }

    /// The open questions, for a studio that just connected.
    pub fn asks_open(&self) -> CkResult<Vec<Ask>> {
        Ok(self.asks()?.open.clone())
    }

    /// The studio answers: an option or free text (typed or spoken).
    pub fn ask_answer(&self, req: &AnswerRequest) -> CkResult<Vec<Ask>> {
        if too_long(&req.answer) {
            return Err(bad("an answer is 1 to 500 bytes"));
        }
        let mut asks = self.asks()?;
        let n = asks
            .open
            .iter()
            .position(|a| a.id == req.id)
            .ok_or_else(|| {
                CkError::new(
                    ErrorCode::NotFound,
                    format!("no open question {:?}", req.id),
                )
            })?;
        asks.open.remove(n);
        asks.answers
            .push((req.id.clone(), req.answer.trim().to_owned()));
        self.publish_asks(&asks);
        Ok(asks.open.clone())
    }
}
