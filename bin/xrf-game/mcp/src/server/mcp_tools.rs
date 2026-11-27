use serde_json::{Value, json};
use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_bridge::GameBridge;
use crate::wire::game_request_kind::GameRequestKind;

/// The bridge's tools: their names, what an agent reads of each, the arguments they take, and the bridge call each
/// makes.
pub(crate) struct McpTools;

impl McpTools {
  /// Lines of the log `game_log` answers unless told otherwise.
  const LOG_LINES: usize = 100;

  /// Every tool as `tools/list` answers it.
  pub(crate) fn list() -> Value {
    let point = |what: &str| json!({ "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3, "description": what });

    json!([
      {
        "name": "game_start",
        "description": "Start the native game on a level of an X-Ray game tree and wait until it shows it. Fly mode; \
          a first run compiles shaders for a while. One game at a time.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "root": { "type": "string", "description": "The game tree's root folder: an installation, volumes or gamedata" },
            "level": { "type": "string", "description": "The level's name, as `zaton`" },
            "arguments": { "type": "array", "items": { "type": "string" }, "description": "More arguments, as `--set window.mode=windowed`" }
          },
          "required": ["root", "level"]
        }
      },
      {
        "name": "game_status",
        "description": "Where the running game stands: its level, tick, time of day and camera (engine space).",
        "inputSchema": { "type": "object", "properties": {} }
      },
      {
        "name": "game_pose",
        "description": "Stand the camera at a position looking at a target, both in engine space.",
        "inputSchema": {
          "type": "object",
          "properties": { "position": point("Where the camera stands"), "target": point("The point it looks at") },
          "required": ["position", "target"]
        }
      },
      {
        "name": "game_capture",
        "description": "Read the window's next frame back to a PNG file and answer its path.",
        "inputSchema": { "type": "object", "properties": {} }
      },
      {
        "name": "game_log",
        "description": "The last lines of the game's newest log, read from disk: works while no game runs, or after one \
          crashed.",
        "inputSchema": {
          "type": "object",
          "properties": { "lines": { "type": "integer", "minimum": 1, "description": "How many lines, 100 unless given" } }
        }
      },
      {
        "name": "game_quit",
        "description": "Quit the running game and wait for it to exit.",
        "inputSchema": { "type": "object", "properties": {} }
      }
    ])
  }

  /// Calls the tool named `name` with `arguments`, none for a tool there is none of.
  pub(crate) fn call(bridge: &mut GameBridge, name: &str, arguments: &Value) -> Option<XrfResult<String>> {
    Some(match name {
      "game_start" => Self::start(bridge, arguments),
      "game_status" => bridge.ask(GameRequestKind::Status),
      "game_pose" => Self::read_point(arguments, "position").and_then(|position| {
        let target: [f32; 3] = Self::read_point(arguments, "target")?;

        bridge.ask(GameRequestKind::Pose { position, target })
      }),
      "game_capture" => bridge.ask(GameRequestKind::Capture),
      "game_log" => bridge.read_log(
        arguments
          .get("lines")
          .and_then(Value::as_u64)
          .and_then(|lines| usize::try_from(lines).ok())
          .unwrap_or(Self::LOG_LINES),
      ),
      "game_quit" => bridge.quit(),
      _ => return None,
    })
  }

  fn start(bridge: &mut GameBridge, arguments: &Value) -> XrfResult<String> {
    let extra: Vec<String> = arguments
      .get("arguments")
      .and_then(Value::as_array)
      .map(|items| items.iter().filter_map(Value::as_str).map(String::from).collect())
      .unwrap_or_default();

    bridge.start(
      Self::read_text(arguments, "root")?,
      Self::read_text(arguments, "level")?,
      &extra,
    )
  }

  fn read_text<'a>(arguments: &'a Value, name: &str) -> XrfResult<&'a str> {
    arguments
      .get(name)
      .and_then(Value::as_str)
      .ok_or_else(|| XrfError::new_invalid_error(format!("'{name}' must be a string")))
  }

  fn read_point(arguments: &Value, name: &str) -> XrfResult<[f32; 3]> {
    let invalid = || XrfError::new_invalid_error(format!("'{name}' must be three numbers"));
    let items: &Vec<Value> = arguments.get(name).and_then(Value::as_array).ok_or_else(invalid)?;
    let numbers: Vec<f32> = items
      .iter()
      .filter_map(Value::as_f64)
      .map(|number| number as f32)
      .collect();

    numbers.try_into().map_err(|_| invalid())
  }
}
