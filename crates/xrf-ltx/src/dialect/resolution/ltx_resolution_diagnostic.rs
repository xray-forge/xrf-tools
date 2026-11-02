/// Something a dialect wants said about a config tree, short of refusing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LtxResolutionDiagnostic {
  pub section: String,
  pub file: Option<String>,
  pub message: String,
  /// What the engine does with the same input, where that differs from reporting it.
  ///
  /// The reason these are worth emitting at all: a modder cannot see what the game silently drops, so a warning that
  /// says "the game loads this and says nothing" is more use than the same warning without it.
  pub engine_behaviour: Option<String>,
}
