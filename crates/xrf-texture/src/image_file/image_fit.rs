//! Making an image exactly the size something else needs it.

use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba, RgbaImage};
use xrf_error::XrfResult;
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
