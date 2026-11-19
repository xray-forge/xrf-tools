use serde::{Deserialize, Serialize};

/// How a root spawn is built beyond its levels.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnRootOptions {
  /// The level the actor starts on, if not its own: xrAI's `-start`.
  pub start: Option<String>,
  /// Refuse, rather than report, a restrictor cutting an AI map apart: xrAI's check without `-no_separator_check`,
  /// kept to real cuts.
  pub is_separator_strict: bool,
}
