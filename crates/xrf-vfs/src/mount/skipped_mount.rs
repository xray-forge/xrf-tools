use std::path::PathBuf;

/// A source a plan named that could not be opened, and why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XraySkippedMount {
  /// Where the source that failed to open lives.
  pub path: PathBuf,
  /// How the plan described it, such as an `fsgame.ltx` alias.
  pub origin: String,
  /// Why it could not be opened, rendered for a person.
  pub reason: String,
}
