//! Reading an image a caller names into the pixels every recipe here works in.

use std::path::Path;

use image::RgbaImage;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// Reads any image the decoder recognises as RGBA.
///
/// Not limited to DDS: a height map or a gloss mask is authoring input, and an author has it as whatever
/// their painting tool writes. What the engine loads is the output of these recipes, not their input.
///
/// # Errors
///
/// Returns an error naming the path when the file cannot be read or is in no format the decoder knows.
pub fn read_image_as_rgba<P: AsRef<Path>>(path: P) -> XrfResult<RgbaImage> {
  Ok(
    image::open(path.as_ref())
      .map_err(|error| {
        XrfError::new_texture_processing_error(format!("Failed to read image {}: {error}", format_path(path.as_ref())))
      })?
      .to_rgba8(),
  )
}
