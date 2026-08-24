use std::collections::BTreeSet;

use uuid::Uuid;
use xrf_chunk::ChunkReader;
use xrf_db::{GraphCrossTable, GraphLevel, SpawnFile, SpawnGraphsChunk, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::levels::level_engine_constants::SPAWNS_DIRECTORY;
use crate::{Finding, GamedataProject, GamedataVerificationRule};

/// Single level the game graph can send the player to.
pub(crate) struct RosterLevel {
  pub(crate) name: String,
  pub(crate) id: u8,
  pub(crate) guid: Uuid,
  pub(crate) graph_guid: Uuid,
  /// Spawn asset whose game graph declared this level.
  pub(crate) source: String,
  /// Cross table addressed by this level, absent when the graph ships fewer tables than levels.
  pub(crate) cross_table_level_guid: Option<Uuid>,
  pub(crate) cross_table_game_guid: Option<Uuid>,
  pub(crate) cross_table_nodes_count: Option<u32>,
}

/// Levels the game graph declares, reconciled across every graph bearing spawn asset.
pub(crate) struct LevelRoster {
  pub(crate) levels: Vec<RosterLevel>,
  pub(crate) findings: Vec<Finding>,
  pub(crate) sources_count: u32,
}

impl LevelRoster {
  /// Collect the level roster from every spawn asset carrying a game graph.
  ///
  /// Only the graph chunk is parsed, so an ALife object class without a CLSID mapping cannot
  /// prevent the roster from being determined. Whole spawn file validity belongs to `verify_spawns`.
  ///
  /// A spawn file that cannot be read at all is a checker error rather than a finding: the roster
  /// is unknown, and falling back to the directory listing would make every reconciliation rule
  /// vacuously pass.
  pub(crate) fn read(project: &GamedataProject) -> XrfResult<Self> {
    let mut roster: Self = Self {
      levels: Vec::new(),
      findings: Vec::new(),
      sources_count: 0,
    };

    let sources: Vec<String> = project
      .vfs()
      .scoped(project.scope())
      .list_entries_of_type(AssetType::Spawn)
      .into_iter()
      .filter(|location| location.get_logical_path().is_under(SPAWNS_DIRECTORY).unwrap_or(false))
      .map(|location| location.get_logical_path().to_string())
      .collect();

    for source in &sources {
      let path: &String = source;

      // Narrow read rather than the parsed seam: this needs the graphs chunk of a spawn that runs to 97MB in a shipped
      // game, and parsing the whole file to reach one chunk would cost more than every other level check together.
      let graphs: SpawnGraphsChunk = project
        .read_bytes(path)
        .and_then(ChunkReader::from_vec)
        .and_then(|mut chunk| SpawnFile::read_graphs_from_chunk::<XRayByteOrder, _>(&mut chunk))
        .map_err(|error| {
          XrfError::new_verify_error(format!("Failed to read level roster from spawn file {source}: {error}"))
        })?;

      roster.sources_count += 1;
      roster.add_graph(source, &graphs);
    }

    roster
      .findings
      .sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    Ok(roster)
  }

  /// Whether any spawn asset provided a game graph.
  pub(crate) fn has_source(&self) -> bool {
    self.sources_count > 0
  }

  pub(crate) fn find(&self, name: &str) -> Option<&RosterLevel> {
    self.levels.iter().find(|level| level.name == name)
  }

  pub(crate) fn names(&self) -> BTreeSet<&str> {
    self.levels.iter().map(|level| level.name.as_str()).collect()
  }

  fn add_graph(&mut self, source: &str, graphs: &SpawnGraphsChunk) {
    self.report_graph_duplicates(source, graphs);

    for (rank, level) in Self::ranked_levels(graphs).into_iter().enumerate() {
      let cross_table: Option<&GraphCrossTable> = graphs.cross_tables.get(rank);
      let name: String = level.name.to_lowercase();

      if let Some(existing) = self.levels.iter().find(|it| it.name == name) {
        // Repeated names inside a single graph are reported as a duplicate declaration, not as a
        // conflict between graphs.
        if existing.source != source && (existing.guid != level.guid || existing.id != level.id) {
          self.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::LevelsRosterConflict,
            source,
            format!(
              "Level [{name}] is declared by [{}] with id {} guid {} and by [{source}] with id {} guid {}",
              existing.source, existing.id, existing.guid, level.id, level.guid
            ),
          ));
        }

        continue;
      }

      self.levels.push(RosterLevel {
        name,
        id: level.id,
        guid: level.guid,
        graph_guid: graphs.header.guid,
        source: String::from(source),
        cross_table_level_guid: cross_table.map(|it| it.level_guid),
        cross_table_game_guid: cross_table.map(|it| it.game_guid),
        cross_table_nodes_count: cross_table.map(|it| it.nodes_count),
      });
    }
  }

  /// Order levels the way the engine does.
  ///
  /// `CGameGraph::set_current_level` walks `header().levels()`, a map keyed by level id, advancing
  /// the cross table pointer once per level. Cross tables are therefore addressed by the level's
  /// rank in ascending id order - not by the id itself, and not by the order levels appear in the
  /// file.
  fn ranked_levels(graphs: &SpawnGraphsChunk) -> Vec<&GraphLevel> {
    let mut levels: Vec<&GraphLevel> = graphs.levels.iter().collect();

    levels.sort_by_key(|level| level.id);

    levels
  }

  fn report_graph_duplicates(&mut self, source: &str, graphs: &SpawnGraphsChunk) {
    let mut seen_names: BTreeSet<String> = BTreeSet::new();
    let mut seen_ids: BTreeSet<u8> = BTreeSet::new();

    for level in &graphs.levels {
      let name: String = level.name.to_lowercase();

      if !seen_names.insert(name.clone()) {
        self.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsGraphDuplicate,
          source,
          format!("Game graph declares level name [{name}] more than once"),
        ));
      }

      if !seen_ids.insert(level.id) {
        self.findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsGraphDuplicate,
          source,
          format!(
            "Game graph declares level id [{}] more than once, latest is [{name}]",
            level.id
          ),
        ));
      }
    }
  }
}
