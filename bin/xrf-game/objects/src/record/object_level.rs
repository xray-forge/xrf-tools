/// A level of the game graph, which records name by its id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLevel {
  /// Its id in the game graph (`GameGraph::SLevel::id`).
  pub id: u8,
  pub name: String,
}
