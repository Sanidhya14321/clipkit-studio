//! The packed transcript (spec 8): phrases with times, marked silences and fillers, about
//! 12 KB per 10 minutes. One line per phrase or silence:
//! `#<first word> <start>-<end> text with (filler) words` and `~ <start>-<end> silence`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::transcript::{Transcript, Word};

/// A gap between words at least this long ends a phrase.
pub const PHRASE_GAP: f64 = 0.3;
/// A gap at least this long is marked as a silence line.
pub const SILENCE_GAP: f64 = 0.5;
/// Words marked as fillers (lowercase, punctuation stripped).
pub const FILLERS: &[&str] = &["um", "uh", "er", "ah", "erm", "hmm", "mm", "like"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PackedTranscript {
    pub words: usize,
    pub fillers: usize,
    pub lines: Vec<String>,
    /// Lines left out to keep the answer under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// Lowercase with surrounding punctuation stripped.
pub fn norm(text: &str) -> String {
    text.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'')
        .to_lowercase()
}

pub fn is_filler(w: &Word) -> bool {
    FILLERS.contains(&norm(&w.text).as_str())
}

/// Word index ranges `[first, last]` of each phrase.
pub fn phrases(words: &[Word]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut first = 0;
    for n in 1..=words.len() {
        let ends = n == words.len() || words[n].start - words[n - 1].end >= PHRASE_GAP;
        if ends {
            out.push((first, n - 1));
            first = n;
        }
    }
    if words.is_empty() {
        out.clear();
    }
    out
}

pub fn pack(t: &Transcript) -> PackedTranscript {
    let words = &t.words;
    let mut lines = Vec::new();
    let mut prev_end = 0.0;
    for (a, b) in phrases(words) {
        let start = words[a].start;
        if start - prev_end >= SILENCE_GAP {
            lines.push(format!("~ {prev_end:.2}-{start:.2} silence"));
        }
        let text: Vec<String> = words[a..=b]
            .iter()
            .map(|w| {
                if is_filler(w) {
                    format!("({})", w.text)
                } else {
                    w.text.clone()
                }
            })
            .collect();
        lines.push(format!(
            "#{a} {start:.2}-{:.2} {}",
            words[b].end,
            text.join(" ")
        ));
        prev_end = words[b].end;
    }
    PackedTranscript {
        words: words.len(),
        fillers: words.iter().filter(|w| is_filler(w)).count(),
        lines,
        omitted: 0,
    }
}
