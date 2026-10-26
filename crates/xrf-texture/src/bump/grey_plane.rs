use image::RgbaImage;

/// One channel of an image as the bytes the SDK's bump converter reads, off the edge included.
///
/// Held apart from [`RgbaImage`] because the derivation reads a single plane many times per pixel and reads it off the
/// edge, and because the plane the SDK differentiates is not a channel of the source at all: it is the average of its
/// three colour channels, stored back as a byte.
pub(crate) struct GreyPlane {
  pub(crate) width: u32,
  pub(crate) height: u32,
  bytes: Vec<u8>,
}

impl GreyPlane {
  /// What the convolver multiplies a byte by to read it as a height, `byte_to_float` (`NVI_Convolution.cpp`).
  const BYTE_TO_FLOAT: f32 = 1.0 / 255.0;

  /// The plane the SDK differentiates, `AverageRGBToAlpha` (`NVI_Image.cpp`): the mean of red, green and blue,
  /// truncated into the alpha byte the convolver then reads.
  ///
  /// The mean rather than a luminance weighting, so a height map authored in any one channel and copied across the
  /// other two arrives unchanged.
  pub(crate) fn from_rgb_average(image: &RgbaImage) -> Self {
    Self::from_pixels(image, |[red, green, blue, _]| {
      ((f32::from(red) + f32::from(green) + f32::from(blue)) / 3.0) as u8
    })
  }

  /// One channel of an image, taken as it stands.
  pub(crate) fn from_channel(image: &RgbaImage, channel: usize) -> Self {
    Self::from_pixels(image, |pixel| pixel[channel])
  }

  /// One byte everywhere.
  pub(crate) fn filled(width: u32, height: u32, byte: u8) -> Self {
    Self {
      width,
      height,
      bytes: vec![byte; width as usize * height as usize],
    }
  }

  /// The byte at a coordinate, with anything outside the image taken from the opposite edge.
  pub(crate) fn get_wrapped(&self, x: i64, y: i64) -> u8 {
    let x: usize = x.rem_euclid(i64::from(self.width)) as usize;
    let y: usize = y.rem_euclid(i64::from(self.height)) as usize;

    self.bytes[y * self.width as usize + x]
  }

  /// The byte at a coordinate as the convolver reads it, in `0..=1`.
  pub(crate) fn get_level_wrapped(&self, x: i64, y: i64) -> f32 {
    f32::from(self.get_wrapped(x, y)) * Self::BYTE_TO_FLOAT
  }

  /// Every byte, row by row.
  pub(crate) fn get_bytes(&self) -> &[u8] {
    &self.bytes
  }

  /// The mean of every byte, in the `0..=1` range a gloss is measured in.
  ///
  /// Accumulated in `f64` where the SDK accumulates in `f32`. A texture is millions of texels, and once an `f32`
  /// running total passes a million its steps are coarser than the values still being added to it: summing 0.6 two
  /// million times that way answers 0.613. That figure is what the gloss verdict is measured against, so the drift
  /// would decide whether a surface is called too dark by how large it is.
  pub(crate) fn average(&self) -> f32 {
    (self.bytes.iter().map(|byte| f64::from(*byte)).sum::<f64>() / self.bytes.len() as f64 / 255.0) as f32
  }

  fn from_pixels(image: &RgbaImage, byte: impl Fn([u8; 4]) -> u8) -> Self {
    Self {
      width: image.width(),
      height: image.height(),
      bytes: image.pixels().map(|pixel| byte(pixel.0)).collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};

  use super::GreyPlane;

  #[test]
  fn the_average_does_not_drift_over_a_texture_sized_plane() {
    // The failing size: two million texels of one value. Accumulated in `f32` this answers 0.613 rather than 0.6,
    // because the running total outgrows the steps being added to it, and the gloss verdict reads that number.
    let plane: GreyPlane = GreyPlane::filled(1024, 2048, 153);

    assert!(
      (plane.average() - 153.0 / 255.0).abs() < 1e-5,
      "expected the mean to stay put, got {}",
      plane.average()
    );
  }

  #[test]
  fn a_plane_reads_the_average_of_the_colour_channels_truncated_as_the_sdk_stores_it() {
    // `(10 + 10 + 11) / 3` is 10.33, stored as the byte 10.
    let image: RgbaImage = RgbaImage::from_pixel(2, 2, Rgba([10, 10, 11, 255]));

    assert_eq!(GreyPlane::from_rgb_average(&image).get_wrapped(0, 0), 10);
    assert_eq!(GreyPlane::from_channel(&image, 2).get_wrapped(-1, -1), 11);
  }
}
