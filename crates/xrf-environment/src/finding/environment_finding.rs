use serde::Serialize;

use crate::finding::environment_rule::EnvironmentRule;

/// One problem with a game's environment configs, placed at the file, section and key it is about.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentFinding {
  pub rule: EnvironmentRule,
  /// The config it is in, as a logical path.
  pub file: String,
  /// The section it is in, where it is about one.
  pub section: Option<String>,
  /// The key it is about, where it is about one.
  pub key: Option<String>,
  pub message: String,
}
