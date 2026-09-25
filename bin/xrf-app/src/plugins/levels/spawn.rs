//! What the game spawns on a level, out of the one spawn it keeps for every level.

use std::sync::Arc;
use std::time::Instant;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_spawn::{AlifeObject, GraphVertexLevels, SpawnALifeSpawnsChunk, SpawnFile, XRayByteOrder};
use xrf_vfs::XrayProbe;

use crate::plugins::levels::read::read_asset;
use crate::plugins::levels::state::{LevelSpawn, SelectedLevel};

/// The game's spawn, which places what every level starts with.
const SPAWN_FILE: &str = "spawns\\all.spawn";

/// The open level's spawned objects, read the first time anything asks and kept with the level.
///
/// # Errors
///
/// Returns the reason when the spawn cannot be read or the level has no name to find its objects by, which every
/// later ask answers with again.
pub fn get_level_spawn(current: &SelectedLevel, probe: &XrayProbe) -> Result<Arc<LevelSpawn>, String> {
  current
    .spawn
    .get_or_init(|| {
      let directory: Option<String> = current.source.get_logical_directory();
      let level: &str = directory
        .as_deref()
        .and_then(|it| it.rsplit('\\').next())
        .ok_or_else(|| format!("Level {} has no name to find its spawn by", current.source.get_label()))?;

      read_level_spawn(probe, level).map(Arc::new)
    })
    .clone()
}

/// The objects standing on a level, by the level each game vertex belongs to, reading nothing of the spawn but its
/// objects and the head of its graph.
fn read_level_spawn(probe: &XrayProbe, level: &str) -> Result<LevelSpawn, String> {
  let started: Instant = Instant::now();
  let failure = |error: xrf_error::XrfError| format!("Failed to read '{SPAWN_FILE}': {error}");
  let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(read_asset(probe, SPAWN_FILE)?)
    .and_then(|mut it| it.read_children())
    .map_err(failure)?;
  let spawns: SpawnALifeSpawnsChunk =
    SpawnFile::read_alife_spawns_from_chunks::<XRayByteOrder, _>(&chunks).map_err(failure)?;
  let levels: GraphVertexLevels =
    SpawnFile::read_vertex_levels_from_chunks::<XRayByteOrder, _>(&chunks).map_err(failure)?;
  let total: usize = spawns.objects.len();
  let objects: Vec<AlifeObject> = spawns
    .objects
    .into_iter()
    .filter(|object| {
      levels
        .get_object_level(object)
        .is_some_and(|it| it.name.eq_ignore_ascii_case(level))
    })
    .collect();

  log::info!(
    "Read the spawn of {level}: {} of {total} objects in {:?}",
    objects.len(),
    started.elapsed()
  );

  Ok(LevelSpawn { objects })
}
