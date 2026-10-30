//! Interchange exports (spec 8.6): OpenTimelineIO (`.otio`, JSON) and SRT subtitles.
//!
//! OTIO: footage becomes clips with external references to the original media, voice items
//! audio clips of their active take, generated clips generator references (`clipkit:<clip>`
//! with their props), gaps gaps. Overlapping items on one ClipKit track are spread over extra
//! OTIO tracks because an OTIO track plays its children one after another.
//!
//! SRT: speech with real word timing: voice takes with a transcript (from `voice captions`)
//! and footage whose source has a transcript, mapped through the cuts to timeline time.

use std::path::Path;

use ck_core::model::{Item, ItemBody, Project, TrackKind};
use ck_core::{CkError, CkResult, ErrorCode};
use serde_json::{Value, json};

use super::Session;

fn rt(seconds: f64, rate: f64) -> Value {
    json!({ "OTIO_SCHEMA": "RationalTime.1", "rate": rate, "value": (seconds * rate).round() })
}

fn range(start: f64, dur: f64, rate: f64) -> Value {
    json!({ "OTIO_SCHEMA": "TimeRange.1", "start_time": rt(start, rate), "duration": rt(dur, rate) })
}

fn gap(dur: f64, rate: f64) -> Value {
    json!({ "OTIO_SCHEMA": "Gap.1", "name": "", "source_range": range(0.0, dur, rate),
            "effects": [], "markers": [], "enabled": true, "metadata": {} })
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

/// Splits items into lanes where no two overlap (first fit by start time).
fn lanes<'a>(project: &Project, items: &'a [Item]) -> Vec<Vec<(&'a Item, f64, f64)>> {
    let mut spans: Vec<(&Item, f64, f64)> = items
        .iter()
        .filter_map(|i| ck_core::timeline::span(project, i).map(|s| (i, s.start, s.end)))
        .collect();
    spans.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut out: Vec<Vec<(&Item, f64, f64)>> = Vec::new();
    for s in spans {
        match out
            .iter_mut()
            .find(|l| l.last().is_none_or(|x| x.2 <= s.1 + 1e-9))
        {
            Some(l) => l.push(s),
            None => out.push(vec![s]),
        }
    }
    out
}

impl Session {
    fn otio_clip(&self, project: &Project, item: &Item, rate: f64) -> Option<Value> {
        let (media, source_range) = match &item.body {
            ItemBody::Footage {
                src,
                src_in,
                src_out,
                ..
            } => {
                let s = project.sources.iter().find(|s| &s.id == src)?;
                (
                    json!({ "OTIO_SCHEMA": "ExternalReference.1",
                            "target_url": file_url(&self.resolve(&s.path)),
                            "available_range": s.duration_sec.map(|d| range(0.0, d, rate)),
                            "metadata": { "clipkit": { "source": s.id, "hash": s.hash } } }),
                    range(*src_in, src_out - src_in, rate),
                )
            }
            ItemBody::Voice { slot_id, .. } => {
                let slot = project.voice_slots.iter().find(|s| &s.id == slot_id)?;
                let take = slot
                    .takes
                    .iter()
                    .find(|t| Some(&t.id) == slot.active_take.as_ref())?;
                (
                    json!({ "OTIO_SCHEMA": "ExternalReference.1",
                            "target_url": file_url(&self.resolve(&take.path)),
                            "available_range": range(0.0, take.duration_sec, rate),
                            "metadata": { "clipkit": { "slot": slot.id, "take": take.id } } }),
                    range(0.0, take.duration_sec, rate),
                )
            }
            ItemBody::Clip {
                clip_id,
                dur,
                props,
                ..
            } => (
                json!({ "OTIO_SCHEMA": "GeneratorReference.1",
                        "name": clip_id.clone().unwrap_or_default(),
                        "generator_kind": format!("clipkit:{}", clip_id.as_deref().unwrap_or("clip")),
                        "parameters": props,
                        "available_range": range(0.0, *dur, rate),
                        "metadata": {} }),
                range(0.0, *dur, rate),
            ),
            ItemBody::Audio { .. } | ItemBody::Caption { .. } => return None,
        };
        Some(json!({
            "OTIO_SCHEMA": "Clip.2",
            "name": if item.name.is_empty() { item.id.clone() } else { item.name.clone() },
            "source_range": source_range,
            "media_references": { "DEFAULT_MEDIA": media },
            "active_media_reference_key": "DEFAULT_MEDIA",
            "effects": [], "markers": [], "enabled": true,
            "metadata": { "clipkit": { "id": item.id } }
        }))
    }

    /// The project as an OpenTimelineIO timeline.
    pub fn otio(&self, project: &Project) -> Value {
        let rate = f64::from(project.settings.fps);
        let mut tracks = Vec::new();
        for track in &project.tracks {
            let kind = match track.kind {
                TrackKind::Audio => "Audio",
                TrackKind::Caption => continue,
                TrackKind::Video | TrackKind::Overlay => "Video",
            };
            for (n, lane) in lanes(project, &track.items).into_iter().enumerate() {
                let mut children = Vec::new();
                let mut at = 0.0;
                for (item, start, end) in lane {
                    let Some(clip) = self.otio_clip(project, item, rate) else {
                        continue;
                    };
                    if start - at > 1e-6 {
                        children.push(gap(start - at, rate));
                    }
                    children.push(clip);
                    at = end;
                }
                if children.is_empty() {
                    continue;
                }
                let name = if n == 0 {
                    track.id.clone()
                } else {
                    format!("{}-{}", track.id, n + 1)
                };
                tracks.push(json!({ "OTIO_SCHEMA": "Track.1", "name": name, "kind": kind,
                                    "children": children, "source_range": null,
                                    "effects": [], "markers": [], "enabled": true, "metadata": {} }));
            }
        }
        json!({
            "OTIO_SCHEMA": "Timeline.1",
            "name": project.name,
            "global_start_time": null,
            "tracks": { "OTIO_SCHEMA": "Stack.1", "name": "tracks", "children": tracks,
                        "source_range": null, "effects": [], "markers": [], "enabled": true,
                        "metadata": {} },
            "metadata": { "clipkit": { "project": project.id, "rev": project.rev } }
        })
    }

    /// Timed words on the timeline: voice takes' and footage sources' transcripts.
    pub fn timeline_words(&self, project: &Project) -> Vec<(f64, f64, String)> {
        let mut words = Vec::new();
        for item in project.items() {
            match &item.body {
                ItemBody::Voice { slot_id, at } => {
                    let Some(slot) = project.voice_slots.iter().find(|s| &s.id == slot_id) else {
                        continue;
                    };
                    let take = slot
                        .takes
                        .iter()
                        .find(|t| Some(&t.id) == slot.active_take.as_ref());
                    let Some(path) = take.and_then(|t| t.transcript_path.as_ref()) else {
                        continue;
                    };
                    if let Ok(t) = ck_voice::load(&self.resolve(path)) {
                        words.extend(
                            t.words
                                .into_iter()
                                .map(|w| (at + w.start, at + w.end, w.text)),
                        );
                    }
                }
                ItemBody::Footage {
                    src,
                    src_in,
                    src_out,
                    at,
                    speed,
                    ..
                } => {
                    let Some(s) = project.sources.iter().find(|s| &s.id == src) else {
                        continue;
                    };
                    let Some(path) = &s.transcript_path else {
                        continue;
                    };
                    if let Ok(t) = ck_voice::load(&self.resolve(path)) {
                        words.extend(
                            t.words
                                .into_iter()
                                .filter(|w| w.start >= *src_in && w.end <= *src_out)
                                .map(|w| {
                                    (
                                        at + (w.start - src_in) / speed,
                                        at + (w.end - src_in) / speed,
                                        w.text,
                                    )
                                }),
                        );
                    }
                }
                _ => {}
            }
        }
        words.sort_by(|a, b| a.0.total_cmp(&b.0));
        words
    }
}

/// Groups words into cues: at most 7 words or 3 s, breaking on pauses of 0.4 s or more.
pub fn srt(words: &[(f64, f64, String)]) -> String {
    let mut cues: Vec<(f64, f64, Vec<&str>)> = Vec::new();
    for (s, e, w) in words {
        let start_new = match cues.last() {
            None => true,
            Some((cs, ce, ws)) => ws.len() >= 7 || e - cs > 3.0 || s - ce >= 0.4,
        };
        if start_new {
            cues.push((*s, *e, vec![w.as_str()]));
        } else if let Some(c) = cues.last_mut() {
            c.1 = *e;
            c.2.push(w.as_str());
        }
    }
    let ts = |t: f64| {
        let ms = (t.max(0.0) * 1000.0).round() as u64;
        format!(
            "{:02}:{:02}:{:02},{:03}",
            ms / 3_600_000,
            ms / 60_000 % 60,
            ms / 1000 % 60,
            ms % 1000
        )
    };
    cues.iter()
        .enumerate()
        .map(|(i, (s, e, ws))| format!("{}\n{} --> {}\n{}\n", i + 1, ts(*s), ts(*e), ws.join(" ")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Writes `text` to `out` (creating its folder).
pub fn write_text(out: &Path, text: &str) -> CkResult<()> {
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", dir.display())))?;
    }
    ck_core::storage::atomic_write(out, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srt_cues_break_on_pauses_and_length() {
        let w = |s: f64, e: f64, t: &str| (s, e, t.to_owned());
        let words = vec![
            w(1.0, 1.4, "Welcome"),
            w(1.45, 1.7, "to"),
            w(1.75, 2.0, "ClipKit"),
            w(3.0, 3.3, "today"),
            w(3.3, 3.45, "we"),
        ];
        assert_eq!(
            srt(&words),
            "1\n00:00:01,000 --> 00:00:02,000\nWelcome to ClipKit\n\n2\n00:00:03,000 --> 00:00:03,450\ntoday we\n"
        );
        let many: Vec<_> = (0..9)
            .map(|i| w(f64::from(i) * 0.2, f64::from(i) * 0.2 + 0.15, "x"))
            .collect();
        assert_eq!(srt(&many).matches(" --> ").count(), 2);
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum TimelineFormat {
    Otio,
    Srt,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimelineExportRequest {
    pub format: TimelineFormat,
    /// Output path relative to the project folder; default `exports/<project>.<otio|srt>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimelineExportResult {
    pub path: String,
    pub format: TimelineFormat,
    /// OTIO tracks, or SRT cues.
    pub entries: usize,
}

impl Session {
    pub fn export_timeline(&self, req: &TimelineExportRequest) -> CkResult<TimelineExportResult> {
        let project = self.project()?;
        let ext = match req.format {
            TimelineFormat::Otio => "otio",
            TimelineFormat::Srt => "srt",
        };
        let rel = match &req.out {
            Some(o) => {
                let p = Path::new(o);
                let ok = p
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
                    && p.extension().is_some_and(|e| e == ext);
                if !ok {
                    return Err(CkError::new(
                        ErrorCode::Request,
                        format!(
                            "export path {o:?} must be a relative .{ext} path inside the project"
                        ),
                    ));
                }
                p.to_path_buf()
            }
            None => Path::new("exports").join(format!("{}.{ext}", project.id)),
        };
        let out = self.workdir().join(&rel);
        let (text, entries) = match req.format {
            TimelineFormat::Otio => {
                let v = self.otio(&project);
                let n = v["tracks"]["children"].as_array().map_or(0, Vec::len);
                let text = serde_json::to_string_pretty(&v)
                    .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
                (text, n)
            }
            TimelineFormat::Srt => {
                let words = self.timeline_words(&project);
                if words.is_empty() {
                    return Err(
                        CkError::new(ErrorCode::Request, "no timed speech to caption")
                            .with_hint("Transcribe the footage or run ck voice captions first."),
                    );
                }
                let text = srt(&words);
                let n = text.matches(" --> ").count();
                (text, n)
            }
        };
        write_text(&out, &text)?;
        Ok(TimelineExportResult {
            path: rel.to_string_lossy().replace('\\', "/"),
            format: req.format,
            entries,
        })
    }
}
