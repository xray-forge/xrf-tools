use serde::{Deserialize, Serialize};

/// A drawable kind beside the static scene, as a report names it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDrawableKind {
  /// The grass's tufts.
  Grass,
}
