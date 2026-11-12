use std::time::Duration;

use crate::acoustics::scene::simulation_pace::SimulationPace;
use crate::scene::render::bank::sound_bank::SoundBank;
use crate::scene::render::stream_launcher::StreamLauncher;

/// How a control side runs: where its sounds are read and its streams decode, how often it ticks, and how its scenes'
/// acoustics keep up.
pub(crate) struct ControlRuntime {
  pub(crate) bank: SoundBank,
  pub(crate) streams: StreamLauncher,
  /// How long passes between two ticks, at most.
  pub(crate) tick: Duration,
  pub(crate) pace: SimulationPace,
}

impl ControlRuntime {
  /// The control thread's: sounds read on a thread of their own, each stream on its own thread, every acoustics
  /// simulation in real time.
  pub(crate) fn threaded(tick: Duration) -> Self {
    Self {
      bank: SoundBank::threaded(),
      streams: StreamLauncher::Threads,
      tick,
      pace: SimulationPace::RealTime,
    }
  }
}
