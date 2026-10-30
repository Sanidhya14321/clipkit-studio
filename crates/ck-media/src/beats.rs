//! Beat grid from audio: an onset envelope from decoded PCM, tempo by autocorrelation, phase by
//! the comb that collects the most onset energy. Plain Rust over ffmpeg output.

use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use ck_core::CkError;

use crate::run::{check_path, run};

const RATE: usize = 8000;
const HOP: usize = 160;
/// Envelope frames per second.
const FRAMES: f64 = (RATE / HOP) as f64;
const MIN_BPM: f64 = 70.0;
const MAX_BPM: f64 = 180.0;

/// Measured tempo and beat times, seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Beats {
    pub tempo: f64,
    pub beats: Vec<f64>,
}

/// Decodes `src` to mono PCM and measures its beats.
pub fn measure_beats(src: &Path) -> Result<Beats, CkError> {
    check_path(src)?;
    let args: Vec<&OsStr> = vec![
        OsStr::new("-v"),
        OsStr::new("error"),
        OsStr::new("-i"),
        src.as_os_str(),
        OsStr::new("-ac"),
        OsStr::new("1"),
        OsStr::new("-ar"),
        OsStr::new("8000"),
        OsStr::new("-f"),
        OsStr::new("s16le"),
        OsStr::new("-"),
    ];
    let out = run("ffmpeg", &args, Duration::from_secs(120))?;
    let samples: Vec<f64> = out
        .stdout
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| f64::from(i16::from_le_bytes(*b)))
        .collect();
    Ok(beats_from_samples(&samples))
}

/// Positive energy rise per hop.
fn onsets(samples: &[f64]) -> Vec<f64> {
    let energy: Vec<f64> = samples
        .chunks(HOP)
        .map(|c| c.iter().map(|s| s * s).sum::<f64>().sqrt())
        .collect();
    energy.windows(2).map(|w| (w[1] - w[0]).max(0.0)).collect()
}

fn beats_from_samples(samples: &[f64]) -> Beats {
    let env = onsets(samples);
    let (lo, hi) = (
        (FRAMES * 60.0 / MAX_BPM) as usize,
        (FRAMES * 60.0 / MIN_BPM) as usize,
    );
    let lag = (lo..=hi.min(env.len().saturating_sub(1)))
        .max_by(|a, b| {
            let score = |l: usize| env.iter().zip(&env[l..]).map(|(x, y)| x * y).sum::<f64>();
            score(*a).total_cmp(&score(*b))
        })
        .unwrap_or(lo.max(1));
    let phase = (0..lag)
        .max_by(|a, b| {
            let comb = |p: usize| env.iter().skip(p).step_by(lag).sum::<f64>();
            comb(*a).total_cmp(&comb(*b))
        })
        .unwrap_or(0);
    Beats {
        tempo: (FRAMES * 60.0 / lag as f64 * 10.0).round() / 10.0,
        beats: (phase..env.len())
            .step_by(lag)
            // The rise is between frames i and i+1.
            .map(|f| ((f + 1) as f64 * 1000.0 / FRAMES).round() / 1000.0)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_tempo_and_phase_of_clicks() {
        // 120 BPM clicks from 0.2 s, 10 s long.
        let mut s = vec![0.0; RATE * 10];
        for n in 0..20 {
            let at = (0.2 + f64::from(n) * 0.5) * RATE as f64;
            for x in &mut s[at as usize..at as usize + 200] {
                *x = 10000.0;
            }
        }
        let b = beats_from_samples(&s);
        assert!((b.tempo - 120.0).abs() < 1.0, "{}", b.tempo);
        assert!((b.beats[0] - 0.2).abs() < 0.03, "{:?}", &b.beats[..2]);
        assert!(b.beats.len() >= 19);
    }

    #[test]
    fn silence_gives_a_grid_without_panic() {
        assert!(!beats_from_samples(&vec![0.0; RATE]).tempo.is_nan());
        assert!(beats_from_samples(&[]).beats.is_empty());
    }
}
