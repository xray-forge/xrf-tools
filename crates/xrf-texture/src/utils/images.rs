use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use xrf_dds::{DdsEncodeOptions, DdsFile, ImageFormat as DDSImageFormat, Mipmaps, Quality};
use xrf_error::XrfResult;
use xrf_output::OutputOptions;
use xrf_utils::{assert, format_path};

/// Scale an image to the given bounds and centre it on a transparent canvas of exactly that size.
///
/// Scaling preserves the aspect ratio, so an image whose proportions differ from the bounds ends up
/// letterboxed rather than distorted. An image that already matches the bounds is returned untouched.
pub fn fit_image_into_bounds(image: DynamicImage, width: u32, height: u32, source: &Path) -> XrfResult<DynamicImage> {
  let image_width: u32 = image.width();
  let image_height: u32 = image.height();

  if image_width == width && image_height == height {
    return Ok(image);
  }

  log::info!(
    "Rescaling image to bounds: {}x{} from {}x{} {}",
    width,
    height,
    image_width,
    image_height,
    format_path(source)
  );

  let rescaled_image: DynamicImage = image.resize(width, height, FilterType::Lanczos3);
  let rescaled_width: u32 = rescaled_image.width();
  let rescaled_height: u32 = rescaled_image.height();

  if rescaled_width == width && rescaled_height == height {
    return Ok(rescaled_image);
  }

  log::info!(
    "Re-center rescaled image to bounds: {}x{} from {}x{} {}",
    width,
    height,
    rescaled_width,
    rescaled_height,
    format_path(source)
  );

  let mut centered: ImageBuffer<Rgba<u8>, Vec<u8>> = RgbaImage::new(width, height);

  assert(
    rescaled_width <= width,
    "Unexpected width {rescaled_width} > {width} when rescaling",
  )?;
  assert(
    rescaled_height <= height,
    "Unexpected height {rescaled_height} > {height} when rescaling",
  )?;

  centered.copy_from(
    &rescaled_image,
    (width - rescaled_width) / 2,
    (height - rescaled_height) / 2,
  )?;

  Ok(centered.into())
}

/// Write an image as a dds file with the given format and mip chain.
///
/// Dimensions do not have to be multiples of 4. The block compressor pads every mip level out to whole
/// 4x4 blocks itself and records the unpadded size in the header, so the file keeps the exact
/// dimensions the image was built with.
pub fn save_image_as_ui_dds(path: &Path, image: &RgbaImage, format: DDSImageFormat, mipmaps: Mipmaps) -> XrfResult {
  DdsFile::encode_rgba(image, DdsEncodeOptions::new(format, Quality::Slow, mipmaps))?.write_to_path(path)
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
