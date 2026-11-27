use serde_json::json;

use crate::bridge::game_line::GameLine;
use crate::wire::game_event::GameEvent;
use crate::wire::game_request::GameRequest;
use crate::wire::game_request_kind::GameRequestKind;
use crate::wire::game_response::GameResponse;

#[test]
fn reads_a_request_line_with_its_kinds_arguments_beside_its_id() {
  assert_eq!(
    GameRequest::parse(r#" {"id":"3","kind":"pose","position":[1,2,3],"target":[1,2,4]} "#),
    Some(GameRequest {
      id: String::from("3"),
      kind: GameRequestKind::Pose {
        position: [1.0, 2.0, 3.0],
        target: [1.0, 2.0, 4.0],
      },
    })
  );
  assert_eq!(
    GameRequest::parse(r#"{"id":"4","kind":"status"}"#).map(|request| request.kind),
    Some(GameRequestKind::Status)
  );
  assert_eq!(GameRequest::parse(r#"{"id":"5","kind":"dance"}"#), None);
  assert_eq!(GameRequest::parse("not json"), None);
}

#[test]
fn writes_a_request_as_one_flat_object() {
  let line: String = serde_json::to_string(&GameRequest {
    id: String::from("1"),
    kind: GameRequestKind::Capture,
  })
  .expect("it serializes");

  assert_eq!(line, r#"{"id":"1","kind":"capture"}"#);
}

#[test]
fn tells_an_answer_from_an_event_and_skips_anything_else() {
  assert_eq!(
    GameLine::parse(r#"{"event":"ready","level":"zaton"}"#),
    Some(GameLine::Event(GameEvent::Ready {
      level: String::from("zaton")
    }))
  );
  assert_eq!(
    GameLine::parse(r#"{"id":"2","ok":true,"result":{"tick":5}}"#),
    Some(GameLine::Response(GameResponse::succeed("2", json!({ "tick": 5 }))))
  );
  assert_eq!(
    GameLine::parse(r#"{"id":"2","ok":false,"error":"no frame"}"#),
    Some(GameLine::Response(GameResponse::fail("2", "no frame")))
  );
  assert_eq!(GameLine::parse("[2026] a stray line"), None);
}
