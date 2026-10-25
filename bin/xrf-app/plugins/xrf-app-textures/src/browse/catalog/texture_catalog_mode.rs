use serde::{Deserialize, Serialize};

/// How a listing addressed what it found.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureCatalogMode {
  /// The game tree: listed by engine reference, archives included, files outside `textures\` counted not listed.
  #[default]
  Roots,
  /// A plain directory: every `.dds` under it, addressed by its own path.
  LooseDirectory,
}
