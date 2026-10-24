use serde::Deserialize;
use xrf_texture::{TextureEncodingFormat, TextureEncodingQuality};

/// What a level map is exported as: `CDemoRecord`'s own uncompressed TGA, a PNG, a DDS as the game loads a map, or, for
/// its annotations alone, their SVG.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum MapExportFormat {
  Png,
  Tga,
  Svg,
  Dds {
    format: TextureEncodingFormat,
    quality: TextureEncodingQuality,
    /// Whether it carries its whole mip chain, reduced with the X-Ray converter's Kaiser kernel.
    is_mipmapped: bool,
  },
}
