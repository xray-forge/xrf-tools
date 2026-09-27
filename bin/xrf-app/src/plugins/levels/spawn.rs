//! What the game spawns on a level, out of the one spawn it keeps for every level.

use std::sync::Arc;
use std::time::Instant;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_spawn::{SpawnFile, SpawnLevelObjects, XRayByteOrder};
use xrf_vfs::{XrayLogicalPath, XrayProbe};

use crate::core::assets::read_located_asset;
use crate::plugins::levels::report::report_spawn;
use crate::plugins::levels::state::{LevelSource, LevelSpawn, SelectedLevel};

/// Where the game keeps its spawn (`$game_spawn$`).
const SPAWNS_DIRECTORY: &str = "spawns";

/// The game's spawn, which places what every level starts with.
const SPAWN_FILE: &str = "all.spawn";

/// The open level's spawned objects, read the first time anything asks and kept with the level.
///
/// # Errors
///
/// Returns the reason when the spawn cannot be read or the level has no name to find its objects by, which every
/// later ask answers with again.
pub fn get_level_spawn(current: &SelectedLevel, probe: &XrayProbe) -> Result<Arc<LevelSpawn>, String> {
  current
    .spawn
    .get_or_init(|| read_source_spawn(&current.source, probe))
    .clone()
}

/// The spawned objects of a level, read by its name.
///
/// # Errors
///
/// Returns the reason when the spawn cannot be read or the level has no name to find its objects by.
pub fn read_source_spawn(source: &LevelSource, probe: &XrayProbe) -> Result<Arc<LevelSpawn>, String> {
  let level: String = source
    .get_name()
    .ok_or_else(|| format!("Level {} has no name to find its spawn by", source.get_label()))?;

  read_level_spawn(probe, &level).map(Arc::new)
}

/// The objects standing on a level, by the level each game vertex belongs to, reading nothing of the spawn but its
/// objects and the head of its graph.
fn read_level_spawn(probe: &XrayProbe, level: &str) -> Result<LevelSpawn, String> {
  let started: Instant = Instant::now();
  let path: XrayLogicalPath = XrayLogicalPath::new(SPAWNS_DIRECTORY)
    .and_then(|directory| directory.join(SPAWN_FILE))
    .map_err(|error| format!("Failed to name '{SPAWN_FILE}': {error}"))?;
  let failure = |error: xrf_error::XrfError| format!("Failed to read '{path}': {error}");
  let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = read_located_asset(probe, path.as_str())
    .and_then(ChunkReader::from_vec)
    .and_then(|mut it| it.read_children())
    .map_err(failure)?;
  let read: SpawnLevelObjects =
    SpawnFile::read_level_objects_from_chunks::<XRayByteOrder, _>(&chunks, level).map_err(failure)?;

  report_spawn(level, path.as_str(), &read, started);

  Ok(LevelSpawn {
    arrivals: read.arrivals,
    objects: read.objects,
  })
}
