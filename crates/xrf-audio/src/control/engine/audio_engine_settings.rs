/// How an audio engine is sized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioEngineSettings {
  /// Voices that can play at once: the mixer's pool, preallocated.
  pub voices_count: usize,
  /// Voices the World and Ambience buses may hold together, at most the pool. The rest is kept for Music and Previews,
  /// so a full scene, or two fading into each other, never refuses a track or a preview.
  pub world_voices_count: usize,
  /// Commands the control side can send before the audio thread takes them.
  pub commands_count: usize,
}

impl Default for AudioEngineSettings {
  /// Extended's 1024 targets, its console's maximum, for the world and ambience, plus 32 kept for previews and music.
  fn default() -> Self {
    Self {
      voices_count: 1056,
      world_voices_count: 1024,
      commands_count: 4096,
    }
  }
}
