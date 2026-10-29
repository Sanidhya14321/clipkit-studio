//! Looks: design tokens plus a motion personality (spec 7.1, 7.5, 8.5).
//!
//! Sizes (radius, spacing unit) are fractions of frame height, like the type scale in the
//! typography taste file. Layers refer to color roles, never to hex values.

use std::path::Path;

use ck_core::CkError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::color::{contrast, parse_hex};
use crate::{asset_error, list_files, read_file, stem};

/// Minimum WCAG contrast for every text role on the color it sits on (spec 4.3 readable contrast).
pub const MIN_TEXT_CONTRAST: f64 = 4.5;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Look {
    pub id: String,
    pub name: String,
    pub description: String,
    pub colors: Colors,
    pub fonts: Fonts,
    /// Ratio between neighbouring type sizes.
    pub type_scale: f64,
    pub radius: f64,
    pub spacing_unit: f64,
    pub motion: Motion,
}

/// Color roles as `#RRGGBB`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Colors {
    pub background: String,
    pub surface: String,
    pub text: String,
    pub muted_text: String,
    pub accent: String,
    pub accent_text: String,
}

/// Two roles, so a look never uses more than two font families (spec 4.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Fonts {
    pub display: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum MotionPersonality {
    Snappy,
    Smooth,
    Calm,
    Slow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Easing {
    Linear,
    QuadIn,
    QuadOut,
    CubicIn,
    CubicOut,
    ExpoOut,
    SineInOut,
    BackOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Motion {
    pub personality: MotionPersonality,
    /// Multiplies the durations of the motion taste file.
    pub duration_scale: f64,
    pub enter_easing: Easing,
    pub exit_easing: Easing,
    /// Seconds between staggered entrances.
    pub stagger: f64,
}

impl Look {
    /// Parses and validates one look file; `path` names the file in errors and gives the id.
    pub fn parse(path: &Path, text: &str) -> Result<Self, CkError> {
        let look: Self = serde_json::from_str(text).map_err(|e| asset_error(path, e))?;
        look.validate(path)?;
        Ok(look)
    }

    fn validate(&self, path: &Path) -> Result<(), CkError> {
        if self.id != stem(path) {
            return Err(asset_error(
                path,
                format!("id `{}` must match the file name", self.id),
            ));
        }
        if self.fonts.display.trim().is_empty() || self.fonts.text.trim().is_empty() {
            return Err(asset_error(
                path,
                "fonts.display and fonts.text must be named",
            ));
        }
        let c = &self.colors;
        let hex = |field: &str, value: &str| {
            parse_hex(value).ok_or_else(|| asset_error(path, format!("colors.{field} `{value}`")))
        };
        let background = hex("background", &c.background)?;
        let surface = hex("surface", &c.surface)?;
        let text = hex("text", &c.text)?;
        let muted = hex("mutedText", &c.muted_text)?;
        let accent = hex("accent", &c.accent)?;
        let accent_text = hex("accentText", &c.accent_text)?;
        for (pair, fg, bg) in [
            ("text on background", text, background),
            ("text on surface", text, surface),
            ("mutedText on background", muted, background),
            ("accentText on accent", accent_text, accent),
        ] {
            let ratio = contrast(fg, bg);
            if ratio < MIN_TEXT_CONTRAST {
                return Err(asset_error(
                    path,
                    format!("{pair} contrast {ratio:.2}, below {MIN_TEXT_CONTRAST}"),
                ));
            }
        }
        let ranges = [
            ("typeScale", self.type_scale, 1.05, 1.62),
            ("radius", self.radius, 0.0, 0.1),
            ("spacingUnit", self.spacing_unit, 0.002, 0.05),
            ("motion.durationScale", self.motion.duration_scale, 0.5, 2.0),
            ("motion.stagger", self.motion.stagger, 0.0, 0.5),
        ];
        for (field, value, lo, hi) in ranges {
            if !(lo..=hi).contains(&value) {
                return Err(asset_error(
                    path,
                    format!("{field} {value} outside {lo} to {hi}"),
                ));
            }
        }
        Ok(())
    }
}

/// Loads every `*.json` look in `dir`, sorted by id.
pub fn load_looks(dir: &Path) -> Result<Vec<Look>, CkError> {
    list_files(dir, "json")?
        .iter()
        .map(|path| Look::parse(path, &read_file(path)?))
        .collect()
}
