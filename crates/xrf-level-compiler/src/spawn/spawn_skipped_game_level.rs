/// A `[levels]` entry the build cannot take, as xrAI's `read_levels` passes one over with a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnSkippedGameLevel {
  pub section: String,
  /// Its directory name, where the section names one.
  pub name: Option<String>,
  pub reason: String,
}
