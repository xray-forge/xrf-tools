use std::path::{Path, PathBuf};

use image::RgbaImage;
use image::imageops::crop_imm;
use xrf_error::{XrfError, XrfResult};

use crate::build::recipe::build_texture_bump_pair::BuildTextureBumpPair;
use crate::bump::GenerateBumpProcessor;

/// How a source image is laid out into the texture written from it.
#[derive(Clone, Debug, PartialEq)]
pub enum BuildTextureLayout {
  /// One face, as written: an image, a normal map or a terrain texture alike, which the SDK compresses the same way.
  Planar,
  /// Six square faces side by side, `+x -x +y -y +z -z` from the left, written as a cube (`ttCubeMap`).
  ///
  /// The strip is the SDK's cube source: `CImageManager` takes a source six times as wide as it is tall for a cube,
  /// and `ETools_cm.cpp` reads face `n` from the `n`-th square of it.
  CubeStrip,
  /// The `<name>.dds` and `<name>#.dds` pair a bump descriptor builds (`ttBumpMap`), from one source holding the height
  /// in its colour and the gloss in its alpha, as `DXTCompressBump` reads it.
  BumpPair(BuildTextureBumpPair),
}

impl BuildTextureLayout {
  /// Faces of a cube.
  pub const CUBE_FACES: u32 = 6;

  /// Every file a build of this layout writes at `destination`: the texture, and for a bump pair its `#` companion.
  pub fn get_written_paths(&self, destination: &Path) -> Vec<PathBuf> {
    match self {
      Self::BumpPair(_) => GenerateBumpProcessor::pair_paths(&BuildTextureBumpPair::bump_of(destination)).to_vec(),
      Self::Planar | Self::CubeStrip => vec![destination.to_path_buf()],
    }
  }

  /// The faces the source holds, left to right; a bump pair's one source is a single face.
  ///
  /// # Errors
  ///
  /// Returns an error for a cube source that is not six square faces side by side.
  pub fn split(&self, source: &RgbaImage) -> XrfResult<Vec<RgbaImage>> {
    match self {
      Self::Planar | Self::BumpPair(_) => Ok(vec![source.clone()]),
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
  use std::path::{Path, PathBuf};

  use image::{Rgba, RgbaImage};
  use xrf_error::XrfResult;

  use super::BuildTextureLayout;
  use crate::build::recipe::build_texture_bump_pair::BuildTextureBumpPair;

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
  fn names_every_file_a_build_writes() {
    let pair: BuildTextureLayout = BuildTextureLayout::BumpPair(BuildTextureBumpPair {
      virtual_height: 0.05,
      normal_map: None,
    });

    assert_eq!(
      pair.get_written_paths(Path::new("stone_bump.dds")),
      [PathBuf::from("stone_bump.dds"), PathBuf::from("stone_bump#.dds")]
    );
    assert_eq!(
      BuildTextureLayout::CubeStrip.get_written_paths(Path::new("sky.dds")),
      [PathBuf::from("sky.dds")]
    );
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
