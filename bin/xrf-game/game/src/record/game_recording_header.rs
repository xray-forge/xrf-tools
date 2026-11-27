use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::LoopClock;
use xrf_game_movement::MovementMode;

use crate::session::game_load_request::GameLoadRequest;
use crate::session::game_load_source::GameLoadSource;

/// A recording's first line: what its session started from, which a replay starts from again. The game tree is not
/// named: a replay is told its own, and one whose tree differs parts from the recording at its first digest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRecordingHeader {
  /// The recording format's version; a recording of another is refused rather than guessed at.
  pub version: u32,
  /// The level's name in the tree.
  pub level: String,
  /// The new game's seed.
  pub seed: u64,
  /// How the player moved as the session began.
  pub mode: MovementMode,
  /// Ticks a second the loop stepped at.
  pub tick_rate: u32,
}

impl GameRecordingHeader {
  /// The format this build writes and replays. A recording is a debugging aid, not a save: it holds no promise to
  /// replay on a later build, whose steps may differ.
  pub const VERSION: u32 = 1;

  /// The header of a session `request` opens, stepped at the loop's default rate.
  pub fn of(request: &GameLoadRequest) -> Self {
    let GameLoadSource::New { seed } = request.game;

    Self {
      version: Self::VERSION,
      level: request.level.clone(),
      seed,
      mode: request.mode,
      tick_rate: LoopClock::DEFAULT_TICK_RATE,
    }
  }

  /// Whether this build replays the recording.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for another format version or tick rate than this build's.
  pub fn check(&self) -> XrfResult<()> {
    if self.version != Self::VERSION {
      return Err(XrfError::new_invalid_error(format!(
        "the recording is of format {}, and this build replays format {}",
        self.version,
        Self::VERSION
      )));
    }

    if self.tick_rate != LoopClock::DEFAULT_TICK_RATE {
      return Err(XrfError::new_invalid_error(format!(
        "the recording stepped {} ticks a second, and this build steps {}",
        self.tick_rate,
        LoopClock::DEFAULT_TICK_RATE
      )));
    }

    Ok(())
  }
}
