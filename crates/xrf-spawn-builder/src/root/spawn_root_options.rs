use serde::{Deserialize, Serialize};

/// How a root spawn is built beyond its levels.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnRootOptions {
  /// The level the actor starts on, where it is not the actor's own: xrAI's `-start`.
  pub start: Option<String>,
  /// Refuse the build where a restrictor cuts the AI map apart, rather than report it: xrAI's check without
  /// `-no_separator_check`, kept to real cuts.
  pub is_separator_strict: bool,
}
