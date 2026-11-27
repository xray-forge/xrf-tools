use serde_json::json;

use crate::bridge::game_line::GameLine;
use crate::wire::game_event::GameEvent;
use crate::wire::game_input::GameInput;
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
  assert_eq!(
    GameRequest::parse(r#"{"id":"8","kind":"console","line":"list shadows"}"#).map(|request| request.kind),
    Some(GameRequestKind::Console {
      line: String::from("list shadows"),
    })
  );
  assert_eq!(GameRequest::parse(r#"{"id":"5","kind":"dance"}"#), None);
  assert_eq!(GameRequest::parse("not json"), None);
}

#[test]
fn reads_input_steps_by_type_with_optional_fields_defaulted() {
  assert_eq!(
    GameRequest::parse(
      r#"{"id":"6","kind":"input","inputs":[{"type":"press","key":"Grave"},{"type":"wait","ms":100},
      {"type":"text","text":"help"},{"type":"move","dx":5,"dy":-2},{"type":"click","x":10,"y":20}]}"#
    )
    .map(|request| request.kind),
    Some(GameRequestKind::Input {
      inputs: vec![
        GameInput::Press {
          key: String::from("Grave")
        },
        GameInput::Wait { ms: 100 },
        GameInput::Text {
          text: String::from("help")
        },
        GameInput::Move {
          dx: 5.0,
          dy: -2.0,
          x: 0.0,
          y: 0.0
        },
        GameInput::Click {
          x: 10.0,
          y: 20.0,
          button: None
        },
      ]
    })
  );
  assert_eq!(
    GameRequest::parse(r#"{"id":"7","kind":"input","inputs":[{"type":"jump"}]}"#),
    None
  );
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
