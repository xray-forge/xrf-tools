use uuid::Uuid;

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
