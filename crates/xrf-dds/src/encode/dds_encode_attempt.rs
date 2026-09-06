use std::time::{Duration, Instant};

use image::RgbaImage;
use image_dds::Quality;
use xrf_error::{XrfError, XrfResult};

use crate::encode::dds_encode_candidate::DdsEncodeCandidate;
use crate::encode::dds_encoding::DdsEncoding;
use crate::encode::dds_image_difference::DdsImageDifference;
use crate::file::dds_file::DdsFile;
use crate::mip::dds_mip_chain::DdsMipChain;

/// One candidate format tried against a texture, with what it cost and what it lost.
///
/// The encoded file travels with the figures rather than being thrown away and rebuilt when somebody accepts it: a
/// comparison of five candidates is five encodes already done, and re-running the chosen one would spend the time
/// again to produce the same bytes.
pub struct DdsEncodeAttempt {
  pub candidate: DdsEncodeCandidate,
  /// The encoded texture, ready to be written or uploaded.
  pub file: DdsFile,
  /// Bytes the file occupies on disk, header included.
  pub file_bytes: u64,
  /// Bytes the texture occupies once uploaded, which is the payload with its whole mip chain and no header.
  pub gpu_bytes: u64,
  pub encode_duration: Duration,
  /// How far the encoded base level sits from the pixels it was given.
  ///
  /// Additional loss relative to what was passed in. For a texture already stored in a lossy format, the reference is
  /// itself lossy, so this is what a re-encode costs on top - never distance from an original nobody has.
  pub difference: DdsImageDifference,
}

impl DdsEncodeAttempt {
  /// Encodes a chain as one candidate and measures the result against the chain's own base level.
  ///
  /// # Errors
  ///
  /// Returns an error when the chain holds no levels, when the encoder refuses the pixels, or when the encoded
  /// texture cannot be decoded back to compare it.
  pub fn measure(chain: &DdsMipChain, candidate: DdsEncodeCandidate, quality: Quality) -> XrfResult<Self> {
    let started: Instant = Instant::now();
    let file: DdsFile = DdsEncoding::new(candidate.to_image_format(), quality).encode(chain)?;
    let encode_duration: Duration = started.elapsed();

    let metadata = file.metadata();
    let reference: &RgbaImage = chain
      .levels()
      .first()
      .ok_or_else(|| XrfError::new_texture_processing_error("Cannot measure a chain holding no levels"))?;

    Ok(Self {
      candidate,
      file_bytes: metadata.file_size,
      // Taken from the payload the encoder produced rather than computed from block arithmetic, so a format whose
      // blocks this crate has the size of wrong cannot report a size the file does not have.
      gpu_bytes: metadata.file_size.saturating_sub(metadata.metadata_size),
      encode_duration,
      difference: DdsImageDifference::between(reference, &file.decode_rgba(0)?)?,
      file,
    })
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use image_dds::Quality;
  use xrf_error::XrfResult;

  use super::DdsEncodeAttempt;
  use crate::encode::dds_encode_candidate::DdsEncodeCandidate;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  /// A picture with detail in every channel, so a lossy candidate has something to lose.
  fn noisy(width: u32, height: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
      Rgba([
        (x * 7 + y * 13) as u8,
        (x * 31 + y * 3) as u8,
        (x * 17 + y * 29) as u8,
        u8::MAX,
      ])
    })
  }

  #[test]
  fn prices_each_candidate_against_the_pixels_it_was_given() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&noisy(32, 32), DdsMipmaps::Filtered(DdsMipFilter::Kaiser))?;

    let uncompressed: DdsEncodeAttempt = DdsEncodeAttempt::measure(&chain, DdsEncodeCandidate::Rgba8, Quality::Fast)?;
    let compressed: DdsEncodeAttempt = DdsEncodeAttempt::measure(&chain, DdsEncodeCandidate::Bc1, Quality::Fast)?;

    // Uncompressed is the base and every mip below it at four bytes a pixel, which is the sum of the chain.
    assert_eq!(uncompressed.gpu_bytes, (1024 + 256 + 64 + 16 + 4 + 1) * 4);
    assert_eq!(
      uncompressed.difference.psnr, None,
      "Expect no loss where nothing is compressed"
    );

    // BC1 spends eight bytes on each 4x4 block, and a level smaller than one block still pays for a whole one - so
    // the chain is 64 blocks, then 16, 4, and then a single block three times over, not an eighth of the figure
    // above. That tail is what a size reported by pixel arithmetic gets wrong.
    assert_eq!(compressed.gpu_bytes, (64 + 16 + 4 + 1 + 1 + 1) * 8);
    assert!(
      compressed.difference.psnr.is_some_and(|psnr| psnr > 0.0),
      "Expect a block format to cost something measurable"
    );

    // The header is what the file has and the upload does not.
    assert!(compressed.file_bytes > compressed.gpu_bytes);

    Ok(())
  }
}
