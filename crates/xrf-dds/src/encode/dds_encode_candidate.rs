use ddsfile::DxgiFormat;
use image_dds::ImageFormat;

use crate::renderer::dds_format_support::DdsFormatSupport;
use crate::renderer::dds_renderer::DdsRenderer;

/// A format a texture can be re-encoded into, of the five worth offering.
///
/// The list is short on purpose. It is what the engine's own textures are authored in - the three DXT families and
/// uncompressed - plus BC7, which is the one modern block format worth the trade and which only the DX11 path loads.
/// Everything else `image_dds` can write is either a channel layout no X-Ray shader samples or a floating point
/// format the engine has no path for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DdsEncodeCandidate {
  Bc1,
  Bc2,
  Bc3,
  Rgba8,
  Bc7,
}

impl DdsEncodeCandidate {
  pub const ALL: [Self; 5] = [Self::Bc1, Self::Bc2, Self::Bc3, Self::Rgba8, Self::Bc7];

  /// Name of the format, with the DXT name the descriptor and the SDK use for it.
  pub const fn label(self) -> &'static str {
    match self {
      Self::Bc1 => "BC1 (DXT1)",
      Self::Bc2 => "BC2 (DXT3)",
      Self::Bc3 => "BC3 (DXT5)",
      Self::Rgba8 => "RGBA8",
      Self::Bc7 => "BC7",
    }
  }

  /// The encoder's name for the layout.
  pub const fn to_image_format(self) -> ImageFormat {
    match self {
      Self::Bc1 => ImageFormat::BC1RgbaUnorm,
      Self::Bc2 => ImageFormat::BC2RgbaUnorm,
      Self::Bc3 => ImageFormat::BC3RgbaUnorm,
      Self::Rgba8 => ImageFormat::Rgba8Unorm,
      Self::Bc7 => ImageFormat::BC7RgbaUnorm,
    }
  }

  /// The header's name for the layout.
  pub const fn to_dxgi_format(self) -> DxgiFormat {
    match self {
      Self::Bc1 => DxgiFormat::BC1_UNorm,
      Self::Bc2 => DxgiFormat::BC2_UNorm,
      Self::Bc3 => DxgiFormat::BC3_UNorm,
      Self::Rgba8 => DxgiFormat::R8G8B8A8_UNorm,
      Self::Bc7 => DxgiFormat::BC7_UNorm,
    }
  }

  /// Bits one pixel costs on the GPU, before any mip chain.
  ///
  /// A block format is priced by its block: BC1 spends 64 bits on sixteen pixels, the rest 128.
  pub const fn bits_per_pixel(self) -> u32 {
    match self {
      Self::Bc1 => 4,
      Self::Bc2 | Self::Bc3 | Self::Bc7 => 8,
      Self::Rgba8 => 32,
    }
  }

  /// Whether the layout carries an alpha channel worth storing.
  ///
  /// BC1 has one bit of it, which is a cutout rather than a channel, so it answers `false`: a texture whose alpha
  /// means something loses it here, and that is the point of saying so.
  pub const fn has_alpha_channel(self) -> bool {
    matches!(self, Self::Bc2 | Self::Bc3 | Self::Rgba8 | Self::Bc7)
  }

  /// Whether a renderer loads a texture written in this format.
  pub const fn support(self, renderer: DdsRenderer) -> DdsFormatSupport {
    renderer.supports(self.to_dxgi_format())
  }

  /// What the renderers together make of this format, as a sentence a surface can show beside it.
  ///
  /// Says the unverified path by name rather than folding it into the supported ones, because "we did not check GL"
  /// and "GL loads it" are different claims and only one of them is true.
  pub fn support_summary(self) -> String {
    let unsupported: Vec<&'static str> = Self::renderers_answering(self, DdsFormatSupport::Unsupported);
    let unverified: Vec<&'static str> = Self::renderers_answering(self, DdsFormatSupport::Unverified);

    match (unsupported.as_slice(), unverified.as_slice()) {
      ([], []) => String::from("all renderers"),
      ([], unverified) => format!("all renderers, {} unverified", unverified.join(" and ")),
      (unsupported, []) => format!("not on {}", unsupported.join(" or ")),
      (unsupported, unverified) => format!(
        "not on {}, {} unverified",
        unsupported.join(" or "),
        unverified.join(" and ")
      ),
    }
  }

  fn renderers_answering(self, answer: DdsFormatSupport) -> Vec<&'static str> {
    DdsRenderer::ALL
      .into_iter()
      .filter(|renderer| self.support(*renderer) == answer)
      .map(DdsRenderer::label)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use crate::encode::dds_encode_candidate::DdsEncodeCandidate;
  use crate::renderer::dds_format_support::DdsFormatSupport;
  use crate::renderer::dds_renderer::DdsRenderer;

  #[test]
  fn the_legacy_families_load_everywhere_that_was_checked() {
    // The three DXT families and uncompressed are what the engine's own textures are authored in, so every path that
    // was read loads them. GL was not read, and says so rather than being counted as agreement.
    for candidate in [
      DdsEncodeCandidate::Bc1,
      DdsEncodeCandidate::Bc2,
      DdsEncodeCandidate::Bc3,
      DdsEncodeCandidate::Rgba8,
    ] {
      assert_eq!(candidate.support(DdsRenderer::Dx9), DdsFormatSupport::Supported);
      assert_eq!(candidate.support(DdsRenderer::Dx11), DdsFormatSupport::Supported);
      assert_eq!(candidate.support(DdsRenderer::OpenGl), DdsFormatSupport::Unverified);
      assert_eq!(candidate.support_summary(), "all renderers, GL unverified");
    }
  }

  #[test]
  fn bc7_is_refused_by_the_d3d9_path() {
    // D3DX9 predates BC7 entirely, so this is the one candidate that costs a renderer.
    assert_eq!(
      DdsEncodeCandidate::Bc7.support(DdsRenderer::Dx9),
      DdsFormatSupport::Unsupported
    );
    assert_eq!(
      DdsEncodeCandidate::Bc7.support(DdsRenderer::Dx11),
      DdsFormatSupport::Supported
    );
    assert_eq!(DdsEncodeCandidate::Bc7.support_summary(), "not on DX9, GL unverified");
  }

  #[test]
  fn bc1_reports_no_alpha_channel() {
    // One bit of cutout is not a channel, and a texture whose alpha means something loses it here.
    assert!(!DdsEncodeCandidate::Bc1.has_alpha_channel());
    assert_eq!(
      DdsEncodeCandidate::ALL
        .iter()
        .filter(|it| it.has_alpha_channel())
        .count(),
      4
    );
  }

  #[test]
  fn block_formats_are_priced_by_their_block() {
    // BC1 spends 64 bits on sixteen pixels and the other block formats 128, against 32 bits a pixel uncompressed.
    assert_eq!(DdsEncodeCandidate::Bc1.bits_per_pixel(), 4);
    assert_eq!(DdsEncodeCandidate::Bc3.bits_per_pixel(), 8);
    assert_eq!(DdsEncodeCandidate::Bc7.bits_per_pixel(), 8);
    assert_eq!(DdsEncodeCandidate::Rgba8.bits_per_pixel(), 32);
  }
}
