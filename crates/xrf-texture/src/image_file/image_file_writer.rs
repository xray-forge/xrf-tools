use std::path::Path;

use image::{ImageFormat, RgbaImage};
use xrf_dds::{DdsEncoding, DdsFile, DdsMetadata, DdsMipChain, DdsMipmaps, ImageFormat as DDSImageFormat, Quality};
use xrf_error::{MapXrfErr, XrfResult};

use crate::image_file::ImageFileReshape;

/// Writes an image out as the texture file a caller asked for.
pub struct ImageFileWriter;

impl ImageFileWriter {
  /// Mip chain written into a packed UI sprite sheet.
  ///
  /// None, following vanilla. The engine cannot use a chain here anyway, since the UI is authored on a 1024x768 canvas
  /// that is scaled up to the real resolution, so a sheet is magnified rather than minified and only level 0 is ever
  /// sampled.
  pub const UI_MIPMAPS: DdsMipmaps = DdsMipmaps::Disabled;

  /// Number of mip levels [`Self::UI_MIPMAPS`] produces, for comparing against an existing sheet.
  pub const UI_MIPMAP_LEVELS: u32 = 1;

  /// Write an image as a dds file with the given format and mip chain.
  ///
  /// Dimensions do not have to be multiples of 4. The block compressor pads every mip level out to whole
  /// 4x4 blocks itself and records the unpadded size in the header, so the file keeps the exact
  /// dimensions the image was built with.
  pub fn write_dds(path: &Path, image: &RgbaImage, format: DDSImageFormat, mipmaps: DdsMipmaps) -> XrfResult {
    DdsEncoding::new(format, Quality::Slow)
      .encode(&DdsMipChain::build(image, mipmaps)?)?
      .write_to_path(path)
  }

  pub fn write_png(path: &Path, image: &RgbaImage) -> XrfResult {
    image
      .save_with_format(path, ImageFormat::Png)
      .map_texture_processing_err()
  }

  /// How the sheet about to be written at `path` differs from the one it replaces, if it does.
  ///
  /// Packing is meant to replace a sheet's pixels, not its geometry. Canvas size and mip chain length are
  /// resource state that the packed sprite rectangles do not fully describe, so a sheet that quietly
  /// changes shape diverges from its pristine form and from the other resource repositories with nothing
  /// in the log to say so. One sheet can also be described by several description files, and packing only
  /// some of them would otherwise shrink it without a word. Nothing at `path` is no difference.
  pub fn compare_replaced_dds(path: &Path, width: u32, height: u32, mipmap_levels: u32) -> Option<ImageFileReshape> {
    if !path.is_file() {
      return None;
    }

    let metadata: DdsMetadata = match DdsFile::read_from_path(path) {
      Ok(existing) => existing.metadata(),
      Err(error) => {
        return Some(ImageFileReshape::Unreadable {
          reason: error.to_string(),
        });
      }
    };

    (metadata.width != width || metadata.height != height || metadata.mipmap_levels != mipmap_levels).then_some(
      ImageFileReshape::Reshaped {
        width: metadata.width,
        height: metadata.height,
        mipmap_levels: metadata.mipmap_levels,
      },
    )
  }
}
