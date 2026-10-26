use image::RgbaImage;
use image::imageops::crop_imm;
use xrf_error::{XrfError, XrfResult};

/// How a source image is laid out into the texture written from it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildTextureLayout {
  /// One face, as written: an image, a normal map or a terrain texture alike, which the SDK compresses the same way.
  Planar,
  /// Six square faces side by side, `+x -x +y -y +z -z` from the left, written as a cube (`ttCubeMap`).
  ///
  /// The strip is the SDK's cube source: `CImageManager` takes a source six times as wide as it is tall for a cube,
  /// and `ETools_cm.cpp` reads face `n` from the `n`-th square of it.
  CubeStrip,
}

impl BuildTextureLayout {
  /// Faces of a cube.
  pub const CUBE_FACES: u32 = 6;

  /// The faces the source holds, left to right.
  ///
  /// # Errors
  ///
  /// Returns an error for a cube source that is not six square faces side by side.
  pub fn split(self, source: &RgbaImage) -> XrfResult<Vec<RgbaImage>> {
    match self {
      Self::Planar => Ok(vec![source.clone()]),
      Self::CubeStrip => {
        let (width, height): (u32, u32) = source.dimensions();

        if height == 0 || height.checked_mul(Self::CUBE_FACES) != Some(width) {
          return Err(XrfError::new_texture_processing_error(format!(
            "A cube map is built from six square faces side by side, so a {width}x{height} source is not one"
          )));
        }

        Ok(
          (0..Self::CUBE_FACES)
            .map(|face| crop_imm(source, face * height, 0, height, height).to_image())
            .collect(),
        )
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use xrf_error::XrfResult;

  use super::BuildTextureLayout;

  #[test]
  fn takes_a_cubes_faces_from_the_left_in_dds_order() -> XrfResult {
    // Each face its own grey, `+x` darkest and leftmost.
    let strip: RgbaImage = RgbaImage::from_fn(12, 2, |x, _| Rgba([(x / 2) as u8 * 40, 0, 0, u8::MAX]));
    let faces: Vec<RgbaImage> = BuildTextureLayout::CubeStrip.split(&strip)?;

    assert_eq!(
      faces
        .iter()
        .map(|face| (face.dimensions(), face.get_pixel(1, 1).0[0]))
        .collect::<Vec<_>>(),
      [0, 40, 80, 120, 160, 200].map(|grey| ((2, 2), grey))
    );

    Ok(())
  }

  #[test]
  fn refuses_a_cube_source_that_is_not_six_squares() {
    assert!(BuildTextureLayout::CubeStrip.split(&RgbaImage::new(12, 3)).is_err());
    assert!(BuildTextureLayout::CubeStrip.split(&RgbaImage::new(0, 0)).is_err());
    assert_eq!(
      BuildTextureLayout::Planar
        .split(&RgbaImage::new(12, 3))
        .map(|faces| faces.len())
        .ok(),
      Some(1)
    );
  }
}
