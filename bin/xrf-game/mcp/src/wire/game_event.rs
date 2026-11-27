use serde::{Deserialize, Serialize};

/// A line the game writes unasked, named by its `event`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
  /// The level is shown and the game takes requests.
  Ready { level: String },
}
