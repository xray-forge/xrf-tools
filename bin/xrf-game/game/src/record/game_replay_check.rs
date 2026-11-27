use std::collections::BTreeMap;

use xrf_game_loop::{LoopStepper, LoopWitness};

use crate::record::game_state_digest::GameStateDigest;
use crate::state::game_command::GameCommand;
use crate::state::game_state::GameState;

/// Checks a replay against its recording as the loop steps it: at each tick the recording holds a digest for, the
/// state's digest is compared with it. The first that differs is logged as an error with its tick, the replay having
/// parted from the recording since the digest before; the recording's end is logged with how many matched.
pub struct GameReplayCheck {
  /// The recording's digests by tick.
  digests: BTreeMap<u64, String>,
  /// The tick the recording ends at.
  end: u64,
  matched: usize,
  /// The tick of the first digest that differed.
  parted_at: Option<u64>,
}

impl GameReplayCheck {
  /// A check of a replay to tick `end` against the recording's `digests` by tick.
  pub fn new(digests: BTreeMap<u64, String>, end: u64) -> Self {
    Self {
      digests,
      end,
      matched: 0,
      parted_at: None,
    }
  }

  /// How many digests matched so far.
  pub fn count_matched(&self) -> usize {
    self.matched
  }

  /// The tick of the first digest that differed from the recording's; none while the replay follows it.
  pub fn get_parted_at(&self) -> Option<u64> {
    self.parted_at
  }

  /// Compares `state` at `tick` with the recording, where it holds a digest for the tick and the replay still follows
  /// it.
  pub fn compare(&mut self, tick: u64, state: &GameState) {
    if self.parted_at.is_some() {
      return;
    }

    let Some(recorded) = self.digests.get(&tick) else {
      return;
    };
    let digest: String = GameStateDigest::of(tick, state).to_string();

    if digest == *recorded {
      self.matched += 1;
    } else {
      self.parted_at = Some(tick);
      log::error!(
        "The replay parted from its recording by tick {tick}: the state's digest is {digest}, recorded {recorded}"
      );
    }
  }

  fn log_end(&self) {
    let count: usize = self.digests.range(..=self.end).count();

    match self.parted_at {
      None => log::info!(
        "The replay followed its recording to its end at tick {}: all {count} state digests matched",
        self.end
      ),
      Some(tick) => log::error!(
        "The replay reached its recording's end at tick {} having parted from it by tick {tick}: {} of {count} state \
         digests matched before",
        self.end,
        self.matched
      ),
    }
  }
}

impl LoopWitness<GameState> for GameReplayCheck {
  fn see_commands(&mut self, _tick: u64, _commands: &[GameCommand]) {}

  fn see_step(&mut self, stepper: &LoopStepper<GameState>) {
    let tick: u64 = stepper.get_time().tick;

    if tick > self.end {
      return;
    }

    self.compare(tick, stepper.get_state());

    if tick == self.end {
      self.log_end();
    }
  }
}
