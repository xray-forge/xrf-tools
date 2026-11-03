use crate::host::sound::world_sound::WorldSound;

/// One of a level's static sounds, `SStaticSound` (`level.snd_static`): where it stands in engine space, how loud and
/// how fast it plays, and its windows in milliseconds, each `[from, to]`, both nought for none.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldStaticSound {
  pub sound: WorldSound,
  pub position: [f32; 3],
  pub volume: f32,
  pub frequency: f32,
  /// Game time of day it plays within, `m_ActiveTime`.
  pub active: [u32; 2],
  /// How long a looped play lasts, drawn between the two, `m_PlayTime`; nought plays the sound through once.
  pub play: [u32; 2],
  /// How long it waits between plays, `m_PauseTime`; nought loops it forever.
  pub pause: [u32; 2],
}
