//! Recipes: pre-wired scenes with typed slots (spec 7.1, 7.4, 7.5).
//!
//! A recipe names its slots and its layers. Each layer places one core clip at a fraction of the
//! scene's duration; its props refer to slots as `"{slot}"` and to look tokens as `"$role.name"`
//! (for example `"$colors.accent"`). The expander checks the slot values, fills the props, keeps
//! the token references (so a re-theme needs no agent tokens) and records provenance.

use std::path::Path;

pub use ck_core::model::Provenance;
use ck_core::model::{Item, ItemBody, Pass, TrackKind};
use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::look::Look;
use crate::{asset_error, list_files, read_file, stem};

/// Core clip ids a recipe layer may use. Phase 4 step 7 builds these natively in Remotion.
pub const CORE_CLIPS: &[&str] = &[
    "backdrop",
    "bullet-list",
    "device-frame",
    "glitch-text",
    "kinetic-text",
    "lower-third",
    "quote-card",
    "split-compare",
    "stat-counter",
];

/// Brief fields a layer may depend on (the essentials of spec 8.1).
pub const BRIEF_FIELDS: &[&str] = &[
    "goal", "cta", "audience", "platform", "aspect", "length", "pacing", "look", "voice",
];

/// Token roles a prop may refer to, with the names each role has in a look.
const TOKEN_ROLES: &[(&str, &[&str])] = &[
    (
        "colors",
        &[
            "background",
            "surface",
            "text",
            "mutedText",
            "accent",
            "accentText",
        ],
    ),
    ("fonts", &["display", "text"]),
    ("motion", &["enterEasing", "exitEasing", "stagger"]),
    ("size", &["radius", "spacingUnit", "typeScale"]),
];

/// Longest text slot, most list items and longest recipe file.
pub const MAX_TEXT_CHARS: usize = 160;
pub const MAX_LIST_ITEMS: usize = 8;
pub const MAX_RECIPE_BYTES: usize = 6 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    /// When an agent should pick this recipe, in one line.
    pub use_when: String,
    pub tags: Vec<String>,
    pub duration: DurationRange,
    pub slots: Vec<Slot>,
    pub layers: Vec<LayerTemplate>,
}

/// Scene duration in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DurationRange {
    pub default: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Slot {
    pub name: String,
    pub kind: SlotKind,
    pub required: bool,
    pub description: String,
    /// Text slots: most characters. List slots: most characters per item.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<usize>,
    /// List slots: least and most items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SlotKind {
    /// A non-empty string.
    Text,
    /// A finite number.
    Number,
    /// A list of non-empty strings.
    List,
    /// A source id from the project (an image or video the user supplied).
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LayerTrack {
    Video,
    Overlay,
}

impl From<LayerTrack> for TrackKind {
    fn from(t: LayerTrack) -> Self {
        match t {
            LayerTrack::Video => TrackKind::Video,
            LayerTrack::Overlay => TrackKind::Overlay,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LayerTemplate {
    /// Layer name, unique within the recipe; the layer id is `<prefix>-<name>`.
    pub name: String,
    pub purpose: String,
    pub track: LayerTrack,
    pub pass: Pass,
    /// One of `CORE_CLIPS`.
    pub clip: String,
    /// Start and end as fractions of the scene duration.
    pub start: f64,
    pub end: f64,
    #[serde(default)]
    pub props: serde_json::Map<String, Value>,
    pub brief_deps: Vec<String>,
}

/// A placed layer: the item and the kind of track it belongs on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExpandedLayer {
    pub track: TrackKind,
    #[schemars(with = "Value")]
    pub item: Item,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Expansion {
    pub layers: Vec<ExpandedLayer>,
    pub provenance: Provenance,
}

/// What to expand: where, how long, and the slot values.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpandInput<'a> {
    /// Prefix for layer ids; layer ids are `<prefix>-<layer name>`.
    pub prefix: &'a str,
    pub at: f64,
    /// Scene duration; the recipe default when `None`.
    pub duration: Option<f64>,
    pub slots: &'a serde_json::Map<String, Value>,
    /// Source ids in the project, for `source` slots.
    pub sources: &'a [String],
}

impl Recipe {
    /// Parses and validates one recipe file; `path` names the file in errors and gives the id.
    pub fn parse(path: &Path, text: &str) -> Result<Self, CkError> {
        if text.len() > MAX_RECIPE_BYTES {
            return Err(asset_error(
                path,
                format!("{} bytes, over {MAX_RECIPE_BYTES}", text.len()),
            ));
        }
        let recipe: Self = serde_json::from_str(text).map_err(|e| asset_error(path, e))?;
        recipe.validate(path)?;
        Ok(recipe)
    }

    fn validate(&self, path: &Path) -> Result<(), CkError> {
        let err = |what: String| Err(asset_error(path, what));
        if self.id != stem(path) {
            return err(format!("id `{}` must match the file name", self.id));
        }
        let d = self.duration;
        if !(d.min > 0.0 && d.min <= d.default && d.default <= d.max && d.max <= 60.0) {
            return err("duration needs 0 < min <= default <= max <= 60".into());
        }
        if self.slots.iter().all(|s| !s.required) {
            return err("at least one slot must be required".into());
        }
        let mut slot_names = Vec::new();
        for s in &self.slots {
            if slot_names.contains(&s.name.as_str()) {
                return err(format!("slot `{}` declared twice", s.name));
            }
            slot_names.push(s.name.as_str());
            let chars_ok = s
                .max_chars
                .is_none_or(|n| (1..=MAX_TEXT_CHARS).contains(&n));
            let needs_chars = matches!(s.kind, SlotKind::Text | SlotKind::List);
            if !chars_ok || needs_chars != s.max_chars.is_some() {
                return err(format!(
                    "slot `{}`: text and list slots need maxChars 1 to {MAX_TEXT_CHARS}, others none",
                    s.name
                ));
            }
            let is_list = s.kind == SlotKind::List;
            let items_ok = match (s.min_items, s.max_items) {
                (Some(lo), Some(hi)) => is_list && 1 <= lo && lo <= hi && hi <= MAX_LIST_ITEMS,
                (None, None) => !is_list,
                _ => false,
            };
            if !items_ok {
                return err(format!(
                    "slot `{}`: list slots need 1 <= minItems <= maxItems <= {MAX_LIST_ITEMS}, others none",
                    s.name
                ));
            }
        }
        if self.layers.is_empty() {
            return err("a recipe needs at least one layer".into());
        }
        let mut used = Vec::new();
        let mut names = Vec::new();
        for l in &self.layers {
            if names.contains(&l.name.as_str()) {
                return err(format!("layer `{}` declared twice", l.name));
            }
            names.push(&l.name);
            if !CORE_CLIPS.contains(&l.clip.as_str()) {
                return err(format!("layer `{}`: unknown clip `{}`", l.name, l.clip));
            }
            if !(0.0 <= l.start && l.start < l.end && l.end <= 1.0) {
                return err(format!("layer `{}`: need 0 <= start < end <= 1", l.name));
            }
            if let Some(dep) = l
                .brief_deps
                .iter()
                .find(|d| !BRIEF_FIELDS.contains(&d.as_str()))
            {
                return err(format!("layer `{}`: unknown brief field `{dep}`", l.name));
            }
            for value in l.props.values() {
                check_refs(value, &slot_names, &mut used)
                    .map_err(|what| asset_error(path, format!("layer `{}`: {what}", l.name)))?;
            }
        }
        if let Some(s) = slot_names.iter().find(|s| !used.contains(*s)) {
            return err(format!("slot `{s}` is not used by any layer"));
        }
        Ok(())
    }

    /// Builds the named layers for one scene. Fails on a missing, unknown or out-of-range slot;
    /// an optional slot left out drops the props that use it (never a placeholder value).
    pub fn expand(
        &self,
        look: &Look,
        text: &str,
        input: &ExpandInput,
    ) -> Result<Expansion, CkError> {
        let duration = input.duration.unwrap_or(self.duration.default);
        if !(self.duration.min..=self.duration.max).contains(&duration) {
            return Err(invalid(format!(
                "duration {duration} outside {} to {} for recipe `{}`",
                self.duration.min, self.duration.max, self.id
            )));
        }
        if !(input.at.is_finite() && input.at >= 0.0) {
            return Err(invalid(format!("at {} must be zero or more", input.at)));
        }
        if let Some(k) = input
            .slots
            .keys()
            .find(|k| !self.slots.iter().any(|s| &s.name == *k))
        {
            return Err(invalid(format!(
                "recipe `{}` has no slot `{k}`; ck recipe search lists its slots",
                self.id
            )));
        }
        let mut adaptations = Vec::new();
        for slot in &self.slots {
            match input.slots.get(&slot.name) {
                Some(v) => {
                    check_slot(slot, v, input.sources)?;
                    adaptations.push(slot.name.clone());
                }
                None if slot.required => {
                    return Err(invalid(format!(
                        "slot `{}` is required: {}",
                        slot.name, slot.description
                    )));
                }
                None => {}
            }
        }
        // The hash is of the recipe file, so it is FNV-1a 64 of `text`.
        let provenance = Provenance {
            source_asset: format!("recipe:{}", self.id),
            hash: fnv1a(text),
            mode: "recipe".into(),
            adaptations,
            license: "original".into(),
        };
        // Token references are kept as written; resolving them against the look proves they exist.
        let layers = self
            .layers
            .iter()
            .map(|l| {
                let mut props = serde_json::Map::new();
                for (k, v) in &l.props {
                    if let Some(v) = fill(v, input.slots, look)? {
                        props.insert(k.clone(), v);
                    }
                }
                let round = |x: f64| (x * 1000.0).round() / 1000.0;
                Ok(ExpandedLayer {
                    track: l.track.into(),
                    item: Item {
                        id: format!("{}-{}", input.prefix, l.name),
                        name: format!("{} {}", self.name, l.name),
                        purpose: l.purpose.clone(),
                        pass: Some(l.pass),
                        asset_ref: Some(format!("recipe:{}", self.id)),
                        brief_deps: l.brief_deps.clone(),
                        provenance: Some(provenance.clone()),
                        rev: 0,
                        body: ItemBody::Clip {
                            clip_id: Some(l.clip.clone()),
                            derived_id: None,
                            at: round(input.at + l.start * duration),
                            dur: round((l.end - l.start) * duration),
                            props: Value::Object(props),
                        },
                    },
                })
            })
            .collect::<Result<_, CkError>>()?;
        Ok(Expansion { layers, provenance })
    }
}

/// Loads every `*.json` recipe in `dir`, sorted by id.
pub fn load_recipes(dir: &Path) -> Result<Vec<Recipe>, CkError> {
    list_files(dir, "json")?
        .iter()
        .map(|path| Recipe::parse(path, &read_file(path)?))
        .collect()
}

fn invalid(what: String) -> CkError {
    CkError::new(ErrorCode::Request, what)
}

/// `"{slot}"` names a slot; `"$role.name"` names a token.
fn slot_ref(s: &str) -> Option<&str> {
    s.strip_prefix('{').and_then(|s| s.strip_suffix('}'))
}

fn check_refs<'a>(v: &'a Value, slots: &[&str], used: &mut Vec<&'a str>) -> Result<(), String> {
    match v {
        Value::String(s) => {
            if let Some(name) = slot_ref(s) {
                if !slots.contains(&name) {
                    return Err(format!("unknown slot `{name}`"));
                }
                used.push(name);
            } else if let Some(token) = s.strip_prefix('$')
                && !token_known(token)
            {
                return Err(format!("unknown token `${token}`"));
            }
            Ok(())
        }
        Value::Array(a) => a.iter().try_for_each(|v| check_refs(v, slots, used)),
        Value::Object(o) => o.values().try_for_each(|v| check_refs(v, slots, used)),
        _ => Ok(()),
    }
}

fn token_known(token: &str) -> bool {
    token.split_once('.').is_some_and(|(role, name)| {
        TOKEN_ROLES
            .iter()
            .any(|(r, names)| *r == role && names.contains(&name))
    })
}

/// Fills slot references; `None` when the value uses an optional slot that was left out.
fn fill(
    v: &Value,
    slots: &serde_json::Map<String, Value>,
    look: &Look,
) -> Result<Option<Value>, CkError> {
    Ok(match v {
        Value::String(s) => match slot_ref(s) {
            Some(name) => slots.get(name).cloned(),
            None => {
                if let Some(token) = s.strip_prefix('$') {
                    token_value(look, token).ok_or_else(|| {
                        CkError::new(
                            ErrorCode::Asset,
                            format!("look `{}` has no token `${token}`", look.id),
                        )
                    })?;
                }
                Some(v.clone())
            }
        },
        Value::Array(a) => {
            let mut out = Vec::with_capacity(a.len());
            for x in a {
                match fill(x, slots, look)? {
                    Some(x) => out.push(x),
                    None => return Ok(None),
                }
            }
            Some(Value::Array(out))
        }
        Value::Object(o) => {
            let mut out = serde_json::Map::new();
            for (k, x) in o {
                match fill(x, slots, look)? {
                    Some(x) => {
                        out.insert(k.clone(), x);
                    }
                    None => return Ok(None),
                }
            }
            Some(Value::Object(out))
        }
        _ => Some(v.clone()),
    })
}

/// The value a token reference resolves to in a look.
pub fn token_value(look: &Look, token: &str) -> Option<Value> {
    if !token_known(token) {
        return None;
    }
    let (role, name) = token.split_once('.')?;
    let source = match role {
        "size" => serde_json::to_value(look).ok()?,
        _ => serde_json::to_value(look).ok()?.get(role)?.clone(),
    };
    source.get(name).cloned()
}

fn check_slot(slot: &Slot, v: &Value, sources: &[String]) -> Result<(), CkError> {
    let bad = |what: &str| invalid(format!("slot `{}` {what}", slot.name));
    let max = slot.max_chars.unwrap_or(MAX_TEXT_CHARS);
    let text_ok = |s: &str| !s.trim().is_empty() && s.chars().count() <= max;
    match slot.kind {
        SlotKind::Text => match v.as_str() {
            Some(s) if text_ok(s) => Ok(()),
            _ => Err(bad(&format!(
                "needs non-empty text of at most {max} characters"
            ))),
        },
        SlotKind::Number => match v.as_f64() {
            Some(n) if n.is_finite() => Ok(()),
            _ => Err(bad("needs a number")),
        },
        SlotKind::List => {
            let lo = slot.min_items.unwrap_or(1);
            let hi = slot.max_items.unwrap_or(MAX_LIST_ITEMS);
            match v.as_array() {
                Some(a)
                    if (lo..=hi).contains(&a.len())
                        && a.iter().all(|x| x.as_str().is_some_and(text_ok)) =>
                {
                    Ok(())
                }
                _ => Err(bad(&format!(
                    "needs {lo} to {hi} items of non-empty text, at most {max} characters each"
                ))),
            }
        }
        SlotKind::Source => match v.as_str() {
            Some(s) if sources.iter().any(|x| x == s) => Ok(()),
            _ => Err(bad("needs the id of a source in the project")),
        },
    }
}

/// FNV-1a 64-bit hash as 16 hex digits (stable, no extra dependency).
pub fn fnv1a(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}
