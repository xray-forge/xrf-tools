/// A graph point connection to a level outside the build, which the merge leaves out as xrAI does with a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpawnSkippedConnection {
  pub level: String,
  pub point: String,
  /// The level the point names, by its caption.
  pub target_level: String,
}
