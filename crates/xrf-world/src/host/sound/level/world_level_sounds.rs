use xrf_engine_target::XrayEngine;

use crate::host::sound::level::world_music_track::WorldMusicTrack;
use crate::host::sound::level::world_static_sound::WorldStaticSound;
use crate::host::sound::level::world_zone_sound::WorldZoneSound;

/// What a level plays whatever its weather: its static sounds and its music tracks (`CLevelSoundManager`), in the order
/// the game reads them, and its zones' sounds in the spawn's order, played as its engine plays them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldLevelSounds {
  pub engine: XrayEngine,
  pub statics: Vec<WorldStaticSound>,
  pub music: Vec<WorldMusicTrack>,
  pub zones: Vec<WorldZoneSound>,
}
