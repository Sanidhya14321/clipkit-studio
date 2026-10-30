//! Render engine seam (UPDATE_5 section 2.1): job control and capabilities, mirroring the
//! worker's EngineAdapter. Only Remotion ships until the U5-1 engine decision is approved.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// What an engine can run, as reported to the planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EngineCapabilities {
    /// Stable engine id.
    pub engine: String,
    /// Animation runtimes the engine seeks deterministically.
    pub runtimes: Vec<String>,
    /// Whether motion blur is deterministic on this engine.
    pub motion_blur: bool,
}

/// Job control for one render engine.
pub trait RenderEngine {
    /// Capabilities of this engine.
    fn capabilities(&self) -> EngineCapabilities;
    /// Worker script, relative to the render worker folder.
    fn worker_script(&self) -> &'static str;
}

/// The current default engine.
#[derive(Debug, Clone, Copy, Default)]
pub struct RemotionEngine;

impl RenderEngine for RemotionEngine {
    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            engine: "remotion".into(),
            runtimes: ["remotion", "css", "gsap", "motion"]
                .map(String::from)
                .to_vec(),
            motion_blur: true,
        }
    }

    fn worker_script(&self) -> &'static str {
        "src/render.mjs"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remotion_reports_the_spike_verified_runtimes() {
        let caps = RemotionEngine.capabilities();
        assert_eq!(caps.engine, "remotion");
        assert!(caps.runtimes.iter().any(|r| r == "gsap"));
        assert!(caps.runtimes.iter().any(|r| r == "motion"));
        assert_eq!(RemotionEngine.worker_script(), "src/render.mjs");
    }
}
