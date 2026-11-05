use std::time::Duration;

use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::control::audio_engine::AudioEngine;
use crate::render::bank::sound_bank::SoundBank;
use crate::render::stream_launcher::StreamLauncher;

/// What a control tick hands each scene: the engine its voices play on, where their sounds are read and streamed, how
/// placed sounds reach the ears, and the mixer's clock it plays on to.
pub(crate) struct SceneTick<'a> {
  pub(crate) engine: &'a mut AudioEngine,
  pub(crate) bank: &'a mut SoundBank,
  pub(crate) streams: &'a mut StreamLauncher,
  pub(crate) spatialization: AudioSpatialization,
  /// The mixer's frame the scene plays on to.
  pub(crate) mixer_frame: u64,
  /// The mixer's rate, frames a second.
  pub(crate) rate: u32,
  /// How far past the mixer's frame a sound starting is given its voice now, stamped to the frame it starts at: past the
  /// next tick and the block its commands land in, so no start waits for a tick and lands late.
  pub(crate) lookahead: Duration,
}
