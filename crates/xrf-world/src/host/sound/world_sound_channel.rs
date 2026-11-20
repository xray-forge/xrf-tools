use crate::host::sound::world_sound::WorldSound;

/// A sound channel of an ambient, `CEnvAmbient::SSndChannel`: its sounds, one of which plays at a time around the
/// camera, how far away, and how long it waits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldSoundChannel {
  pub sounds: Vec<WorldSound>,
  /// Metres from the camera, nearest then farthest, `m_sound_dist`.
  pub distance: [f32; 2],
  /// Milliseconds, `m_sound_period`: the first sound's wait between the first two, every later one's between the last
  /// two after the sound before it ends.
  pub period: [u32; 4],
}
