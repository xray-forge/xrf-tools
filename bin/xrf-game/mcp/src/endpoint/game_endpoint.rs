use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use xrf_error::{XrfError, XrfResult};

use crate::wire::game_attach::GameAttach;
use crate::wire::game_event::GameEvent;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// Where the endpoint's answers go: the stream of whoever drives the game, none while nobody does.
type GameEndpointOutput = Arc<Mutex<Option<Box<dyn Write + Send>>>>;

/// The game's side of the control endpoint: a thread reading request lines into a channel the game drains between
/// frames, and the driver's stream written a line an answer.
///
/// - Started by the bridge (`start`), it reads the game's standard input and answers on its standard output, and its
///   input ending quits the game, so a bridge that dies leaves no game holding its executable.
/// - Started by hand (`listen`), it listens on a loopback port and writes how to attach (`GameAttach`) into the game's
///   folder, removed as the game exits. A connection's first line must be the run's token; one at a time drives the
///   game, and one leaving quits nothing, so the next attaches.
pub struct GameEndpoint {
  requests: Receiver<GameRequest>,
  output: GameEndpointOutput,
  /// The attach file this endpoint wrote, removed as it is dropped.
  attach_file: Option<PathBuf>,
}

impl GameEndpoint {
  /// How long a connection is given to send its token.
  const TOKEN_WAIT: Duration = Duration::from_secs(5);

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
      output: Arc::new(Mutex::new(Some(Box::new(io::stdout())))),
      attach_file: None,
    })
  }

  /// Listens on a loopback port for a bridge to attach, and writes how into `directory`'s `attach.json`, for the game
  /// showing `level`.
  ///
  /// # Errors
  ///
  /// Returns why the port cannot be opened, the attach file written, or the listening thread started.
  pub fn listen(directory: &Path, level: &str) -> XrfResult<Self> {
    let listener: TcpListener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let attach: GameAttach = GameAttach {
      port: listener.local_addr()?.port(),
      token: uuid::Uuid::new_v4().simple().to_string(),
      process: std::process::id(),
      level: level.to_owned(),
    };
    let attach_file: PathBuf = directory.join(GameAttach::FILE_NAME);

    fs::create_dir_all(directory)?;
    fs::write(
      &attach_file,
      serde_json::to_vec_pretty(&attach).map_err(|error| XrfError::new_serialization_error(error.to_string()))?,
    )?;

    let (sender, requests) = mpsc::channel::<GameRequest>();
    let output: GameEndpointOutput = Arc::new(Mutex::new(None));
    let driven: GameEndpointOutput = Arc::clone(&output);
    let token: String = attach.token;

    thread::Builder::new()
      .name(String::from("xrf-game-mcp-listen"))
      .spawn(move || Self::serve_attached(&listener, &token, &sender, &driven))?;

    log::info!(
      "Control endpoint listens on 127.0.0.1:{}; a bridge attaches by '{}'",
      attach.port,
      attach_file.display()
    );

    Ok(Self {
      requests,
      output,
      attach_file: Some(attach_file),
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

  /// Sends each request the bridge's `input` holds, as [`Self::forward_requests`] does, then a quit once it ends.
  pub(crate) fn read_requests(input: impl BufRead, sender: &Sender<GameRequest>) {
    if Self::forward_requests(input, sender) {
      let _ = sender.send(GameRequest {
        id: String::from(GameRequest::END_OF_INPUT),
        kind: GameRequestKind::Quit,
      });
    }
  }

  /// Sends each request `input` holds, a line each, skipping a line that holds none, until it ends; answers whether the
  /// game still takes requests.
  pub(crate) fn forward_requests(input: impl BufRead, sender: &Sender<GameRequest>) -> bool {
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
            return false;
          }
        }
        None if line.trim().is_empty() => {}
        None => log::warn!("Control endpoint skips a line that is no request: {line}"),
      }
    }

    true
  }

  /// Takes connections one at a time while the game takes requests: one whose first line is `token` drives the game,
  /// its answers going to it, until it leaves.
  fn serve_attached(listener: &TcpListener, token: &str, sender: &Sender<GameRequest>, output: &GameEndpointOutput) {
    for stream in listener.incoming() {
      let stream: TcpStream = match stream {
        Ok(stream) => stream,
        Err(error) => {
          log::warn!("Control endpoint cannot take a connection: {error}");

          continue;
        }
      };
      // A connection that never sends its token would hold the listener from every other.
      if stream.set_read_timeout(Some(Self::TOKEN_WAIT)).is_err() {
        continue;
      }

      let Ok(writer) = stream.try_clone() else {
        continue;
      };
      let mut reader: BufReader<TcpStream> = BufReader::new(stream);
      let mut first: String = String::new();

      if reader.read_line(&mut first).is_err() || first.trim() != token {
        log::warn!("Control endpoint turned away a connection without this run's token");

        continue;
      }

      if reader.get_ref().set_read_timeout(None).is_err() {
        continue;
      }

      log::info!("A bridge attached to the control endpoint");
      *output.lock().unwrap_or_else(PoisonError::into_inner) = Some(Box::new(writer));

      let is_open: bool = Self::forward_requests(reader, sender);

      *output.lock().unwrap_or_else(PoisonError::into_inner) = None;
      log::info!("The bridge left the control endpoint");

      if !is_open {
        return;
      }
    }
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
    let Some(writer) = output.as_mut() else {
      return;
    };

    if let Err(error) = writeln!(writer, "{line}").and_then(|()| writer.flush()) {
      log::warn!("Control endpoint's output is closed: {error}");
    }
  }
}

impl Drop for GameEndpoint {
  fn drop(&mut self) {
    if let Some(file) = &self.attach_file
      && let Err(error) = fs::remove_file(file)
    {
      log::warn!("The attach file '{}' was not removed: {error}", file.display());
    }
  }
}
