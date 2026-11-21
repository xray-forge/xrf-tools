use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelAiFile, LevelGameFile};
use xrf_spawn::{AlifeObject, GraphCrossTableCell, LevelSpawnFile, SpawnSectionClasses};

use crate::gamedata::spawn_game_level::SpawnGameLevel;
use crate::graph::spawn_graph_points::SpawnGraphPoints;
use crate::graph::spawn_level_game_graph::SpawnLevelGameGraph;
use crate::map::spawn_ai_map::SpawnAiMap;
use crate::map::spawn_cross_table::SpawnCrossTable;
use crate::merge::spawn_merge_level::SpawnMergeLevel;
use crate::root::spawn_root_level::SpawnRootLevel;

/// A level's files read from its directory, with what `CGameGraphBuilder` builds from the level alone before the
/// merge: its graph points, cross table and game graph.
pub struct SpawnLevelFiles {
  map: SpawnAiMap,
  /// The objects of `level.spawn`, typed once for the graph points and the entities both.
  objects: Vec<AlifeObject>,
  game: Option<LevelGameFile>,
  points: SpawnGraphPoints,
  graph: SpawnLevelGameGraph,
  cells: Vec<GraphCrossTableCell>,
}

impl SpawnLevelFiles {
  /// Whether a directory holds `level.ai`, `level.spawn` and `level.game`.
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
  /// Returns an error naming the level when a file does not read or its graph cannot be built, as when no AI map path
  /// reaches a graph point from its neighbours.
  pub fn read(directory: &Path, name: &str, classes: &SpawnSectionClasses) -> XrfResult<Self> {
    Self::read_unnamed(directory, classes)
      .map_err(|error| XrfError::new_invalid_error(format!("Level {name}: {error}")))
  }

  /// The objects of a spawn list in order, each typed as the class `classes` gives its section.
  ///
  /// # Errors
  ///
  /// Returns an error when a packet does not read as its class.
  pub fn read_objects(spawn: &LevelSpawnFile, classes: &SpawnSectionClasses) -> XrfResult<Vec<AlifeObject>> {
    spawn
      .objects
      .iter()
      .map(|packet| packet.read_alife_object_by(classes))
      .collect()
  }

  /// The build's input for this level under its entry in the game's level list.
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
      objects: &self.objects,
      game: self.game.as_ref(),
      removed: &self.points.removed,
    }
  }

  fn read_unnamed(directory: &Path, classes: &SpawnSectionClasses) -> XrfResult<Self> {
    let (file, nodes) = LevelAiFile::read_with_nodes_from_path(&directory.join(LevelAiFile::FILE_NAME))?;
    let spawn: LevelSpawnFile = LevelSpawnFile::read_from_path(&directory.join(LevelSpawnFile::FILE_NAME))?;
    let game_path = directory.join(LevelGameFile::FILE_NAME);
    let game: Option<LevelGameFile> = game_path
      .is_file()
      .then(|| LevelGameFile::read_from_path(&game_path))
      .transpose()?;
    let map: SpawnAiMap = SpawnAiMap::new(file.header, &nodes)?;
    let objects: Vec<AlifeObject> = Self::read_objects(&spawn, classes)?;
    let points: SpawnGraphPoints = SpawnGraphPoints::read(&objects, &map);
    let nodes_of_points: Vec<u32> = points.points.iter().map(|point| point.node).collect();
    let cells: Vec<GraphCrossTableCell> = SpawnCrossTable::build(&map, &nodes_of_points)?;
    let graph: SpawnLevelGameGraph = SpawnLevelGameGraph::build(&map, &points.points, &cells)?;

    Ok(Self {
      map,
      objects,
      game,
      points,
      graph,
      cells,
    })
  }
}
