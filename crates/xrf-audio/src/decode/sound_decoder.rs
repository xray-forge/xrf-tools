use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Timestamp;
use xrf_error::{XrfError, XrfResult};

use crate::decode::sound_channels::SoundChannels;
use crate::decode::sound_format::SoundFormat;

/// A sound's audio track decoded packet by packet from its bytes: Ogg/Vorbis as the game ships, or WAV as the SDK keeps
/// sources.
pub struct SoundDecoder {
  reader: Box<dyn FormatReader>,
  decoder: Box<dyn AudioDecoder>,
  track_id: u32,
  sample_rate: u32,
  channels: SoundChannels,
  frames_count: u64,
  /// The first frame the next decoded packet may give: frames before it, left by a seek landing early, are dropped.
  next_frame: u64,
  /// Set by a seek to the sound's end, which containers refuse as out of range: there is nothing left to decode.
  is_at_end: bool,
}

impl SoundDecoder {
  /// Opens the sound in `bytes` at its first frame.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a container symphonia reads, carry no audio track, or carry one without
  /// a sample rate, a length, or one or two channels.
  pub fn open(bytes: Arc<[u8]>) -> XrfResult<Self> {
    let stream: MediaSourceStream<'static> =
      MediaSourceStream::new(Box::new(Cursor::new(bytes)), MediaSourceStreamOptions::default());
    let reader: Box<dyn FormatReader> = symphonia::default::get_probe()
      .probe(
        &Hint::new(),
        stream,
        FormatOptions::default(),
        MetadataOptions::default(),
      )
      .map_err(|error| Self::new_error("Could not open sound", &error))?;
    let track = reader
      .default_track(TrackType::Audio)
      .ok_or_else(|| XrfError::new_read_error("Sound has no audio track"))?;
    let Some(CodecParameters::Audio(parameters)) = track.codec_params.as_ref() else {
      return Err(XrfError::new_read_error(
        "Sound's track carries no audio codec parameters",
      ));
    };
    let sample_rate: u32 = parameters
      .sample_rate
      .filter(|rate| *rate > 0)
      .ok_or_else(|| XrfError::new_read_error("Sound has no sample rate"))?;
    let channels_count: usize = parameters.channels.as_ref().map_or(0, |channels| channels.count());
    let channels: SoundChannels = SoundChannels::from_count(channels_count)
      .ok_or_else(|| XrfError::new_read_error(format!("Sound has {channels_count} channels, not one or two")))?;
    let frames_count: u64 = track
      .num_frames
      .ok_or_else(|| XrfError::new_read_error("Sound does not state its length"))?;
    let track_id: u32 = track.id;
    let decoder: Box<dyn AudioDecoder> = symphonia::default::get_codecs()
      .make_audio_decoder(parameters, &AudioDecoderOptions::default())
      .map_err(|error| Self::new_error("Could not open sound's decoder", &error))?;

    Ok(Self {
      reader,
      decoder,
      track_id,
      sample_rate,
      channels,
      frames_count,
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

  /// Frames each channel plays, as the track states it.
  pub fn get_frames_count(&self) -> u64 {
    self.frames_count
  }

  pub fn get_duration(&self) -> Duration {
    Duration::from_secs_f64(self.frames_count as f64 / f64::from(self.sample_rate))
  }

  /// The track's rate, channels and length together.
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

    // symphonia sometimes stamps the first packets of the Ogg page a seek lands on hundreds of frames off, and the
    // error carries forward from them; landing a second early and decoding on to the frame lets the next pages'
    // stamps, which are right, place it.
    let landing: u64 = frame.saturating_sub(u64::from(self.sample_rate));
    let target: Timestamp = Timestamp::new(i64::try_from(landing).unwrap_or(i64::MAX));

    self
      .reader
      .seek(
        SeekMode::Accurate,
        SeekTo::Timestamp {
          ts: target,
          track_id: self.track_id,
        },
      )
      .map_err(|error| Self::new_error("Could not seek in sound", &error))?;
    self.decoder.reset();
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

    loop {
      let packet = match self.reader.next_packet() {
        Ok(Some(packet)) => packet,
        Ok(None) => {
          samples.clear();
          return Ok(false);
        }
        Err(error) => return Err(Self::new_error("Could not read sound", &error)),
      };

      if packet.track_id != self.track_id {
        continue;
      }

      let first_frame: u64 = u64::try_from(packet.pts.get()).unwrap_or(0);
      let buffer = match self.decoder.decode(&packet) {
        Ok(buffer) => buffer,
        Err(SymphoniaError::DecodeError(_)) => continue,
        Err(error) => return Err(Self::new_error("Could not decode sound", &error)),
      };

      if buffer.num_planes() != channels {
        return Err(XrfError::new_read_error(format!(
          "Sound packet decoded {} channels, not the track's {channels}",
          buffer.num_planes()
        )));
      }

      buffer.copy_to_vec_interleaved(samples);

      let skipped: usize = usize::try_from(self.next_frame.saturating_sub(first_frame)).unwrap_or(usize::MAX);

      samples.drain(..(skipped * channels).min(samples.len()));
      self.next_frame = self.next_frame.max(first_frame) + (samples.len() / channels) as u64;

      if !samples.is_empty() {
        return Ok(true);
      }
    }
  }

  fn new_error(context: &str, error: &SymphoniaError) -> XrfError {
    XrfError::new_read_error(format!("{context}: {error}"))
  }
}
