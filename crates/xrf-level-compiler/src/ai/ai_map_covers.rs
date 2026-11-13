use serde::{Deserialize, Serialize};

/// Which covers the AI map compiler computes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiMapCovers {
  /// None, as xrAI's `-draft` leaves them: every value 0, and no `build.cform` needed.
  None,
  /// Each cover node traced against `build.cform` and the other nodes mixed from them, as xrAI's full run does.
  #[default]
  Full,
}
