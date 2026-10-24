use serde::Deserialize;
use xrf_dds::{DdsEncoding, DdsMipFilter, DdsMipmaps};
use xrf_texture::{TextureEncodingFormat, TextureEncodingQuality};

/// A DDS an export writes, as the game loads a map or a cubemap: its format, how hard it is encoded, and its mipmaps.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewerDdsExport {
  pub format: TextureEncodingFormat,
  pub quality: TextureEncodingQuality,
  /// Whether it carries its whole mip chain, reduced with the X-Ray converter's Kaiser kernel.
  pub is_mipmapped: bool,
}

impl ViewerDdsExport {
  /// The encoder it writes with, and its mipmaps: the whole chain, reduced with the X-Ray converter's Kaiser kernel,
  /// or none.
  pub fn to_encoding(self) -> (DdsEncoding, DdsMipmaps) {
    let mipmaps: DdsMipmaps = if self.is_mipmapped {
      DdsMipmaps::Filtered(DdsMipFilter::Kaiser)
    } else {
      DdsMipmaps::Disabled
    };

    (
      DdsEncoding::new(self.format.to_candidate().to_image_format(), self.quality.to_quality()),
      mipmaps,
    )
  }
}
