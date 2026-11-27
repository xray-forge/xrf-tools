use std::io::Cursor;
use std::sync::mpsc;

use crate::endpoint::game_endpoint::GameEndpoint;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;

#[test]
fn reads_each_request_skips_what_is_none_and_quits_when_its_input_ends() {
  let (sender, receiver) = mpsc::channel::<GameRequest>();
  let input: Cursor<&str> =
    Cursor::new("{\"id\":\"1\",\"kind\":\"status\"}\nnonsense\n\n{\"id\":\"2\",\"kind\":\"capture\"}\n");

  GameEndpoint::read_requests(input, &sender);

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
}
