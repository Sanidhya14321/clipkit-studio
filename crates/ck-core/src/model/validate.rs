//! Structural validation of a project (the schema gate, spec 4.2). Quality checks live in ck-pipeline.

use std::collections::HashSet;

use super::{ItemBody, Project, SCHEMA_VERSION};
use crate::error::{CkError, CkResult, ErrorCode};

fn fail(msg: String) -> CkError {
    CkError::new(ErrorCode::Schema, msg)
}

fn time(field: &str, id: &str, v: f64) -> CkResult<()> {
    if v.is_finite() && v >= 0.0 {
        Ok(())
    } else {
        Err(fail(format!(
            "{id}.{field} must be a finite number >= 0, got {v}"
        )))
    }
}

impl Project {
    /// Check ids, references and numbers. Returns the first problem found.
    pub fn validate(&self) -> CkResult<()> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(fail(format!("schemaVersion must be {SCHEMA_VERSION}")));
        }
        let s = &self.settings;
        if s.fps == 0 || s.width == 0 || s.height == 0 {
            return Err(fail("settings.fps, width and height must be > 0".into()));
        }

        let mut ids = HashSet::new();
        let mut unique = |kind: &str, id: &str| {
            if id.is_empty() || id.starts_with('@') || !ids.insert(id.to_owned()) {
                Err(fail(format!(
                    "{kind} id {id:?} is empty, starts with '@' or is not unique"
                )))
            } else {
                Ok(())
            }
        };
        for src in &self.sources {
            unique("source", &src.id)?;
            if let Some(d) = src.duration_sec {
                time("durationSec", &src.id, d)?;
            }
        }
        for slot in &self.voice_slots {
            unique("voice slot", &slot.id)?;
        }
        for track in &self.tracks {
            unique("track", &track.id)?;
            for item in &track.items {
                unique("item", &item.id)?;
            }
        }

        let source_ids: HashSet<&str> = self.sources.iter().map(|s| s.id.as_str()).collect();
        let slot_ids: HashSet<&str> = self.voice_slots.iter().map(|s| s.id.as_str()).collect();
        for item in self.items() {
            let id = item.id.as_str();
            match &item.body {
                ItemBody::Footage {
                    src,
                    src_in,
                    src_out,
                    at,
                    speed,
                    volume,
                } => {
                    if !source_ids.contains(src.as_str()) {
                        return Err(fail(format!("{id}.src {src:?} is not a source id")));
                    }
                    time("srcIn", id, *src_in)?;
                    time("srcOut", id, *src_out)?;
                    time("at", id, *at)?;
                    time("volume", id, *volume)?;
                    if src_out <= src_in {
                        return Err(fail(format!("{id}.srcOut must be greater than srcIn")));
                    }
                    if !(speed.is_finite() && *speed > 0.0) {
                        return Err(fail(format!("{id}.speed must be > 0")));
                    }
                }
                ItemBody::Clip { at, dur, .. } => {
                    time("at", id, *at)?;
                    time("dur", id, *dur)?;
                    if *dur <= 0.0 {
                        return Err(fail(format!("{id}.dur must be > 0")));
                    }
                }
                ItemBody::Voice { slot_id, at } => {
                    if !slot_ids.contains(slot_id.as_str()) {
                        return Err(fail(format!("{id}.slotId {slot_id:?} is not a voice slot")));
                    }
                    time("at", id, *at)?;
                }
                ItemBody::Audio {
                    src,
                    at,
                    ranges,
                    volume,
                    ..
                } => {
                    if !source_ids.contains(src.as_str()) {
                        return Err(fail(format!("{id}.src {src:?} is not a source id")));
                    }
                    time("at", id, *at)?;
                    time("volume", id, *volume)?;
                    for r in ranges {
                        time("ranges.start", id, r.start)?;
                        time("ranges.end", id, r.end)?;
                        if r.end <= r.start {
                            return Err(fail(format!("{id}.ranges end must be after start")));
                        }
                    }
                }
                ItemBody::Caption { .. } => {}
            }
        }
        Ok(())
    }
}
