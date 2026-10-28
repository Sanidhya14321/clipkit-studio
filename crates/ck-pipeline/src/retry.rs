//! The retry gate (spec 4.2): the same failing call with the same arguments is refused after
//! two failed attempts, with a hint to read the error. A success clears the count.

use std::collections::HashMap;

use ck_core::{CkError, CkResult, ErrorCode};
use serde_json::Value;

pub const MAX_FAILED_ATTEMPTS: u32 = 2;

#[derive(Debug, Default)]
pub struct RetryGate {
    failures: HashMap<String, u32>,
}

impl RetryGate {
    /// Call before running `op`. Refuses once the same call has failed `MAX_FAILED_ATTEMPTS` times.
    pub fn check(&self, op: &str, args: &Value) -> CkResult<()> {
        let n = self.failures.get(&key(op, args)).copied().unwrap_or(0);
        if n >= MAX_FAILED_ATTEMPTS {
            return Err(CkError::new(
                ErrorCode::Retry,
                format!("{op} failed {n} times with these exact arguments"),
            ));
        }
        Ok(())
    }

    /// Call after running `op` with its outcome.
    pub fn record(&mut self, op: &str, args: &Value, ok: bool) {
        let k = key(op, args);
        if ok {
            self.failures.remove(&k);
        } else {
            *self.failures.entry(k).or_default() += 1;
        }
    }
}

/// Same call = same op and same arguments, whatever the key order.
fn key(op: &str, args: &Value) -> String {
    format!("{op}:{}", canonical(args))
}

fn canonical(v: &Value) -> String {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", Value::String(k.clone()), canonical(&map[k])))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn third_identical_failing_call_is_refused() {
        let mut g = RetryGate::default();
        let args = json!({"layer": "a", "at": 1.0});
        for _ in 0..2 {
            g.check("patch", &args).unwrap();
            g.record("patch", &args, false);
        }
        assert_eq!(g.check("patch", &args).unwrap_err().code, ErrorCode::Retry);
        // Same arguments in another key order are the same call.
        assert!(g.check("patch", &json!({"at": 1.0, "layer": "a"})).is_err());
        // Different arguments or another op are fine.
        g.check("patch", &json!({"layer": "a", "at": 2.0})).unwrap();
        g.check("lock", &args).unwrap();
    }

    #[test]
    fn success_resets_the_count() {
        let mut g = RetryGate::default();
        let args = json!([1, {"b": 2, "a": 1}]);
        g.record("x", &args, false);
        g.record("x", &args, true);
        g.record("x", &args, false);
        g.check("x", &args).unwrap();
    }
}
