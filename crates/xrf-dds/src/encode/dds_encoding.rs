use ddsfile::{D3DFormat, Dds, NewD3dParams};
use image_dds::{ImageFormat, Mipmaps, Quality};
use xrf_error::{XrfError, XrfResult};

use crate::file::dds_file::DdsFile;
use crate::mip::dds_mip_chain::DdsMipChain;

/// How a texture is written: the layout its pixels are packed into, and how hard the encoder works at it.
///
/// The one door out of this crate for producing a DDS. It takes levels rather than an image because the levels are
/// the expensive part and are worth keeping: weighing five formats against one texture is one [`DdsMipChain`] and five
/// encodings of it, and a door taking an image would have reduced the same picture five times.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DdsEncoding {
  pub format: ImageFormat,
  pub quality: Quality,
}

impl DdsEncoding {
  pub fn new(format: ImageFormat, quality: Quality) -> Self {
    Self { format, quality }
  }

  /// Writes the chain into a texture, level for level.
  ///
  /// The encoder is told to take the levels it is handed rather than produce its own: the X-Ray converter's filter
  /// family lives in [`crate::DdsMipFilter`] and has no counterpart in `image_dds`, whose generator is a fixed box
  /// reduction.
  ///
  /// # Errors
  ///
  /// Returns an error when the chain cannot be laid out as a surface, or when the encoder refuses the pixels.
  pub fn encode(&self, chain: &DdsMipChain) -> XrfResult<DdsFile> {
    let surface = chain
      .to_surface()?
      .encode(self.format, self.quality, Mipmaps::FromSurface)
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

    let dds: Dds = match to_legacy_format(self.format) {
      Some(format) => {
        let mut dds: Dds = Dds::new_d3d(NewD3dParams {
          height: surface.height,
          width: surface.width,
          depth: None,
          format,
          mipmap_levels: (surface.mipmaps > 1).then_some(surface.mipmaps),
          caps2: None,
        })
        .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

        dds.data = surface.data;
        dds
      }
      None => surface
        .to_dds()
        .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?,
    };

    DdsFile::from_encoded(dds)
  }
}

/// The legacy `FourCC` spelling of a layout, for the formats that predate the DX10 header extension.
///
/// The encoder's own `to_dds` prefers a DX10 header whenever DXGI can name the format, which is every candidate but
/// nothing X-Ray ships: across `gamedata`, `gamedata-anomaly` and the vanilla trees, 13,274 textures carry a legacy
/// header and not one carries a DX10 header - Anomaly included, which runs DX11.
///
/// The reason it matters is the D3D9 path. `D3DXCreateTextureFromFileInMemoryEx` predates the extension and reads
/// only the old header, so a BC3 texture written with a DX10 header is refused by a renderer that supports BC3
/// perfectly well. Writing the legacy spelling wherever one exists is what makes the compatibility this crate reports
/// true of the files it produces.
///
/// BC7 has no legacy spelling and keeps the DX10 header, which is the same fact its `DX9: unsupported` badge states.
fn to_legacy_format(format: ImageFormat) -> Option<D3DFormat> {
  match format {
    ImageFormat::BC1RgbaUnorm => Some(D3DFormat::DXT1),
    ImageFormat::BC2RgbaUnorm => Some(D3DFormat::DXT3),
    ImageFormat::BC3RgbaUnorm => Some(D3DFormat::DXT5),
    // Byte for byte the same layout: `Rgba8Unorm` is red first, which is what D3D calls `A8B8G8R8`.
    ImageFormat::Rgba8Unorm => Some(D3DFormat::A8B8G8R8),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use image_dds::{ImageFormat, Quality};
  use xrf_error::XrfResult;

  use super::DdsEncoding;
  use crate::encode::dds_encode_candidate::DdsEncodeCandidate;
  use crate::file::dds_file::DdsFile;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  fn encode(width: u32, height: u32, mipmaps: DdsMipmaps) -> XrfResult<DdsFile> {
    let base: RgbaImage = RgbaImage::from_fn(width, height, |x, y| Rgba([(x * 16) as u8, (y * 32) as u8, 0, u8::MAX]));

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast).encode(&DdsMipChain::build(&base, mipmaps)?)
  }

  #[test]
  fn writes_exactly_the_levels_the_chain_holds() -> XrfResult {
    // The encoder is handed the levels rather than asked to make its own, so the file's mip count is the chain's
    // length and nothing else. A texture 1023 wide reduces nine times before it runs out.
    assert_eq!(
      encode(1023, 1020, DdsMipmaps::Filtered(DdsMipFilter::Kaiser))?
        .metadata()
        .mipmap_levels,
      10
    );
    assert_eq!(encode(1023, 1020, DdsMipmaps::Disabled)?.metadata().mipmap_levels, 1);

    Ok(())
  }

  #[test]
  fn every_candidate_encodes_and_reads_back_as_the_format_it_named() -> XrfResult {
    // Driven off the candidate list itself, so a format added to it is covered the day it is added rather than the
    // day somebody remembers to write a case for it.
    let base: RgbaImage = RgbaImage::from_fn(16, 16, |x, y| {
      Rgba([(x * 16) as u8, (y * 16) as u8, ((x + y) * 8) as u8, u8::MAX])
    });
    let chain: DdsMipChain = DdsMipChain::build(&base, DdsMipmaps::Filtered(DdsMipFilter::Box))?;

    for candidate in DdsEncodeCandidate::ALL {
      let file: DdsFile = DdsEncoding::new(candidate.to_image_format(), Quality::Fast).encode(&chain)?;
      let bytes: Vec<u8> = file.write_to_bytes()?;
      let read: DdsFile = DdsFile::read_from_bytes(&bytes)?;
      let metadata = read.metadata();

      assert_eq!(
        (metadata.width, metadata.height, metadata.mipmap_levels),
        (16, 16, 5),
        "{}",
        candidate.label()
      );

      // Every candidate decodes back to pixels, which is what the preview and the comparison both need.
      assert_eq!(read.decode_rgba(0)?.dimensions(), (16, 16), "{}", candidate.label());

      // BC7 is the one candidate with no legacy spelling, so it is the one written through a DX10 header - twenty
      // bytes longer, and a layout a reader that only knew the old header would miss entirely.
      // BC7 is the one candidate with no legacy `FourCC` spelling, so it is the only one written through a DX10
      // header. Everything else takes the old header the engine's own textures use, because the D3D9 loader reads
      // no other kind.
      let expected_header: u64 = if candidate == DdsEncodeCandidate::Bc7 { 148 } else { 128 };

      assert_eq!(metadata.metadata_size, expected_header, "{}", candidate.label());
      assert_eq!(
        metadata.dx10_format.is_some(),
        candidate == DdsEncodeCandidate::Bc7,
        "{}",
        candidate.label()
      );
    }

    Ok(())
  }

  #[test]
  fn keeps_dimensions_that_are_not_multiples_of_four() -> XrfResult {
    // The block compressor pads each level out to whole 4x4 blocks and records the unpadded size, so a texture keeps
    // the shape it was authored at.
    let metadata = encode(1023, 1020, DdsMipmaps::Disabled)?.metadata();

    assert_eq!((metadata.width, metadata.height), (1023, 1020));

    Ok(())
  }
}
