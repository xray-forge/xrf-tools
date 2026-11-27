use serde::{Deserialize, Serialize};

use crate::wire::game_request_kind::GameRequestKind;

/// One request line the bridge writes to the game's standard input: an id its answer echoes, and what it asks, as
/// `{"id":"3","kind":"pose","position":[1,2,3],"target":[1,2,4]}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameRequest {
  pub id: String,
  #[serde(flatten)]
  pub kind: GameRequestKind,
}

impl GameRequest {
  /// The id of the quit the game's reader asks for itself once its input ends, which no answer is read for.
  pub const END_OF_INPUT: &'static str = "end-of-input";

  /// The request one line holds, none for a line that is not one: the reader skips it.
  pub fn parse(line: &str) -> Option<Self> {
    serde_json::from_str(line.trim()).ok()
  }
}
