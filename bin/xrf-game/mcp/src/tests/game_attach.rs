use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::bridge::game_attached::GameAttached;
use crate::endpoint::game_endpoint::GameEndpoint;
use crate::wire::game_attach::GameAttach;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

/// An empty folder of its own under the system's temporary one.
fn create_directory(name: &str) -> PathBuf {
  let directory: PathBuf = std::env::temp_dir().join(format!("xrf-game-mcp-attach-{name}-{}", std::process::id()));

  let _ = fs::remove_dir_all(&directory);

  directory
}

/// Answers `endpoint`'s first `count` requests as the game would, a status with its level, waiting for them until
/// `until`; the endpoint dropped after, as the game exiting drops it.
fn serve(endpoint: GameEndpoint, mut count: usize, until: Instant) {
  while count > 0 && Instant::now() < until {
    if let Some(GameRequest { id, kind }) = endpoint.take_request() {
      let response: GameResponse = match kind {
        GameRequestKind::Status => GameResponse::succeed(&id, json!({ "level": "zaton" })),
        _ => GameResponse::fail(&id, String::from("not in this test")),
      };

      endpoint.answer(&response);
      count -= 1;
    }

    thread::sleep(Duration::from_millis(5));
  }
}

/// Attaches as `attach` says, asks for the status and leaves, answering whether the game answered.
fn attach_once(attach: GameAttach) -> bool {
  let mut bridge: GameAttached = GameAttached::connect(attach).expect("attached");
  let response: GameResponse = bridge
    .channel
    // Past the token wait a silent connection before it may hold the listener for.
    .request(GameRequestKind::Status, Duration::from_secs(10))
    .expect("answered");

  bridge.leave();

  response.ok && response.result == Some(json!({ "level": "zaton" }))
}

#[test]
fn a_bridge_attaches_by_the_token_written_beside_the_game_and_a_stranger_does_not() {
  let directory: PathBuf = create_directory("token");
  let endpoint: GameEndpoint = GameEndpoint::listen(&directory, "zaton").expect("listening");
  let file: PathBuf = directory.join(GameAttach::FILE_NAME);
  let attach: GameAttach = serde_json::from_slice(&fs::read(&file).expect("the attach file")).expect("read");

  assert_eq!(attach.level, "zaton");
  assert_eq!(attach.process, std::process::id());

  // A stranger's first line is no token: turned away, its request never reaching the game.
  let mut stranger: TcpStream = TcpStream::connect((Ipv4Addr::LOCALHOST, attach.port)).expect("connected");

  writeln!(stranger, "guess\n{{\"id\":\"9\",\"kind\":\"quit\"}}").expect("written");

  let mut closed: String = String::new();

  stranger
    .set_read_timeout(Some(Duration::from_secs(5)))
    .expect("a timeout");
  assert_eq!(BufReader::new(&stranger).read_line(&mut closed).ok(), Some(0));
  assert!(endpoint.take_request().is_none());

  // A silent connection is let go once its token wait runs out, holding nobody up.
  let silent: TcpStream = TcpStream::connect((Ipv4Addr::LOCALHOST, attach.port)).expect("connected");
  let server = thread::spawn(move || serve(endpoint, 2, Instant::now() + Duration::from_secs(20)));

  // One bridge leaving quits nothing: the next attaches.
  assert!(attach_once(attach.clone()));
  assert!(attach_once(attach));
  server.join().expect("served");
  drop(silent);

  assert!(!file.exists(), "the attach file goes with the endpoint");

  let _ = fs::remove_dir_all(&directory);
}
