use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_app_core::session::{SessionPart, SessionParts, SessionSnapshot};
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_spawn::{AlifeObject, ClsId, SpawnFile, SpawnLevelObjects};
use xrf_vfs::{XrayInputRecord, XrayLogicalPath, XraySearch};

use crate::level::{LevelSource, SelectedLevel};
use crate::parts::LevelSpawn;
use crate::report::LevelReport;
use crate::spawn::LevelSpawnRelease;

/// What a change event calls a level's spawn when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SPAWN_PART: &str = "spawn";

/// What the game spawns on a level, out of the one spawn it keeps for every level.
pub(crate) struct LevelSpawnPart;

impl SessionPart for LevelSpawnPart {
  type Value = Result<Arc<LevelSpawn>, String>;

  const NAME: &'static str = LEVEL_SPAWN_PART;
}

impl LevelSpawnPart {
  /// Where the game keeps its spawn (`$game_spawn$`).
  const SPAWNS_DIRECTORY: &'static str = "spawns";

  /// The game's spawn, which places what every level starts with.
  const SPAWN_FILE: &'static str = "all.spawn";

  /// The open level's spawned objects, read as it opened and kept with the level.
  ///
  /// # Errors
  ///
  /// Returns the reason when the spawn cannot be read or the level has no name to find its objects by, which every
  /// later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<LevelSpawn>, String> {
    current
      .parts
      .get_or_record::<Self>(search, |search| Self::read(&current.source, search))
  }

  /// `parts` with the spawn an open read from `inputs` kept as the level's.
  pub fn with(parts: SessionParts, spawn: Result<Arc<LevelSpawn>, String>, inputs: XrayInputRecord) -> SessionParts {
    parts.with::<Self>(spawn, inputs)
  }

  /// The spawned objects of a level, read by its name.
  ///
  /// # Errors
  ///
  /// Returns the reason when the spawn cannot be read or the level has no name to find its objects by.
  pub fn read(source: &LevelSource, search: &XraySearch) -> Result<Arc<LevelSpawn>, String> {
    Self::read_level(search, &source.require_name()?).map(Arc::new)
  }

  /// The game's spawn's root chunks, and the path it was read by.
  ///
  /// # Errors
  ///
  /// Returns why the spawn cannot be named or read.
  pub fn read_chunks(
    search: &XraySearch,
  ) -> Result<(XrayLogicalPath, Vec<ChunkReader<InMemoryChunkDataSource>>), String> {
    let path: XrayLogicalPath = XrayLogicalPath::new(Self::SPAWNS_DIRECTORY)
      .and_then(|directory| directory.join(Self::SPAWN_FILE))
      .map_err(|error| format!("Failed to name '{}': {error}", Self::SPAWN_FILE))?;
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = search
      .read(path.as_str())
      .and_then(ChunkReader::from_vec)
      .and_then(|mut it| it.read_children())
      .map_err(|error| format!("Failed to read '{path}': {error}"))?;

    Ok((path, chunks))
  }

  /// The objects standing on a level when a new game starts, by the level each game vertex belongs to, reading nothing
  /// of the spawn but its objects and the head of its graph.
  fn read_level(search: &XraySearch, level: &str) -> Result<LevelSpawn, String> {
    let started: Instant = Instant::now();
    let (path, chunks): (XrayLogicalPath, Vec<ChunkReader<InMemoryChunkDataSource>>) = Self::read_chunks(search)?;
    let read: SpawnLevelObjects = SpawnFile::read_level_objects_from_chunks::<XRayByteOrder, _>(&chunks, level)
      .map_err(|error| format!("Failed to read '{path}': {error}"))?;

    LevelReport::spawn(level, path.as_str(), &read, started);

    let released: HashMap<String, LevelSpawnRelease> = LevelSpawnRelease::read_new_game(search);
    let objects: Vec<AlifeObject> = read.objects;
    let releases: HashMap<usize, LevelSpawnRelease> = objects
      .iter()
      .enumerate()
      .filter_map(|(index, object)| released.get(&object.name).map(|release| (index, *release)))
      // Anomaly's `game_setup.script` replaces no inventory box's item: a box only goes when it is removed outright.
      .filter(|(index, release)| *release != LevelSpawnRelease::ReplaceItems || objects[*index].clsid != ClsId::SInvBox)
      .collect();

    LevelReport::new_game_releases(level, releases.len());

    Ok(LevelSpawn {
      arrivals: read.arrivals,
      objects,
      releases,
    })
  }
}
