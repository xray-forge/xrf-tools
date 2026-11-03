use std::time::Duration;

/// What the engine reads of a sound file as it loads it (`CSoundRender_Source::LoadWave`): how long it plays, whether
/// it is stereo, which always plays 2D, and its source parameters, the engine's defaults where it carries none.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldSoundInfo {
  /// Its length at its own rate, `get_length_sec`.
  pub length: Duration,
  pub is_stereo: bool,
  /// Metres within which it plays at full volume, and past which it is not heard.
  pub min_distance: f32,
  pub max_distance: f32,
  pub base_volume: f32,
  /// Whether its game type is `SOUND_TYPE_WORLD_AMBIENT`, which the sound renderer never occludes.
  pub is_world_ambient: bool,
}

impl Default for WorldSoundInfo {
  /// `SoundSourceInfo`'s defaults, a mono second.
  fn default() -> Self {
    Self {
      length: Duration::from_secs(1),
      is_stereo: false,
      min_distance: 1.0,
      max_distance: 300.0,
      base_volume: 1.0,
      is_world_ambient: false,
    }
  }
}
