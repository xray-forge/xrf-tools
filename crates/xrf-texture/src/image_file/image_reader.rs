//! Reading an image a caller names into the pixels every recipe here works in.

use std::fs;
use std::path::Path;

use image::{DynamicImage, ImageFormat, ImageResult, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// Reads a BMP, DDS, JPEG, PNG or TGA image as RGBA, in the format its extension names.
///
/// Not limited to DDS: a height map or a gloss mask is authoring input, and an author has it as whatever
/// their painting tool writes. What the engine loads is the output of these recipes, not their input.
///
/// # Errors
///
/// Returns an error naming the path when the file cannot be read or its extension names no format this decodes.
pub fn read_image_as_rgba<P: AsRef<Path>>(path: P) -> XrfResult<RgbaImage> {
  Ok(
    decode_image(path.as_ref())
      .map_err(|error| {
        XrfError::new_texture_processing_error(format!("Failed to read image {}: {error}", format_path(path.as_ref())))
      })?
      .to_rgba8(),
  )
}

/// Decodes the image at `path` in the format its extension names, as `image::open` does.
///
/// The file is read whole and decoded from a borrowed cursor, the reader every decode in the workspace shares: `image`
/// compiles each codec once per reader type, so streaming from the file would build all of them a second time.
///
/// # Errors
///
/// Returns an error when the extension names no format the decoder knows, or the file cannot be read or decoded.
pub(crate) fn decode_image(path: &Path) -> ImageResult<DynamicImage> {
  let format: ImageFormat = ImageFormat::from_path(path)?;

  image::load_from_memory_with_format(&fs::read(path)?, format)
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;
  use std::path::PathBuf;

  use image::{DynamicImage, ImageFormat, RgbImage};
  use xrf_test_utils::GeneratedResources;

  use super::{decode_image, read_image_as_rgba};

  /// A 3x2 picture encoded as `format`, written under `name`.
  fn written(name: &str, format: ImageFormat) -> PathBuf {
    let mut bytes: Vec<u8> = Vec::new();

    DynamicImage::ImageRgb8(RgbImage::new(3, 2))
      .write_to(&mut Cursor::new(&mut bytes), format)
      .expect("picture encodes");

    GeneratedResources::write(&format!("xrf-texture/image-reader/{name}"), bytes).expect("picture written")
  }

  #[test]
  fn decodes_each_raster_format_an_x_ray_tree_carries() {
    for (name, format) in [
      ("picture.bmp", ImageFormat::Bmp),
      ("picture.jpg", ImageFormat::Jpeg),
      ("picture.png", ImageFormat::Png),
      ("picture.tga", ImageFormat::Tga),
    ] {
      let image: DynamicImage = decode_image(&written(name, format)).expect("picture decodes");

      assert_eq!((image.width(), image.height()), (3, 2), "{name}");
    }
  }

  #[test]
  fn the_extension_picks_the_decoder_rather_than_the_bytes() {
    assert!(decode_image(&written("png-bytes.bmp", ImageFormat::Png)).is_err());
  }

  #[test]
  fn a_format_left_out_of_the_build_is_refused_naming_the_path() {
    let error: String = read_image_as_rgba(written("picture.webp", ImageFormat::Png))
      .expect_err("WebP is not built")
      .to_string();

    assert!(error.contains("picture.webp"), "got: {error}");
  }
}
