//! Asset cards (spec 7.2): about 250 to 400 tokens, front matter then fixed body sections.
//! The agent reads cards instead of sources. Regeneration keeps the owner-notes section.
//!
//! Front matter is `key: value` lines between `---` lines; lists are `[a, b]` (no YAML crate,
//! like the taste files). Card text is data, never instructions.

use std::path::Path;

use ck_core::CkError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::asset_error;
use crate::taste::parse_list;

/// Most bytes a card may have (about 600 tokens; cards aim for 250 to 400).
pub const MAX_CARD_BYTES: usize = 2400;

/// Body sections, in order. "Owner notes" is never overwritten by regeneration.
pub const SECTIONS: &[&str] = &[
    "What it is",
    "Use when",
    "Avoid when",
    "Inputs",
    "Safe to adapt",
    "Structural parts",
    "Cannot do",
    "Pairs well with",
    "Taste notes",
    "Adaptation presets",
    "Owner notes",
];

macro_rules! kebab_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $v:ident = $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name { $($(#[$vm])* $v),+ }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$v),+];

            pub fn as_str(self) -> &'static str {
                match self { $($name::$v => $s),+ }
            }

            pub fn parse(s: &str) -> Option<Self> {
                match s { $($s => Some($name::$v),)+ _ => None }
            }
        }
    };
}

kebab_enum!(
    /// What an asset is.
    AssetKind {
        RemotionClip = "remotion-clip",
        ReactComponent = "react-component",
        CssEffect = "css-effect",
        Shader = "shader",
        Lottie = "lottie",
        Lut = "lut",
        OverlayVideo = "overlay-video",
        Sfx = "sfx",
        Font = "font",
        Icon = "icon",
        AeReference = "ae-reference",
        PremiereReference = "premiere-reference",
    }
);

kebab_enum!(
    /// How an asset may be used.
    Runtime {
        Direct = "direct",
        DeriveOnly = "derive-only",
        ReferenceOnly = "reference-only",
    }
);

kebab_enum!(
    /// Whether rendering a frame depends only on the frame number.
    Determinism {
        Native = "native",
        Wrappable = "wrappable",
        ReferenceOnly = "reference-only",
    }
);

kebab_enum!(
    /// License classes (spec 7.6).
    LicenseClass {
        Permissive = "permissive",
        Attribution = "attribution",
        Copyleft = "copyleft",
        /// For example Commons Clause or non-commercial terms.
        CommercialRestricted = "commercial-restricted",
        AccountRequired = "account-required",
        ReferenceOnly = "reference-only",
        Unknown = "unknown",
    }
);

kebab_enum!(
    /// License mode (spec 7.6): a brief field that search, recipes and export respect.
    LicenseMode {
        CommercialSafe = "commercial-safe",
        Personal = "personal",
        Any = "any",
    }
);

kebab_enum!(
    Grade {
        A = "A",
        B = "B",
        C = "C",
    }
);

impl LicenseMode {
    /// Whether an asset of `class` may be used (not just read about) in this mode. Unknown is
    /// never usable; reference-only assets are re-implemented natively, never used.
    pub fn allows(self, class: LicenseClass) -> bool {
        use LicenseClass as L;
        match (self, class) {
            (_, L::Unknown | L::ReferenceOnly) => false,
            (LicenseMode::CommercialSafe, L::Permissive | L::Attribution) => true,
            (LicenseMode::CommercialSafe, _) => false,
            (LicenseMode::Personal | LicenseMode::Any, _) => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    /// `<source>/<asset>`.
    pub id: String,
    pub name: String,
    pub source: String,
    pub source_commit: String,
    pub kind: AssetKind,
    pub runtime: Runtime,
    pub determinism: Determinism,
    /// The license as the source states it.
    pub license: String,
    /// SPDX id, or empty for custom and unknown licenses (never a guess).
    pub spdx: String,
    pub license_class: LicenseClass,
    pub grade: Grade,
    pub deps: Vec<String>,
    /// Relative cost to render, 1 (cheap) to 5.
    pub cost: u8,
    /// Natural length in seconds, when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    pub aspects: Vec<String>,
    pub tags: Vec<String>,
    pub collections: Vec<String>,
    pub reviewed: bool,
    /// Credit line for attribution licenses (CC BY and similar).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    /// Section title to text, in [`SECTIONS`] order.
    pub sections: Vec<(String, String)>,
}

fn one<T>(path: &Path, field: &str, v: Option<T>) -> Result<T, CkError> {
    v.ok_or_else(|| asset_error(path, format!("missing `{field}`")))
}

fn list(path: &Path, field: &str, value: &str) -> Result<Vec<String>, CkError> {
    if value.trim() == "[]" {
        return Ok(Vec::new());
    }
    parse_list(value).ok_or_else(|| asset_error(path, format!("`{field}` must be a [list]")))
}

fn enum_of<T>(
    path: &Path,
    field: &str,
    value: &str,
    parse: fn(&str) -> Option<T>,
) -> Result<T, CkError> {
    parse(value).ok_or_else(|| asset_error(path, format!("`{field}` has unknown value `{value}`")))
}

impl Card {
    pub fn parse(path: &Path, text: &str) -> Result<Self, CkError> {
        if text.len() > MAX_CARD_BYTES {
            return Err(asset_error(
                path,
                format!("{} bytes, over the {MAX_CARD_BYTES} byte cap", text.len()),
            ));
        }
        let rest = text
            .strip_prefix("---\n")
            .ok_or_else(|| asset_error(path, "missing front matter"))?;
        let (front, body) = rest
            .split_once("\n---\n")
            .ok_or_else(|| asset_error(path, "unclosed front matter"))?;
        let mut f: std::collections::BTreeMap<&str, &str> = Default::default();
        for line in front.lines() {
            let (k, v) = line
                .split_once(':')
                .ok_or_else(|| asset_error(path, format!("bad front matter line `{line}`")))?;
            if f.insert(k.trim(), v.trim()).is_some() {
                return Err(asset_error(path, format!("`{}` twice", k.trim())));
            }
        }
        let mut take = |k: &str| f.remove(k);
        let card = Self {
            id: one(path, "id", take("id"))?.to_owned(),
            name: one(path, "name", take("name"))?.to_owned(),
            source: one(path, "source", take("source"))?.to_owned(),
            source_commit: one(path, "sourceCommit", take("sourceCommit"))?.to_owned(),
            kind: enum_of(
                path,
                "kind",
                one(path, "kind", take("kind"))?,
                AssetKind::parse,
            )?,
            runtime: enum_of(
                path,
                "runtime",
                one(path, "runtime", take("runtime"))?,
                Runtime::parse,
            )?,
            determinism: enum_of(
                path,
                "determinism",
                one(path, "determinism", take("determinism"))?,
                Determinism::parse,
            )?,
            license: one(path, "license", take("license"))?.to_owned(),
            spdx: one(path, "spdx", take("spdx"))?.to_owned(),
            license_class: enum_of(
                path,
                "licenseClass",
                one(path, "licenseClass", take("licenseClass"))?,
                LicenseClass::parse,
            )?,
            grade: enum_of(
                path,
                "grade",
                one(path, "grade", take("grade"))?,
                Grade::parse,
            )?,
            deps: list(path, "deps", one(path, "deps", take("deps"))?)?,
            cost: one(path, "cost", take("cost"))?
                .parse()
                .ok()
                .filter(|c| (1..=5).contains(c))
                .ok_or_else(|| asset_error(path, "`cost` must be 1 to 5"))?,
            duration: match take("duration") {
                Some(d) => Some(
                    d.parse()
                        .ok()
                        .filter(|d: &f64| d.is_finite() && *d > 0.0)
                        .ok_or_else(|| asset_error(path, "`duration` must be seconds"))?,
                ),
                None => None,
            },
            aspects: list(path, "aspects", one(path, "aspects", take("aspects"))?)?,
            tags: list(path, "tags", one(path, "tags", take("tags"))?)?,
            collections: list(
                path,
                "collections",
                one(path, "collections", take("collections"))?,
            )?,
            reviewed: match one(path, "reviewed", take("reviewed"))? {
                "true" => true,
                "false" => false,
                _ => return Err(asset_error(path, "`reviewed` must be true or false")),
            },
            attribution: take("attribution").map(str::to_owned),
            sections: sections(path, body)?,
        };
        if let Some(extra) = f.keys().next() {
            return Err(asset_error(path, format!("unknown field `{extra}`")));
        }
        if card.license_class == LicenseClass::Attribution && card.attribution.is_none() {
            return Err(asset_error(
                path,
                "attribution licenses need an `attribution` line",
            ));
        }
        if card.id
            != format!(
                "{}/{}",
                card.source,
                card.id.rsplit('/').next().unwrap_or("")
            )
        {
            return Err(asset_error(path, "id must be <source>/<asset>"));
        }
        Ok(card)
    }

    /// Text of a section, empty when missing.
    pub fn section(&self, title: &str) -> &str {
        self.sections
            .iter()
            .find(|(t, _)| t == title)
            .map_or("", |(_, b)| b.as_str())
    }

    /// The card as a file. `owner_notes` from an earlier version is kept as is.
    pub fn render(&self, owner_notes: Option<&str>) -> String {
        let l = |v: &[String]| format!("[{}]", v.join(", "));
        let mut s = String::from("---\n");
        let mut kv = |k: &str, v: &str| {
            s.push_str(k);
            s.push_str(": ");
            s.push_str(v);
            s.push('\n');
        };
        kv("id", &self.id);
        kv("name", &self.name);
        kv("source", &self.source);
        kv("sourceCommit", &self.source_commit);
        kv("kind", self.kind.as_str());
        kv("runtime", self.runtime.as_str());
        kv("determinism", self.determinism.as_str());
        kv("license", &self.license);
        kv("spdx", &self.spdx);
        kv("licenseClass", self.license_class.as_str());
        kv("grade", self.grade.as_str());
        kv("deps", &l(&self.deps));
        kv("cost", &self.cost.to_string());
        if let Some(d) = self.duration {
            kv("duration", &d.to_string());
        }
        kv("aspects", &l(&self.aspects));
        kv("tags", &l(&self.tags));
        kv("collections", &l(&self.collections));
        kv("reviewed", if self.reviewed { "true" } else { "false" });
        if let Some(a) = &self.attribution {
            kv("attribution", a);
        }
        s.push_str("---\n");
        for title in SECTIONS {
            let text = if *title == "Owner notes" {
                owner_notes.unwrap_or_else(|| self.section(title))
            } else {
                self.section(title)
            };
            s.push_str(&format!("\n## {title}\n\n{}\n", text.trim()));
        }
        s
    }
}

fn sections(path: &Path, body: &str) -> Result<Vec<(String, String)>, CkError> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            out.push((title.trim().to_owned(), String::new()));
        } else if let Some((_, text)) = out.last_mut() {
            text.push_str(line);
            text.push('\n');
        }
    }
    for (_, t) in &mut out {
        *t = t.trim().to_owned();
    }
    let titles: Vec<&str> = out.iter().map(|(t, _)| t.as_str()).collect();
    if titles != SECTIONS {
        return Err(asset_error(
            path,
            format!("sections must be {SECTIONS:?} in order, found {titles:?}"),
        ));
    }
    for (title, text) in &out {
        let needed = !matches!(
            title.as_str(),
            "Owner notes" | "Pairs well with" | "Adaptation presets"
        );
        if needed && text.is_empty() {
            return Err(asset_error(path, format!("section `{title}` is empty")));
        }
    }
    Ok(out)
}

/// One compact catalog line (`assets/catalog.jsonl`) per card, for search and listing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub source: String,
    pub kind: AssetKind,
    pub runtime: Runtime,
    pub license_class: LicenseClass,
    pub grade: Grade,
    pub aspects: Vec<String>,
    pub tags: Vec<String>,
    pub collections: Vec<String>,
    /// First line of "Use when".
    pub use_when: String,
}

impl From<&Card> for CatalogEntry {
    fn from(c: &Card) -> Self {
        Self {
            id: c.id.clone(),
            name: c.name.clone(),
            source: c.source.clone(),
            kind: c.kind,
            runtime: c.runtime,
            license_class: c.license_class,
            grade: c.grade,
            aspects: c.aspects.clone(),
            tags: c.tags.clone(),
            collections: c.collections.clone(),
            use_when: c
                .section("Use when")
                .lines()
                .next()
                .unwrap_or("")
                .trim_start_matches("- ")
                .to_owned(),
        }
    }
}

#[cfg(test)]
mod tests;
