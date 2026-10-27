use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_spawn::{SpawnFile, SpawnLevelGraph};
use xrf_vfs::{XrayLogicalPath, XraySearch};

use crate::level::SelectedLevel;
use crate::parts::LevelSpawnPart;

/// What a change event calls a level's graph when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_GRAPH_PART: &str = "graph";

/// A level's graph points and patrols and every level's place in the world, which its level maps' annotations draw.
pub(crate) struct LevelGraphPart;

impl SessionPart for LevelGraphPart {
  type Value = Result<Arc<SpawnLevelGraph>, String>;

  const NAME: &'static str = LEVEL_GRAPH_PART;
}

impl LevelGraphPart {
  /// The open level's graph points and patrols and every level's place in the world: read from the spawn the first
  /// time they are drawn, rather than with its objects on every level opened, and kept with the level.
  ///
  /// # Errors
  ///
  /// Returns the reason when the spawn's graph or patrols cannot be read or the level has no name to find them by,
  /// which every later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<SpawnLevelGraph>, String> {
    current.parts.get_or_record::<Self>(search, |search| {
      let level: String = current.source.require_name()?;
      let (path, chunks): (XrayLogicalPath, Vec<ChunkReader<InMemoryChunkDataSource>>) =
        LevelSpawnPart::read_chunks(search)?;

      SpawnFile::read_level_graph_from_chunks::<XRayByteOrder, _>(&chunks, &level)
        .map(Arc::new)
        .map_err(|error| format!("Failed to read the graph of '{path}': {error}"))
    })
  }
}
