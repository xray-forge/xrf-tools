use std::sync::Arc;
use std::time::Duration;

use xrf_error::XrfResult;

use crate::decode::sound_channels::SoundChannels;
use crate::decode::sound_decoder::SoundDecoder;

/// A sound decoded whole, its samples interleaved at its own rate: what a short sound plays from, shared by every
/// voice playing it.
#[derive(Debug)]
pub struct DecodedSound {
  sample_rate: u32,
  channels: SoundChannels,
  samples: Box<[f32]>,
}

impl DecodedSound {
  /// # Panics
  ///
  /// Panics when `sample_rate` is zero or `samples` does not hold whole frames of `channels`.
  pub fn new(sample_rate: u32, channels: SoundChannels, samples: Vec<f32>) -> Self {
    assert!(sample_rate > 0, "a sound plays at a sample rate above zero");
    assert_eq!(samples.len() % channels.get_count(), 0, "a sound holds whole frames");

    Self {
      sample_rate,
      channels,
      samples: samples.into_boxed_slice(),
    }
  }

  /// Decodes every frame of the sound in `bytes`.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound [`SoundDecoder`] opens or its audio fails to decode.
  pub fn decode(bytes: Arc<[u8]>) -> XrfResult<Self> {
    let mut decoder: SoundDecoder = SoundDecoder::open(bytes)?;
    let channels: SoundChannels = decoder.get_channels();
    let mut samples: Vec<f32> = Vec::with_capacity(
      usize::try_from(decoder.get_frames_count())
        .unwrap_or(0)
        .saturating_mul(channels.get_count()),
    );
    let mut packet: Vec<f32> = Vec::new();

    while decoder.decode_next(&mut packet)? {
      samples.extend_from_slice(&packet);
    }

    Ok(Self::new(decoder.get_sample_rate(), channels, samples))
  }

  pub fn get_sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn get_channels(&self) -> SoundChannels {
    self.channels
  }

  /// Interleaved samples, [`Self::get_channels`] to a frame.
  pub fn get_samples(&self) -> &[f32] {
    &self.samples
  }

  pub fn get_frames_count(&self) -> usize {
    self.samples.len() / self.channels.get_count()
  }

  pub fn get_duration(&self) -> Duration {
    Duration::from_secs_f64(self.get_frames_count() as f64 / f64::from(self.sample_rate))
  }
}
