/// How the mixer keeps up, as the audio thread last reported it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AudioMixerLoad {
  /// Frames rendered since the mixer was made: the mixer's clock, running through a device loss.
  pub frames: u64,
  /// The last buffer's render time over the time it plays for: 1 is a buffer that took as long as it lasts.
  pub load: f32,
  /// The highest load since the last time the load was read.
  pub peak_load: f32,
  /// Buffers whose render took longer than they play for, since the mixer was made.
  pub overruns: u64,
  /// Voices the mixer held after the last buffer.
  pub voices: u32,
}
