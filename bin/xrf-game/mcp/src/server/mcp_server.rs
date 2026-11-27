use std::io::{BufRead, Write};

use serde_json::{Map, Value, json};
use xrf_error::XrfResult;

use crate::bridge::game_bridge::GameBridge;
use crate::server::mcp_tools::McpTools;

/// An MCP server over standard streams, newline-delimited JSON-RPC 2.0 as the engine's own speaks it
/// (`xrf-engine/cli/mcp/McpStdioServer.ts`): `initialize`, `ping`, `tools/list` and `tools/call`, notifications
/// ignored. Calls run one at a time, a start waiting for its level.
pub struct McpServer {
  bridge: GameBridge,
}

impl McpServer {
  /// Protocol versions it speaks, the latest first; a client asking for another gets the latest.
  pub const PROTOCOL_VERSIONS: [&'static str; 5] =
    ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05", "2024-10-07"];

  /// The name it registers as and introduces itself by.
  pub const NAME: &'static str = "xrf-game";

  const PARSE_ERROR: i64 = -32_700;
  const INVALID_REQUEST: i64 = -32_600;
  const METHOD_NOT_FOUND: i64 = -32_601;
  const INVALID_PARAMS: i64 = -32_602;

  /// How an agent uses the tools, shown beside them.
  const INSTRUCTIONS: &'static str = "Tools for the native XRF game (xrf-tools' xrf-game), a separate program from the \
    C++ game the xrf-engine-game server drives. Build it with `cargo build --profile dev-optimized -p xrf-game -p \
    xrf-game-mcp` in xrf-tools, then call game_start with a tree's root and a level. Read game_log after a failure: \
    it reads the log from disk.";

  pub fn new(bridge: GameBridge) -> Self {
    Self { bridge }
  }

  /// Answers each line of `input` on `output` until the input ends, then quits a game still running.
  ///
  /// # Errors
  ///
  /// Returns why the client's streams failed.
  pub fn serve(&mut self, input: impl BufRead, mut output: impl Write) -> XrfResult<()> {
    for line in input.lines() {
      let line: String = line?;

      if line.trim().is_empty() {
        continue;
      }

      if let Some(response) = self.handle_line(&line) {
        writeln!(output, "{response}")?;
        output.flush()?;
      }
    }

    self.bridge.quit()?;

    Ok(())
  }

  /// The response to one message, none for a notification.
  pub fn handle_line(&mut self, line: &str) -> Option<Value> {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
      return Some(Self::fail(&Value::Null, Self::PARSE_ERROR, "Parse error."));
    };
    let Some(message) = message.as_object() else {
      return Some(Self::fail(
        &Value::Null,
        Self::INVALID_REQUEST,
        "Expected a JSON-RPC message object.",
      ));
    };
    let id: &Value = message.get("id")?;
    let Some(method) = message.get("method").and_then(Value::as_str) else {
      return Some(Self::fail(id, Self::INVALID_REQUEST, "Expected a method name."));
    };
    let parameters: Value = message
      .get("params")
      .cloned()
      .unwrap_or_else(|| Value::Object(Map::new()));

    Some(match method {
      "initialize" => {
        let asked: Option<&str> = parameters.get("protocolVersion").and_then(Value::as_str);
        let version: &str = asked
          .filter(|version| Self::PROTOCOL_VERSIONS.contains(version))
          .unwrap_or(Self::PROTOCOL_VERSIONS[0]);

        Self::succeed(
          id,
          json!({
            "protocolVersion": version,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": Self::NAME, "version": env!("CARGO_PKG_VERSION") },
            "instructions": Self::INSTRUCTIONS,
          }),
        )
      }
      "ping" => Self::succeed(id, json!({})),
      "tools/list" => Self::succeed(id, json!({ "tools": McpTools::list() })),
      "tools/call" => self.call_tool(id, &parameters),
      _ => Self::fail(id, Self::METHOD_NOT_FOUND, &format!("Method '{method}' not found.")),
    })
  }

  fn call_tool(&mut self, id: &Value, parameters: &Value) -> Value {
    let name: &str = parameters.get("name").and_then(Value::as_str).unwrap_or_default();
    let arguments: Value = parameters.get("arguments").cloned().unwrap_or_else(|| json!({}));

    match McpTools::call(&mut self.bridge, name, &arguments) {
      None => Self::fail(id, Self::INVALID_PARAMS, &format!("Unknown tool '{name}'.")),
      Some(result) => {
        let (text, is_error) = match result {
          Ok(text) => (text, false),
          Err(error) => (error.to_string(), true),
        };

        Self::succeed(
          id,
          json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }),
        )
      }
    }
  }

  fn succeed(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
  }

  fn fail(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
  }
}
