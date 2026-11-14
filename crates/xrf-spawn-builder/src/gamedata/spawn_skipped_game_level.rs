/// A `[levels]` entry the build cannot take, as xrAI's `read_levels` passes one over with a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpawnSkippedGameLevel {
  pub(crate) section: String,
  /// Its directory name, where the section names one.
  pub(crate) name: Option<String>,
  pub(crate) reason: String,
}
