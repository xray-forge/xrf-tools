use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use xrf_directories::DataDirectories;
use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_attached::GameAttached;
use crate::bridge::game_child::GameChild;
use crate::bridge::game_link::GameLink;
use crate::bridge::game_logs::GameLogs;
use crate::wire::game_attach::GameAttach;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// What the bridge does for each of its tools: starts the game beside it as its child, or attaches to one started by
/// hand with `--listen`, and asks it; or reads its log from disk, which answers whether a game runs or not.
pub struct GameBridge {
  executable: PathBuf,
  logs: GameLogs,
  /// Where a listening game writes how to attach to it, best first.
  attach_files: Vec<PathBuf>,
  game: Option<GameLink>,
}

impl GameBridge {
  /// How long a start waits for the level: a first run compiles the renderer's shaders.
  const READY_WAIT: Duration = Duration::from_secs(300);

  /// How long a request waits for its answer; a capture waits for the next frame.
  const ANSWER_WAIT: Duration = Duration::from_secs(30);

  /// The game's local data identifier and the folder it keeps its own files in under `appdata`, as the game names them.
  const IDENTIFIER: &'static str = "com.xrf.game";
  const FOLDER: &'static str = "xrf-game";

  /// A bridge over `executable`, the game's, its logs in `logs`, a listening game's attach file among `attach_files`.
  pub fn new(executable: PathBuf, logs: GameLogs, attach_files: Vec<PathBuf>) -> Self {
    Self {
      executable,
      logs,
      attach_files,
      game: None,
    }
  }

  /// A bridge over the game built beside it, its logs and its attach file where the game writes them: its own folder
  /// beside the executable both sit beside, else its local data directory.
  pub fn locate() -> Self {
    let executable: PathBuf = std::env::current_exe()
      .ok()
      .and_then(|bridge| bridge.parent().map(PathBuf::from))
      .unwrap_or_default()
      .join(format!("xrf-game{}", std::env::consts::EXE_SUFFIX));
    let data: DataDirectories = DataDirectories::for_identifier(Self::IDENTIFIER);
    let attach_files: Vec<PathBuf> = DataDirectories::get_writable_executable_directory()
      .map(|directory| directory.join(DataDirectories::APP_DATA_DIRECTORY).join(Self::FOLDER))
      .into_iter()
      .chain(data.get_local_data().map(PathBuf::from))
      .map(|directory| directory.join(GameAttach::FILE_NAME))
      .collect();

    Self::new(executable, GameLogs::locate(), attach_files)
  }

  /// Starts the game on `level` of the tree at `root`, with `arguments` after them, and waits until it shows it.
  ///
  /// # Errors
  ///
  /// Returns why it did not come up: one runs already, it cannot start, or it exited first (its log says why).
  pub fn start(&mut self, root: &str, level: &str, arguments: &[String]) -> XrfResult<String> {
    self.refuse_second()?;

    let mut all: Vec<String> = vec![
      String::from("--root"),
      String::from(root),
      String::from("--level"),
      String::from(level),
    ];

    all.extend_from_slice(arguments);

    let mut game: GameChild = GameChild::start(&self.executable, &all)?;
    let shown: String = game.channel.wait_ready(Self::READY_WAIT)?;

    self.game = Some(GameLink::Child(game));

    Ok(format!("The game shows '{shown}'"))
  }

  /// Attaches to the game started by hand with `--listen`, read from the first attach file found, and confirms it
  /// answers.
  ///
  /// # Errors
  ///
  /// Returns why it cannot attach: one is driven already, no game listens, or it does not answer.
  pub fn attach(&mut self) -> XrfResult<String> {
    self.refuse_second()?;

    let file: &PathBuf = self.attach_files.iter().find(|file| file.is_file()).ok_or_else(|| {
      XrfError::new_not_found_error("no game listens; start one by hand with --listen, or use game_start")
    })?;
    let attach: GameAttach = serde_json::from_slice(&fs::read(file)?)
      .map_err(|error| XrfError::new_invalid_error(format!("'{}' does not read: {error}", file.display())))?;
    let mut game: GameAttached = GameAttached::connect(attach).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "the game '{}' names does not answer ({error}); it may have crashed",
        file.display()
      ))
    })?;

    game.channel.request(GameRequestKind::Status, Self::ANSWER_WAIT)?;

    let answer: String = format!(
      "Attached to the game showing '{}' (process {})",
      game.attach.level, game.attach.process
    );

    self.game = Some(GameLink::Attached(game));

    Ok(answer)
  }

  /// Leaves the game attached to, which plays on; a game started as the bridge's child is quit instead.
  ///
  /// # Errors
  ///
  /// Returns why a child's exit could not be waited for.
  pub fn detach(&mut self) -> XrfResult<String> {
    match self.game.take() {
      Some(GameLink::Attached(mut game)) => {
        game.leave();

        Ok(String::from("Detached; the game plays on"))
      }
      Some(GameLink::Child(game)) => Ok(format!("The game was the bridge's own, so it quit: {}", game.quit()?)),
      None => Ok(String::from("No game is driven")),
    }
  }

  /// Asks the game driven, answering what it found.
  ///
  /// # Errors
  ///
  /// Returns why it could not answer: no game is driven, it did not answer in time, or it failed the request.
  pub fn ask(&mut self, kind: GameRequestKind) -> XrfResult<String> {
    let none = || XrfError::new_invalid_error("no game runs; start one with game_start or attach with game_attach");
    let game: &mut GameLink = self.game.as_mut().ok_or_else(none)?;

    if !game.is_running() {
      return Err(none());
    }

    let response: GameResponse = game.get_channel_mut().request(kind, Self::ANSWER_WAIT)?;

    match (response.ok, response.result) {
      (true, Some(result)) => Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())),
      (true, None) => Ok(String::from("done")),
      // The game refused the request, in its own words.
      (false, _) => Err(XrfError::new_invalid_error(response.error.unwrap_or_default())),
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

  /// Quits the game driven: a child is closed and waited for; an attached game is asked to quit, as its close button
  /// does.
  ///
  /// # Errors
  ///
  /// Returns why a child's exit could not be waited for, or an attached game did not take the quit.
  pub fn quit(&mut self) -> XrfResult<String> {
    match self.game.take() {
      Some(GameLink::Child(game)) => Ok(format!("The game exited: {}", game.quit()?)),
      Some(GameLink::Attached(mut game)) => {
        game.channel.request(GameRequestKind::Quit, Self::ANSWER_WAIT)?;

        Ok(String::from("The attached game is quitting"))
      }
      None => Ok(String::from("No game runs")),
    }
  }

  /// Refuses a second game while one is driven.
  fn refuse_second(&mut self) -> XrfResult<()> {
    if self.game.as_mut().is_some_and(GameLink::is_running) {
      return Err(XrfError::new_invalid_error(
        "a game is driven already; quit it or game_detach first",
      ));
    }

    Ok(())
  }
}
