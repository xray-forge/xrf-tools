use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_child::GameChild;
use crate::bridge::game_logs::GameLogs;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// What the bridge does for each of its tools: starts the game beside it as its child and asks it, or reads its log
/// from disk, which answers whether a game runs or not.
pub struct GameBridge {
  executable: PathBuf,
  logs: GameLogs,
  game: Option<GameChild>,
}

impl GameBridge {
  /// How long a start waits for the level: a first run compiles the renderer's shaders.
  const READY_WAIT: Duration = Duration::from_secs(300);

  /// How long a request waits for its answer; a capture waits for the next frame.
  const ANSWER_WAIT: Duration = Duration::from_secs(30);

  /// A bridge over `executable`, the game's.
  pub fn new(executable: PathBuf, logs: GameLogs) -> Self {
    Self {
      executable,
      logs,
      game: None,
    }
  }

  /// A bridge over the game built beside it, its logs where the game writes them.
  pub fn locate() -> Self {
    let executable: PathBuf = std::env::current_exe()
      .ok()
      .and_then(|bridge| bridge.parent().map(PathBuf::from))
      .unwrap_or_default()
      .join(format!("xrf-game{}", std::env::consts::EXE_SUFFIX));

    Self::new(executable, GameLogs::locate())
  }

  /// Starts the game on `level` of the tree at `root`, with `arguments` after them, and waits until it shows it.
  ///
  /// # Errors
  ///
  /// Returns why it did not come up: one runs already, it cannot start, or it exited first (its log says why).
  pub fn start(&mut self, root: &str, level: &str, arguments: &[String]) -> XrfResult<String> {
    if self.game.as_mut().is_some_and(GameChild::is_running) {
      return Err(XrfError::new_invalid_error("a game runs already; quit it first"));
    }

    let mut all: Vec<String> = vec![
      String::from("--root"),
      String::from(root),
      String::from("--level"),
      String::from(level),
    ];

    all.extend_from_slice(arguments);

    let mut game: GameChild = GameChild::start(&self.executable, &all)?;
    let shown: String = game.wait_ready(Self::READY_WAIT)?;

    self.game = Some(game);

    Ok(format!("The game shows '{shown}'"))
  }

  /// Asks the running game, answering what it found.
  ///
  /// # Errors
  ///
  /// Returns why it could not answer: no game runs, it did not answer in time, or it failed the request.
  pub fn ask(&mut self, kind: GameRequestKind) -> XrfResult<String> {
    if !self.game.as_mut().is_some_and(GameChild::is_running) {
      return Err(XrfError::new_invalid_error("no game runs; start one with game_start"));
    }

    let game: &mut GameChild = self.game.as_mut().expect("a running game was found above");
    let response: GameResponse = game.request(kind, Self::ANSWER_WAIT)?;

    match (response.ok, response.result) {
      (true, Some(result)) => Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())),
      (true, None) => Ok(String::from("done")),
      (false, _) => Err(XrfError::new_io_error(
        response.error.unwrap_or_default(),
        ErrorKind::Other,
      )),
    }
  }

  /// The last `lines` lines of the game's newest log.
  ///
  /// # Errors
  ///
  /// Returns why no log can be read.
  pub fn read_log(&self, lines: usize) -> XrfResult<String> {
    self.logs.read_tail(lines)
  }

  /// Quits the running game and waits for it to exit.
  ///
  /// # Errors
  ///
  /// Returns why its exit could not be waited for.
  pub fn quit(&mut self) -> XrfResult<String> {
    match self.game.take() {
      Some(game) => Ok(format!("The game exited: {}", game.quit()?)),
      None => Ok(String::from("No game runs")),
    }
  }
}
