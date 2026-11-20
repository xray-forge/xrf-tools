use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_sound::VorbisDecoder;

use crate::decode::source::wave_source::WaveSource;

/// Where a [`crate::SoundDecoder`] takes a sound's packets from, each with the frame it starts on: `xrf-sound`'s
/// Ogg/Vorbis decoder for the game's sounds, symphonia's WAV reader for the SDK's sources.
pub(crate) enum SoundSource {
  Vorbis(VorbisDecoder<Arc<[u8]>>),
  Wave(WaveSource),
}

impl SoundSource {
  /// Opens the sound in `bytes` by its container: Ogg as `xrf-sound` tells it, anything else as WAV.
  ///
  /// # Errors
  ///
  /// Returns an error when the source the bytes name cannot open them.
  pub(crate) fn open(bytes: Arc<[u8]>) -> XrfResult<Self> {
    if VorbisDecoder::<Arc<[u8]>>::is_ogg(&bytes) {
      Ok(Self::Vorbis(VorbisDecoder::open(bytes)?))
    } else {
      Ok(Self::Wave(WaveSource::open(bytes)?))
    }
  }

  pub(crate) fn get_sample_rate(&self) -> u32 {
    match self {
      Self::Vorbis(decoder) => decoder.get_sample_rate(),
      Self::Wave(source) => source.get_sample_rate(),
    }
  }

  pub(crate) fn get_channels_count(&self) -> usize {
    match self {
      Self::Vorbis(decoder) => usize::from(decoder.get_channels()),
      Self::Wave(source) => source.get_channels_count(),
    }
  }

  /// Frames each channel plays: `ov_pcm_total` for Vorbis, as the engine counts it, the track's stated length for WAV.
  pub(crate) fn get_frames_count(&self) -> u64 {
    match self {
      Self::Vorbis(decoder) => decoder.get_samples_count(),
      Self::Wave(source) => source.get_frames_count(),
    }
  }

  /// Moves to a packet starting at or before `frame`.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot seek there.
  pub(crate) fn seek(&mut self, frame: u64) -> XrfResult<()> {
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
  pub(crate) fn decode_next(&mut self, samples: &mut Vec<f32>) -> XrfResult<Option<u64>> {
    match self {
      Self::Vorbis(decoder) => decoder.decode_next(samples),
      Self::Wave(source) => source.decode_next(samples),
    }
  }
}
