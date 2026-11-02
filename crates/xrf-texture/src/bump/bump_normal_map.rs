use image::{Rgba, RgbaImage};
use xrf_math::Vector3d;

use crate::bump::bump_vector::BumpVector;
use crate::bump::grey_plane::GreyPlane;

/// The normals a height plane implies, as the bytes a bump texture stores: `ConvertAlphaToNormalMap_4x` and
/// `CalculateNormalMap` (`xray/trunk/xrDXT/NormalMapGen.cpp`).
pub(crate) struct BumpNormalMap;

impl BumpNormalMap {
  /// What the SDK multiplies the virtual height by before differentiating.
  ///
  /// Two other factors are in that source, commented out beside this one, against the 3x3 and a gentler 4x kernel.
  /// This is the pairing that shipped.
  const HEIGHT_SCALE: f32 = 200.0;

  /// The slope along each axis is the difference between the two neighbours either side, halved, which is the
  /// narrowest kernel that is still centred. Taking neighbours from the opposite edge rather than clamping is what
  /// makes a tiling texture tile without a seam, and the SDK passes `wrap` for exactly that.
  ///
  /// The cross product of the two slope vectors `(1, 0, du)` and `(0, 1, dv)` is `(-du, -dv, 1)`, so a flat region
  /// answers straight up and a steep one leans against the slope. Each kernel is summed in the order its elements are
  /// listed, as the convolver sums them.
  pub(crate) fn derive(height: &GreyPlane, virtual_height: f32) -> RgbaImage {
    let scale: f32 = virtual_height * Self::HEIGHT_SCALE;

    RgbaImage::from_fn(height.width, height.height, |x, y| {
      let (x, y): (i64, i64) = (i64::from(x), i64::from(y));
      let du: f32 = (height.get_level_wrapped(x - 1, y) * -0.5 + height.get_level_wrapped(x + 1, y) * 0.5) * scale;
      // The v kernel reads the row below as positive, which is what puts the normal in the handedness the packed pair
      // is authored against.
      let dv: f32 = (height.get_level_wrapped(x, y + 1) * 0.5 + height.get_level_wrapped(x, y - 1) * -0.5) * scale;
      let magnitude: f32 = (du * du + dv * dv + 1.0).sqrt();
      let [red, green, blue] = BumpVector::pack(&Vector3d::new(-du / magnitude, -dv / magnitude, 1.0 / magnitude));

      // Alpha is left at nought here; the gloss reversal fills it with the normal's own red a step later.
      Rgba([red, green, blue, 0])
    })
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};

  use super::BumpNormalMap;
  use crate::bump::grey_plane::GreyPlane;

  /// A plane whose value is chosen per pixel, written into all three colour channels.
  fn plane(width: u32, height: u32, value: impl Fn(u32, u32) -> u8) -> GreyPlane {
    GreyPlane::from_rgb_average(&RgbaImage::from_fn(width, height, |x, y| {
      let level: u8 = value(x, y);

      Rgba([level, level, level, 0])
    }))
  }

  #[test]
  fn a_flat_height_answers_straight_up() {
    // Nothing slopes anywhere, so every normal is `(0, 0, 1)`, which packs to the 127 the shipped textures use.
    let normals: RgbaImage = BumpNormalMap::derive(&plane(4, 4, |_, _| 128), 0.05);

    assert!(
      normals.pixels().all(|pixel| pixel.0[..3] == [127, 127, 255]),
      "expected every normal to point up, got {:?}",
      normals.get_pixel(0, 0)
    );
  }

  #[test]
  fn a_slope_leans_against_its_own_rise() {
    // Height climbing with x means `du` is positive, and the normal's x is `-du`, so it leans back below the middle.
    let normals: RgbaImage = BumpNormalMap::derive(&plane(8, 1, |x, _| (x * 16) as u8), 0.05);
    let leaning = normals.get_pixel(4, 0).0;

    assert!(leaning[0] < 127, "expected a leftward lean, got {leaning:?}");
    assert_eq!(leaning[1], 127, "expected no lean on an axis that does not slope");
  }

  #[test]
  fn the_kernel_reads_across_the_edge_so_a_tiling_texture_has_no_seam() {
    // The two columns either side of the wrap see the same step as any other pair, so the seam is not a ridge.
    let normals: RgbaImage = BumpNormalMap::derive(&plane(8, 1, |x, _| (x * 16) as u8), 0.05);

    assert_eq!(
      normals.get_pixel(0, 0).0,
      normals.get_pixel(7, 0).0,
      "the two edges of a wrap see a mirrored step and should answer alike"
    );
  }

  #[test]
  fn a_taller_virtual_height_leans_further() {
    // Virtual height is the only thing scaling the slope, so the same picture bends more at a larger one.
    let gentle: RgbaImage = BumpNormalMap::derive(&plane(8, 1, |x, _| (x * 4) as u8), 0.02);
    let steep: RgbaImage = BumpNormalMap::derive(&plane(8, 1, |x, _| (x * 4) as u8), 0.2);

    assert!(
      steep.get_pixel(4, 0).0[0] < gentle.get_pixel(4, 0).0[0],
      "expected the taller height to lean further"
    );
  }
}
