use std::sync::Arc;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::decode::sound_channels::SoundChannels;
use crate::decode::sound_format::SoundFormat;
use crate::decode::source::sound_source::SoundSource;

/// A sound's audio decoded packet by packet from its bytes: Ogg/Vorbis as the game ships, or WAV as the SDK keeps
/// sources.
pub struct SoundDecoder {
  source: SoundSource,
  sample_rate: u32,
  channels: SoundChannels,
  frames_count: u64,
  /// The first frame the next decoded packet may give: frames before it, left by a seek landing on a packet that
  /// starts earlier, are dropped.
  next_frame: u64,
  /// Set by a seek to the sound's end, which containers refuse as out of range: there is nothing left to decode.
  is_at_end: bool,
}

impl SoundDecoder {
  /// Opens the sound in `bytes` at its first frame.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are neither an Ogg/Vorbis sound libvorbis opens nor a WAV symphonia reads, or
  /// carry a sound without a sample rate, a length, or one or two channels.
  pub fn open(bytes: Arc<[u8]>) -> XrfResult<Self> {
    let source: SoundSource = SoundSource::open(bytes)?;
    let channels_count: usize = source.get_channels_count();
    let channels: SoundChannels = SoundChannels::from_count(channels_count)
      .ok_or_else(|| XrfError::new_read_error(format!("Sound has {channels_count} channels, not one or two")))?;

    Ok(Self {
      sample_rate: source.get_sample_rate(),
      channels,
      frames_count: source.get_frames_count(),
      source,
      next_frame: 0,
      is_at_end: false,
    })
  }

  pub fn get_sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn get_channels(&self) -> SoundChannels {
    self.channels
  }

  /// Frames each channel plays: `ov_pcm_total` for Ogg/Vorbis, as the engine counts it, and the track's stated length
  /// for WAV.
  pub fn get_frames_count(&self) -> u64 {
    self.frames_count
  }

  pub fn get_duration(&self) -> Duration {
    Duration::from_secs_f64(self.frames_count as f64 / f64::from(self.sample_rate))
  }

  /// The sound's rate, channels and length together.
  pub fn get_format(&self) -> SoundFormat {
    SoundFormat {
      sample_rate: self.sample_rate,
      channels: self.channels,
      frames_count: self.frames_count,
    }
  }

  /// Moves to `frame`: the next decoded samples start exactly there, and a frame at or past the sound's end leaves
  /// nothing to decode.
  ///
  /// # Errors
  ///
  /// Returns an error when the container cannot seek to it.
  pub fn seek(&mut self, frame: u64) -> XrfResult<()> {
    self.is_at_end = frame >= self.frames_count;

    if self.is_at_end {
      self.next_frame = self.frames_count;
      return Ok(());
    }

    self.source.seek(frame)?;
    self.next_frame = frame;

    Ok(())
  }

  /// Decodes the next packet of audio into `samples`, interleaved, replacing what it held. Answers `false` once the
  /// sound has no audio left, leaving `samples` empty.
  ///
  /// A packet the codec refuses as corrupt is skipped, as players do; any other failure ends the decode.
  ///
  /// # Errors
  ///
  /// Returns an error when the container cannot be read further or the codec fails for good.
  pub fn decode_next(&mut self, samples: &mut Vec<f32>) -> XrfResult<bool> {
    let channels: usize = self.channels.get_count();

    if self.is_at_end {
      samples.clear();
      return Ok(false);
    }

    while let Some(first_frame) = self.source.decode_next(samples)? {
      let skipped: usize = usize::try_from(self.next_frame.saturating_sub(first_frame)).unwrap_or(usize::MAX);

      samples.drain(..(skipped * channels).min(samples.len()));
      self.next_frame = self.next_frame.max(first_frame) + (samples.len() / channels) as u64;

      if !samples.is_empty() {
        return Ok(true);
      }
    }

    samples.clear();

    Ok(false)
  }
}
