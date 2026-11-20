use xrf_audio_scene::AudioSceneFrame;

/// A scene's sound at a moment of its level's clock, on its way to the host. Frames come from a pool the sink hands
/// back, so a warm world allocates none.
#[derive(Clone, Debug, Default)]
pub struct WorldSoundFrame {
  /// The audio's frame, its `scene` the scene's `RenderSceneId` and its `time` the level's clock. Only the scene the
  /// listener's viewport shows is heard; every other plays on silent.
  pub sound: Box<AudioSceneFrame>,
  /// Whether a burst draws the frame at a time of its own. A burst's frames play from their own start, the same each
  /// time a burst from that time is drawn.
  pub is_burst: bool,
}
