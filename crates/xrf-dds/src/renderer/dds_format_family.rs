//! Which family a layout belongs to, which is what every renderer's answer is phrased in.
//!
//! Kept apart from the renderers because it is a fact about the format rather than about any loader: the same
//! three sets answer for the D3D9 path, the DX11 path, and whatever reads them next.

use ddsfile::DxgiFormat;

/// The three block families D3DX9 knows, which are DXT1 through DXT5 under their modern names.
pub(crate) const fn is_legacy_block_compressed(format: DxgiFormat) -> bool {
  matches!(
    format,
    DxgiFormat::BC1_UNorm
      | DxgiFormat::BC1_UNorm_sRGB
      | DxgiFormat::BC2_UNorm
      | DxgiFormat::BC2_UNorm_sRGB
      | DxgiFormat::BC3_UNorm
      | DxgiFormat::BC3_UNorm_sRGB
  )
}

/// Every block-compressed layout, the three families above plus the ones D3DX9 never saw.
pub(crate) const fn is_block_compressed(format: DxgiFormat) -> bool {
  is_legacy_block_compressed(format)
    || matches!(
      format,
      DxgiFormat::BC4_UNorm
        | DxgiFormat::BC4_SNorm
        | DxgiFormat::BC5_UNorm
        | DxgiFormat::BC5_SNorm
        | DxgiFormat::BC6H_UF16
        | DxgiFormat::BC6H_SF16
        | DxgiFormat::BC7_UNorm
        | DxgiFormat::BC7_UNorm_sRGB
    )
}

/// The uncompressed layouts a texture of this era is authored in.
pub(crate) const fn is_uncompressed(format: DxgiFormat) -> bool {
  matches!(
    format,
    DxgiFormat::R8G8B8A8_UNorm
      | DxgiFormat::R8G8B8A8_UNorm_sRGB
      | DxgiFormat::B8G8R8A8_UNorm
      | DxgiFormat::B8G8R8A8_UNorm_sRGB
      | DxgiFormat::B8G8R8X8_UNorm
      | DxgiFormat::B5G6R5_UNorm
      | DxgiFormat::B5G5R5A1_UNorm
      | DxgiFormat::B4G4R4A4_UNorm
      | DxgiFormat::A8_UNorm
      | DxgiFormat::R8_UNorm
      | DxgiFormat::R8G8_UNorm
  )
}
