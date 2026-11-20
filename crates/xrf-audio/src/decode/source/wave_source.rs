use std::io::Cursor;
use std::sync::Arc;

use symphonia::core::codecs::CodecParameters;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Timestamp;
use xrf_error::{XrfError, XrfResult};

/// A WAV sound, as the SDK keeps sources, read and decoded by symphonia: its audio track's packets, each with the frame
/// it starts on.
pub(crate) struct WaveSource {
  reader: Box<dyn FormatReader>,
  decoder: Box<dyn AudioDecoder>,
  track_id: u32,
  sample_rate: u32,
  channels_count: usize,
  frames_count: u64,
}

impl WaveSource {
  /// Opens the sound in `bytes` at its first frame.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a container symphonia reads, or carry no audio track, or one without a
  /// sample rate or a length.
  pub(crate) fn open(bytes: Arc<[u8]>) -> XrfResult<Self> {
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
      channels_count,
      frames_count,
    })
  }

  pub(crate) fn get_sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub(crate) fn get_channels_count(&self) -> usize {
    self.channels_count
  }

  pub(crate) fn get_frames_count(&self) -> u64 {
    self.frames_count
  }

  /// Moves to the packet holding `frame`, which may start before it.
  ///
  /// # Errors
  ///
  /// Returns an error when the container cannot seek to it.
  pub(crate) fn seek(&mut self, frame: u64) -> XrfResult<()> {
    self
      .reader
      .seek(
        SeekMode::Accurate,
        SeekTo::Timestamp {
          ts: Timestamp::new(i64::try_from(frame).unwrap_or(i64::MAX)),
          track_id: self.track_id,
        },
      )
      .map_err(|error| Self::new_error("Could not seek in sound", &error))?;
    self.decoder.reset();

    Ok(())
  }

  /// Decodes the next packet into `samples`, interleaved, replacing what they held, and answers the frame it starts
  /// on; none once the track has no audio left. A packet the codec refuses as corrupt is skipped.
  ///
  /// # Errors
  ///
  /// Returns an error when the container cannot be read further, the codec fails for good, or a packet decodes to
  /// another count of channels than the track's.
  pub(crate) fn decode_next(&mut self, samples: &mut Vec<f32>) -> XrfResult<Option<u64>> {
    loop {
      let packet = match self.reader.next_packet() {
        Ok(Some(packet)) => packet,
        Ok(None) => return Ok(None),
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

      if buffer.num_planes() != self.channels_count {
        return Err(XrfError::new_read_error(format!(
          "Sound packet decoded {} channels, not the track's {}",
          buffer.num_planes(),
          self.channels_count
        )));
      }

      buffer.copy_to_vec_interleaved(samples);

      return Ok(Some(first_frame));
    }
  }

  fn new_error(context: &str, error: &SymphoniaError) -> XrfError {
    XrfError::new_read_error(format!("{context}: {error}"))
  }
}
