//! The MCP tool table (spec 6.2). Each tool is the twin of a ck subcommand: the name is the
//! command path joined with `_` (`ck project get` is `project_get`), the input is the JSON the
//! command takes, and the output is the one line of compact JSON the command prints.

use std::path::Path;

use ck_assets::library::{self, IdRequest, LookListRequest, TasteSearchRequest};
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_server::idle::DEFAULT_IDLE_SECS;
use ck_server::launch::{self, EnsureRequest, Launcher};
use ck_server::ops::{Ops, caps};
use ck_server::session::brief::{BriefDiffRequest, BriefProposeRequest, BriefWaitRequest};
use ck_server::session::{HeartbeatRequest, LockRequest, UnlockRequest};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `status` takes no input.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatusArgs {}

/// `project get`: the summary (default) or one layer in full.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectGetArgs {
    /// Ids, kinds and spans per track (the default).
    #[serde(default)]
    pub summary: bool,
    /// One layer by id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
}

/// One MCP tool: its CLI twin, description, output cap and input schema.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    pub name: &'static str,
    pub command: &'static str,
    pub description: &'static str,
    /// Output cap in bytes of compact JSON (the same cap as the CLI command).
    pub cap: usize,
    pub input_schema: Value,
}

/// Every tool, in CLI help order.
pub fn specs() -> Vec<ToolSpec> {
    vec![
        spec::<EnsureRequest>(
            "studio",
            "ck studio",
            "Start the studio in the background unless it is running (ck studio --ensure); \
             `open` opens its browser tab once.",
            caps::STUDIO,
        ),
        spec::<StatusArgs>(
            "status",
            "ck status",
            "Project rev, mode, pipeline stage, agent state and live locks.",
            caps::STATUS,
        ),
        spec::<ProjectGetArgs>(
            "project_get",
            "ck project get",
            "The project summary (ids, kinds and spans per track), or one layer in full with \
             `layer`.",
            caps::SUMMARY.max(caps::LAYER),
        ),
        spec::<PatchRequest>(
            "project_patch",
            "ck project patch",
            "Apply an RFC 6902 patch at baseRev. Lock the layers first.",
            caps::PATCH,
        ),
        spec::<LockRequest>(
            "lock",
            "ck lock",
            "Lock layers before touching them; returns the holder's live locks.",
            caps::LOCK,
        ),
        spec::<UnlockRequest>(
            "unlock",
            "ck unlock",
            "Release layers (all of the holder's locks when layerIds is omitted).",
            caps::LOCK,
        ),
        spec::<HeartbeatRequest>(
            "heartbeat",
            "ck heartbeat",
            "Renew the holder's live locks so they do not expire; returns them.",
            caps::LOCK,
        ),
        spec::<BriefProposeRequest>(
            "brief_propose",
            "ck brief propose",
            "Send inferred brief values only (and at most 3 questions, each naming what it \
             affects); the studio shows the Brief page. Then call brief_wait.",
            caps::BRIEF_PROPOSE,
        ),
        spec::<BriefWaitRequest>(
            "brief_wait",
            "ck brief wait",
            "Block until the user confirms the brief (or it is auto), at most 25 s; call again \
             while ready is false. Returns the confirmed brief.",
            caps::BRIEF,
        ),
        spec::<BriefDiffRequest>(
            "brief_diff",
            "ck brief diff",
            "Only what changed in the brief since a rev you read; patch only layers whose \
             briefDeps changed.",
            caps::BRIEF,
        ),
        spec::<StatusArgs>(
            "brief_get",
            "ck brief get",
            "The brief in compact form (value and source per field) with its rev.",
            caps::BRIEF,
        ),
        spec::<TasteSearchRequest>(
            "taste_search",
            "ck taste search",
            "Rank the taste files by query words (id, title, tags); read at most 3 of them.",
            library::caps::TASTE_SEARCH,
        ),
        spec::<IdRequest>(
            "taste_read",
            "ck taste read",
            "One taste file in full: direction with concrete numbers.",
            library::caps::TASTE_READ,
        ),
        spec::<LookListRequest>(
            "look_list",
            "ck look list",
            "The looks in short: id, name, description and motion personality.",
            library::caps::LOOK_LIST,
        ),
        spec::<IdRequest>(
            "look_get",
            "ck look get",
            "One look with all its design tokens (colors, fonts, type scale, motion).",
            library::caps::LOOK_GET,
        ),
    ]
}

/// Whether `name` runs over the built-in asset library, without a studio.
pub fn is_offline(name: &str) -> bool {
    matches!(
        name,
        "taste_search" | "taste_read" | "look_list" | "look_get"
    )
}

/// Tools over the built-in asset library: they need no studio. `None` when `name` is not one.
pub fn offline(name: &str, args: Value) -> Option<CkResult<String>> {
    let out = match name {
        "taste_search" => parse(args).and_then(|r| json(&library::taste_search(&r)?)),
        "taste_read" => parse(args).and_then(|r| json(&library::taste_read(&r)?)),
        "look_list" => parse(args).and_then(|r| json(&library::look_list(&r)?)),
        "look_get" => parse(args).and_then(|r| json(&library::look_get(&r)?)),
        _ => return None,
    };
    Some(out)
}

fn spec<T: JsonSchema>(
    name: &'static str,
    command: &'static str,
    description: &'static str,
    cap: usize,
) -> ToolSpec {
    // Serializing a schemars schema does not fail; fall back to the widest valid shape anyway.
    let mut input_schema = serde_json::to_value(schemars::schema_for!(T))
        .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
    if let Value::Object(map) = &mut input_schema {
        // MCP clients only need the shape; the draft URI and title are noise in the tool list.
        map.remove("$schema");
        map.remove("title");
    }
    ToolSpec {
        name,
        command,
        description,
        cap,
        input_schema,
    }
}

/// Whether `name` is a tool.
pub fn exists(name: &str) -> bool {
    specs().iter().any(|s| s.name == name)
}

/// Run tool `name` with `args` against the studio and return its compact JSON output.
pub fn call(ops: &Ops, name: &str, args: Value) -> CkResult<String> {
    match name {
        "status" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.status()?)
        }
        "project_get" => match parse::<ProjectGetArgs>(args)? {
            ProjectGetArgs {
                summary: true,
                layer: Some(_),
            } => Err(request_err("give summary or layer, not both".to_owned())),
            ProjectGetArgs {
                layer: Some(id), ..
            } => json(&ops.layer(&id)?),
            ProjectGetArgs { layer: None, .. } => json(&ops.summary()?),
        },
        "project_patch" => json(&ops.patch(&parse(args)?)?),
        "lock" => json(&ops.lock(&parse(args)?)?),
        "unlock" => json(&ops.unlock(&parse(args)?)?),
        "heartbeat" => json(&ops.heartbeat(&parse(args)?)?),
        "brief_propose" => json(&ops.brief_propose(&parse(args)?)?),
        "brief_wait" => json(&ops.brief_wait(&parse(args)?)?),
        "brief_diff" => json(&ops.brief_diff(&parse(args)?)?),
        "brief_get" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.brief_get()?)
        }
        other => Err(request_err(format!("no tool named {other:?}"))),
    }
}

/// The `studio` tool: `ck studio --ensure` for the project whose `.clipkit` folder is `root`.
/// It needs no running server, so it does not go through `Ops`.
pub fn studio(root: &Path, args: Value) -> CkResult<String> {
    let req: EnsureRequest = parse(args)?;
    json(&launch::ensure(
        project_dir(root),
        &req,
        &Launcher::current(DEFAULT_IDLE_SECS)?,
    )?)
}

/// Ops for every other tool; the first call of a session starts the studio (spec 5.1), as the
/// CLI does.
pub fn connect(root: &Path) -> CkResult<Ops> {
    launch::connect(project_dir(root), &Launcher::current(DEFAULT_IDLE_SECS)?)
}

fn project_dir(root: &Path) -> &Path {
    root.parent().unwrap_or(Path::new("."))
}

/// Missing arguments count as an empty object, so `status` can be called bare.
fn parse<T: DeserializeOwned>(args: Value) -> CkResult<T> {
    let args = if args.is_null() {
        Value::Object(Default::default())
    } else {
        args
    };
    serde_json::from_value(args).map_err(|e| request_err(format!("arguments: {e}")))
}

fn json<T: Serialize>(value: &T) -> CkResult<String> {
    serde_json::to_string(value).map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))
}

fn request_err(message: String) -> CkError {
    CkError::new(ErrorCode::Request, message)
}
