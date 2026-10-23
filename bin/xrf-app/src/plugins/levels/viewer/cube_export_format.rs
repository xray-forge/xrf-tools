use serde::Deserialize;

use xrf_texture::{TextureEncodingFormat, TextureEncodingQuality};

/// What a cubemap is exported as: one cube DDS, six TGAs named `1` to `6` as `CDemoRecord` writes its faces, or six
/// PNGs named by face.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum CubeExportFormat {
  Png,
  Tga,
  Dds {
    format: TextureEncodingFormat,
    quality: TextureEncodingQuality,
    /// Whether each face carries its whole mip chain, reduced with the X-Ray converter's Kaiser kernel.
    is_mipmapped: bool,
  },
}
