//! Writing an image out as the texture file a caller asked for.

use std::path::Path;

use image::{ImageFormat, RgbaImage};
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipmaps, ImageFormat as DDSImageFormat, Quality};
use xrf_error::XrfResult;
use xrf_output::OutputOptions;
use xrf_utils::format_path;

/// Mip chain written into a packed UI sprite sheet.
///
/// None, following vanilla. The engine cannot use a chain here anyway, since the UI is authored on a 1024x768 canvas
/// that is scaled up to the real resolution, so a sheet is magnified rather than minified and only level 0 is ever
/// sampled.
pub(crate) const UI_MIPMAPS: DdsMipmaps = DdsMipmaps::Disabled;

/// Number of mip levels [`UI_MIPMAPS`] produces, for comparing against an existing sheet.
pub(crate) const UI_MIPMAP_LEVELS: u32 = 1;

/// Write an image as a dds file with the given format and mip chain.
///
/// Dimensions do not have to be multiples of 4. The block compressor pads every mip level out to whole
/// 4x4 blocks itself and records the unpadded size in the header, so the file keeps the exact
/// dimensions the image was built with.
pub fn save_image_as_ui_dds(path: &Path, image: &RgbaImage, format: DDSImageFormat, mipmaps: DdsMipmaps) -> XrfResult {
  DdsEncoding::new(format, Quality::Slow)
    .encode(&DdsMipChain::build(image, mipmaps)?)?
    .write_to_path(path)
}

/// Warn when the sheet about to be written at `path` is shaped differently from the one it replaces.
///
/// Packing is meant to replace a sheet's pixels, not its geometry. Canvas size and mip chain length are
/// resource state that the packed sprite rectangles do not fully describe, so a sheet that quietly
/// changes shape diverges from its pristine form and from the other resource repositories with nothing
/// in the log to say so. One sheet can also be described by several description files, and packing only
/// some of them would otherwise shrink it without a word.
pub fn warn_on_reshaped_ui_dds(output: &OutputOptions, path: &Path, width: u32, height: u32, mipmap_levels: u32) {
  if !path.is_file() {
    return;
  }

  let existing: DdsFile = match DdsFile::read_from_path(path) {
    Ok(existing) => existing,
    Err(error) => {
      xrf_output::warning!(
        output,
        "Cannot compare shape against replaced file {}: {}",
        format_path(path),
        error
      );

      return;
    }
  };
  let metadata = existing.metadata();

  if metadata.width != width || metadata.height != height || metadata.mipmap_levels != mipmap_levels {
    xrf_output::warning!(
      output,
      "Replacing {} of {}x{} with {} mipmap levels by {}x{} with {} mipmap levels",
      format_path(path),
      metadata.width,
      metadata.height,
      metadata.mipmap_levels,
      width,
      height,
      mipmap_levels
    );
  }
}

pub fn save_image_as_ui_png(path: &Path, image: &RgbaImage) -> XrfResult {
  Ok(image.save_with_format(path, ImageFormat::Png)?)
}
