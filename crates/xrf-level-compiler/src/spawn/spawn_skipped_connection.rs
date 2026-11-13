/// A level changer connection the merge leaves out, as xrAI does with a message: a graph point naming a level outside
/// the build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnSkippedConnection {
  pub level: String,
  pub point: String,
  /// The level the point names, by its caption.
  pub target_level: String,
}
