/// How an audio engine is sized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioEngineSettings {
  /// Voices that can play at once: the mixer's pool, preallocated.
  pub voices_count: usize,
  /// Commands the control side can send before the audio thread takes them.
  pub commands_count: usize,
}

impl Default for AudioEngineSettings {
  /// Extended's 1024 targets, its console's maximum, plus room for previews and music.
  fn default() -> Self {
    Self {
      voices_count: 1056,
      commands_count: 4096,
    }
  }
}
