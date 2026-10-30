//! `pipeline detect` and `pipeline next`: the agent's explicit steps through the stage machine
//! (spec 4.1). Writes advance the rest from evidence.

use ck_core::model::{Mode, Pass, Stage};
use ck_core::{CkError, CkResult, ErrorCode};
use ck_pipeline::stage::{Op, Step, advance, next_steps};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Session;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub stage: Stage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass: Option<Pass>,
    pub mode: Mode,
    /// What the mode was read from: sources by kind.
    pub sources: usize,
}

impl Session {
    fn pipeline_result(&self) -> CkResult<PipelineResult> {
        let p = self.project()?;
        Ok(PipelineResult {
            stage: p.pipeline.stage,
            pass: p.pipeline.pass,
            mode: p.mode,
            sources: p.sources.len(),
        })
    }

    /// `pipeline detect`: read the mode from the project's sources and open the brief stage.
    pub fn pipeline_detect(&self) -> CkResult<PipelineResult> {
        {
            let mut doc = self.doc()?;
            self.gate(doc.project(), Op::SetMode)?;
            let mut state = doc.project().pipeline.clone();
            if matches!(state.stage, Stage::Boot | Stage::Detect) {
                state.stage = Stage::Detect;
                state.pass = None;
                doc.set_pipeline(state);
                self.persist(&mut doc)?;
                self.publish_snapshot(&doc);
            }
        }
        self.pipeline_result()
    }

    /// `pipeline next`: finish the current step. Passes run in order; VERIFY is left by handoff.
    pub fn pipeline_next(&self) -> CkResult<PipelineResult> {
        {
            let mut doc = self.doc()?;
            let project = doc.project();
            let mut state = project.pipeline.clone();
            let from = Step::of(&state);
            if !matches!(
                state.stage,
                Stage::Base | Stage::Passes | Stage::Iterate | Stage::Plan
            ) {
                return Err(CkError::new(
                    ErrorCode::Stage,
                    format!("pipeline next does not apply in {:?}", state.stage).to_lowercase(),
                )
                .with_hint("Before PLAN follow the brief; from VERIFY run ck handoff."));
            }
            let brief = self.brief_status(project);
            let to = next_steps(from)
                .first()
                .copied()
                .ok_or_else(|| CkError::new(ErrorCode::Stage, "no next step"))?;
            advance(&mut state, to, brief)?;
            doc.set_pipeline(state);
            self.persist(&mut doc)?;
            self.publish_snapshot(&doc);
        }
        self.pipeline_result()
    }
}
