//! Bounded, deterministic onset/tempo analysis. Estimates are candidates, never a confirmed grid.
use crate::run::{check_path, run_bounded};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

pub const SAMPLE_RATE: usize = 48000;
pub const MAX_SECONDS: usize = 600;
const HOP: usize = 480;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MusicAnalysis {
    pub duration_sec: f64,
    pub peak: f64,
    pub rms_dbfs: Option<f64>,
    pub onset_sec: Option<f64>,
    pub bpm: Option<f64>,
    pub confidence: f64,
    pub beats: Vec<f64>,
    pub onsets: Vec<f64>,
    /// One RMS value per second, in linear amplitude; not perceptual loudness.
    pub energy: Vec<f64>,
    /// Coarse energy rises in seconds, not confirmed musical drops.
    pub energy_rise_candidates: Vec<f64>,
    /// Must be confirmed by a user/planner before beat-anchored edits.
    pub confirmed: bool,
}

fn error(message: &str) -> CkError {
    CkError::new(ErrorCode::Media, message)
        .with_hint("Use audio shorter than 10 minutes; confirm estimated tempo before editing.")
}

pub fn analyze(path: &Path) -> CkResult<MusicAnalysis> {
    check_path(path)?;
    let meta = std::fs::metadata(path).map_err(|e| error(&e.to_string()))?;
    if !meta.is_file() || meta.len() > 128 * 1024 * 1024 {
        return Err(error("audio must be a regular file under 128 MiB"));
    }
    let info = crate::probe(path)?;
    if info.audio.is_empty() || !info.duration.is_finite() || info.duration > MAX_SECONDS as f64 {
        return Err(error(
            "audio is missing or exceeds the analysis duration limit",
        ));
    }
    // At most 57.6 MB decoded, with an explicit bounded subprocess cap. Decode
    // one extra sample so underreported container duration cannot silently truncate.
    let limit = (MAX_SECONDS * SAMPLE_RATE + 1).to_string();
    let channels = info.audio[0].channels;
    if channels == 0 || channels > 8 {
        return Err(error("analysis supports one to eight source channels"));
    }
    // Maximum channel magnitude preserves audible anti-phase stereo events.
    // Levels are therefore envelope levels; source LUFS is measured separately.
    let mut magnitude = "abs(val(0))".to_owned();
    for channel in 1..channels {
        magnitude = format!("max({magnitude},abs(val({channel})))");
    }
    let filter = format!("aresample=48000,aeval='{magnitude}':c=mono,atrim=end_sample={limit}");
    let out = run_bounded(
        "ffmpeg",
        &[
            "-hide_banner".as_ref(),
            "-nostdin".as_ref(),
            "-loglevel".as_ref(),
            "error".as_ref(),
            "-y".as_ref(),
            "-i".as_ref(),
            path.as_os_str(),
            "-map".as_ref(),
            "0:a:0".as_ref(),
            "-vn".as_ref(),
            "-ac".as_ref(),
            "1".as_ref(),
            "-ar".as_ref(),
            "48000".as_ref(),
            "-af".as_ref(),
            filter.as_ref(),
            "-f".as_ref(),
            "s16le".as_ref(),
            "pipe:1".as_ref(),
        ],
        Duration::from_secs(60),
        64 * 1024 * 1024,
    )?;
    let samples: Vec<f32> = out
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0)
        .collect();
    analyze_samples(&samples)
}

// Occupancy in one-second bins prevents extrapolating through silent sections.
fn energy_coverage(envelope: &[f64]) -> f64 {
    let max = envelope.iter().copied().fold(0.0_f64, f64::max);
    let bins = envelope.chunks(100);
    let count = envelope.len().div_ceil(100).max(1);
    bins.filter(|bin| bin.iter().any(|e| *e > max * 0.02 && *e > 1e-5))
        .count() as f64
        / count as f64
}

/// Fixed 48 kHz mono input. Pure function shared by fixtures and file analysis.
pub fn analyze_samples(samples: &[f32]) -> CkResult<MusicAnalysis> {
    if samples.is_empty()
        || samples.len() > MAX_SECONDS * SAMPLE_RATE
        || samples.iter().any(|s| !s.is_finite())
    {
        return Err(error("empty, non-finite or oversized decoded audio"));
    }
    let peak = samples
        .iter()
        .map(|s| f64::from(s.abs()))
        .fold(0.0_f64, f64::max);
    let power = |s: &[f32]| s.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / s.len() as f64;
    let rms = power(samples).sqrt();
    let envelope: Vec<f64> = samples.chunks(HOP).map(|s| power(s).sqrt()).collect();
    // Positive energy flux is deliberately conservative: ambiguous music stays
    // low-confidence. This does not pretend to infer meter or true downbeats.
    let mut flux = vec![0.0; envelope.len()];
    for i in 0..envelope.len() {
        flux[i] = (envelope[i] - if i == 0 { 0.0 } else { envelope[i - 1] }).max(0.0);
    }
    let max_flux = flux.iter().copied().fold(0.0_f64, f64::max);
    let mut onsets = Vec::new();
    if peak > 1e-5 {
        for i in 0..flux.len() {
            let t = i as f64 * HOP as f64 / SAMPLE_RATE as f64;
            if flux[i] > max_flux * 0.2
                && flux[i] > peak * 0.01
                && (i == 0 || flux[i] >= flux[i - 1])
                && (i + 1 == flux.len() || flux[i] > flux[i + 1])
                && onsets.last().is_none_or(|last| t - last >= 0.08)
            {
                onsets.push(t);
            }
        }
    }
    // Dominant inter-onset period, robust to a few missing hits. Keep octave
    // ambiguity explicit through confidence; do not fabricate a bar/drop map.
    let mut votes = vec![0.0_f64; 181]; // 60..240 BPM
    for pair in onsets.windows(2) {
        let dt = pair[1] - pair[0];
        if dt <= 0.0 {
            continue;
        }
        let mut bpm = 60.0 / dt;
        while bpm < 60.0 {
            bpm *= 2.0;
        }
        while bpm > 240.0 {
            bpm /= 2.0;
        }
        if (60.0..=240.0).contains(&bpm) {
            votes[(bpm.round() as usize - 60).min(180)] += 1.0;
        }
    }
    let best = (0..votes.len())
        .max_by(|a, b| votes[*a].total_cmp(&votes[*b]))
        .unwrap_or(0);
    let candidate = (best + 60) as f64;
    let periods: Vec<f64> = onsets
        .windows(2)
        .map(|p| p[1] - p[0])
        .filter(|dt| (60.0 / dt / candidate - 1.0).abs() < 0.08)
        .collect();
    let initial_period = if periods.is_empty() {
        0.5
    } else {
        periods.iter().sum::<f64>() / periods.len() as f64
    };
    let n = onsets.len().max(1) as f64;
    let first = onsets.first().copied().unwrap_or(0.0);
    let indexes: Vec<f64> = onsets
        .iter()
        .map(|t| ((t - first) / initial_period).round())
        .collect();
    let mean_x = indexes.iter().sum::<f64>() / n;
    let mean_y = onsets.iter().sum::<f64>() / n;
    let variance = indexes.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>();
    let period = if variance > 0.0 {
        indexes
            .iter()
            .zip(&onsets)
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>()
            / variance
    } else {
        initial_period
    };
    let offset = (mean_y - period * mean_x).max(0.0);
    let aligned = onsets
        .iter()
        .filter(|t| {
            let phase = (**t - offset) / period;
            (phase - phase.round()).abs() * period <= 0.025
        })
        .count();
    let duration = samples.len() as f64 / SAMPLE_RATE as f64;
    let confidence = if onsets.len() >= 5 {
        let coverage =
            ((onsets.last().copied().unwrap_or(0.0) - onsets[0] + period) / duration).min(1.0);
        let active = energy_coverage(&envelope);
        aligned as f64 / onsets.len() as f64 * coverage.min(active)
    } else {
        0.0
    };
    let bpm = (confidence >= 0.85 && !periods.is_empty()).then_some(60.0 / period);
    let beats = bpm.map_or_else(Vec::new, |bpm| {
        let start = offset;
        (0..=(duration * bpm / 60.0).ceil() as usize)
            .map(|i| start + i as f64 * 60.0 / bpm)
            .take_while(|t| *t < duration)
            .collect()
    });
    let energy: Vec<f64> = samples
        .chunks(SAMPLE_RATE)
        .map(|s| power(s).sqrt())
        .collect();
    let mut rises = Vec::new();
    for i in 3..energy.len().saturating_sub(1) {
        let before = energy[i - 3..i].iter().sum::<f64>() / 3.0;
        let after = energy[i];
        if after > 0.05
            && after > before.max(0.001) * 1.8
            && rises.last().is_none_or(|last| i as f64 - last >= 4.0)
        {
            rises.push(i as f64);
        }
    }
    rises.truncate(3);
    Ok(MusicAnalysis {
        duration_sec: duration,
        peak,
        rms_dbfs: (rms > 0.0).then(|| 20.0 * rms.log10()),
        onset_sec: onsets.first().copied(),
        bpm,
        confidence,
        beats,
        onsets,
        energy,
        energy_rise_candidates: rises,
        confirmed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oversized_files_are_refused_before_decode_and_energy_rises_are_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("too-large.wav");
        std::fs::File::create(&path)
            .unwrap()
            .set_len(128 * 1024 * 1024 + 1)
            .unwrap();
        assert_eq!(analyze(&path).unwrap_err().code, ErrorCode::Media);
        let samples: Vec<f32> = (0..SAMPLE_RATE * 10)
            .map(|i| if i < SAMPLE_RATE * 6 { 0.01 } else { 0.5 })
            .collect();
        let analysis = analyze_samples(&samples).unwrap();
        assert_eq!(analysis.energy_rise_candidates, vec![6.0]);
        assert!(!analysis.confirmed);
    }
    fn write_wav(path: &Path, samples: &[f32], channels: u16) -> std::io::Result<()> {
        let data: Vec<u8> = samples
            .iter()
            .flat_map(|s| ((*s * 30000.0) as i16).to_le_bytes())
            .collect();
        let mut wav = Vec::new();
        wav.extend(b"RIFF");
        wav.extend((36 + data.len() as u32).to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16_u32.to_le_bytes());
        wav.extend(1_u16.to_le_bytes());
        wav.extend(channels.to_le_bytes());
        wav.extend((SAMPLE_RATE as u32).to_le_bytes());
        wav.extend((SAMPLE_RATE as u32 * 2 * u32::from(channels)).to_le_bytes());
        wav.extend((2 * channels).to_le_bytes());
        wav.extend(16_u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend((data.len() as u32).to_le_bytes());
        wav.extend(data);
        std::fs::write(path, wav)
    }
    #[test]
    fn opposite_phase_stereo_events_are_not_silent() {
        let mono = pulses(180.0, 5, false);
        let stereo: Vec<f32> = mono.iter().flat_map(|v| [*v, -*v]).collect();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stereo.wav");
        write_wav(&path, &stereo, 2).unwrap();
        let a = analyze(&path).unwrap();
        assert!(a.peak > 0.8);
        assert!((a.bpm.unwrap() - 180.0).abs() < 0.2);
    }
    fn pulses(bpm: f64, seconds: usize, shift: bool) -> Vec<f32> {
        let mut samples = vec![0.0; SAMPLE_RATE * seconds];
        let mut time = 0.2;
        while time < seconds as f64 - 0.1 {
            let t = time
                + if shift && time > seconds as f64 / 2.0 {
                    0.12
                } else {
                    0.0
                };
            let at = (t * SAMPLE_RATE as f64).round() as usize;
            if at + 20 < samples.len() {
                for i in 0..20 {
                    samples[at + i] = (20 - i) as f32 / 20.0;
                }
            }
            time += 60.0 / bpm;
        }
        samples
    }
    #[test]
    fn fractional_and_fast_tempos_do_not_drift_or_fold_to_half_time() {
        for bpm in [123.45, 180.0, 200.0, 239.0] {
            let a = analyze_samples(&pulses(bpm, 40, false)).unwrap();
            assert!((a.bpm.unwrap() - bpm).abs() < 0.2, "{bpm}: {:?}", a.bpm);
            for beat in &a.beats {
                assert!(a.onsets.iter().any(|onset| (beat - onset).abs() < 0.026));
            }
        }
        let a = analyze_samples(&pulses(120.0, 20, true)).unwrap();
        assert!(
            a.bpm.is_none(),
            "phase discontinuity must require correction"
        );
        let mut tail = pulses(120.0, 4, false);
        tail.resize(SAMPLE_RATE * 40, 0.0);
        let sparse = analyze_samples(&tail).unwrap();
        assert!(sparse.bpm.is_none());
        assert!(sparse.confidence < 0.2);
        let mut separated = pulses(120.0, 4, false);
        separated.resize(SAMPLE_RATE * 40, 0.0);
        separated.extend(pulses(120.0, 4, false));
        let sparse = analyze_samples(&separated).unwrap();
        assert!(sparse.bpm.is_none());
        assert!(sparse.confidence < 0.25);
    }
    #[test]
    fn decoding_48khz_retains_more_than_25_seconds_and_matches_pure_analysis() {
        let samples = pulses(180.0, 31, false);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clicks.wav");
        write_wav(&path, &samples, 1).unwrap();
        let a = analyze(&path).unwrap();
        assert!((a.duration_sec - 31.0).abs() < 0.001);
        assert!((a.bpm.unwrap() - 180.0).abs() < 0.2);
        assert_eq!(a.beats, analyze_samples(&samples).unwrap().beats);
    }
    #[test]
    fn silence_and_invalid_input_never_invent_a_grid() {
        let a = analyze_samples(&vec![0.0; SAMPLE_RATE]).unwrap();
        assert_eq!(a.bpm, None);
        assert!(a.beats.is_empty());
        assert_eq!(a.onset_sec, None);
        assert!(analyze_samples(&[]).is_err());
        assert!(analyze_samples(&[f32::NAN]).is_err());
    }
    #[test]
    fn known_offset_and_tempo_are_deterministic_and_gain_invariant() {
        let mut samples = vec![0.0; SAMPLE_RATE * 8];
        for n in 0..15 {
            let start = SAMPLE_RATE / 5 + n * SAMPLE_RATE / 2;
            for i in 0..20 {
                samples[start + i] = ((20 - i) as f32 / 20.0) * if i % 2 == 0 { 1.0 } else { -1.0 };
            }
        }
        let a = analyze_samples(&samples).unwrap();
        assert_eq!(a.bpm, Some(120.0));
        assert!((a.onset_sec.unwrap() - 0.2).abs() < 0.011);
        assert!(!a.confirmed);
        assert!(a.confidence > 0.9);
        let quiet: Vec<_> = samples.iter().map(|v| v * 0.1).collect();
        assert_eq!(analyze_samples(&quiet).unwrap().beats, a.beats);
        assert_eq!(analyze_samples(&samples).unwrap(), a);
        assert!(serde_json::to_vec(&a).unwrap().len() < 4096);
    }
}
