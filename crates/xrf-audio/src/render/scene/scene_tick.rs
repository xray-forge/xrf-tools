use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::control::audio_engine::AudioEngine;
use crate::render::sound_bank::SoundBank;
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
}
