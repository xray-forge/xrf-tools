use serde::{Deserialize, Serialize};

use crate::scene::SceneSource;

/// What opening a Level Editor scene was asked to read, and where its library objects are looked for.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneOpenRequest {
  pub source: SceneSource,
  /// Directories the scene objects' `.object` files are looked for in, in search order; `objects` beside the directory
  /// holding the `.level` where none are given.
  pub objects_roots: Option<Vec<String>>,
}
