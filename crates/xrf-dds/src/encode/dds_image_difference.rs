use image::RgbaImage;
use xrf_error::{XrfError, XrfResult};

/// Channel order the per-channel figures are reported in.
const CHANNELS: usize = 4;

/// How far one image sits from another, measured rather than judged.
///
/// The point of comparing here is a re-encode: the candidate is the same picture through a lossy format, and what a
/// person deciding whether to accept it wants is the size it saves against the error it costs. Both figures are
/// relative to whatever was passed as the reference, which for a texture already stored in a lossy format is itself
/// lossy - so this is additional loss on top of what the file had, never distance from an original nobody has.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DdsImageDifference {
  /// Root mean square error per channel, in the eight-bit units the pixels are stored in.
  pub channel_rmse: [f64; CHANNELS],
  /// Root mean square error over every channel at once.
  pub rmse: f64,
  /// Peak signal-to-noise ratio in decibels, or `None` for two images that do not differ at all.
  ///
  /// Absent rather than infinite, because a report that prints `inf` invites a reader to treat it as a number.
  pub psnr: Option<f64>,
}

impl DdsImageDifference {
  /// Compares two images of the same size, pixel for pixel.
  ///
  /// # Errors
  ///
  /// Returns an error when the images differ in size, which would make every figure meaningless, or when either has
  /// no pixels to compare.
  pub fn between(reference: &RgbaImage, candidate: &RgbaImage) -> XrfResult<Self> {
    if reference.dimensions() != candidate.dimensions() {
      return Err(XrfError::new_texture_processing_error(format!(
        "Cannot compare images of different sizes: {}x{} against {}x{}",
        reference.width(),
        reference.height(),
        candidate.width(),
        candidate.height()
      )));
    }

    let pixels: usize = reference.pixels().len();

    if pixels == 0 {
      return Err(XrfError::new_texture_processing_error(
        "Cannot compare images with no pixels",
      ));
    }

    let mut squares: [f64; CHANNELS] = [0.0; CHANNELS];

    for (left, right) in reference.pixels().zip(candidate.pixels()) {
      for (total, (left, right)) in squares.iter_mut().zip(left.0.iter().zip(right.0.iter())) {
        let difference: f64 = f64::from(*left) - f64::from(*right);

        *total += difference * difference;
      }
    }

    let channel_rmse: [f64; CHANNELS] = squares.map(|total| (total / pixels as f64).sqrt());
    let rmse: f64 = (squares.iter().sum::<f64>() / (pixels * CHANNELS) as f64).sqrt();

    Ok(Self {
      channel_rmse,
      rmse,
      // Undefined at zero error, which is the one case a reader has to be able to tell from a very large number.
      psnr: (rmse > 0.0).then(|| 20.0 * (f64::from(u8::MAX) / rmse).log10()),
    })
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use xrf_error::XrfResult;

  use super::DdsImageDifference;

  fn filled(width: u32, height: u32, pixel: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba(pixel))
  }

  #[test]
  fn two_identical_images_differ_by_nothing_and_have_no_ratio() -> XrfResult {
    // The one case a reader has to be able to tell from a very large number, which is why it is absent rather than
    // infinite.
    let difference: DdsImageDifference =
      DdsImageDifference::between(&filled(4, 4, [10, 20, 30, 40]), &filled(4, 4, [10, 20, 30, 40]))?;

    assert_eq!(difference.channel_rmse, [0.0; 4]);
    assert_eq!(difference.rmse, 0.0);
    assert_eq!(difference.psnr, None);

    Ok(())
  }

  #[test]
  fn reports_the_error_of_each_channel_apart() -> XrfResult {
    // Every pixel differs by the same amount, so each channel's root mean square is that amount exactly: 3 in red,
    // 4 in green, nothing in blue or alpha.
    let difference: DdsImageDifference =
      DdsImageDifference::between(&filled(8, 8, [10, 20, 30, 40]), &filled(8, 8, [13, 16, 30, 40]))?;

    assert_eq!(difference.channel_rmse, [3.0, 4.0, 0.0, 0.0]);

    // Over all four channels at once: `sqrt((9 + 16 + 0 + 0) / 4)`.
    assert!(
      (difference.rmse - 2.5).abs() < 1e-9,
      "unexpected rmse {}",
      difference.rmse
    );

    // `20 log10(255 / 2.5)`, which is the ratio a lossy re-encode is judged by.
    let psnr: f64 = difference.psnr.expect("Expect a ratio where the images differ");

    assert!(
      (psnr - 20.0 * (255.0_f64 / 2.5).log10()).abs() < 1e-9,
      "unexpected psnr {psnr}"
    );

    Ok(())
  }

  #[test]
  fn refuses_a_comparison_that_would_mean_nothing() {
    assert!(DdsImageDifference::between(&filled(4, 4, [0; 4]), &filled(4, 8, [0; 4])).is_err());
    assert!(DdsImageDifference::between(&RgbaImage::new(0, 0), &RgbaImage::new(0, 0)).is_err());
  }
}
