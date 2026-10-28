//! The rmcp handler: lists the tool table and runs each call through `ck_server::ops`. Ops use a
//! blocking HTTP client, so every call runs on the blocking pool.

use std::path::PathBuf;
use std::sync::Arc;

use ck_core::{CkError, CkResult, ErrorCode};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;

use crate::tools;

#[derive(Debug, Clone)]
pub struct CkMcp {
    /// The project's `.clipkit` folder; each call finds the studio through its server.json.
    root: PathBuf,
}

impl CkMcp {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Serve MCP on stdin and stdout until the client closes the stream.
    pub async fn serve_stdio(self) -> CkResult<()> {
        let running = self
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| server_err(format!("MCP start: {e}")))?;
        running
            .waiting()
            .await
            .map_err(|e| server_err(format!("MCP stopped: {e}")))?;
        Ok(())
    }
}

/// The tool list as MCP tools.
pub fn tool_list() -> Vec<Tool> {
    tools::specs()
        .into_iter()
        .map(|s| {
            let schema = match s.input_schema {
                Value::Object(map) => map,
                _ => Default::default(),
            };
            Tool::new(s.name, s.description, Arc::new(schema))
        })
        .collect()
}

impl ServerHandler for CkMcp {
    fn get_info(&self) -> ServerConfig {
        let mut implementation = Implementation::default();
        implementation.name = "ck".to_owned();
        implementation.version = env!("CARGO_PKG_VERSION").to_owned();
        let mut info = ServerConfig::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = implementation;
        info.instructions =
            Some("ClipKit Studio. Tools mirror the ck CLI; outputs are compact JSON.".to_owned());
        info
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: tool_list(),
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let name = request.name.to_string();
        if !tools::exists(&name) {
            return Err(ErrorData::invalid_params(
                format!("no tool named {name:?}"),
                None,
            ));
        }
        let args = request.arguments.map_or(Value::Null, Value::Object);
        let root = self.root.clone();
        let outcome = tokio::task::spawn_blocking(move || match name.as_str() {
            "studio" => tools::studio(&root, args),
            _ => tools::connect(&root).and_then(|ops| tools::call(&ops, &name, args)),
        })
        .await
        .unwrap_or_else(|e| Err(server_err(format!("tool task failed: {e}"))));
        Ok(tool_result(outcome).into())
    }
}

/// The CLI's stdout line as text content, or on failure the CkError JSON the CLI prints on
/// stderr, flagged as a tool error.
fn tool_result(outcome: CkResult<String>) -> CallToolResult {
    match outcome {
        Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
        Err(e) => {
            let text = serde_json::to_string(&e).unwrap_or_else(|_| e.to_string());
            CallToolResult::error(vec![ContentBlock::text(text)])
        }
    }
}

fn server_err(message: String) -> CkError {
    CkError::new(ErrorCode::Server, message)
}
