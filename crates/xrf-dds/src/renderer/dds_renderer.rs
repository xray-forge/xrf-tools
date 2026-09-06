use ddsfile::{D3DFormat, DxgiFormat};

use crate::renderer::dds_format_family::{is_block_compressed, is_legacy_block_compressed, is_uncompressed};

use crate::renderer::dds_format_support::DdsFormatSupport;

/// A renderer path of the engine, as far as what it will load off disk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DdsRenderer {
  Dx9,
  Dx11,
  OpenGl,
}

impl DdsRenderer {
  pub const ALL: [Self; 3] = [Self::Dx9, Self::Dx11, Self::OpenGl];

  pub const fn label(self) -> &'static str {
    match self {
      Self::Dx9 => "DX9",
      Self::Dx11 => "DX11",
      Self::OpenGl => "GL",
    }
  }

  /// Whether this renderer loads a texture stored in `format`.
  ///
  /// DX11 accepts whatever DirectXTex parses and the device supports, which covers every block-compressed family and
  /// the uncompressed layouts. DX9 is narrower and not by the engine's choice: `D3DXCreateTextureFromFileInMemoryEx`
  /// is a D3DX9 entry point, and D3DX9 predates BC4, BC5 and BC7 entirely, so those are refused by the library rather
  /// than by any code that can be read. GL was never checked, so it answers [`DdsFormatSupport::Unverified`] for
  /// everything rather than guessing in either direction.
  pub const fn supports(self, format: DxgiFormat) -> DdsFormatSupport {
    match self {
      Self::OpenGl => DdsFormatSupport::Unverified,
      Self::Dx11 => {
        if is_block_compressed(format) || is_uncompressed(format) {
          DdsFormatSupport::Supported
        } else {
          DdsFormatSupport::Unverified
        }
      }
      Self::Dx9 => {
        if is_legacy_block_compressed(format) || is_uncompressed(format) {
          DdsFormatSupport::Supported
        } else {
          DdsFormatSupport::Unsupported
        }
      }
    }
  }
}

impl DdsRenderer {
  /// Whether this renderer loads a texture whose header names a legacy D3D9 format.
  ///
  /// DX9 answers yes to all of them by construction: this enum is the D3D9 format set, and `D3DXCreateTexture*` is a
  /// D3D9 entry point. DX11 answers for the three DXT families, which are BC1 through BC3 under their older names,
  /// and leaves the rest unverified - DirectXTex converts many legacy layouts on load, but which ones was not read,
  /// and a guess in either direction would be a claim about whether somebody's texture appears in the game.
  pub const fn supports_d3d(self, format: D3DFormat) -> DdsFormatSupport {
    match self {
      Self::OpenGl => DdsFormatSupport::Unverified,
      Self::Dx9 => DdsFormatSupport::Supported,
      Self::Dx11 => match format {
        D3DFormat::DXT1 | D3DFormat::DXT2 | D3DFormat::DXT3 | D3DFormat::DXT4 | D3DFormat::DXT5 => {
          DdsFormatSupport::Supported
        }
        _ => DdsFormatSupport::Unverified,
      },
    }
  }
}
