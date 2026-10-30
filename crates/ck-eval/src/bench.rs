//! `ck bench report` (spec 10.3): medians, ranges and pass rates per arm and scenario from the
//! run lines `bench/run.sh` appends. Tokens are reported only from the agents' own usage;
//! missing usage is "not measured", never estimated.

use std::collections::BTreeMap;
use std::path::Path;

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod caps {
    pub const REPORT: usize = 16 * 1024;
}

/// An eval score at or above this passes the scenario's rubric.
pub const PASS_SCORE: u32 = 80;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunLine {
    pub arm: String,
    pub scenario: String,
    pub run: u32,
    pub wall_sec: f64,
    pub eval_score: u32,
    /// Agent-reported tokens; null when the agent reports none.
    #[serde(default)]
    pub tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub arm: String,
    pub scenario: String,
    pub runs: usize,
    pub pass_rate: f64,
    pub score: Stat,
    pub wall_sec: Stat,
    /// None when no run of this cell reported tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<Stat>,
    /// Runs short of the 3 the spec asks for.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub too_few_runs: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BenchReport {
    pub cells: Vec<Cell>,
    /// Runs whose tokens were not reported.
    pub tokens_not_measured: usize,
    pub limits: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BenchReportRequest {
    /// The results file (one JSON run per line).
    pub results: String,
}

fn stat(mut v: Vec<f64>) -> Option<Stat> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let median = if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    };
    Some(Stat {
        median,
        min: v[0],
        max: v[n - 1],
    })
}

pub fn aggregate(runs: &[RunLine]) -> BenchReport {
    let mut groups: BTreeMap<(String, String), Vec<&RunLine>> = BTreeMap::new();
    for r in runs {
        groups
            .entry((r.arm.clone(), r.scenario.clone()))
            .or_default()
            .push(r);
    }
    let cells = groups
        .into_iter()
        .map(|((arm, scenario), rs)| {
            let passed = rs.iter().filter(|r| r.eval_score >= PASS_SCORE).count();
            let zero = Stat {
                median: 0.0,
                min: 0.0,
                max: 0.0,
            };
            Cell {
                runs: rs.len(),
                pass_rate: passed as f64 / rs.len() as f64,
                score: stat(rs.iter().map(|r| f64::from(r.eval_score)).collect())
                    .unwrap_or(zero.clone()),
                wall_sec: stat(rs.iter().map(|r| r.wall_sec).collect()).unwrap_or(zero),
                tokens: stat(
                    rs.iter()
                        .filter_map(|r| r.tokens.map(|t| t as f64))
                        .collect(),
                ),
                too_few_runs: rs.len() < 3,
                arm,
                scenario,
            }
        })
        .collect();
    BenchReport {
        cells,
        tokens_not_measured: runs.iter().filter(|r| r.tokens.is_none()).count(),
        limits: vec![
            "Scores come from ck eval (output only); OCR checks are not measured.".into(),
            "Tokens are the agents' own usage reports; missing reports are not estimated.".into(),
            "Fairness (same model, agent version, machine, prompts) is the operator's to keep."
                .into(),
        ],
    }
}

pub fn report(base: &Path, req: &BenchReportRequest) -> CkResult<BenchReport> {
    let path = {
        let p = Path::new(&req.results);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            base.join(p)
        }
    };
    let text = std::fs::read_to_string(&path).map_err(|e| {
        CkError::new(ErrorCode::NotFound, format!("{}: {e}", path.display()))
            .with_hint("Run bench/run.sh first; results are PLACEHOLDER until then.")
    })?;
    let runs = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(n, l)| {
            serde_json::from_str(l).map_err(|e| {
                CkError::new(
                    ErrorCode::Request,
                    format!("{} line {}: {e}", path.display(), n + 1),
                )
            })
        })
        .collect::<CkResult<Vec<RunLine>>>()?;
    Ok(aggregate(&runs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(arm: &str, run: u32, score: u32, wall: f64, tokens: Option<u64>) -> RunLine {
        RunLine {
            arm: arm.into(),
            scenario: "teaser".into(),
            run,
            wall_sec: wall,
            eval_score: score,
            tokens,
        }
    }

    #[test]
    fn cells_report_medians_ranges_and_pass_rates() {
        let r = aggregate(&[
            line("B", 1, 100, 60.0, Some(20_000)),
            line("B", 2, 90, 80.0, Some(24_000)),
            line("B", 3, 60, 70.0, Some(22_000)),
            line("A0", 1, 40, 200.0, None),
        ]);
        let b = r.cells.iter().find(|c| c.arm == "B").unwrap();
        assert_eq!(b.runs, 3);
        assert!((b.pass_rate - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(
            b.score,
            Stat {
                median: 90.0,
                min: 60.0,
                max: 100.0
            }
        );
        assert_eq!(b.tokens.as_ref().unwrap().median, 22_000.0);
        assert!(!b.too_few_runs);
        let a0 = r.cells.iter().find(|c| c.arm == "A0").unwrap();
        assert_eq!(a0.tokens, None, "never estimated");
        assert!(a0.too_few_runs);
        assert_eq!(r.tokens_not_measured, 1);
    }

    #[test]
    fn even_counts_take_the_middle_mean_and_bad_lines_are_refused() {
        assert_eq!(stat(vec![1.0, 3.0]).unwrap().median, 2.0);
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("r.jsonl"), "{\"arm\":1}\n").unwrap();
        let err = report(
            dir.path(),
            &BenchReportRequest {
                results: "r.jsonl".into(),
            },
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::Request);
        let missing = report(
            dir.path(),
            &BenchReportRequest {
                results: "none".into(),
            },
        )
        .unwrap_err();
        assert_eq!(missing.code, ErrorCode::NotFound);
    }

    #[test]
    fn the_scenario_file_lists_the_spec_scenarios_and_arms() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../bench/scenarios.json"
        ))
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["scenarios"].as_array().unwrap().len(), 15);
        let arms: Vec<&str> = v["arms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["id"].as_str().unwrap())
            .collect();
        for a in ["A0", "A1", "B"] {
            assert!(arms.contains(&a));
        }
        assert!(v["runsPerArm"].as_u64().unwrap() >= 3);
    }
}
