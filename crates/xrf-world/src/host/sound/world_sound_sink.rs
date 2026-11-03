use crate::host::sound::world_sound_frame::WorldSoundFrame;

/// Where a scene's sound frames go: the host's audio, which the world cannot depend on. A sink hands back the frames it
/// is done with, which the scene fills again.
pub trait WorldSoundSink: Send + 'static {
  /// Takes the scene's latest frame; one that cannot go on is dropped, the next replacing it.
  fn send(&mut self, frame: WorldSoundFrame);

  /// A frame the sink is done with, to fill again; none while it holds them all.
  fn take_spent(&mut self) -> Option<WorldSoundFrame>;
}
