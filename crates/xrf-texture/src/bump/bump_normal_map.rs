use image::RgbaImage;

use crate::bump::bump_vector::pack_vector;

/// What the SDK multiplies the virtual height by before differentiating (`NormalMapGen.cpp`).
///
/// Two other factors are in that source, commented out beside this one, against the 3x3 and a gentler 4x kernel. This
/// is the pairing that shipped.
const HEIGHT_SCALE: f32 = 200.0;

/// The normals a height plane implies, as the bytes a bump texture stores.
///
/// `ConvertAlphaToNormalMap_4x`: the slope along each axis is the difference between the two neighbours either side,
/// halved, which is the narrowest kernel that is still centred. Taking neighbours from the opposite edge rather than
/// clamping is what makes a tiling texture tile without a seam, and the SDK passes `wrap` for exactly that.
///
/// The cross product of the two slope vectors `(1, 0, du)` and `(0, 1, dv)` is `(-du, -dv, 1)`, so a flat region
/// answers straight up and a steep one leans against the slope.
pub(crate) fn derive_normal_map(height: &GreyPlane, virtual_height: f32) -> RgbaImage {
  let scale: f32 = virtual_height * HEIGHT_SCALE;
  let mut normals: RgbaImage = RgbaImage::new(height.width, height.height);

  for y in 0..height.height {
    for x in 0..height.width {
      let du: f32 = 0.5 * (height.wrapped(x as i64 + 1, y as i64) - height.wrapped(x as i64 - 1, y as i64)) * scale;
      // The v kernel reads the row below as positive, which is what puts the normal in the handedness the packed
      // pair is authored against.
      let dv: f32 = 0.5 * (height.wrapped(x as i64, y as i64 + 1) - height.wrapped(x as i64, y as i64 - 1)) * scale;
      let packed: [u8; 3] = pack_vector([-du, -dv, 1.0]);

      // Alpha is left at nought here; the gloss reversal fills it with the normal's own red a step later.
      normals.get_pixel_mut(x, y).0 = [packed[0], packed[1], packed[2], 0];
    }
  }

  normals
}

/// One channel of an image, as the values a kernel reads.
///
/// Held apart from [`RgbaImage`] because the derivation reads a single plane many times per pixel and reads it off the
/// edge, and because the plane the SDK differentiates is not a channel of the source at all: it is the average of its
/// three colour channels.
pub(crate) struct GreyPlane {
  pub(crate) width: u32,
  pub(crate) height: u32,
  values: Vec<f32>,
}

impl GreyPlane {
  /// The plane the SDK differentiates, `AverageRGBToAlpha` (`NVI_Image.cpp`): the mean of red, green and blue.
  ///
  /// The mean rather than a luminance weighting, so a height map authored in any one channel and copied across the
  /// other two arrives unchanged.
  pub(crate) fn from_rgb_average(image: &RgbaImage) -> Self {
    Self {
      width: image.width(),
      height: image.height(),
      values: image
        .pixels()
        .map(|pixel| (f32::from(pixel.0[0]) + f32::from(pixel.0[1]) + f32::from(pixel.0[2])) / 3.0 / 255.0)
        .collect(),
    }
  }

  /// One channel of an image, taken as it stands.
  pub(crate) fn from_channel(image: &RgbaImage, channel: usize) -> Self {
    Self {
      width: image.width(),
      height: image.height(),
      values: image
        .pixels()
        .map(|pixel| f32::from(pixel.0[channel]) / 255.0)
        .collect(),
    }
  }

  /// The value at a coordinate, with anything outside the image taken from the opposite edge.
  pub(crate) fn wrapped(&self, x: i64, y: i64) -> f32 {
    let x: usize = x.rem_euclid(i64::from(self.width)) as usize;
    let y: usize = y.rem_euclid(i64::from(self.height)) as usize;

    self.values[y * self.width as usize + x]
  }

  /// The mean of every value, in the `0..=1` range the plane holds.
  ///
  /// Accumulated in `f64` rather than in the plane's own `f32`. A texture is millions of texels, and once an `f32`
  /// running total passes a million its steps are coarser than the values still being added to it: summing 0.6 two
  /// million times that way answers 0.613. That figure is what the gloss verdict is measured against, so the drift
  /// would decide whether a surface is called too dark by how large it is.
  pub(crate) fn average(&self) -> f32 {
    (self.values.iter().map(|value| f64::from(*value)).sum::<f64>() / self.values.len() as f64) as f32
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};

  use super::{GreyPlane, derive_normal_map};

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
    let normals: RgbaImage = derive_normal_map(&plane(4, 4, |_, _| 128), 0.05);

    assert!(
      normals.pixels().all(|pixel| pixel.0[..3] == [127, 127, 255]),
      "expected every normal to point up, got {:?}",
      normals.get_pixel(0, 0)
    );
  }

  #[test]
  fn a_slope_leans_against_its_own_rise() {
    // Height climbing with x means `du` is positive, and the normal's x is `-du`, so it leans back below the middle.
    let normals: RgbaImage = derive_normal_map(&plane(8, 1, |x, _| (x * 16) as u8), 0.05);
    let leaning = normals.get_pixel(4, 0).0;

    assert!(leaning[0] < 127, "expected a leftward lean, got {leaning:?}");
    assert_eq!(leaning[1], 127, "expected no lean on an axis that does not slope");
  }

  #[test]
  fn the_kernel_reads_across_the_edge_so_a_tiling_texture_has_no_seam() {
    // The two columns either side of the wrap see the same step as any other pair, so the seam is not a ridge.
    let normals: RgbaImage = derive_normal_map(&plane(8, 1, |x, _| (x * 16) as u8), 0.05);

    assert_eq!(
      normals.get_pixel(0, 0).0,
      normals.get_pixel(7, 0).0,
      "the two edges of a wrap see a mirrored step and should answer alike"
    );
  }

  #[test]
  fn a_taller_virtual_height_leans_further() {
    // Virtual height is the only thing scaling the slope, so the same picture bends more at a larger one.
    let gentle: RgbaImage = derive_normal_map(&plane(8, 1, |x, _| (x * 4) as u8), 0.02);
    let steep: RgbaImage = derive_normal_map(&plane(8, 1, |x, _| (x * 4) as u8), 0.2);

    assert!(
      steep.get_pixel(4, 0).0[0] < gentle.get_pixel(4, 0).0[0],
      "expected the taller height to lean further"
    );
  }

  #[test]
  fn the_average_does_not_drift_over_a_texture_sized_plane() {
    // The failing size: two million texels of one value. Accumulated in `f32` this answers 0.613 rather than 0.6,
    // because the running total outgrows the steps being added to it, and the gloss verdict reads that number.
    let plane: GreyPlane = plane(1024, 2048, |_, _| 153);

    assert!(
      (plane.average() - 153.0 / 255.0).abs() < 1e-5,
      "expected the mean to stay put, got {}",
      plane.average()
    );
  }

  #[test]
  fn a_plane_reads_the_average_of_the_colour_channels() {
    let image: RgbaImage = RgbaImage::from_pixel(2, 2, Rgba([30, 60, 90, 255]));

    assert!((GreyPlane::from_rgb_average(&image).average() - 60.0 / 255.0).abs() < 1e-6);
    assert!((GreyPlane::from_channel(&image, 2).average() - 90.0 / 255.0).abs() < 1e-6);
  }
}
