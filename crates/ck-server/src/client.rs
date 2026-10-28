//! Blocking loopback client for a running `ck studio`: finds it through `server.json`, sends the
//! session token, and turns error bodies back into `CkError`. Used by the CLI and the MCP server.

use std::path::Path;
use std::time::Duration;

use ck_core::{CkError, CkResult, ErrorCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::auth::TOKEN_HEADER;
use crate::info::ServerInfo;

/// Writes fsync; anything slower than this means the server is stuck.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Largest response body read, kept small so a stray server cannot make the client buffer much.
/// It sits above every `ops` cap; a longer reply is refused as E_OUTPUT_CAP.
pub const RESPONSE_LIMIT: u64 = 64 * 1024;

pub struct Client {
    base: String,
    token: String,
    agent: ureq::Agent,
}

impl Client {
    /// Connect to the server described by `<root>/server.json` (root is the `.clipkit` folder).
    pub fn connect(root: &Path) -> CkResult<Self> {
        let info = ServerInfo::read(root).map_err(|e| {
            CkError::new(
                ErrorCode::Server,
                format!("no running studio: {}", e.message),
            )
        })?;
        Ok(Self::new(&info))
    }

    pub fn new(info: &ServerInfo) -> Self {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            // Never route loopback calls (and the token) through an environment proxy.
            .proxy(None)
            // The token header must never follow a redirect anywhere.
            .max_redirects(0)
            .timeout_global(Some(TIMEOUT))
            .build()
            .into();
        Self {
            base: format!("http://127.0.0.1:{}", info.port),
            token: info.token.clone(),
            agent,
        }
    }

    pub fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> CkResult<T> {
        let mut req = self
            .agent
            .get(format!("{}{path}", self.base))
            .header(TOKEN_HEADER, &self.token);
        for (k, v) in query {
            req = req.query(*k, *v);
        }
        self.finish(req.call())
    }

    pub fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> CkResult<T> {
        let bytes = serde_json::to_vec(body)
            .map_err(|e| CkError::new(ErrorCode::Request, e.to_string()))?;
        let resp = self
            .agent
            .post(format!("{}{path}", self.base))
            .header(TOKEN_HEADER, &self.token)
            .header("content-type", "application/json")
            .send(&bytes[..]);
        self.finish(resp)
    }

    fn finish<T: DeserializeOwned>(
        &self,
        resp: Result<ureq::http::Response<ureq::Body>, ureq::Error>,
    ) -> CkResult<T> {
        let mut resp = resp.map_err(|e| {
            CkError::new(
                ErrorCode::Server,
                format!("studio at {} did not answer: {e}", self.base),
            )
        })?;
        let status = resp.status();
        let bytes = resp
            .body_mut()
            .with_config()
            .limit(RESPONSE_LIMIT)
            .read_to_vec()
            .map_err(|e| match e {
                ureq::Error::BodyExceedsLimit(_) => CkError::new(
                    ErrorCode::OutputCap,
                    format!("reply is over {RESPONSE_LIMIT} bytes"),
                ),
                e => CkError::new(ErrorCode::Server, format!("reading reply: {e}")),
            })?;
        if status.is_success() {
            return serde_json::from_slice(&bytes)
                .map_err(|e| CkError::new(ErrorCode::Server, format!("bad reply: {e}")));
        }
        Err(
            serde_json::from_slice::<CkError>(&bytes).unwrap_or_else(|_| {
                CkError::new(ErrorCode::Server, format!("studio replied {status}"))
            }),
        )
    }
}
