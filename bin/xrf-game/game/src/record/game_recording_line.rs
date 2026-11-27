use serde::{Deserialize, Serialize};

use crate::record::game_recorded_command::GameRecordedCommand;
use crate::record::game_recording_header::GameRecordingHeader;

/// One line of a recording, a JSON value a line as the control endpoint's protocol is: the header first, then each
/// command with the tick that took it and a digest of the state every [`crate::GameStateDigest::INTERVAL`] ticks, in
/// tick order, and the tick the session stopped at last. A recording cut short by a crash has no last line and ends
/// where its lines do.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameRecordingLine {
  Header(GameRecordingHeader),
  Command {
    tick: u64,
    command: GameRecordedCommand,
  },
  /// The state's digest as it stood at `tick`, sixteen hexadecimal digits.
  Digest {
    tick: u64,
    digest: String,
  },
  End {
    tick: u64,
  },
}
