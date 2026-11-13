use std::path::Path;

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelAiFile, LevelGameFile};
use xrf_spawn::{GraphCrossTableCell, LevelSpawnFile};

use crate::spawn::spawn_ai_map::SpawnAiMap;
use crate::spawn::spawn_cross_table::SpawnCrossTable;
use crate::spawn::spawn_game_level::SpawnGameLevel;
use crate::spawn::spawn_graph_points::SpawnGraphPoints;
use crate::spawn::spawn_level_game_graph::SpawnLevelGameGraph;
use crate::spawn::spawn_merge_level::SpawnMergeLevel;
use crate::spawn::spawn_root_level::SpawnRootLevel;

/// A level's files as the root spawn build reads them from its directory, and what is built from the level alone: its
/// graph points, cross table and game graph (`CGameGraphBuilder`, run over each level before the merge).
pub struct SpawnLevelFiles {
  map: SpawnAiMap,
  spawn: LevelSpawnFile,
  game: Option<LevelGameFile>,
  points: SpawnGraphPoints,
  graph: SpawnLevelGameGraph,
  cells: Vec<GraphCrossTableCell>,
}

impl SpawnLevelFiles {
  /// Whether a directory holds what a level of the build needs, its `level.ai` and `level.spawn`, and the `level.game`
  /// its patrols come from.
  pub fn is_complete(directory: &Path) -> bool {
    [
      LevelAiFile::FILE_NAME,
      LevelSpawnFile::FILE_NAME,
      LevelGameFile::FILE_NAME,
    ]
    .iter()
    .all(|file| directory.join(file).is_file())
  }

  /// Reads the level `name` from its directory and builds its graph; a level without `level.game` has no patrols.
  ///
  /// # Errors
  ///
  /// Returns an error naming the level when a file does not read, or its graph cannot be built (a graph point no path
  /// on the AI map reaches from a neighbour among them).
  pub fn read(directory: &Path, name: &str) -> XrfResult<Self> {
    Self::read_unnamed(directory).map_err(|error| XrfError::new_invalid_error(format!("Level {name}: {error}")))
  }

  /// The level as the build takes it, by its entry in the game's level list.
  pub fn get_input(&self, level: &SpawnGameLevel) -> SpawnRootLevel<'_> {
    SpawnRootLevel {
      merge: SpawnMergeLevel {
        section: level.section.clone(),
        name: level.name.clone(),
        caption: level.caption.clone(),
        id: level.id,
        offset: level.offset,
        map: &self.map,
        points: &self.points.points,
        graph: &self.graph,
        cells: &self.cells,
      },
      spawn: &self.spawn,
      game: self.game.as_ref(),
      removed: &self.points.removed,
    }
  }

  fn read_unnamed(directory: &Path) -> XrfResult<Self> {
    let (file, nodes) =
      LevelAiFile::read_with_nodes_from_path::<XRayByteOrder, _>(&directory.join(LevelAiFile::FILE_NAME))?;
    let spawn: LevelSpawnFile =
      LevelSpawnFile::read_from_path::<XRayByteOrder, _>(&directory.join(LevelSpawnFile::FILE_NAME))?;
    let game_path = directory.join(LevelGameFile::FILE_NAME);
    let game: Option<LevelGameFile> = game_path
      .is_file()
      .then(|| LevelGameFile::read_from_path::<XRayByteOrder, _>(&game_path))
      .transpose()?;
    let map: SpawnAiMap = SpawnAiMap::new(file.header, &nodes);
    let points: SpawnGraphPoints = SpawnGraphPoints::read(&spawn, &map)?;
    let nodes_of_points: Vec<u32> = points.points.iter().map(|point| point.node).collect();
    let cells: Vec<GraphCrossTableCell> = SpawnCrossTable::build(&map, &nodes_of_points)?;
    let graph: SpawnLevelGameGraph = SpawnLevelGameGraph::build(&map, &points.points, &cells)?;

    Ok(Self {
      map,
      spawn,
      game,
      points,
      graph,
      cells,
    })
  }
}
