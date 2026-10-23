//! What the game spawns on a level, out of the one spawn it keeps for every level.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_spawn::{AlifeObject, ClsId, SpawnFile, SpawnLevelGraph, SpawnLevelObjects};
use xrf_vfs::{XrayInputRecord, XrayLogicalPath, XrayProbe};

use crate::core::assets::read_located_asset;
use crate::core::session::{SessionPart, SessionParts, SessionSnapshot};
use crate::plugins::levels::new_game::read_new_game_releases;
use crate::plugins::levels::report::{report_new_game_releases, report_spawn};
use crate::plugins::levels::state::{LevelSource, LevelSpawn, LevelSpawnRelease, SelectedLevel};

/// Where the game keeps its spawn (`$game_spawn$`).
const SPAWNS_DIRECTORY: &str = "spawns";

/// The game's spawn, which places what every level starts with.
const SPAWN_FILE: &str = "all.spawn";

/// What the game spawns on a level.
struct LevelSpawnPart;

impl SessionPart for LevelSpawnPart {
  type Value = Result<Arc<LevelSpawn>, String>;
}

/// A level's graph points and patrols and every level's place in the world, which its level maps' annotations draw.
struct LevelGraphPart;

impl SessionPart for LevelGraphPart {
  type Value = Result<Arc<SpawnLevelGraph>, String>;
}

/// The open level's spawned objects, read as it opened and kept with the level.
///
/// # Errors
///
/// Returns the reason when the spawn cannot be read or the level has no name to find its objects by, which every
/// later ask answers with again.
pub fn get_level_spawn(current: &SessionSnapshot<SelectedLevel>, probe: &XrayProbe) -> Result<Arc<LevelSpawn>, String> {
  current
    .parts
    .get_or_record::<LevelSpawnPart>(probe, |probe| read_source_spawn(&current.source, probe))
}

/// `parts` with the spawn an open read from `inputs` kept as the level's.
pub fn with_level_spawn(
  parts: SessionParts,
  spawn: Result<Arc<LevelSpawn>, String>,
  inputs: XrayInputRecord,
) -> SessionParts {
  parts.with::<LevelSpawnPart>(spawn, inputs)
}

/// The open level's graph points and patrols and every level's place in the world, which a level map's annotations
/// draw: read from the spawn the first time they are drawn, rather than with its objects on every level opened, and
/// kept with the level.
///
/// # Errors
///
/// Returns the reason when the spawn's graph or patrols cannot be read or the level has no name to find them by,
/// which every later ask answers with again.
pub fn get_level_graph(
  current: &SessionSnapshot<SelectedLevel>,
  probe: &XrayProbe,
) -> Result<Arc<SpawnLevelGraph>, String> {
  current.parts.get_or_record::<LevelGraphPart>(probe, |probe| {
    let level: String = current
      .source
      .get_name()
      .ok_or_else(|| format!("Level {} has no name to find its graph by", current.source.get_label()))?;
    let (path, chunks): (XrayLogicalPath, Vec<ChunkReader<InMemoryChunkDataSource>>) = read_spawn_chunks(probe)?;

    SpawnFile::read_level_graph_from_chunks::<XRayByteOrder, _>(&chunks, &level)
      .map(Arc::new)
      .map_err(|error| format!("Failed to read the graph of '{path}': {error}"))
  })
}

/// The game's spawn's root chunks, and the path it was read by.
fn read_spawn_chunks(
  probe: &XrayProbe,
) -> Result<(XrayLogicalPath, Vec<ChunkReader<InMemoryChunkDataSource>>), String> {
  let path: XrayLogicalPath = XrayLogicalPath::new(SPAWNS_DIRECTORY)
    .and_then(|directory| directory.join(SPAWN_FILE))
    .map_err(|error| format!("Failed to name '{SPAWN_FILE}': {error}"))?;
  let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = read_located_asset(probe, path.as_str())
    .and_then(ChunkReader::from_vec)
    .and_then(|mut it| it.read_children())
    .map_err(|error| format!("Failed to read '{path}': {error}"))?;

  Ok((path, chunks))
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

/// The objects standing on a level when a new game starts, by the level each game vertex belongs to, reading nothing of
/// the spawn but its objects and the head of its graph.
fn read_level_spawn(probe: &XrayProbe, level: &str) -> Result<LevelSpawn, String> {
  let started: Instant = Instant::now();
  let (path, chunks): (XrayLogicalPath, Vec<ChunkReader<InMemoryChunkDataSource>>) = read_spawn_chunks(probe)?;
  let read: SpawnLevelObjects = SpawnFile::read_level_objects_from_chunks::<XRayByteOrder, _>(&chunks, level)
    .map_err(|error| format!("Failed to read '{path}': {error}"))?;

  report_spawn(level, path.as_str(), &read, started);

  let released: HashMap<String, LevelSpawnRelease> = read_new_game_releases(probe);
  let objects: Vec<AlifeObject> = read.objects;
  let releases: HashMap<usize, LevelSpawnRelease> = objects
    .iter()
    .enumerate()
    .filter_map(|(index, object)| released.get(&object.name).map(|release| (index, *release)))
    // Anomaly's `game_setup.script` replaces no inventory box's item: a box only goes when it is removed outright.
    .filter(|(index, release)| *release != LevelSpawnRelease::ReplaceItems || objects[*index].clsid != ClsId::SInvBox)
    .collect();

  report_new_game_releases(level, releases.len());

  Ok(LevelSpawn {
    arrivals: read.arrivals,
    objects,
    releases,
  })
}
