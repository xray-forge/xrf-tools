use crate::wire::game_event::GameEvent;
use crate::wire::game_response::GameResponse;

/// One line the game wrote on its standard output: an answer, or a line it sent unasked.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GameLine {
  Response(GameResponse),
  Event(GameEvent),
}

impl GameLine {
  /// What `line` holds, none for a line that is neither: the bridge skips it.
  pub(crate) fn parse(line: &str) -> Option<Self> {
    let line: &str = line.trim();

    serde_json::from_str::<GameEvent>(line)
      .map(Self::Event)
      .or_else(|_| serde_json::from_str::<GameResponse>(line).map(Self::Response))
      .ok()
  }
}
