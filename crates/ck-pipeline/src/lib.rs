//! Stage state machine and server-enforced guardrails (spec 4).
//!
//! Phase 1 covers the stage gate and the retry gate. Scope, budget, destructive, quality and
//! honesty gates arrive with the features they guard (Phases 4, 6 and 8).

pub mod retry;
pub mod stage;

pub use retry::RetryGate;
pub use stage::{Op, Step, advance, check_op, next_steps};
