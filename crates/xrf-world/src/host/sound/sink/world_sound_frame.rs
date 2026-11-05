use xrf_audio_scene::AudioSceneFrame;

/// A scene's sound as of a moment on its level's clock, on its way to the host: the audio's frame of it, filled by the
/// world, and whether a burst draws it. Frames come from a pool the sink hands back, so a warm world allocates none.
#[derive(Clone, Debug, Default)]
pub struct WorldSoundFrame {
  /// What plays, from where it is heard, and whether it is heard: the listener's viewport shows the scene, every other
  /// scene silent and playing on. Its `scene` is the scene's `RenderSceneId`, its `time` the level's clock.
  pub sound: Box<AudioSceneFrame>,
  /// Whether a burst draws the frame at a time of its own, heard by the burst's viewport: the scene's sound plays the
  /// burst's frames from a start of their own, the same however often a burst from that time is drawn.
  pub is_burst: bool,
}
