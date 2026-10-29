//! A block between two marker lines in a text file the user owns (Codex config.toml and
//! AGENTS.md). Init adds or replaces only the block; uninstall cuts it out and leaves every other
//! byte as it was.

use ck_core::{CkError, CkResult, ErrorCode};

pub struct Markers {
    pub begin: &'static str,
    pub end: &'static str,
}

/// Byte range of the marked block (begin line through end line and its newline), if present.
fn find(text: &str, m: &Markers) -> CkResult<Option<(usize, usize)>> {
    let mut offset = 0;
    let mut start = None;
    for line in text.split_inclusive('\n') {
        let next = offset + line.len();
        let bare = line.trim_end();
        match start {
            None if bare == m.begin => start = Some(offset),
            Some(s) if bare == m.end => return Ok(Some((s, next))),
            _ => {}
        }
        offset = next;
    }
    match start {
        None => Ok(None),
        Some(_) => Err(CkError::new(
            ErrorCode::Request,
            format!("found {:?} without {:?}", m.begin, m.end),
        )
        .with_hint("Remove the partial ClipKit block by hand, then run ck init again.")),
    }
}

/// Whether `text` holds the marked block.
pub fn contains(text: &str, m: &Markers) -> CkResult<bool> {
    Ok(find(text, m)?.is_some())
}

/// `text` with the block holding `body`, replaced in place or appended at the end.
pub fn upsert(text: &str, m: &Markers, body: &str) -> CkResult<String> {
    let block = format!("{}\n{}\n{}\n", m.begin, body.trim_end(), m.end);
    Ok(match find(text, m)? {
        Some((s, e)) => format!("{}{block}{}", &text[..s], &text[e..]),
        None if text.is_empty() || text.ends_with('\n') => format!("{text}{block}"),
        None => format!("{text}\n{block}"),
    })
}

/// `text` without the block.
pub fn remove(text: &str, m: &Markers) -> CkResult<String> {
    Ok(match find(text, m)? {
        Some((s, e)) => format!("{}{}", &text[..s], &text[e..]),
        None => text.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    const M: Markers = Markers {
        begin: "# begin",
        end: "# end",
    };

    #[test]
    fn upsert_then_remove_restores_the_text() {
        for text in ["", "a = 1\n", "[x]\ny = 2\n\n"] {
            let added = upsert(text, &M, "body\n").unwrap();
            assert!(added.starts_with(text));
            assert!(added.ends_with("# begin\nbody\n# end\n"));
            assert_eq!(remove(&added, &M).unwrap(), text);
        }
    }

    #[test]
    fn upsert_replaces_in_place_and_is_idempotent() {
        let text = "a\n# begin\nold\n# end\nb\n";
        let new = upsert(text, &M, "new").unwrap();
        assert_eq!(new, "a\n# begin\nnew\n# end\nb\n");
        assert_eq!(upsert(&new, &M, "new").unwrap(), new);
        assert_eq!(remove(&new, &M).unwrap(), "a\nb\n");
    }

    #[test]
    fn missing_newline_gets_one_before_the_block() {
        assert_eq!(upsert("a", &M, "b").unwrap(), "a\n# begin\nb\n# end\n");
    }

    #[test]
    fn a_begin_without_end_is_refused() {
        let e = upsert("# begin\nx\n", &M, "b").unwrap_err();
        assert_eq!(e.code, ErrorCode::Request);
        assert!(remove("# begin\n", &M).is_err());
    }
}
