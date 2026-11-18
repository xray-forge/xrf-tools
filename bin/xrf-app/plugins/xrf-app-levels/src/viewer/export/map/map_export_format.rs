use serde::Deserialize;

use crate::viewer::ViewerDdsExport;

/// A level map export format: a PNG, an uncompressed TGA as `CDemoRecord` writes, a DDS as the game loads maps, or SVG
/// for the annotations alone.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum MapExportFormat {
  Png,
  Tga,
  Svg,
  Dds(ViewerDdsExport),
}
