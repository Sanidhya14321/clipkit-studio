//! Taste profile (spec 8.8): a local, editable file built from what the user picks. Every brief
//! field the user sets (confirming or editing the brief, tuning chips) counts as a pick. The
//! profile fills brief fields the agent left out (source `profile`), never anything the agent
//! inferred or the user chose. It can be disabled and cleared.
//!
//! Location: `CK_PROFILE` when set, else `~/.clipkit/profile.json`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ck_core::model::{Brief, BriefSource};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROFILE_ENV: &str = "CK_PROFILE";
/// Fields the profile learns and suggests (not per-project text like the goal or CTA).
pub const LEARNED: &[&str] = &[
    "platform",
    "aspect",
    "length",
    "pacing",
    "look",
    "voice",
    "licenseMode",
];
/// A value needs this many picks before it becomes a default.
pub const MIN_PICKS: u32 = 2;

pub mod caps {
    pub const PROFILE: usize = 2 * 1024;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub enabled: bool,
    /// Field to (value as JSON text) to the number of times the user picked it.
    pub picks: BTreeMap<String, BTreeMap<String, u32>>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            enabled: true,
            picks: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileView {
    pub path: String,
    pub enabled: bool,
    /// Field to the default it would fill and how many picks back it.
    pub defaults: BTreeMap<String, (Value, u32)>,
    pub picks: usize,
}

/// `CK_PROFILE`, else `~/.clipkit/profile.json` (None when there is no home folder).
pub fn default_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os(PROFILE_ENV).filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(p));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".clipkit/profile.json"))
}

pub fn load(path: &Path) -> Profile {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, p: &Profile) -> CkResult<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", dir.display())))?;
    }
    let text =
        serde_json::to_vec_pretty(p).map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
    ck_core::storage::atomic_write(path, &text)
}

impl Profile {
    /// The most picked value per learned field, once it has at least `MIN_PICKS` picks.
    pub fn defaults(&self) -> BTreeMap<String, (Value, u32)> {
        self.picks
            .iter()
            .filter_map(|(field, values)| {
                let (v, n) = values
                    .iter()
                    .max_by_key(|(v, n)| (**n, std::cmp::Reverse(*v)))?;
                let value = serde_json::from_str(v).ok()?;
                (*n >= MIN_PICKS).then(|| (field.clone(), (value, *n)))
            })
            .collect()
    }

    /// Counts user-sourced learned fields that `after` changed relative to `before`.
    pub fn learn(&mut self, before: &Brief, after: &Brief) -> bool {
        let mut changed = false;
        for field in LEARNED {
            let Some(f) = after
                .fields
                .get(*field)
                .filter(|f| f.source == BriefSource::User)
            else {
                continue;
            };
            if before
                .fields
                .get(*field)
                .is_some_and(|b| b.value == f.value && b.source == f.source)
            {
                continue;
            }
            let key = f.value.to_string();
            *self
                .picks
                .entry((*field).to_owned())
                .or_default()
                .entry(key)
                .or_default() += 1;
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ck_core::model::BriefField;
    use serde_json::json;

    fn brief(fields: &[(&str, Value, BriefSource)]) -> Brief {
        let mut b = Brief::default();
        for (k, v, s) in fields {
            b.fields.insert(
                (*k).to_owned(),
                BriefField {
                    value: v.clone(),
                    source: *s,
                    confidence: 1.0,
                },
            );
        }
        b
    }

    #[test]
    fn user_picks_count_and_become_defaults_after_two() {
        let mut p = Profile::default();
        let empty = Brief::default();
        let picked = brief(&[
            ("pacing", json!("punchy"), BriefSource::User),
            ("look", json!("cinematic"), BriefSource::Inferred),
            ("goal", json!("Launch"), BriefSource::User),
        ]);
        assert!(p.learn(&empty, &picked));
        assert!(p.defaults().is_empty(), "one pick is not a habit yet");
        // The same brief again changes nothing; a second project counts.
        assert!(!p.learn(&picked, &picked));
        p.learn(&empty, &picked);
        assert_eq!(p.defaults()["pacing"], (json!("punchy"), 2));
        assert!(
            !p.defaults().contains_key("look"),
            "inferred values are not picks"
        );
        assert!(
            !p.picks.contains_key("goal"),
            "per-project text is not learned"
        );
    }
}

/// What `ck profile` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ProfileAction {
    #[default]
    Show,
    Clear,
    Disable,
    Enable,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileCommandRequest {
    #[serde(default)]
    pub action: ProfileAction,
}

/// `ck profile show|clear|disable|enable`.
pub fn run(path: Option<&Path>, req: &ProfileCommandRequest) -> CkResult<ProfileView> {
    let path = path.ok_or_else(|| {
        CkError::new(ErrorCode::NotFound, "no profile location")
            .with_hint("Set CK_PROFILE to a file path.")
    })?;
    let mut p = load(path);
    match req.action {
        ProfileAction::Show => {}
        ProfileAction::Clear => {
            p.picks.clear();
            save(path, &p)?;
        }
        ProfileAction::Disable => {
            p.enabled = false;
            save(path, &p)?;
        }
        ProfileAction::Enable => {
            p.enabled = true;
            save(path, &p)?;
        }
    }
    Ok(ProfileView {
        path: path.to_string_lossy().into_owned(),
        enabled: p.enabled,
        defaults: p.defaults(),
        picks: p
            .picks
            .values()
            .flat_map(|v| v.values())
            .map(|n| *n as usize)
            .sum(),
    })
}
