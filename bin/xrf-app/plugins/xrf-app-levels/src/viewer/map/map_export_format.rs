use serde::Deserialize;

use crate::viewer::ViewerDdsExport;

/// What a level map is exported as: `CDemoRecord`'s own uncompressed TGA, a PNG, a DDS as the game loads a map, or, for
/// its annotations alone, their SVG.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MapExportFormat {
  Png,
  Tga,
  Svg,
  Dds(ViewerDdsExport),
}
