//! Local axum HTTP and WebSocket server, event bus and studio serving (spec 5, 6.2).
//!
//! The server binds 127.0.0.1 only, checks Host and Origin on every request and the session
//! token on every API and WebSocket call. All writes go through `Session`, which applies the
//! revision, schema and lock gates from ck-core, so turn-taking holds even if the UI is bypassed.

pub mod api;
pub mod auth;
pub mod client;
pub mod clock;
pub mod debug;
pub mod event;
pub mod http_error;
pub mod idle;
pub mod info;
pub mod launch;
pub mod media;
pub mod ops;
pub mod serve;
pub mod session;
pub mod studio;
pub mod ts_types;
pub mod view;
mod ws;

pub use clock::{Clock, SystemClock};
pub use event::Event;
pub use info::ServerInfo;
pub use serve::Server;
pub use session::Session;

#[cfg(test)]
mod tests;
