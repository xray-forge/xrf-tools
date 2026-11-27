use std::io::{self, BufRead, Stdout, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, PoisonError};
use std::thread;

use serde::Serialize;
use xrf_error::XrfResult;

use crate::wire::game_event::GameEvent;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// The game's side of the control endpoint: a thread reading request lines from its standard input into a channel the
/// game drains between frames, and its standard output written a line an answer. Its input ending quits the game, so a
/// bridge that dies leaves no game holding its executable.
pub struct GameEndpoint {
  requests: Receiver<GameRequest>,
  output: Mutex<Stdout>,
}

impl GameEndpoint {
  /// Starts reading the game's standard input.
  ///
  /// # Errors
  ///
  /// Returns why the reader's thread cannot be started.
  pub fn start() -> XrfResult<Self> {
    let (sender, requests) = mpsc::channel::<GameRequest>();

    thread::Builder::new()
      .name(String::from("xrf-game-mcp"))
      .spawn(move || Self::read_requests(io::stdin().lock(), &sender))?;

    log::info!("Control endpoint reads the game's standard input");

    Ok(Self {
      requests,
      output: Mutex::new(io::stdout()),
    })
  }

  /// The next request waiting, none where none is.
  pub fn take_request(&self) -> Option<GameRequest> {
    self.requests.try_recv().ok()
  }

  pub fn answer(&self, response: &GameResponse) {
    self.write_line(response);
  }

  pub fn announce(&self, event: &GameEvent) {
    self.write_line(event);
  }

  /// Sends each request `input` holds, a line each, skipping a line that holds none, then a quit once it ends.
  pub(crate) fn read_requests(input: impl BufRead, sender: &Sender<GameRequest>) {
    for line in input.lines() {
      let line: String = match line {
        Ok(line) => line,
        Err(error) => {
          log::warn!("Control endpoint cannot read its input: {error}");

          break;
        }
      };

      match GameRequest::parse(&line) {
        Some(request) => {
          if sender.send(request).is_err() {
            return;
          }
        }
        None if line.trim().is_empty() => {}
        None => log::warn!("Control endpoint skips a line that is no request: {line}"),
      }
    }

    let _ = sender.send(GameRequest {
      id: String::from(GameRequest::END_OF_INPUT),
      kind: GameRequestKind::Quit,
    });
  }

  fn write_line(&self, message: &impl Serialize) {
    let line: String = match serde_json::to_string(message) {
      Ok(line) => line,
      Err(error) => {
        log::error!("Control endpoint cannot write an answer: {error}");

        return;
      }
    };
    let mut output = self.output.lock().unwrap_or_else(PoisonError::into_inner);

    if let Err(error) = writeln!(output, "{line}").and_then(|()| output.flush()) {
      log::warn!("Control endpoint's output is closed: {error}");
    }
  }
}
