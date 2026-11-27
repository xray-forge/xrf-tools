//! The bridge: an MCP server over its standard streams that starts the native game beside it as its child and drives it
//! over the child's. Its own executable, never a subcommand of the game's, so an agent session holding it open never
//! keeps the game's executable from being built again.

use std::io::{self, BufReader};
use std::process::ExitCode;

use xrf_game_mcp::{GameBridge, McpServer};

fn main() -> ExitCode {
  // Its standard output carries the protocol, so it logs nothing: the game's log file holds every record.
  let mut server: McpServer = McpServer::new(GameBridge::locate());

  match server.serve(BufReader::new(io::stdin().lock()), io::stdout().lock()) {
    Ok(()) => ExitCode::SUCCESS,
    Err(_) => ExitCode::FAILURE,
  }
}
