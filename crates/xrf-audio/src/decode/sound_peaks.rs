use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::decode::sound_decoder::SoundDecoder;

/// A sound's waveform: the loudest sample of each equal stretch of its frames, over every channel, and how long it
/// plays.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundPeaks {
  /// Each stretch's largest absolute sample, from 0 to 1.
  pub peaks: Vec<f32>,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
}

impl SoundPeaks {
  /// Peaks of `count` stretches of the sound in `bytes`, decoded as it streams so a long sound is never held whole.
  /// A sound shorter than `count` frames gives one peak a frame.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound [`SoundDecoder`] opens or its audio fails to decode.
  pub fn read(bytes: Arc<[u8]>, count: usize) -> XrfResult<Self> {
    let mut decoder: SoundDecoder = SoundDecoder::open(bytes)?;
    let channels: usize = decoder.get_channels().get_count();
    let frames_count: u64 = decoder.get_frames_count();
    let count: usize = count
      .max(1)
      .min(usize::try_from(frames_count).unwrap_or(usize::MAX).max(1));
    let mut peaks: Vec<f32> = vec![0.0; count];
    let mut packet: Vec<f32> = Vec::new();
    let mut frame: u64 = 0;

    while decoder.decode_next(&mut packet)? {
      for samples in packet.chunks_exact(channels) {
        let stretch: usize = Self::get_stretch(frame, frames_count, count);
        let loudest: f32 = samples
          .iter()
          .fold(0.0, |loudest: f32, sample| loudest.max(sample.abs()));

        peaks[stretch] = peaks[stretch].max(loudest.min(1.0));
        frame += 1;
      }
    }

    Ok(Self {
      peaks,
      duration: decoder.get_duration(),
    })
  }

  fn get_stretch(frame: u64, frames_count: u64, count: usize) -> usize {
    let stretch: u128 = u128::from(frame) * count as u128 / u128::from(frames_count.max(1));

    usize::try_from(stretch).unwrap_or(usize::MAX).min(count - 1)
  }
}
