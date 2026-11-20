use serde::{Deserialize, Serialize};

/// What a viewer asks of a level's music.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WorldMusicCommand {
  /// Back to the game's schedule: a track played by hand stops, and the next is picked as the game would.
  Schedule,
  /// Plays one track by its entry's name from its start, and again each time it ends, until the schedule is asked back.
  Play { track: String },
  /// Plays the track playing from `position` milliseconds in.
  Seek { position: u64 },
}
