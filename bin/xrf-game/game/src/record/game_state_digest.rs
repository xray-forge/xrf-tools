use std::fmt::{self, Display, Formatter, Write};

use crate::state::game_progress::GameProgress;
use crate::state::game_state::GameState;

/// A number standing for the game's state between two ticks: two sessions whose states agree have the same one, and
/// two that parted almost surely do not, so a replay is compared with its recording a number at a time. It covers what
/// the steps move: the game's seed and time, every table of the object registry (the player's record among them),
/// what A-Life keeps between updates, the level's place on the game graph and the player's movement. The physics world
/// is covered through what it moves, the player, as no engine compares one whole.
///
/// It reads the state as its `Debug` text, which names every field of every table, so a field added to the state is
/// covered without a line here, and hashes that text with FNV-1a, the same on every build. A registry of twenty
/// thousand objects takes milliseconds this way, so it is taken at an interval, never every tick.
// todo: hash the state's saved bytes instead, once the game's state is written to a save.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameStateDigest(u64);

impl GameStateDigest {
  /// Ticks between two digests of a recorded or replayed session: one a simulated second.
  pub const INTERVAL: u64 = 100;

  /// The digest of `state` as it stands at `tick`.
  pub fn of(tick: u64, state: &GameState) -> Self {
    let GameProgress {
      seed,
      time,
      objects,
      alife,
      player,
    } = &state.progress;
    let mut hasher: DigestHasher = DigestHasher::default();

    // Writing into the hasher never fails.
    let _ = write!(
      hasher,
      "{tick} {seed} {time:?} {objects:?} {:?} {:?} {player:?} {:?} {:?}",
      alife.get_until_update(),
      alife.get_schedule(),
      state.level.graph_level,
      state.movement,
    );

    Self(hasher.hash)
  }
}

impl Display for GameStateDigest {
  /// Sixteen hexadecimal digits, as a recording writes it.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    write!(formatter, "{:016x}", self.0)
  }
}

/// FNV-1a over the text written into it.
struct DigestHasher {
  hash: u64,
}

impl DigestHasher {
  const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
  const PRIME: u64 = 0x0000_0100_0000_01b3;
}

impl Default for DigestHasher {
  fn default() -> Self {
    Self {
      hash: Self::OFFSET_BASIS,
    }
  }
}

impl Write for DigestHasher {
  fn write_str(&mut self, text: &str) -> fmt::Result {
    for byte in text.bytes() {
      self.hash = (self.hash ^ u64::from(byte)).wrapping_mul(Self::PRIME);
    }

    Ok(())
  }
}
