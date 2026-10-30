//! Per-layer locks (spec 4.2 lock gate). Time is passed in as `now_ms` so tests control the clock.

use crate::error::{CkError, CkResult, ErrorCode};
use crate::model::{Author, Holder, Lock};

/// A lock expires when no heartbeat arrives within this window (DECISIONS.md, Phase 1).
pub const DEFAULT_TTL_MS: u64 = 30_000;

/// Who holds a live lock on the layer, if anyone.
pub fn holder_of(locks: &[Lock], layer_id: &str, now_ms: u64) -> Option<Holder> {
    locks
        .iter()
        .find(|l| l.layer_id == layer_id && l.is_live(now_ms))
        .map(|l| l.holder)
}

/// Lock every listed layer for `holder`, or none of them. Re-acquiring your own lock refreshes it.
pub fn acquire(
    locks: &mut Vec<Lock>,
    holder: Holder,
    layer_ids: &[String],
    now_ms: u64,
    ttl_ms: u64,
) -> CkResult<()> {
    prune(locks, now_ms);
    let held: Vec<String> = layer_ids
        .iter()
        .filter(|id| holder_of(locks, id, now_ms).is_some_and(|h| h != holder))
        .cloned()
        .collect();
    if !held.is_empty() {
        return Err(
            CkError::new(ErrorCode::LockHeld, "layers are locked by another holder")
                .with_layers(held),
        );
    }
    for id in layer_ids {
        locks.retain(|l| &l.layer_id != id);
        locks.push(Lock {
            layer_id: id.clone(),
            holder,
            acquired_at_ms: now_ms,
            heartbeat_at_ms: now_ms,
            ttl_ms,
        });
    }
    Ok(())
}

/// Refresh every live lock of `holder`. Expired locks are not revived. Returns how many were refreshed.
pub fn heartbeat(locks: &mut [Lock], holder: Holder, now_ms: u64) -> usize {
    let mut n = 0;
    for l in locks
        .iter_mut()
        .filter(|l| l.holder == holder && l.is_live(now_ms))
    {
        l.heartbeat_at_ms = now_ms;
        n += 1;
    }
    n
}

/// Release `holder`'s locks on the listed layers, or all of them when `layer_ids` is `None`.
pub fn release(locks: &mut Vec<Lock>, holder: Holder, layer_ids: Option<&[String]>) {
    locks.retain(|l| l.holder != holder || layer_ids.is_some_and(|ids| !ids.contains(&l.layer_id)));
}

/// Drop expired locks.
pub fn prune(locks: &mut Vec<Lock>, now_ms: u64) {
    locks.retain(|l| l.is_live(now_ms));
}

/// The lock gate for a patch touching `touched` layers.
/// The agent must hold every touched layer; humans (and voice) are refused on agent-held layers.
pub fn check_patch(
    locks: &[Lock],
    author: Author,
    touched: &[String],
    now_ms: u64,
) -> CkResult<()> {
    match author {
        Author::Agent => {
            let missing: Vec<String> = touched
                .iter()
                .filter(|id| holder_of(locks, id, now_ms) != Some(Holder::Agent))
                .cloned()
                .collect();
            if missing.is_empty() {
                Ok(())
            } else {
                Err(
                    CkError::new(ErrorCode::LockNotHeld, "agent does not hold these layers")
                        .with_hint(
                            "Run ck lock with the listed ids first. @project is the lock for \
                             project-level fields such as the brief and settings.",
                        )
                        .with_layers(missing),
                )
            }
        }
        Author::Human | Author::Voice => {
            // During a structural change the whole editor is read-only (spec 5.3).
            if holder_of(locks, crate::patch::STRUCTURE_LAYER, now_ms) == Some(Holder::Agent) {
                return Err(CkError::new(
                    ErrorCode::LockHeld,
                    "the agent is making a structural change; the editor is read-only",
                )
                .with_layers(touched.to_vec()));
            }
            let held: Vec<String> = touched
                .iter()
                .filter(|id| holder_of(locks, id, now_ms) == Some(Holder::Agent))
                .cloned()
                .collect();
            if held.is_empty() {
                Ok(())
            } else {
                Err(
                    CkError::new(ErrorCode::LockHeld, "the agent is working on these layers")
                        .with_layers(held),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn agent_lock_blocks_human_and_expires() {
        let mut locks = Vec::new();
        acquire(&mut locks, Holder::Agent, &ids(&["a", "b"]), 0, 1000).unwrap();
        let err = check_patch(&locks, Author::Human, &ids(&["b", "c"]), 500).unwrap_err();
        assert_eq!(
            (err.code, err.layer_ids),
            (ErrorCode::LockHeld, ids(&["b"]))
        );
        check_patch(&locks, Author::Agent, &ids(&["a", "b"]), 500).unwrap();
        // No heartbeat: expired at 1000.
        check_patch(&locks, Author::Human, &ids(&["b"]), 1000).unwrap();
        let err = check_patch(&locks, Author::Agent, &ids(&["a"]), 1000).unwrap_err();
        assert_eq!(err.code, ErrorCode::LockNotHeld);
    }

    #[test]
    fn heartbeat_extends_and_acquire_is_all_or_nothing() {
        let mut locks = Vec::new();
        acquire(&mut locks, Holder::Agent, &ids(&["a"]), 0, 1000).unwrap();
        assert_eq!(heartbeat(&mut locks, Holder::Agent, 900), 1);
        assert_eq!(holder_of(&locks, "a", 1800), Some(Holder::Agent));
        let err = acquire(&mut locks, Holder::Human, &ids(&["x", "a"]), 1000, 1000).unwrap_err();
        assert_eq!(err.code, ErrorCode::LockHeld);
        assert_eq!(
            holder_of(&locks, "x", 1000),
            None,
            "partial acquire must not happen"
        );
        release(&mut locks, Holder::Agent, None);
        acquire(&mut locks, Holder::Human, &ids(&["x", "a"]), 1000, 1000).unwrap();
    }

    #[derive(Debug, Clone)]
    enum Op {
        Acquire(bool, u8),
        Heartbeat(bool),
        Release(bool, Option<u8>),
        Wait(u64),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (any::<bool>(), 0u8..4).prop_map(|(a, l)| Op::Acquire(a, l)),
            any::<bool>().prop_map(Op::Heartbeat),
            (any::<bool>(), proptest::option::of(0u8..4)).prop_map(|(a, l)| Op::Release(a, l)),
            (0u64..1500).prop_map(Op::Wait),
        ]
    }

    proptest! {
        /// Never two live locks on one layer; a lock is live exactly while its heartbeat is fresh.
        #[test]
        fn lock_invariants_hold(ops in prop::collection::vec(op(), 1..60)) {
            let mut locks = Vec::new();
            let mut now = 0u64;
            let who = |agent: bool| if agent { Holder::Agent } else { Holder::Human };
            for op in ops {
                match op {
                    Op::Acquire(a, l) => { let _ = acquire(&mut locks, who(a), &[format!("l{l}")], now, 1000); }
                    Op::Heartbeat(a) => { heartbeat(&mut locks, who(a), now); }
                    Op::Release(a, l) => {
                        let ids = l.map(|l| vec![format!("l{l}")]);
                        release(&mut locks, who(a), ids.as_deref());
                    }
                    Op::Wait(ms) => now += ms,
                }
                for l in 0..4 {
                    let id = format!("l{l}");
                    let live: Vec<_> = locks.iter().filter(|x| x.layer_id == id && x.is_live(now)).collect();
                    prop_assert!(live.len() <= 1);
                    if let Some(x) = live.first() {
                        prop_assert!(now - x.heartbeat_at_ms < 1000);
                        prop_assert_eq!(holder_of(&locks, &id, now), Some(x.holder));
                    }
                }
            }
        }
    }
}
