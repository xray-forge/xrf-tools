use serde::Deserialize;

use crate::viewer::ViewerDdsExport;

/// What a cubemap is exported as: one cube DDS, six TGAs named `1` to `6` as `CDemoRecord` writes its faces, or six
/// PNGs named by face.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CubeExportFormat {
  Png,
  Tga,
  Dds(ViewerDdsExport),
}
