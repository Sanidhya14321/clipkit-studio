//! Docs and tool agree: every `ck` command the shipped skills, adapters and README name exists,
//! and every brief field they name is one `brief propose` accepts.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

use std::path::{Path, PathBuf};
use std::process::Command;

use ck_assets::recipe::BRIEF_FIELDS;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            walk(&p, out);
        } else if p
            .extension()
            .is_some_and(|x| ["md", "mdc", "toml"].contains(&x.to_str().unwrap()))
        {
            out.push(p);
        }
    }
}

fn docs() -> Vec<(String, String)> {
    let mut files = vec![root().join("README.md")];
    walk(&root().join("plugin"), &mut files);
    walk(&root().join("adapters"), &mut files);
    files
        .into_iter()
        .map(|p| {
            (
                p.display().to_string(),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

fn help(args: &[&str]) -> Option<String> {
    let mut all = args.to_vec();
    all.push("--help");
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(&all)
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| String::from_utf8(out.stdout).unwrap())
}

/// Every `ck word [word]` in `text`, word by word.
fn commands(text: &str) -> Vec<(String, Option<String>)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let clean = |w: &str| {
        w.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-')
            .to_owned()
    };
    words
        .windows(3)
        .filter(|w| w[0].trim_start_matches(['`', '(', '"']) == "ck")
        .filter(|w| !["command", "MCP"].contains(&clean(w[1]).as_str()))
        .map(|w| {
            (
                clean(w[1]),
                Some(clean(w[2].split('|').next().unwrap_or(""))),
            )
        })
        .collect()
}

#[test]
fn every_documented_command_exists() {
    let mut missing = Vec::new();
    for (file, text) in docs() {
        for (cmd, sub) in commands(&text) {
            let Some(h) = help(&[&cmd]) else {
                missing.push(format!("{file}: ck {cmd}"));
                continue;
            };
            // Only a command with subcommands constrains the next word, and only when the doc
            // writes it as code or a command line rather than prose.
            let sub = sub.unwrap();
            if h.contains("Commands:")
                && text.contains(&format!("ck {cmd} {sub}"))
                && !sub.is_empty()
                && help(&[&cmd, &sub]).is_none()
                && !PROSE.contains(&sub.as_str())
            {
                missing.push(format!("{file}: ck {cmd} {sub}"));
            }
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}

/// Words that follow a group command in sentences ("ck pipeline by small patches").
const PROSE: &[&str] = &[
    "by", "with", "and", "or", "is", "the", "to", "for", "commands", "first",
];

#[test]
fn every_documented_brief_field_exists() {
    let mut missing = Vec::new();
    for (file, text) in docs() {
        for part in text.split("brief.").skip(1) {
            let name: String = part.chars().take_while(char::is_ascii_alphabetic).collect();
            if !name.is_empty() && name != "json" && !BRIEF_FIELDS.contains(&name.as_str()) {
                missing.push(format!("{file}: brief.{name}"));
            }
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}
