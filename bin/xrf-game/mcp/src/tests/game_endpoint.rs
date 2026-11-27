use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::task::{Wake, Waker};

use crate::endpoint::game_endpoint::GameEndpoint;
use crate::endpoint::game_endpoint_inbox::GameEndpointInbox;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;

/// Counts the wakes it is asked for.
#[derive(Default)]
struct CountingWake(AtomicU32);

impl Wake for CountingWake {
  fn wake(self: Arc<Self>) {
    self.wake_by_ref();
  }

  fn wake_by_ref(self: &Arc<Self>) {
    self.0.fetch_add(1, Ordering::AcqRel);
  }
}

#[test]
fn reads_each_request_skips_what_is_none_and_quits_when_its_input_ends_waking_the_game_to_each() {
  let (sender, receiver) = mpsc::channel::<GameRequest>();
  let wakes: Arc<CountingWake> = Arc::new(CountingWake::default());
  let inbox: GameEndpointInbox = GameEndpointInbox {
    sender,
    waker: Waker::from(Arc::clone(&wakes)),
  };
  let input: Cursor<&str> =
    Cursor::new("{\"id\":\"1\",\"kind\":\"status\"}\nnonsense\n\n{\"id\":\"2\",\"kind\":\"capture\"}\n");

  GameEndpoint::read_requests(input, &inbox);

  let requests: Vec<GameRequest> = receiver.try_iter().collect();

  assert_eq!(
    requests,
    vec![
      GameRequest {
        id: String::from("1"),
        kind: GameRequestKind::Status,
      },
      GameRequest {
        id: String::from("2"),
        kind: GameRequestKind::Capture,
      },
      GameRequest {
        id: String::from(GameRequest::END_OF_INPUT),
        kind: GameRequestKind::Quit,
      },
    ]
  );
  // One wake a request handed over, none for a line that held none.
  assert_eq!(wakes.0.load(Ordering::Acquire), 3);
}

#[test]
fn stops_reading_and_wakes_nobody_once_the_game_takes_no_more_requests() {
  let (sender, receiver) = mpsc::channel::<GameRequest>();
  let wakes: Arc<CountingWake> = Arc::new(CountingWake::default());
  let inbox: GameEndpointInbox = GameEndpointInbox {
    sender,
    waker: Waker::from(Arc::clone(&wakes)),
  };

  drop(receiver);

  assert!(!GameEndpoint::forward_requests(
    Cursor::new("{\"id\":\"1\",\"kind\":\"status\"}\n"),
    &inbox
  ));
  assert_eq!(wakes.0.load(Ordering::Acquire), 0);
}
