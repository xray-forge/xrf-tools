use std::ops::Range;

use image::RgbaImage;
use rayon::prelude::*;

use crate::mip::dds_mip_filter::DdsMipFilter;

/// Channels a texture carries, which the passes below walk one at a time.
const CHANNELS: usize = 4;

/// Bytes the horizontally reduced rows of one slab may take. A wide kernel reducing a large base would otherwise hold
/// every source row reduced at once, `destination width x source height` values: 268 MB for the first level of a 4096
/// square base.
const SLAB_BYTES: usize = 32 << 20;

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
  /// The horizontal pass answers rows that the vertical pass then reduces, and they are kept as `f64` between the two
  /// so a value is rounded to a byte once, at the end, rather than at every axis. Destination rows are taken in slabs
  /// whose source rows fit [`SLAB_BYTES`], a row of either pass at a time on every worker. Each value is summed over
  /// its window in the window's order whichever worker takes it, so the result does not depend on the split.
  pub(crate) fn resample(&self, source: &RgbaImage) -> RgbaImage {
    let row_length: usize = self.width as usize * CHANNELS;
    let source_row_length: usize = source.width() as usize * CHANNELS;
    let slab_rows: usize = (SLAB_BYTES / (row_length * size_of::<f64>())).max(1);
    let mut destination: RgbaImage = RgbaImage::new(self.width, self.height);
    let bytes: &mut [u8] = &mut destination;
    let mut reduced: Vec<f64> = Vec::new();
    let mut first: usize = 0;

    while first < self.vertical.len() {
      let slab: DdsMipSlab = self.next_slab(first, slab_rows);

      reduced.resize(slab.sources.len() * row_length, 0.0);
      reduced
        .par_chunks_mut(row_length)
        .zip(slab.sources.clone())
        .for_each(|(values, row)| {
          self.reduce_row(&source.as_raw()[row * source_row_length..][..source_row_length], values);
        });
      bytes[slab.rows.start * row_length..slab.rows.end * row_length]
        .par_chunks_mut(row_length)
        .zip(&self.vertical[slab.rows.clone()])
        .for_each_init(
          || vec![0.0; row_length],
          |sums, (row, taps)| Self::reduce_column(taps, &reduced, slab.sources.start, sums, row),
        );

      first = slab.rows.end;
    }

    destination
  }

  /// The slab starting at destination row `first`: as many rows as fit in `slab_rows` source rows, and at least one
  /// whatever its window spans.
  fn next_slab(&self, first: usize, slab_rows: usize) -> DdsMipSlab {
    let mut sources: Range<usize> = self.vertical[first].span();
    let mut last: usize = first + 1;

    while let Some(taps) = self.vertical.get(last) {
      let span: Range<usize> = taps.span();
      let joined: Range<usize> = sources.start.min(span.start)..sources.end.max(span.end);

      if joined.len() > slab_rows {
        break;
      }

      sources = joined;
      last += 1;
    }

    DdsMipSlab {
      rows: first..last,
      sources,
    }
  }

  /// One source row reduced along the horizontal axis, a value per destination pixel and channel.
  fn reduce_row(&self, source: &[u8], reduced: &mut [f64]) {
    for (taps, values) in self.horizontal.iter().zip(reduced.as_chunks_mut::<CHANNELS>().0) {
      let mut sums: [f64; CHANNELS] = [-0.0; CHANNELS];

      for (weight, pixel) in taps
        .weights
        .iter()
        .zip(source[taps.start * CHANNELS..].as_chunks::<CHANNELS>().0)
      {
        for (sum, channel) in sums.iter_mut().zip(pixel) {
          *sum += weight * f64::from(*channel);
        }
      }

      *values = sums;
    }
  }

  /// One destination row: the reduced rows of `slab`, the first of which is source row `slab_start`, weighed down the
  /// vertical window and rounded into bytes.
  fn reduce_column(taps: &DdsMipTaps, slab: &[f64], slab_start: usize, sums: &mut [f64], row: &mut [u8]) {
    let row_length: usize = row.len();

    sums.fill(-0.0);

    for (offset, weight) in taps.weights.iter().enumerate() {
      let reduced: &[f64] = &slab[(taps.start + offset - slab_start) * row_length..][..row_length];

      for (sum, value) in sums.iter_mut().zip(reduced) {
        *sum += weight * value;
      }
    }

    for (byte, sum) in row.iter_mut().zip(sums.iter()) {
      *byte = Self::to_byte(*sum);
    }
  }

  /// Rounds a resampled value back into a byte, clamping the overshoot a kernel with negative lobes produces.
  fn to_byte(value: f64) -> u8 {
    value.round().clamp(0.0, f64::from(u8::MAX)) as u8
  }
}

/// Destination rows reduced together, and the source rows their vertical windows read.
#[derive(Debug, PartialEq, Eq)]
struct DdsMipSlab {
  rows: Range<usize>,
  sources: Range<usize>,
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

  /// The source samples the window covers.
  fn span(&self) -> Range<usize> {
    self.start..self.start + self.weights.len()
  }
}

#[cfg(test)]
mod tests {
  use std::ops::Range;

  use image::{Rgba, RgbaImage};

  use super::{DdsMipResampler, DdsMipSlab, DdsMipTaps};
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

  /// The reduction as one sum per pixel and channel over the whole window, read pixel by pixel: what the slabs must
  /// answer byte for byte.
  fn reduced_directly(resampler: &DdsMipResampler, source: &RgbaImage) -> RgbaImage {
    let weigh = |taps: &DdsMipTaps, sample_at: &dyn Fn(usize) -> f64| -> f64 {
      taps
        .weights
        .iter()
        .enumerate()
        .map(|(offset, weight)| weight * sample_at(taps.start + offset))
        .sum()
    };

    RgbaImage::from_fn(resampler.width, resampler.height, |x, y| {
      let vertical: &DdsMipTaps = &resampler.vertical[y as usize];
      let horizontal: &DdsMipTaps = &resampler.horizontal[x as usize];

      Rgba(std::array::from_fn(|channel| {
        DdsMipResampler::to_byte(weigh(vertical, &|row| {
          weigh(horizontal, &|column| {
            f64::from(source.get_pixel(column as u32, row as u32).0[channel])
          })
        }))
      }))
    })
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
  fn answers_every_pixel_as_one_sum_over_its_window_whatever_the_slabs() {
    // An oblong of noise in every channel, reduced by kernels with and without negative lobes, at sizes whose slabs
    // hold a few rows each: the split must not show in a single byte.
    let byte = |value: u32| u8::try_from(value % 256).unwrap_or_default();
    let source: RgbaImage = RgbaImage::from_fn(67, 45, |x, y| {
      Rgba([
        byte(x * 37 + y * 11),
        byte(x * 5 + y * 71),
        byte(x * y + 13),
        byte((x * 3) ^ (y * 7)),
      ])
    });

    for filter in [
      DdsMipFilter::Box,
      DdsMipFilter::Kaiser,
      DdsMipFilter::Catrom,
      DdsMipFilter::Point,
    ] {
      for destination in [(33, 22), (8, 5), (1, 1)] {
        let resampler: DdsMipResampler = DdsMipResampler::new(source.dimensions(), destination, filter);

        assert_eq!(
          resampler.resample(&source),
          reduced_directly(&resampler, &source),
          "{} to {destination:?}",
          filter.label()
        );
      }
    }
  }

  #[test]
  fn a_slab_holds_what_fits_and_at_least_one_row() {
    let resampler: DdsMipResampler = DdsMipResampler::new((64, 64), (32, 32), DdsMipFilter::Box);
    let slab = |rows: Range<usize>, sources: Range<usize>| DdsMipSlab { rows, sources };

    // A halving box reads two source rows a destination row, so eight source rows hold four destination rows.
    assert_eq!(resampler.next_slab(0, 8), slab(0..4, 0..8));
    assert_eq!(resampler.next_slab(4, 8), slab(4..8, 8..16));
    // A window wider than the slab is still taken, alone.
    assert_eq!(resampler.next_slab(0, 1), slab(0..1, 0..2));
  }

  #[test]
  fn rounds_and_clamps_a_resampled_value_into_a_byte() {
    // Where a kernel with negative lobes lands: Catrom undershoots at a hard edge and overshoots after it, and both
    // are outside a byte before this. Halves round away from zero, which is what decides the middle grey.
    assert_eq!(DdsMipResampler::to_byte(-12.5), 0);
    assert_eq!(DdsMipResampler::to_byte(300.0), u8::MAX);
    assert_eq!(DdsMipResampler::to_byte(127.5), 128);
    assert_eq!(DdsMipResampler::to_byte(126.5), 127);
  }
}
