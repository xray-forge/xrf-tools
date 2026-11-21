use std::sync::Arc;

use xrf_error::XrfResult;

use crate::vorbis::vorbis_decoder::VorbisDecoder;
use crate::wave::wave_decoder::WaveDecoder;

/// A sound's packets, each with the frame it starts on, whichever container holds it: the Ogg/Vorbis the game ships or
/// the WAV the SDK keeps sources in. What a player reads a sound from.
pub enum SoundSource {
  Vorbis(VorbisDecoder<Arc<[u8]>>),
  Wave(WaveDecoder),
}

impl SoundSource {
  /// Opens the sound in `bytes` by its container: Ogg as [`VorbisDecoder::is_ogg`] tells it, anything else as WAV.
  ///
  /// # Errors
  ///
  /// Returns an error when the source the bytes name cannot open them.
  pub fn open(bytes: Arc<[u8]>) -> XrfResult<Self> {
    if VorbisDecoder::<Arc<[u8]>>::is_ogg(&bytes) {
      Ok(Self::Vorbis(VorbisDecoder::open(bytes)?))
    } else {
      Ok(Self::Wave(WaveDecoder::open(bytes)?))
    }
  }

  pub fn get_sample_rate(&self) -> u32 {
    match self {
      Self::Vorbis(decoder) => decoder.get_sample_rate(),
      Self::Wave(source) => source.get_sample_rate(),
    }
  }

  pub fn get_channels(&self) -> u16 {
    match self {
      Self::Vorbis(decoder) => decoder.get_channels(),
      Self::Wave(source) => source.get_channels(),
    }
  }

  /// Frames each channel plays: `ov_pcm_total` for Vorbis, as the engine counts it, the track's stated length for WAV.
  pub fn get_samples_count(&self) -> u64 {
    match self {
      Self::Vorbis(decoder) => decoder.get_samples_count(),
      Self::Wave(source) => source.get_samples_count(),
    }
  }

  /// Moves to a packet starting at or before `frame`.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot seek there.
  pub fn seek(&mut self, frame: u64) -> XrfResult<()> {
    match self {
      Self::Vorbis(decoder) => decoder.seek(frame),
      Self::Wave(source) => source.seek(frame),
    }
  }

  /// Decodes the next packet into `samples`, interleaved, and answers the frame it starts on; none at the end.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot be read or decoded further.
  pub fn decode_next(&mut self, samples: &mut Vec<f32>) -> XrfResult<Option<u64>> {
    match self {
      Self::Vorbis(decoder) => decoder.decode_next(samples),
      Self::Wave(source) => source.decode_next(samples),
    }
  }
}
