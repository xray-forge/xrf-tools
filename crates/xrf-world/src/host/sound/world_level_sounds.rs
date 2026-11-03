use xrf_engine_target::XrayEngine;

use crate::host::sound::world_music_track::WorldMusicTrack;
use crate::host::sound::world_static_sound::WorldStaticSound;

/// What a level plays whatever its weather (`CLevelSoundManager`): its static sounds and its music tracks, in the order
/// the game reads them, played as its engine plays them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldLevelSounds {
  pub engine: XrayEngine,
  pub statics: Vec<WorldStaticSound>,
  pub music: Vec<WorldMusicTrack>,
}
