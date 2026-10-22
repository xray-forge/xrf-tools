use ddsfile::{Caps, Caps2, D3DFormat, Dds, NewD3dParams};
use image_dds::{ImageFormat, Mipmaps, Quality, SurfaceRgba8};
use xrf_error::{XrfError, XrfResult};

use crate::file::dds_file::DdsFile;
use crate::mip::dds_mip_chain::DdsMipChain;

/// How a texture is written: the layout its pixels are packed into, and how hard the encoder works at it.
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
  /// # Errors
  ///
  /// Returns an error when the chain cannot be laid out as a surface, or when the encoder refuses the pixels.
  pub fn encode(&self, chain: &DdsMipChain) -> XrfResult<DdsFile> {
    self.write(chain.to_surface()?)
  }

  /// Writes six chains into a cube texture, `+x -x +y -y +z -z`, each face level for level as [`Self::encode`] does.
  ///
  /// # Errors
  ///
  /// Returns an error unless there are six square faces alike in size and levels, or where [`Self::encode`] does.
  pub fn encode_cube(&self, faces: &[DdsMipChain]) -> XrfResult<DdsFile> {
    self.write(DdsMipChain::to_cube_surface(faces)?)
  }

  /// Writes a surface of one layer, or of six as a cube.
  fn write(&self, surface: SurfaceRgba8<Vec<u8>>) -> XrfResult<DdsFile> {
    let is_cube: bool = surface.layers == 6;
    let surface = surface
      .encode(self.format, self.quality, Mipmaps::FromSurface)
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

    let mut dds: Dds = match to_legacy_format(self.format) {
      Some(format) => {
        let mut dds: Dds = Dds::new_d3d(NewD3dParams {
          height: surface.height,
          width: surface.width,
          depth: None,
          format,
          mipmap_levels: (surface.mipmaps > 1).then_some(surface.mipmaps),
          caps2: is_cube.then_some(Caps2::CUBEMAP | Caps2::CUBEMAP_ALLFACES),
        })
        .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

        dds.data = surface.data;
        dds
      }
      None => surface
        .to_dds()
        .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?,
    };

    // A cube is a complex surface whether or not its faces carry levels, which Direct3D's loaders ask of its header;
    // a DX10 header counts it as one cube, where `image_dds` leaves the count to `ddsfile`, which divides one layer by
    // six into none.
    if is_cube {
      dds.header.caps.insert(Caps::COMPLEX);

      if let Some(header10) = dds.header10.as_mut() {
        header10.array_size = 1;
      }
    }

    DdsFile::from_encoded(dds)
  }
}

/// The legacy `FourCC` spelling of a layout, for the formats that predate the DX10 header extension.
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
  use ddsfile::Dds;
  use image::{Rgba, RgbaImage};
  use image_dds::{ImageFormat, Quality, SurfaceRgba8};
  use xrf_error::{XrfError, XrfResult};

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
  fn writes_six_faces_as_a_cube_in_their_order() -> XrfResult {
    // Each face one flat shade, so the layer it reads back from shows where it was written.
    let new_face = |side: u32, face: u8| {
      DdsMipChain::build(
        &RgbaImage::from_pixel(side, side, Rgba([40 * (face + 1), 0, 0, u8::MAX])),
        DdsMipmaps::Filtered(DdsMipFilter::Kaiser),
      )
    };
    let faces: Vec<DdsMipChain> = (0..6).map(|face| new_face(16, face)).collect::<XrfResult<_>>()?;

    for candidate in [
      DdsEncodeCandidate::Bc3,
      DdsEncodeCandidate::Bc7,
      DdsEncodeCandidate::Rgba8,
    ] {
      let bytes: Vec<u8> = DdsEncoding::new(candidate.to_image_format(), Quality::Fast)
        .encode_cube(&faces)?
        .write_to_bytes()?;
      let read: DdsFile = DdsFile::read_from_bytes(&bytes)?;

      assert!(read.metadata().is_cubemap, "{}", candidate.label());
      assert_eq!(
        (read.metadata().width, read.metadata().mipmap_levels),
        (16, 5),
        "{}",
        candidate.label()
      );

      let dds: Dds =
        Dds::read(&mut bytes.as_slice()).map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

      // A DX10 header counts one cube of six faces; a legacy one has no count.
      assert!(
        dds.header10.as_ref().is_none_or(|header10| header10.array_size == 1),
        "{}",
        candidate.label()
      );

      let surface: SurfaceRgba8<Vec<u8>> = SurfaceRgba8::decode_layers_mipmaps_dds(&dds, 0..6, 0..1)
        .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

      for face in 0..6u8 {
        let red: u8 = surface.data[usize::from(face) * 16 * 16 * 4];

        // Within the four steps BC3's five bits of red round a shade by.
        assert!(
          red.abs_diff(40 * (face + 1)) <= 4,
          "{} face {face} reads {red}",
          candidate.label()
        );
      }
    }

    // Five faces, or faces of two sizes, are no cube.
    let encoding: DdsEncoding = DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast);
    let mut uneven: Vec<DdsMipChain> = (0..5).map(|face| new_face(16, face)).collect::<XrfResult<_>>()?;

    assert!(encoding.encode_cube(&uneven).is_err());

    uneven.push(new_face(8, 5)?);

    assert!(encoding.encode_cube(&uneven).is_err());

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
