use crate::host::sound::world_sound::WorldSound;

/// One track of a level's music, `SMusicTrack`: an entry of the section `game.ltx` names by the level's
/// `music_tracks`, `name = from_hour, to_hour, volume, pause_from_s, pause_to_s`.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldMusicTrack {
  /// The entry's name, the stereo file's reference.
  pub name: String,
  pub stereo: WorldSound,
  /// OpenXRay's halves, `<name>_l` and `<name>_r`, read where the stereo file is missing; none on Monolith.
  pub halves: Option<[WorldSound; 2]>,
  /// Game time of day it is picked within, in milliseconds, `m_ActiveTime`; the hours may cross midnight.
  pub hours: [u32; 2],
  pub volume: f32,
  /// Milliseconds it waits after it ends, drawn between the two, `m_PauseTime`, widened by one where both are equal.
  pub pause: [u32; 2],
}
