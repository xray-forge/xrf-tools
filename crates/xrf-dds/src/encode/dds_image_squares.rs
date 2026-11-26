use xrf_error::{XrfError, XrfResult};

use crate::encode::dds_image_difference::DdsImageDifference;

/// The squared differences of two pictures' channels summed, over as many pixels as were compared: what
/// [`DdsImageDifference`] is figured from, kept apart so pieces of one comparison can be summed as they are decoded.
///
/// Sums of 8-bit differences squared are whole numbers, so they add up in any order to the same total.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DdsImageSquares {
  squares: [u64; DdsImageDifference::CHANNELS],
  pixels: u64,
}

impl DdsImageSquares {
  /// The squares of two runs of RGBA pixels of one length, pixel for pixel.
  pub(crate) fn of(reference: &[u8], candidate: &[u8]) -> Self {
    let mut squares: [u64; DdsImageDifference::CHANNELS] = [0; DdsImageDifference::CHANNELS];
    let (reference, _) = reference.as_chunks::<{ DdsImageDifference::CHANNELS }>();
    let (candidate, _) = candidate.as_chunks::<{ DdsImageDifference::CHANNELS }>();

    for (left, right) in reference.iter().zip(candidate) {
      for (total, (left, right)) in squares.iter_mut().zip(left.iter().zip(right)) {
        let difference: u64 = u64::from(left.abs_diff(*right));

        *total += difference * difference;
      }
    }

    Self {
      squares,
      pixels: reference.len().min(candidate.len()) as u64,
    }
  }

  /// Both comparisons' squares and pixels together.
  pub(crate) fn add(self, other: Self) -> Self {
    let mut squares: [u64; DdsImageDifference::CHANNELS] = self.squares;

    for (total, other) in squares.iter_mut().zip(other.squares) {
      *total += other;
    }

    Self {
      squares,
      pixels: self.pixels + other.pixels,
    }
  }

  /// The difference these squares stand for.
  ///
  /// # Errors
  ///
  /// Returns an error when no pixel was compared.
  pub(crate) fn finish(self) -> XrfResult<DdsImageDifference> {
    if self.pixels == 0 {
      return Err(XrfError::new_texture_processing_error(
        "Cannot compare images with no pixels",
      ));
    }

    // Exact in `f64`: a 4096² picture's total stays far below 2^53.
    let pixels: f64 = self.pixels as f64;
    let channel_rmse: [f64; DdsImageDifference::CHANNELS] = self.squares.map(|total| (total as f64 / pixels).sqrt());
    let rmse: f64 = (self.squares.iter().sum::<u64>() as f64 / (pixels * DdsImageDifference::CHANNELS as f64)).sqrt();

    Ok(DdsImageDifference {
      channel_rmse,
      rmse,
      // Undefined at zero error, which is the one case a reader has to be able to tell from a very large number.
      psnr: (rmse > 0.0).then(|| 20.0 * (f64::from(u8::MAX) / rmse).log10()),
    })
  }
}
