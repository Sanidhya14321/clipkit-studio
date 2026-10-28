//! MCP server exposing the same operations as the CLI (spec 6.2), over stdio.

pub mod server;
pub mod tools;

pub use server::CkMcp;

#[cfg(test)]
mod tests;
