//! The content map: what the agent reads instead of the transcript. Code finds phrases, fillers,
//! pauses and retakes; the agent marks key lines from it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cuts::{CutKind, suggest_cuts};
use crate::transcript::{Transcript, Word};

/// A pause this long ends a phrase, seconds.
const PHRASE_GAP: f64 = 0.25;
/// A phrase is split at this many words.
const MAX_PHRASE_WORDS: usize = 12;
/// Words shorter than this are never emphasis.
const EMPHASIS_CHARS: usize = 6;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Phrase {
    pub start: f64,
    pub end: f64,
    pub text: String,
    /// Start of the phrase's emphasis word.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emphasis: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContentMap {
    pub words: usize,
    pub fillers: usize,
    pub retakes: usize,
    pub pauses: usize,
    /// Phrase index most fit to open: the first question, else the first phrase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hook: Option<usize>,
    /// `index start-end: text`; phrase i runs between grid anchors i and i+1.
    pub lines: Vec<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

fn ends_sentence(w: &Word) -> bool {
    w.text.ends_with(['.', '?', '!'])
}

fn close(words: &[Word]) -> Option<Phrase> {
    let (first, last) = (words.first()?, words.last()?);
    let emphasis = words
        .iter()
        .filter(|w| w.text.trim_matches(|c: char| !c.is_alphanumeric()).len() >= EMPHASIS_CHARS)
        .max_by_key(|w| w.text.len())
        .map(|w| w.start);
    Some(Phrase {
        start: first.start,
        end: last.end,
        text: words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        emphasis,
    })
}

/// The transcript's phrases, split at pauses, sentence ends and a word limit.
pub fn phrases(t: &Transcript) -> Vec<Phrase> {
    let mut out = Vec::new();
    let mut from = 0;
    for (n, w) in t.words.iter().enumerate() {
        let gap = t.words.get(n + 1).map(|x| x.start - w.end);
        let split = gap.is_none_or(|g| g >= PHRASE_GAP)
            || ends_sentence(w)
            || n + 1 - from >= MAX_PHRASE_WORDS;
        if split {
            out.extend(close(&t.words[from..=n]));
            from = n + 1;
        }
    }
    out
}

pub fn content_map(t: &Transcript, duration: f64) -> ContentMap {
    let found = phrases(t);
    let cuts = suggest_cuts(t, duration);
    let count = |k: CutKind| cuts.iter().filter(|c| c.kind == k).count();
    ContentMap {
        words: t.words.len(),
        fillers: count(CutKind::Filler),
        retakes: count(CutKind::Retake),
        pauses: count(CutKind::Silence),
        hook: found
            .iter()
            .position(|p| p.text.ends_with('?'))
            .or((!found.is_empty()).then_some(0)),
        lines: found
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{i} {:.2}-{:.2}: {}", p.start, p.end, p.text))
            .collect(),
        omitted: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(i: usize, text: &str, start: f64, end: f64) -> Word {
        Word {
            i,
            text: text.into(),
            start,
            end,
        }
    }

    fn sample() -> Transcript {
        Transcript {
            language: "en".into(),
            words: vec![
                word(0, "Ready", 0.0, 0.3),
                word(1, "now.", 0.3, 0.6),
                word(2, "Why", 0.7, 0.9),
                word(3, "shipping?", 0.9, 1.3),
                word(4, "Because", 2.0, 2.4),
                word(5, "customers", 2.4, 3.0),
                word(6, "wait", 3.0, 3.2),
            ],
        }
    }

    #[test]
    fn phrases_split_at_sentences_and_pauses_and_mark_emphasis() {
        let p = phrases(&sample());
        assert_eq!(p.len(), 3);
        assert_eq!(p[2].text, "Because customers wait");
        assert_eq!(p[2].emphasis, Some(2.4));
    }

    #[test]
    fn map_names_the_question_as_the_hook() {
        let m = content_map(&sample(), 3.5);
        assert_eq!(m.hook, Some(1));
        assert_eq!(m.lines[1], "1 0.70-1.30: Why shipping?");
    }
}
