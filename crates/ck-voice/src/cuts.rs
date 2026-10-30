//! Cut candidates (spec 8): silences, fillers, false starts and repeated takes. Every cut
//! lands on word boundaries; the export adds the short fades at each join.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pack::{PHRASE_GAP, is_filler, norm, phrases};
use crate::transcript::{Transcript, Word};

/// A gap between words at least this long is a silence candidate.
pub const SILENCE_MIN: f64 = 0.5;
/// Pause kept on each side of a cut silence.
pub const SILENCE_PAD: f64 = 0.15;
/// Shortest repeated opening that marks an abandoned take.
pub const RETAKE_MIN_WORDS: usize = 2;
/// A retake must restart within this many seconds.
pub const RETAKE_WINDOW: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CutKind {
    Silence,
    Filler,
    /// An abandoned take that is said again right after (false start or repeated take).
    Retake,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CutCandidate {
    pub id: String,
    pub kind: CutKind,
    /// Source time range to remove, in seconds.
    pub start: f64,
    pub end: f64,
    /// Removed word index range `[first, last]`; absent for a silence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: Option<[usize; 2]>,
    /// 0.0 to 1.0.
    pub confidence: f64,
}

/// Contractions spelled out so "we'll" matches "we will".
fn tokens(words: &[Word]) -> Vec<String> {
    words
        .iter()
        .filter(|w| !is_filler(w))
        .flat_map(|w| {
            let n = norm(&w.text);
            let expanded = match n.split_once('\'') {
                Some((head, "ll")) => format!("{head} will"),
                Some((head, "re")) => format!("{head} are"),
                Some((head, "ve")) => format!("{head} have"),
                Some((head, "m")) => format!("{head} am"),
                Some((head, "d")) => format!("{head} would"),
                _ => n,
            };
            expanded.split(' ').map(str::to_owned).collect::<Vec<_>>()
        })
        .collect()
}

fn common_prefix(a: &[String], b: &[String]) -> usize {
    a.iter().zip(b).take_while(|(x, y)| x == y).count()
}

/// Candidates sorted by start, overlaps resolved in favour of the earlier, larger cut.
/// `duration` closes a trailing silence.
pub fn suggest_cuts(t: &Transcript, duration: f64) -> Vec<CutCandidate> {
    let w = &t.words;
    let mut cuts: Vec<CutCandidate> = Vec::new();
    let mut push = |kind, start: f64, end: f64, words, confidence| {
        if end > start {
            cuts.push(CutCandidate {
                id: String::new(),
                kind,
                start,
                end,
                words,
                confidence,
            });
        }
    };

    // Repeated takes: a phrase whose opening the next phrase says again.
    let ph = phrases(w);
    for pair in ph.windows(2) {
        let ((a0, a1), (b0, b1)) = (pair[0], pair[1]);
        let (ta, tb) = (tokens(&w[a0..=a1]), tokens(&w[b0..=b1]));
        let shared = common_prefix(&ta, &tb);
        if shared >= RETAKE_MIN_WORDS && w[b0].start - w[a0].start <= RETAKE_WINDOW {
            let conf = if shared >= 3 { 0.8 } else { 0.6 };
            push(
                CutKind::Retake,
                w[a0].start,
                w[b0].start,
                Some([a0, b0 - 1]),
                conf,
            );
        }
    }

    // Fillers: from the filler's start to the next word's start (or its own end at the end).
    for (n, word) in w.iter().enumerate() {
        if is_filler(word) {
            let end = w.get(n + 1).map_or(word.end, |next| next.start);
            let conf = if norm(&word.text) == "like" { 0.4 } else { 0.9 };
            push(CutKind::Filler, word.start, end, Some([n, n]), conf);
        }
    }

    // Silences: gaps between words (and before the first and after the last), keeping a pad.
    // No words means no speech evidence (music, failed STT), not one long silence.
    let mut edges: Vec<(f64, f64)> = Vec::new();
    let mut prev = 0.0;
    for word in w {
        edges.push((prev, word.start));
        prev = word.end;
    }
    if !w.is_empty() {
        edges.push((prev, duration.max(prev)));
    }
    for (n, (a, b)) in edges.into_iter().enumerate() {
        if b - a >= SILENCE_MIN.max(PHRASE_GAP) {
            let start = if n == 0 { a } else { a + SILENCE_PAD };
            let end = if b >= duration { b } else { b - SILENCE_PAD };
            push(CutKind::Silence, start, end, None, 0.9);
        }
    }

    // Earlier start first; at equal start the longer cut first. Drop cuts inside a kept one
    // and trim partial overlaps.
    cuts.sort_by(|a, b| a.start.total_cmp(&b.start).then(b.end.total_cmp(&a.end)));
    let mut out: Vec<CutCandidate> = Vec::new();
    for mut c in cuts {
        if let Some(last) = out.last() {
            if c.end <= last.end {
                continue;
            }
            if c.start < last.end {
                c.start = last.end;
            }
        }
        out.push(c);
    }
    for (n, c) in out.iter_mut().enumerate() {
        c.id = format!("cut-{}", n + 1);
    }
    out
}

/// The kept ranges of `[0, duration]` once the `(start, end)` cuts are removed.
pub fn kept_ranges(cuts: &[(f64, f64)], duration: f64) -> Vec<(f64, f64)> {
    let mut sorted = cuts.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut kept = Vec::new();
    let mut at = 0.0;
    for (s, e) in sorted {
        if s > at {
            kept.push((at, s.min(duration)));
        }
        at = at.max(e);
    }
    if at < duration {
        kept.push((at, duration));
    }
    kept.retain(|(s, e)| e - s > 1e-6);
    kept
}

#[cfg(test)]
mod tests;
