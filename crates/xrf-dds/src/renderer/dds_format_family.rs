use ddsfile::DxgiFormat;

/// Which family a layout belongs to, which is what every renderer's answer is phrased in.
///
/// Kept apart from the renderers because it is a fact about the format rather than about any loader: the same three
/// families answer for the D3D9 path, the DX11 path, and whatever reads them next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DdsFormatFamily {
  /// The three block families D3DX9 knows, which are DXT1 through DXT5 under their modern names.
  LegacyBlockCompressed,
  /// The block families D3DX9 never saw: BC4 through BC7.
  ModernBlockCompressed,
  /// The uncompressed layouts a texture of this era is authored in.
  Uncompressed,
}

impl DdsFormatFamily {
  /// The family `format` belongs to, or `None` for a layout outside all three.
  pub(crate) const fn of(format: DxgiFormat) -> Option<Self> {
    match format {
      DxgiFormat::BC1_UNorm
      | DxgiFormat::BC1_UNorm_sRGB
      | DxgiFormat::BC2_UNorm
      | DxgiFormat::BC2_UNorm_sRGB
      | DxgiFormat::BC3_UNorm
      | DxgiFormat::BC3_UNorm_sRGB => Some(Self::LegacyBlockCompressed),
      DxgiFormat::BC4_UNorm
      | DxgiFormat::BC4_SNorm
      | DxgiFormat::BC5_UNorm
      | DxgiFormat::BC5_SNorm
      | DxgiFormat::BC6H_UF16
      | DxgiFormat::BC6H_SF16
      | DxgiFormat::BC7_UNorm
      | DxgiFormat::BC7_UNorm_sRGB => Some(Self::ModernBlockCompressed),
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
      | DxgiFormat::R8G8_UNorm => Some(Self::Uncompressed),
      _ => None,
    }
  }
}
