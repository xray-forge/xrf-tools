#![doc = include_str!("../README.md")]

pub(crate) mod bridge;
pub(crate) mod endpoint;
pub(crate) mod server;
pub(crate) mod wire;

#[cfg(test)]
mod tests;

pub use crate::bridge::game_bridge::GameBridge;
pub use crate::bridge::game_logs::GameLogs;
pub use crate::endpoint::game_endpoint::GameEndpoint;
pub use crate::server::mcp_server::McpServer;
pub use crate::wire::game_event::GameEvent;
pub use crate::wire::game_input::GameInput;
pub use crate::wire::game_request::GameRequest;
pub use crate::wire::game_request_kind::GameRequestKind;
pub use crate::wire::game_response::GameResponse;
