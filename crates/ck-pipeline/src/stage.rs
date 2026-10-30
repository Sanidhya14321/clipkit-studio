//! The stage machine (spec 4.1) and the stage gate (spec 4.2).

use ck_core::model::{BriefStatus, Pass, PipelineState, Stage};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A position in the pipeline: a stage, plus the pass while in `passes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub stage: Stage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass: Option<Pass>,
}

impl Step {
    pub const fn stage(stage: Stage) -> Self {
        Self { stage, pass: None }
    }

    pub const fn pass(pass: Pass) -> Self {
        Self {
            stage: Stage::Passes,
            pass: Some(pass),
        }
    }

    pub fn of(state: &PipelineState) -> Self {
        Self {
            stage: state.stage,
            pass: state.pass,
        }
    }

    fn label(self) -> String {
        match self.pass {
            Some(p) => format!("{:?} pass", p).to_lowercase(),
            None => format!("{:?}", self.stage).to_lowercase(),
        }
    }
}

/// Every operation declares the stages it may run in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Op {
    /// Read-only calls: status, summaries, slices, context.
    Read,
    Health,
    SetMode,
    ProposeBrief,
    WritePlan,
    Patch,
    RunGates,
    Handoff,
}

impl Op {
    pub const ALL: &'static [Op] = &[
        Op::Read,
        Op::Health,
        Op::SetMode,
        Op::ProposeBrief,
        Op::WritePlan,
        Op::Patch,
        Op::RunGates,
        Op::Handoff,
    ];

    /// Stages this operation is allowed in (`None` = any stage).
    pub fn stages(self) -> Option<&'static [Stage]> {
        Some(match self {
            Op::Read => return None,
            Op::Health => &[Stage::Boot],
            Op::SetMode => &[Stage::Detect],
            Op::ProposeBrief => &[Stage::Brief, Stage::Iterate],
            Op::WritePlan => &[Stage::Plan],
            Op::Patch => &[Stage::Base, Stage::Passes, Stage::Iterate],
            Op::RunGates => &[Stage::Verify],
            Op::Handoff => &[Stage::Handoff],
        })
    }
}

/// The steps reachable from `from`. Stages cannot be skipped; passes run in order.
/// VERIFY may send work back to any pass; ITERATE jobs end in VERIFY again.
pub fn next_steps(from: Step) -> Vec<Step> {
    match (from.stage, from.pass) {
        (Stage::Boot, _) => vec![Step::stage(Stage::Detect)],
        (Stage::Detect, _) => vec![Step::stage(Stage::Brief)],
        (Stage::Brief, _) => vec![Step::stage(Stage::Plan)],
        (Stage::Plan, _) => vec![Step::stage(Stage::Base)],
        (Stage::Base, _) => vec![Step::pass(Pass::Structure)],
        (Stage::Passes, pass) => match pass.and_then(Pass::next) {
            Some(next) => vec![Step::pass(next)],
            None => vec![Step::stage(Stage::Verify)],
        },
        (Stage::Verify, _) => std::iter::once(Step::stage(Stage::Handoff))
            .chain(Pass::ALL.iter().map(|p| Step::pass(*p)))
            .collect(),
        (Stage::Handoff, _) => vec![Step::stage(Stage::Iterate)],
        (Stage::Iterate, _) => vec![Step::stage(Stage::Verify)],
    }
}

fn hint_for(from: Step) -> String {
    let next: Vec<String> = next_steps(from).into_iter().map(Step::label).collect();
    format!(
        "Current step: {}. Next allowed step: {}.",
        from.label(),
        next.join(" or ")
    )
}

/// Move the pipeline to `to`, refusing skips (E_STAGE). Leaving BRIEF needs a confirmed or auto brief.
pub fn advance(state: &mut PipelineState, to: Step, brief: BriefStatus) -> CkResult<()> {
    let from = Step::of(state);
    if !next_steps(from).contains(&to) {
        return Err(CkError::new(
            ErrorCode::Stage,
            format!("cannot go from {} to {}", from.label(), to.label()),
        )
        .with_hint(hint_for(from)));
    }
    if from.stage == Stage::Brief && brief == BriefStatus::Proposed {
        return Err(CkError::new(ErrorCode::Stage, "the brief is not confirmed yet").with_hint(
            "Wait for the user to confirm or skip the Brief page, or mark the brief auto if they asked for no questions.",
        ));
    }
    if from.stage == Stage::Passes {
        state.last_completed_pass = from.pass;
    }
    state.stage = to.stage;
    state.pass = to.pass;
    Ok(())
}

/// The stage a project is in after a write, derived from evidence so the agent never drives it:
/// an unsettled brief is BRIEF, a settled one with no layers is PLAN, layers mean PASSES, and
/// anything after the first handoff is ITERATE.
pub fn stage_after_write(current: Stage, brief: BriefStatus, has_layers: bool) -> Stage {
    match (current, brief, has_layers) {
        (Stage::Handoff | Stage::Iterate, _, _) => Stage::Iterate,
        (_, BriefStatus::Proposed, _) => Stage::Brief,
        (_, _, false) => Stage::Plan,
        (_, _, true) => Stage::Passes,
    }
}

/// The stage gate: refuse an operation outside its stages, naming the next allowed step.
pub fn check_op(state: &PipelineState, op: Op) -> CkResult<()> {
    match op.stages() {
        Some(stages) if !stages.contains(&state.stage) => {
            let allowed: Vec<String> = stages
                .iter()
                .map(|s| format!("{s:?}").to_lowercase())
                .collect();
            Err(CkError::new(
                ErrorCode::Stage,
                format!(
                    "{op:?} is allowed only in {}; current stage is {:?}",
                    allowed.join(", "),
                    state.stage
                )
                .to_lowercase(),
            )
            .with_hint(hint_for(Step::of(state))))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(brief: BriefStatus) -> CkResult<PipelineState> {
        let mut s = PipelineState::default();
        for stage in [Stage::Detect, Stage::Brief, Stage::Plan, Stage::Base] {
            advance(&mut s, Step::stage(stage), brief)?;
        }
        for p in Pass::ALL {
            advance(&mut s, Step::pass(*p), brief)?;
        }
        advance(&mut s, Step::stage(Stage::Verify), brief)?;
        Ok(s)
    }

    #[test]
    fn writes_derive_the_stage_from_evidence() {
        use BriefStatus::{Auto, Confirmed, Proposed};
        assert_eq!(
            stage_after_write(Stage::Boot, Proposed, false),
            Stage::Brief
        );
        assert_eq!(
            stage_after_write(Stage::Brief, Confirmed, false),
            Stage::Plan
        );
        assert_eq!(stage_after_write(Stage::Plan, Auto, true), Stage::Passes);
        assert_eq!(stage_after_write(Stage::Verify, Auto, true), Stage::Passes);
        assert_eq!(
            stage_after_write(Stage::Handoff, Auto, true),
            Stage::Iterate
        );
        assert_eq!(
            stage_after_write(Stage::Iterate, Proposed, true),
            Stage::Iterate
        );
    }

    #[test]
    fn happy_path_runs_every_stage_and_pass_in_order() {
        let mut s = walk(BriefStatus::Confirmed).unwrap();
        assert_eq!(s.last_completed_pass, Some(Pass::Polish));
        advance(&mut s, Step::stage(Stage::Handoff), BriefStatus::Confirmed).unwrap();
        advance(&mut s, Step::stage(Stage::Iterate), BriefStatus::Confirmed).unwrap();
        advance(&mut s, Step::stage(Stage::Verify), BriefStatus::Confirmed).unwrap();
        advance(&mut s, Step::pass(Pass::Text), BriefStatus::Confirmed).unwrap();
    }

    #[test]
    fn unconfirmed_brief_blocks_planning_unless_auto() {
        let err = walk(BriefStatus::Proposed).unwrap_err();
        assert_eq!(err.code, ErrorCode::Stage);
        assert!(err.hint.contains("confirm"));
        walk(BriefStatus::Auto).unwrap();
    }

    /// Exhaustive: from every step, every other step is either listed in `next_steps` and
    /// accepted, or refused with E_STAGE and a hint that names the next allowed step.
    #[test]
    fn every_transition_is_allowed_or_refused_with_hint() {
        let mut all: Vec<Step> = Stage::ALL
            .iter()
            .filter(|s| **s != Stage::Passes)
            .map(|s| Step::stage(*s))
            .collect();
        all.extend(Pass::ALL.iter().map(|p| Step::pass(*p)));
        for from in &all {
            for to in &all {
                let mut s = PipelineState {
                    stage: from.stage,
                    pass: from.pass,
                    last_completed_pass: None,
                };
                let res = advance(&mut s, *to, BriefStatus::Confirmed);
                if next_steps(*from).contains(to) {
                    assert!(res.is_ok(), "{from:?} -> {to:?}");
                    assert_eq!(Step::of(&s), *to);
                } else {
                    let e = res.unwrap_err();
                    assert_eq!(e.code, ErrorCode::Stage);
                    assert!(e.hint.contains("Next allowed step"), "{from:?} -> {to:?}");
                    assert_eq!(Step::of(&s), *from, "refused transitions change nothing");
                }
            }
        }
    }

    #[test]
    fn patch_before_base_is_refused_with_next_step() {
        let s = PipelineState {
            stage: Stage::Brief,
            pass: None,
            last_completed_pass: None,
        };
        let e = check_op(&s, Op::Patch).unwrap_err();
        assert_eq!(e.code, ErrorCode::Stage);
        assert!(e.hint.contains("Next allowed step: plan"), "{}", e.hint);
        for op in Op::ALL {
            for stage in Stage::ALL {
                let s = PipelineState {
                    stage: *stage,
                    pass: None,
                    last_completed_pass: None,
                };
                let ok = op.stages().is_none_or(|st| st.contains(stage));
                assert_eq!(check_op(&s, *op).is_ok(), ok, "{op:?} in {stage:?}");
            }
        }
    }
}
