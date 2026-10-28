//! Structured logging conventions (spec 10.4). The subscriber is set up by the binary (ck-cli);
//! library code only emits `tracing` spans and events with these field names.

use std::fmt;

use serde::{Serialize, Serializer};

/// Field names every call, patch, lock change, render and provider call carries where they apply.
pub mod fields {
    pub const TRACE_ID: &str = "trace_id";
    pub const RUN_ID: &str = "run_id";
    pub const TURN_ID: &str = "turn_id";
    pub const STAGE: &str = "stage";
    pub const PASS: &str = "pass";
    pub const LAYER_ID: &str = "layer_id";
    pub const REV: &str = "rev";
}

/// Context for one agent operation. Enter the returned span around the operation so every event
/// inside it carries these fields.
pub fn op_span(
    trace_id: &str,
    run_id: &str,
    turn_id: &str,
    stage: &str,
    pass: Option<&str>,
) -> tracing::Span {
    tracing::info_span!(
        "ck_op",
        trace_id = trace_id,
        run_id = run_id,
        turn_id = turn_id,
        stage = stage,
        pass = pass.unwrap_or("")
    )
}

/// A value that must never reach logs, traces or error messages: API keys, tokens, consent data.
/// `Debug`, `Display` and `Serialize` all print `[REDACTED]`; read the value with `expose`.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret<T>(T);

const REDACTED: &str = "[REDACTED]";

impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl<T> Serialize for Secret<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(REDACTED)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use serde_json::{Value, json};

    use super::*;
    use crate::model::Author;
    use crate::{Document, patch::PatchRequest};

    #[derive(Clone, Default)]
    struct Buf(Arc<Mutex<Vec<u8>>>);

    impl Write for Buf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn json_lines_carry_standard_fields_and_redact_secrets() {
        let buf = Buf::default();
        let writer = buf.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(false)
            .with_writer(move || writer.clone())
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let _g = op_span("tr1", "run1", "turn1", "passes", Some("text")).entered();
            let mut doc = Document::new(crate::model::tests::sample());
            let req = PatchRequest {
                base_rev: 0,
                layer_revs: Default::default(),
                ops: serde_json::from_value(
                    json!([{"op": "replace", "path": "/tracks/1/items/0/at", "value": 2.0}]),
                )
                .unwrap(),
                author: Author::Human,
                label: "move".into(),
            };
            doc.patch(&req, 1).unwrap();
            let key = Secret::new("sk-live-123");
            tracing::info!(api_key = %key, debug_key = ?key, "provider call");
        });

        let out = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
        assert!(!out.contains("sk-live-123"), "secret leaked: {out}");
        let lines: Vec<Value> = out
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        let commit = lines
            .iter()
            .find(|l| l["message"] == "revision committed")
            .unwrap();
        assert_eq!(commit[fields::REV], 1);
        assert!(commit["layer_ids"].as_str().unwrap().contains('b'));
        for name in [
            fields::TRACE_ID,
            fields::RUN_ID,
            fields::TURN_ID,
            fields::STAGE,
            fields::PASS,
        ] {
            assert!(
                commit["span"].get(name).is_some(),
                "missing {name}: {commit}"
            );
        }
        let call = lines
            .iter()
            .find(|l| l["message"] == "provider call")
            .unwrap();
        assert_eq!(call["api_key"], REDACTED);
        assert_eq!(call["debug_key"], REDACTED);
        assert_eq!(serde_json::to_value(Secret::new(5)).unwrap(), REDACTED);
    }
}
