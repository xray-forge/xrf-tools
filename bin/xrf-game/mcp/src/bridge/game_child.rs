use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_line::GameLine;
use crate::wire::game_event::GameEvent;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// A game the bridge started as its child: its standard input taking requests, its standard output read into a channel
/// of answers and events by a thread of its own. Its standard error is not piped: its log file holds every record.
/// Dropped, it closes the game's input, which quits it, and kills it only where it does not quit in time.
pub(crate) struct GameChild {
  child: Child,
  input: Option<ChildStdin>,
  lines: Receiver<GameLine>,
  next_id: u64,
}

impl GameChild {
  /// How long a dropped or quitting game is given to exit before it is killed.
  const EXIT_WAIT: Duration = Duration::from_secs(15);

  /// How often a wait for the game's exit looks again.
  const EXIT_POLL: Duration = Duration::from_millis(50);

  /// Starts `executable` with `arguments` and its control endpoint on.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot be started.
  pub(crate) fn start(executable: &Path, arguments: &[String]) -> XrfResult<Self> {
    let mut child: Child = Command::new(executable)
      .args(arguments)
      .arg("--mcp")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|error| {
        XrfError::new_io_error(format!("{} cannot start: {error}", executable.display()), error.kind())
      })?;
    let output: ChildStdout = child
      .stdout
      .take()
      .ok_or_else(|| XrfError::new_io_error("the game's output is not piped", ErrorKind::BrokenPipe))?;
    let input: Option<ChildStdin> = child.stdin.take();
    let (sender, lines) = mpsc::channel::<GameLine>();

    thread::Builder::new()
      .name(String::from("xrf-game-mcp-output"))
      .spawn(move || Self::read_lines(BufReader::new(output), &sender))?;

    Ok(Self {
      child,
      input,
      lines,
      next_id: 1,
    })
  }

  /// Whether the game still runs.
  pub(crate) fn is_running(&mut self) -> bool {
    matches!(self.child.try_wait(), Ok(None))
  }

  /// Waits for the game to say it is ready, answering the level it shows.
  ///
  /// # Errors
  ///
  /// Returns why it never said so: it exited first, or `timeout` passed.
  pub(crate) fn wait_ready(&mut self, timeout: Duration) -> XrfResult<String> {
    let deadline: Instant = Instant::now() + timeout;

    loop {
      match self.receive(deadline)? {
        GameLine::Event(GameEvent::Ready { level }) => return Ok(level),
        GameLine::Response(_) => {}
      }
    }
  }

  /// Sends a request and waits up to `timeout` for its answer, skipping any other line.
  ///
  /// # Errors
  ///
  /// Returns why no answer came: the game's input is closed, it exited, or `timeout` passed.
  pub(crate) fn request(&mut self, kind: GameRequestKind, timeout: Duration) -> XrfResult<GameResponse> {
    let id: String = self.next_id.to_string();

    self.next_id += 1;

    let line: String = serde_json::to_string(&GameRequest { id: id.clone(), kind })
      .map_err(|error| XrfError::new_io_error(error.to_string(), ErrorKind::InvalidData))?;
    let input: &mut ChildStdin = self
      .input
      .as_mut()
      .ok_or_else(|| XrfError::new_io_error("the game's input is closed", ErrorKind::BrokenPipe))?;

    writeln!(input, "{line}").and_then(|()| input.flush())?;

    let deadline: Instant = Instant::now() + timeout;

    loop {
      if let GameLine::Response(response) = self.receive(deadline)?
        && response.id == id
      {
        return Ok(response);
      }
    }
  }

  /// Quits the game: closes its input, which it reads as a quit, and waits for it to exit, killing it where it does not.
  ///
  /// # Errors
  ///
  /// Returns why its exit could not be waited for.
  pub(crate) fn quit(mut self) -> XrfResult<ExitStatus> {
    self.close_and_wait()
  }

  fn close_and_wait(&mut self) -> XrfResult<ExitStatus> {
    drop(self.input.take());

    let deadline: Instant = Instant::now() + Self::EXIT_WAIT;

    loop {
      if let Some(status) = self.child.try_wait()? {
        return Ok(status);
      }

      if Instant::now() >= deadline {
        log::warn!("The game did not exit within {:?}; killing it", Self::EXIT_WAIT);
        self.child.kill()?;

        return Ok(self.child.wait()?);
      }

      thread::sleep(Self::EXIT_POLL);
    }
  }

  fn receive(&mut self, deadline: Instant) -> XrfResult<GameLine> {
    match self
      .lines
      .recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
      Ok(line) => Ok(line),
      Err(RecvTimeoutError::Timeout) => Err(XrfError::new_io_error(
        "the game did not answer in time",
        ErrorKind::TimedOut,
      )),
      Err(RecvTimeoutError::Disconnected) => {
        let status: String = match self.child.try_wait() {
          Ok(Some(status)) => status.to_string(),
          _ => String::from("its output closed"),
        };

        Err(XrfError::new_io_error(
          format!("the game exited: {status}"),
          ErrorKind::BrokenPipe,
        ))
      }
    }
  }

  fn read_lines(output: impl BufRead, sender: &Sender<GameLine>) {
    for line in output.lines().map_while(Result::ok) {
      if let Some(line) = GameLine::parse(&line)
        && sender.send(line).is_err()
      {
        return;
      }
    }
  }
}

impl Drop for GameChild {
  fn drop(&mut self) {
    if self.is_running()
      && let Err(error) = self.close_and_wait()
    {
      log::warn!("The game could not be stopped: {error}");
    }
  }
}
