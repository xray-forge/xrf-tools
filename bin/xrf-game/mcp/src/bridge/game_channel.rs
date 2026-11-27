use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::bridge::game_line::GameLine;
use crate::wire::game_event::GameEvent;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// The line protocol with one game, whatever carries it: requests written to its input a line each, and its lines read
/// into a channel of answers and events by a thread of its own, the bridge's child's standard streams or an attached
/// game's socket alike.
pub(crate) struct GameChannel {
  input: Option<Box<dyn Write + Send>>,
  lines: Receiver<GameLine>,
  next_id: u64,
}

impl GameChannel {
  /// The protocol over `input`, the game's requests, and `output`, its answers and events, read on a thread named
  /// `name`.
  ///
  /// # Errors
  ///
  /// Returns why the reading thread cannot be started.
  pub(crate) fn new(input: Box<dyn Write + Send>, output: impl Read + Send + 'static, name: &str) -> XrfResult<Self> {
    let (sender, lines) = mpsc::channel::<GameLine>();

    thread::Builder::new()
      .name(name.to_owned())
      .spawn(move || Self::read_lines(BufReader::new(output), &sender))?;

    Ok(Self {
      input: Some(input),
      lines,
      next_id: 1,
    })
  }

  /// Waits for the game to say it is ready, answering the level it shows.
  ///
  /// # Errors
  ///
  /// Returns why it never said so: its output closed first, or `timeout` passed.
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
  /// Returns why no answer came: the game's input is closed, its output closed, or `timeout` passed.
  pub(crate) fn request(&mut self, kind: GameRequestKind, timeout: Duration) -> XrfResult<GameResponse> {
    let id: String = self.next_id.to_string();

    self.next_id += 1;

    let line: String = serde_json::to_string(&GameRequest { id: id.clone(), kind })
      .map_err(|error| XrfError::new_io_error(error.to_string(), ErrorKind::InvalidData))?;
    let input: &mut Box<dyn Write + Send> = self
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

  /// Closes the game's input: a child reads it as a quit, an attached game as the bridge leaving.
  pub(crate) fn close_input(&mut self) {
    drop(self.input.take());
  }

  /// The next line before `deadline`.
  ///
  /// # Errors
  ///
  /// Returns that none came in time, or that the game's output closed.
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
      Err(RecvTimeoutError::Disconnected) => Err(XrfError::new_io_error(
        "the game is gone: its output closed",
        ErrorKind::BrokenPipe,
      )),
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
