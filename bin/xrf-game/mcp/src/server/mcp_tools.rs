use serde_json::{Value, json};
use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_bridge::GameBridge;
use crate::wire::game_input::GameInput;
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
        "name": "game_attach",
        "description": "Attach to a game the user started by hand with `--listen`, instead of starting one, and drive it \
          with every other tool. Reads how from the game's attach.json; one game at a time.",
        "inputSchema": { "type": "object", "properties": {} }
      },
      {
        "name": "game_detach",
        "description": "Stop driving the game attached to, leaving it running; a game started by game_start is quit \
          instead.",
        "inputSchema": { "type": "object", "properties": {} }
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
        "name": "game_input",
        "description": "Play keyboard and mouse input into the game, as the window would deliver it, and answer once \
          the last step is delivered. Steps run in order, after any played before: press, down and up a key by SDL's \
          name (`W`, `Grave` opens the console, `Return`, `Escape`, `Space`, `Left Shift`, `Up`); text typed while \
          the console is open; move (`dx`, `dy` turn the camera); click at `x`, `y`; wheel; wait `ms`. Hold a key \
          with down, wait, up; wait about 100 ms after opening the console before typing. Waits total at most 20 s.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "inputs": {
              "type": "array",
              "description": "The steps, in order",
              "items": {
                "type": "object",
                "properties": {
                  "type": { "type": "string", "enum": ["press", "down", "up", "text", "move", "click", "wheel", "wait"] },
                  "key": { "type": "string", "description": "SDL's name for the key: press, down, up" },
                  "text": { "type": "string", "description": "What to type: text" },
                  "dx": { "type": "number", "description": "Mouse movement: move" },
                  "dy": { "type": "number", "description": "Mouse movement: move; wheel lines: wheel" },
                  "x": { "type": "number", "description": "Cursor position in window coordinates: move, click" },
                  "y": { "type": "number", "description": "Cursor position in window coordinates: move, click" },
                  "button": { "type": "string", "enum": ["left", "right", "middle"], "description": "click" },
                  "ms": { "type": "integer", "minimum": 0, "description": "How long: wait" }
                },
                "required": ["type"]
              }
            }
          },
          "required": ["inputs"]
        }
      },
      {
        "name": "game_console",
        "description": "Run a line in the game's console, as typed into its drawer, and answer its lines. `help` lists \
          the commands, `list <part>` the variables (the settings, by dotted path); a variable alone shows it, with a \
          value sets it, as `view.shadows.resolution 2048`. A refused line is an error.",
        "inputSchema": {
          "type": "object",
          "properties": { "line": { "type": "string", "description": "The line, as `noclip` or `list shadows`" } },
          "required": ["line"]
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
        "description": "Quit the game driven: one started by game_start is waited for until it exits; an attached one \
          is asked to quit, as its close button does. To leave an attached game running, use game_detach.",
        "inputSchema": { "type": "object", "properties": {} }
      }
    ])
  }

  /// Calls the tool named `name` with `arguments`, none for a tool there is none of.
  pub(crate) fn call(bridge: &mut GameBridge, name: &str, arguments: &Value) -> Option<XrfResult<String>> {
    Some(match name {
      "game_start" => Self::start(bridge, arguments),
      "game_attach" => bridge.attach(),
      "game_detach" => bridge.detach(),
      "game_status" => bridge.ask(GameRequestKind::Status),
      "game_pose" => Self::read_point(arguments, "position").and_then(|position| {
        let target: [f32; 3] = Self::read_point(arguments, "target")?;

        bridge.ask(GameRequestKind::Pose { position, target })
      }),
      "game_input" => Self::read_inputs(arguments).and_then(|inputs| bridge.ask(GameRequestKind::Input { inputs })),
      "game_console" => Self::read_text(arguments, "line")
        .and_then(|line| bridge.ask(GameRequestKind::Console { line: line.to_owned() })),
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

  fn read_inputs(arguments: &Value) -> XrfResult<Vec<GameInput>> {
    let inputs: &Value = arguments
      .get("inputs")
      .ok_or_else(|| XrfError::new_invalid_error("'inputs' must be a list of steps"))?;

    serde_json::from_value(inputs.clone())
      .map_err(|error| XrfError::new_invalid_error(format!("'inputs' is not a list of steps: {error}")))
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
