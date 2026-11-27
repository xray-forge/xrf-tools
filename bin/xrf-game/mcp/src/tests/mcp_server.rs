use std::path::PathBuf;

use serde_json::Value;

use crate::bridge::game_bridge::GameBridge;
use crate::bridge::game_logs::GameLogs;
use crate::server::mcp_server::McpServer;

fn server() -> McpServer {
  McpServer::new(GameBridge::new(
    PathBuf::from("no-such-game.exe"),
    GameLogs::new(Vec::new()),
  ))
}

fn answer(server: &mut McpServer, line: &str) -> Value {
  server.handle_line(line).expect("a request is answered")
}

#[test]
fn introduces_itself_in_the_version_asked_else_its_latest() {
  let mut server: McpServer = server();
  let asked: Value = answer(
    &mut server,
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
  );
  let unknown: Value = answer(
    &mut server,
    r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
  );

  assert_eq!(asked["result"]["protocolVersion"], "2025-06-18");
  assert_eq!(asked["result"]["serverInfo"]["name"], McpServer::NAME);
  assert_eq!(asked["id"], 1);
  assert_eq!(unknown["result"]["protocolVersion"], McpServer::PROTOCOL_VERSIONS[0]);
}

#[test]
fn lists_its_tools_and_answers_a_notification_with_nothing() {
  let mut server: McpServer = server();
  let listed: Value = answer(&mut server, r#"{"jsonrpc":"2.0","id":"a","method":"tools/list"}"#);
  let names: Vec<&str> = listed["result"]["tools"]
    .as_array()
    .expect("a list")
    .iter()
    .filter_map(|tool| tool["name"].as_str())
    .collect();

  assert_eq!(
    names,
    [
      "game_start",
      "game_status",
      "game_pose",
      "game_input",
      "game_console",
      "game_capture",
      "game_log",
      "game_quit"
    ]
  );
  assert!(
    server
      .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
      .is_none()
  );
}

#[test]
fn fails_a_call_as_a_tool_result_and_an_unknown_method_or_tool_as_an_error() {
  let mut server: McpServer = server();
  let status: Value = answer(
    &mut server,
    r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"game_status"}}"#,
  );
  let pose: Value = answer(
    &mut server,
    r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"game_pose","arguments":{"position":[1,2]}}}"#,
  );

  assert_eq!(status["result"]["isError"], true);
  assert!(
    status["result"]["content"][0]["text"]
      .as_str()
      .is_some_and(|text| text.contains("no game runs"))
  );
  assert_eq!(pose["result"]["isError"], true);
  assert_eq!(
    answer(
      &mut server,
      r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"game_dance"}}"#
    )["error"]["code"],
    -32_602
  );
  assert_eq!(
    answer(&mut server, r#"{"jsonrpc":"2.0","id":6,"method":"resources/list"}"#)["error"]["code"],
    -32_601
  );
  assert_eq!(answer(&mut server, "{oops")["error"]["code"], -32_700);
}
