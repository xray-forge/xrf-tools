use image::RgbaImage;

use crate::mip::dds_mip_filter::DdsMipFilter;

/// Channels a texture carries, which the passes below walk one at a time.
const CHANNELS: usize = 4;

/// One reduction of an image to a smaller size, with the windows each destination pixel reads prepared up front.
///
/// Separable: the windows of the two axes are built once and applied in turn, so the cost is proportional to a
/// window's width rather than its area. Preparing them is worth its own step because every row of a pass reads the
/// same horizontal windows, and every column the same vertical ones.
pub(crate) struct DdsMipResampler {
  width: u32,
  height: u32,
  horizontal: Vec<DdsMipTaps>,
  vertical: Vec<DdsMipTaps>,
}

impl DdsMipResampler {
  /// Prepares a reduction from `source` to `destination`, both as `(width, height)`.
  pub(crate) fn new(source: (u32, u32), destination: (u32, u32), filter: DdsMipFilter) -> Self {
    Self {
      width: destination.0,
      height: destination.1,
      horizontal: DdsMipTaps::along_axis(source.0, destination.0, filter),
      vertical: DdsMipTaps::along_axis(source.1, destination.1, filter),
    }
  }

  /// Applies the reduction, one axis after the other.
  ///
  /// The horizontal pass answers full-height rows that the vertical pass then reduces, and they are kept as `f64`
  /// between the two so a value is rounded to a byte once, at the end, rather than at every axis.
  pub(crate) fn resample(&self, source: &RgbaImage) -> RgbaImage {
    let width: usize = self.width as usize;
    let mut rows: Vec<f64> = vec![0.0; width * source.height() as usize * CHANNELS];

    for y in 0..source.height() as usize {
      for (x, taps) in self.horizontal.iter().enumerate() {
        for channel in 0..CHANNELS {
          rows[(y * width + x) * CHANNELS + channel] =
            taps.weigh(|sample| f64::from(source.get_pixel(sample as u32, y as u32).0[channel]));
        }
      }
    }

    let mut destination: RgbaImage = RgbaImage::new(self.width, self.height);

    for (y, taps) in self.vertical.iter().enumerate() {
      for x in 0..width {
        for channel in 0..CHANNELS {
          destination.get_pixel_mut(x as u32, y as u32).0[channel] =
            to_byte(taps.weigh(|sample| rows[(sample * width + x) * CHANNELS + channel]));
        }
      }
    }

    destination
  }
}

/// One destination pixel's window over a source axis.
struct DdsMipTaps {
  /// Index of the first source sample the window covers.
  start: usize,
  /// Weight of each covered sample, already normalized to sum to one.
  weights: Vec<f64>,
}

impl DdsMipTaps {
  /// The window each destination sample of one axis reads, in source samples.
  ///
  /// The kernel is widened by the reduction factor, which is what makes a window cover everything the destination
  /// pixel stands for rather than a fixed few neighbours: a box kernel halving an axis reads two samples, and the same
  /// kernel reducing by eight reads eight.
  fn along_axis(source_len: u32, destination_len: u32, filter: DdsMipFilter) -> Vec<Self> {
    let scale: f64 = f64::from(source_len) / f64::from(destination_len);
    // Only a reduction widens the kernel. An enlargement reads the kernel at its own width, which is what keeps a
    // reconstruction filter reconstructing rather than blurring.
    let kernel_scale: f64 = scale.max(1.0);
    let support: f64 = filter.support() * kernel_scale;

    (0..destination_len)
      .map(|index| {
        let center: f64 = (f64::from(index) + 0.5) * scale;

        // A kernel with no width names one sample rather than a window, which is what `Point` is.
        if support <= 0.0 {
          return Self::at(center, source_len);
        }

        let first: usize = (center - support - 0.5).ceil().max(0.0) as usize;
        let last: usize = ((center + support - 0.5).floor().max(0.0) as usize).min(source_len as usize - 1);
        let weights: Vec<f64> = (first..=last)
          .map(|sample| filter.weight((sample as f64 + 0.5 - center) / kernel_scale))
          .collect();
        let total: f64 = weights.iter().sum();

        // A window that weighs nothing anywhere - which a kernel with negative lobes can produce - would answer
        // black, so it falls back to the sample at its centre.
        if total == 0.0 {
          return Self::at(center, source_len);
        }

        Self {
          start: first,
          weights: weights.into_iter().map(|weight| weight / total).collect(),
        }
      })
      .collect()
  }

  /// A window of the single sample under `center`, for the cases that name a sample rather than a neighbourhood.
  fn at(center: f64, source_len: u32) -> Self {
    Self {
      start: (center.floor().max(0.0) as usize).min(source_len as usize - 1),
      weights: vec![1.0],
    }
  }

  /// The weighted sum of the window, reading each covered sample through `sample_at`.
  fn weigh(&self, sample_at: impl Fn(usize) -> f64) -> f64 {
    self
      .weights
      .iter()
      .enumerate()
      .map(|(offset, weight)| weight * sample_at(self.start + offset))
      .sum()
  }
}

/// Rounds a resampled value back into a byte, clamping the overshoot a kernel with negative lobes produces.
fn to_byte(value: f64) -> u8 {
  value.round().clamp(0.0, f64::from(u8::MAX)) as u8
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};

  use super::{DdsMipResampler, to_byte};
  use crate::mip::dds_mip_filter::DdsMipFilter;

  /// A row of grey steps, opaque, so every figure below is one channel of arithmetic.
  fn steps(values: &[u8]) -> RgbaImage {
    RgbaImage::from_fn(values.len() as u32, 1, |x, _| {
      let value: u8 = values[x as usize];

      Rgba([value, value, value, u8::MAX])
    })
  }

  /// The red channel of a reduction, which for [`steps`] is the whole picture.
  fn reduce(values: &[u8], width: u32, filter: DdsMipFilter) -> Vec<u8> {
    let source: RgbaImage = steps(values);

    DdsMipResampler::new((source.width(), 1), (width, 1), filter)
      .resample(&source)
      .pixels()
      .map(|pixel| pixel.0[0])
      .collect()
  }

  #[test]
  fn the_box_kernel_averages_the_pixels_a_destination_pixel_covers() {
    // Widened by the reduction, so halving reads two source pixels and reducing by four reads four.
    assert_eq!(reduce(&[0, 10, 20, 30], 2, DdsMipFilter::Box), vec![5, 25]);
    assert_eq!(reduce(&[0, 10, 20, 30], 1, DdsMipFilter::Box), vec![15]);
  }

  #[test]
  fn the_point_kernel_takes_one_source_pixel() {
    // No window at all, so each destination pixel is whichever source pixel its centre lands on.
    assert_eq!(reduce(&[0, 10, 20, 30], 2, DdsMipFilter::Point), vec![10, 30]);
    assert_eq!(reduce(&[0, 10, 20, 30], 1, DdsMipFilter::Point), vec![20]);
  }

  #[test]
  fn rounds_and_clamps_a_resampled_value_into_a_byte() {
    // Where a kernel with negative lobes lands: Catrom undershoots at a hard edge and overshoots after it, and both
    // are outside a byte before this. Halves round away from zero, which is what decides the middle grey.
    assert_eq!(to_byte(-12.5), 0);
    assert_eq!(to_byte(300.0), u8::MAX);
    assert_eq!(to_byte(127.5), 128);
    assert_eq!(to_byte(126.5), 127);
  }
}
