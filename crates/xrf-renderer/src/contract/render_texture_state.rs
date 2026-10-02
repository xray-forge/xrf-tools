use serde::{Deserialize, Serialize};

/// Where one texture a viewport's scene samples stands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RenderTextureState {
  Loading,
  #[serde(rename_all = "camelCase")]
  Loaded {
    width: u32,
    height: u32,
    levels: u32,
    /// The file's own layout, as the textures explorer names it.
    layout: String,
    /// Whether it was expanded to eight bits a channel rather than uploaded as stored.
    is_expanded: bool,
  },
  /// Its reference resolved to no file.
  Missing,
  /// Its file could not be read or laid out, and why.
  Failed {
    reason: String,
  },
}
