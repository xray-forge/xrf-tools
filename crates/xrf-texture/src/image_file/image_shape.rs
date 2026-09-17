use std::io::Cursor;

use image::{ImageFormat, ImageReader};
use serde::Serialize;

/// The size of a picture a webview renders as it stands, read from its header alone.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageShape {
  pub width: u32,
  pub height: u32,
  /// The format as the decoder recognised it, which is what the bytes say rather than what the name claims.
  pub format: String,
}

impl ImageShape {
  /// The shape these bytes declare, or `None` when they are not a picture this reads.
  pub fn of_bytes(bytes: &[u8]) -> Option<Self> {
    let reader: ImageReader<Cursor<&[u8]>> = ImageReader::new(Cursor::new(bytes)).with_guessed_format().ok()?;
    let format: ImageFormat = reader.format()?;
    let (width, height) = reader.into_dimensions().ok()?;

    Some(Self {
      width,
      height,
      format: Self::get_format_label(format),
    })
  }

  /// What to call a format on screen, in the spelling an author would recognise.
  fn get_format_label(format: ImageFormat) -> String {
    match format {
      ImageFormat::Png => "PNG".to_owned(),
      ImageFormat::Jpeg => "JPEG".to_owned(),
      ImageFormat::Bmp => "BMP".to_owned(),
      ImageFormat::Tga => "TGA".to_owned(),
      ImageFormat::Gif => "GIF".to_owned(),
      ImageFormat::WebP => "WebP".to_owned(),
      // Everything else the decoder knows and this viewer does not route: named by the crate rather than guessed at.
      other => format!("{other:?}").to_uppercase(),
    }
  }
}

#[cfg(test)]
mod tests {
  use image::{DynamicImage, RgbaImage};

  use super::ImageShape;

  /// One encoded picture of the given size.
  fn encoded(width: u32, height: u32, format: image::ImageFormat) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();

    DynamicImage::ImageRgba8(RgbaImage::new(width, height))
      .write_to(&mut std::io::Cursor::new(&mut bytes), format)
      .unwrap();

    bytes
  }

  #[test]
  fn reads_a_size_out_of_a_header() {
    for (format, label) in [(image::ImageFormat::Png, "PNG"), (image::ImageFormat::Bmp, "BMP")] {
      let shape: ImageShape = ImageShape::of_bytes(&encoded(8, 4, format)).unwrap();

      assert_eq!((shape.width, shape.height), (8, 4));
      assert_eq!(shape.format, label);
    }
  }

  #[test]
  fn names_the_format_the_bytes_are_rather_than_the_one_a_name_claims() {
    // The guess is made from content, so a picture saved under the wrong extension still reports what it is - which
    // is the reading a viewer wants when a tree has been through several hands.
    let shape: ImageShape = ImageShape::of_bytes(&encoded(2, 2, image::ImageFormat::Jpeg)).unwrap();

    assert_eq!(shape.format, "JPEG");
  }

  #[test]
  fn answers_nothing_for_bytes_that_are_not_a_picture() {
    assert_eq!(ImageShape::of_bytes(b"not a picture at all"), None);
    assert_eq!(ImageShape::of_bytes(&[]), None);
  }
}
