#![expect(
  unsafe_code,
  reason = "writes the recording through Media Foundation, whose COM calls the `windows` crate exposes as unsafe"
)]

use std::path::Path;
use std::time::Duration;

use windows::Win32::Media::MediaFoundation::{
  CODECAPI_AVEncMPVDefaultBPictureCount, IMFAttributes, IMFMediaBuffer, IMFMediaType, IMFSample, IMFSinkWriter,
  IMFSinkWriterEx, IMFTransform, MF_MT_AUDIO_AVG_BYTES_PER_SECOND, MF_MT_AUDIO_BITS_PER_SAMPLE,
  MF_MT_AUDIO_BLOCK_ALIGNMENT, MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_AVG_BITRATE,
  MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE,
  MF_MT_MPEG2_PROFILE, MF_MT_PIXEL_ASPECT_RATIO, MF_MT_SUBTYPE, MF_MT_VIDEO_LEVEL,
  MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, MF_SINK_WRITER_DISABLE_THROTTLING, MF_TRANSCODE_CONTAINERTYPE,
  MFAudioFormat_AAC, MFAudioFormat_PCM, MFCreateAttributes, MFCreateMediaType, MFCreateMemoryBuffer, MFCreateSample,
  MFCreateSinkWriterFromURL, MFMediaType_Audio, MFMediaType_Video, MFT_CATEGORY_VIDEO_ENCODER,
  MFT_ENUM_HARDWARE_URL_Attribute, MFT_FRIENDLY_NAME_Attribute, MFTranscodeContainerType_FMPEG4, MFVideoFormat_H264,
  MFVideoFormat_RGB32, MFVideoInterlace_Progressive, eAVEncH264VProfile_High,
};
use windows::core::{GUID, HSTRING, Interface, PCWSTR};

use crate::viewer::recording::writer::{RecordFormat, RecordResampler, RecordSample, RecordSoundFormat, RecordSummary};

/// Media Foundation's Sink Writer writing one recording as fragmented MP4: H.264 High from frames handed over as RGB32,
/// which the writer converts for the encoder, a hardware one where the GPU has it; and AAC from 16-bit PCM, sound at a
/// rate AAC does not take brought to 48 kHz first. Each fragment is whole once written, so a recording cut short plays
/// to its last. Made and used on the thread holding a `MediaSession`.
pub struct MediaSink {
  writer: IMFSinkWriter,
  format: RecordFormat,
  video: u32,
  sound: Option<SinkSound>,
  encoder: String,
  frames: u32,
  end: Duration,
}

/// The sound stream: its index, what the encoder takes, and the resampler that brings the sound to it where it must.
struct SinkSound {
  stream: u32,
  encoded: RecordSoundFormat,
  resampling: Option<SinkResampling>,
}

/// Sound brought to 48 kHz as it comes: a gap in what comes (a block the tap had no room for) starts the resampler
/// again at the next sound's own time, so nothing after it slides.
struct SinkResampling {
  resampler: RecordResampler,
  /// The input frame the sound handed over so far ends at, none before the first.
  next: Option<u64>,
  /// The output frame the resampler last started again at, and the frames it gave since.
  base: u64,
  given: u64,
}

impl MediaSink {
  /// Ticks of Media Foundation's clock a second.
  const TICKS: u64 = 10_000_000;

  /// The video's bits a pixel of each frame, so the bitrate follows the size and the frame rate.
  const BITS_PER_PIXEL: f64 = 0.1;

  /// Macroblocks a second H.264's level 5.2 takes; past it a stream names level 6.2, its highest, which NVIDIA's
  /// encoder writes.
  const LEVEL_5_2_MACROBLOCKS_PER_SECOND: f64 = 2_073_600.0;
  const LEVEL_6_2: u32 = 62;

  /// AAC's highest stereo bitrate the encoder takes, bytes a second, and the bits of each sample handed to it.
  const AAC_BYTES_PER_SECOND: u32 = 24_000;
  const SOUND_BITS: u16 = 16;

  /// Opens `file` for a recording of `format` and begins writing it.
  ///
  /// # Errors
  ///
  /// Returns an error, naming the step, when the writer cannot be made or takes no stream of the format: an odd size,
  /// a size and rate past the encoder, or sound to resample in other than stereo.
  pub fn open(file: &Path, format: RecordFormat) -> Result<Self, String> {
    let fail = |step: &'static str| move |error: windows::core::Error| format!("{step}: {}", error.message());
    let path: HSTRING = HSTRING::from(file.as_os_str());

    // SAFETY: Media Foundation calls on objects this function makes, on a thread holding a session.
    unsafe {
      let attributes: IMFAttributes = Self::new_attributes(3).map_err(fail("Making the writer's attributes"))?;

      attributes
        .SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)
        .map_err(fail("Turning the throttle off"))?;

      attributes
        .SetGUID(&MF_TRANSCODE_CONTAINERTYPE, &MFTranscodeContainerType_FMPEG4)
        .map_err(fail("Asking for fragmented MP4"))?;
      attributes
        .SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)
        .map_err(fail("Allowing hardware encoders"))?;

      let writer: IMFSinkWriter = MFCreateSinkWriterFromURL(PCWSTR(path.as_ptr()), None, &attributes)
        .map_err(|error| format!("Creating the writer of {}: {}", file.display(), error.message()))?;
      let video: u32 = writer
        .AddStream(&Self::new_h264(&format).map_err(fail("Describing the H.264 stream"))?)
        .map_err(fail("Adding the H.264 stream"))?;
      let encoding: IMFAttributes = Self::new_attributes(1).map_err(fail("Making the encoder's settings"))?;

      // No B-frames: the fragmented MP4 sink writes no edit list, so a frame shown after one decoded later would start
      // the picture a frame behind its sound.
      encoding
        .SetUINT32(&CODECAPI_AVEncMPVDefaultBPictureCount, 0)
        .map_err(fail("Turning B-frames off"))?;
      writer
        .SetInputMediaType(
          video,
          &Self::new_rgb32(&format).map_err(fail("Describing the frames"))?,
          &encoding,
        )
        .map_err(|error| {
          format!(
            "Handing {}×{} frames {:.0} a second to the H.264 encoder: {}",
            format.width,
            format.height,
            Self::get_frames_per_second(&format),
            error.message()
          )
        })?;

      let sound: Option<SinkSound> = match format.sound {
        Some(taken) => {
          let resampler: Option<RecordResampler> = RecordResampler::for_rate(taken.rate);

          if resampler.is_some() && taken.channels != 2 {
            return Err(format!(
              "Sound at {} Hz is brought to the 48 kHz AAC takes in stereo only, not {} channels",
              taken.rate, taken.channels
            ));
          }

          let encoded: RecordSoundFormat = RecordSoundFormat {
            rate: resampler.as_ref().map_or(taken.rate, |_| RecordResampler::RATE),
            ..taken
          };
          let stream: u32 = writer
            .AddStream(&Self::new_aac(encoded).map_err(fail("Describing the AAC stream"))?)
            .map_err(fail("Adding the AAC stream"))?;

          writer
            .SetInputMediaType(
              stream,
              &Self::new_pcm(encoded).map_err(fail("Describing the sound"))?,
              None,
            )
            .map_err(|error| {
              format!(
                "Handing {} channels at {} Hz to the AAC encoder: {}",
                encoded.channels,
                encoded.rate,
                error.message()
              )
            })?;

          Some(SinkSound {
            stream,
            encoded,
            resampling: resampler.map(|resampler| SinkResampling {
              resampler,
              next: None,
              base: 0,
              given: 0,
            }),
          })
        }
        None => None,
      };

      writer.BeginWriting().map_err(fail("Beginning to write"))?;

      Ok(Self {
        encoder: Self::find_encoder(&writer, video),
        writer,
        format,
        video,
        sound,
        frames: 0,
        end: Duration::ZERO,
      })
    }
  }

  /// Hands `sample` to its stream's encoder.
  ///
  /// # Errors
  ///
  /// Returns an error when a frame is not the recording's size, sound comes to a recording without it or not in whole
  /// frames, or the writer refuses the sample.
  pub fn write(&mut self, sample: RecordSample) -> Result<(), String> {
    match sample {
      RecordSample::Frame { bgra, time, duration } => {
        let expected: usize = self.format.width as usize * self.format.height as usize * 4;

        if bgra.len() != expected {
          return Err(format!(
            "A frame of {} bytes came to a {}×{} recording",
            bgra.len(),
            self.format.width,
            self.format.height
          ));
        }

        let start: i64 = Self::to_ticks(time);

        // SAFETY: a sample of this sink's own to its video stream.
        unsafe {
          let sample: IMFSample = Self::new_sample(&bgra, start, Self::to_ticks(time + duration) - start)
            .map_err(|error| format!("Making a frame's sample: {}", error.message()))?;

          self
            .writer
            .WriteSample(self.video, &sample)
            .map_err(|error| format!("Writing frame {}: {}", self.frames, error.message()))?;
        }

        self.frames += 1;
        self.end = self.end.max(time + duration);

        Ok(())
      }
      RecordSample::Sound { samples, at } => {
        let Some(sound) = &mut self.sound else {
          return Err(String::from("Sound came to a recording without it"));
        };
        let channels: usize = usize::from(sound.encoded.channels);

        if !samples.len().is_multiple_of(channels) {
          return Err(format!(
            "{} samples came, not whole frames of {channels} channels",
            samples.len()
          ));
        }

        let (samples, at) = match &mut sound.resampling {
          Some(resampling) => resampling.push(&samples, at),
          None => (samples, at),
        };

        Self::write_pcm(&self.writer, sound, &samples, at)
      }
    }
  }

  /// Writes `samples`, 16-bit at the rate the encoder takes, its first frame the sound's frame `at`.
  fn write_pcm(writer: &IMFSinkWriter, sound: &SinkSound, samples: &[f32], at: u64) -> Result<(), String> {
    let frames: u64 = (samples.len() / usize::from(sound.encoded.channels)) as u64;

    if frames == 0 {
      return Ok(());
    }

    let pcm: Vec<u8> = samples
      .iter()
      .flat_map(|sample| ((sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16).to_le_bytes())
      .collect();
    let start: i64 = Self::to_sound_ticks(at, sound.encoded);

    // SAFETY: a sample of this sink's own to its sound stream.
    unsafe {
      let sample: IMFSample = Self::new_sample(&pcm, start, Self::to_sound_ticks(at + frames, sound.encoded) - start)
        .map_err(|error| format!("Making a sound's sample: {}", error.message()))?;

      writer
        .WriteSample(sound.stream, &sample)
        .map_err(|error| format!("Writing sound at frame {at}: {}", error.message()))
    }
  }

  /// Writes what the encoders still hold and closes the file.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer cannot finish the file.
  pub fn finish(self) -> Result<RecordSummary, String> {
    // SAFETY: finalizes this sink's writer once; nothing is written after.
    unsafe { self.writer.Finalize() }.map_err(|error| format!("Finishing the recording: {}", error.message()))?;

    Ok(RecordSummary {
      encoder: self.encoder,
      frames: self.frames,
      duration: self.end,
    })
  }

  /// `time` on Media Foundation's clock, to the nearest tick.
  fn to_ticks(time: Duration) -> i64 {
    ((time.as_nanos() + 50) / 100) as i64
  }

  /// The sound's frame `at` on Media Foundation's clock.
  fn to_sound_ticks(at: u64, sound: RecordSoundFormat) -> i64 {
    (u128::from(at) * u128::from(Self::TICKS) / u128::from(sound.rate)) as i64
  }

  /// Frames a second of `format`'s frames.
  fn get_frames_per_second(format: &RecordFormat) -> f64 {
    Self::TICKS as f64 / Self::to_ticks(format.frame).max(1) as f64
  }

  /// The frame rate the file names for `format`, its frames' own: whole where it is within a thousandth of one, else as
  /// ticks. An encoder drops frames that come faster than it (NVIDIA's did).
  fn get_frame_rate(format: &RecordFormat) -> (u32, u32) {
    let rate: f64 = Self::get_frames_per_second(format);

    if (rate - rate.round()).abs() < 1e-3 {
      (rate.round() as u32, 1)
    } else {
      (Self::TICKS as u32, Self::to_ticks(format.frame).max(1) as u32)
    }
  }

  /// Whether `format` passes what H.264's level 5.2 takes a second, which an encoder left to choose refuses.
  fn is_past_level_5_2(format: &RecordFormat) -> bool {
    let macroblocks: u32 = format.width.div_ceil(16) * format.height.div_ceil(16);

    f64::from(macroblocks) * Self::get_frames_per_second(format) > Self::LEVEL_5_2_MACROBLOCKS_PER_SECOND
  }

  /// The video encoder's name, from the writer's transforms on `stream`: a hardware one's with its device, Microsoft's
  /// software one's as it names itself.
  unsafe fn find_encoder(writer: &IMFSinkWriter, stream: u32) -> String {
    let Ok(writer) = writer.cast::<IMFSinkWriterEx>() else {
      return String::from("unknown");
    };

    for index in 0.. {
      let mut category: GUID = GUID::zeroed();
      let mut transform: Option<IMFTransform> = None;

      // SAFETY: both out-pointers are valid for the call.
      if unsafe { writer.GetTransformForStream(stream, index, Some(&mut category), &mut transform) }.is_err() {
        break;
      }

      if category == MFT_CATEGORY_VIDEO_ENCODER
        && let Some(transform) = transform
      {
        // SAFETY: the transform's attributes, read once.
        let Ok(attributes) = (unsafe { transform.GetAttributes() }) else {
          return String::from("unknown");
        };
        let name = |key: &GUID| Self::read_string(&attributes, key).filter(|name| !name.is_empty());

        return match (
          name(&MFT_FRIENDLY_NAME_Attribute),
          name(&MFT_ENUM_HARDWARE_URL_Attribute),
        ) {
          (Some(friendly), Some(_)) => format!("{friendly} (hardware)"),
          (None, Some(url)) => format!("{url} (hardware)"),
          (Some(friendly), None) => friendly,
          (None, None) => String::from("H.264 Encoder MFT"),
        };
      }
    }

    String::from("unknown")
  }

  fn read_string(attributes: &IMFAttributes, key: &GUID) -> Option<String> {
    let mut buffer: [u16; 256] = [0; 256];
    let mut length: u32 = 0;

    // SAFETY: the buffer is as long as the length it is given.
    unsafe { attributes.GetString(key, &mut buffer, Some(&mut length)) }.ok()?;

    Some(String::from_utf16_lossy(&buffer[..length as usize]))
  }

  unsafe fn new_attributes(count: u32) -> windows::core::Result<IMFAttributes> {
    let mut attributes: Option<IMFAttributes> = None;

    // SAFETY: the out-pointer is valid.
    unsafe { MFCreateAttributes(&mut attributes, count)? };

    attributes.ok_or_else(windows::core::Error::empty)
  }

  unsafe fn new_h264(format: &RecordFormat) -> windows::core::Result<IMFMediaType> {
    let bitrate: f64 =
      f64::from(format.width) * f64::from(format.height) * Self::BITS_PER_PIXEL * Self::get_frames_per_second(format);

    // SAFETY: setting attributes of a new media type.
    unsafe {
      let media: IMFMediaType = Self::new_video(format, &MFVideoFormat_H264)?;

      media.SetUINT32(&MF_MT_AVG_BITRATE, bitrate as u32)?;
      media.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32)?;

      if Self::is_past_level_5_2(format) {
        media.SetUINT32(&MF_MT_VIDEO_LEVEL, Self::LEVEL_6_2)?;
      }

      Ok(media)
    }
  }

  unsafe fn new_rgb32(format: &RecordFormat) -> windows::core::Result<IMFMediaType> {
    // SAFETY: setting attributes of a new media type.
    unsafe {
      let media: IMFMediaType = Self::new_video(format, &MFVideoFormat_RGB32)?;

      // Positive: the rows run top down.
      media.SetUINT32(&MF_MT_DEFAULT_STRIDE, format.width * 4)?;

      Ok(media)
    }
  }

  unsafe fn new_video(format: &RecordFormat, subtype: &GUID) -> windows::core::Result<IMFMediaType> {
    let (numerator, denominator) = Self::get_frame_rate(format);

    // SAFETY: setting attributes of a new media type.
    unsafe {
      let media: IMFMediaType = MFCreateMediaType()?;

      media.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
      media.SetGUID(&MF_MT_SUBTYPE, subtype)?;
      media.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
      media.SetUINT64(
        &MF_MT_FRAME_SIZE,
        (u64::from(format.width) << 32) | u64::from(format.height),
      )?;
      media.SetUINT64(&MF_MT_FRAME_RATE, (u64::from(numerator) << 32) | u64::from(denominator))?;
      media.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1 << 32) | 1)?;

      Ok(media)
    }
  }

  unsafe fn new_aac(sound: RecordSoundFormat) -> windows::core::Result<IMFMediaType> {
    // SAFETY: setting attributes of a new media type.
    unsafe {
      let media: IMFMediaType = Self::new_audio(sound, &MFAudioFormat_AAC)?;

      media.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, Self::AAC_BYTES_PER_SECOND)?;

      Ok(media)
    }
  }

  unsafe fn new_pcm(sound: RecordSoundFormat) -> windows::core::Result<IMFMediaType> {
    let block: u32 = u32::from(sound.channels) * u32::from(Self::SOUND_BITS / 8);

    // SAFETY: setting attributes of a new media type.
    unsafe {
      let media: IMFMediaType = Self::new_audio(sound, &MFAudioFormat_PCM)?;

      media.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, block)?;
      media.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, sound.rate * block)?;

      Ok(media)
    }
  }

  unsafe fn new_audio(sound: RecordSoundFormat, subtype: &GUID) -> windows::core::Result<IMFMediaType> {
    // SAFETY: setting attributes of a new media type.
    unsafe {
      let media: IMFMediaType = MFCreateMediaType()?;

      media.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
      media.SetGUID(&MF_MT_SUBTYPE, subtype)?;
      media.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, u32::from(Self::SOUND_BITS))?;
      media.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, sound.rate)?;
      media.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, u32::from(sound.channels))?;

      Ok(media)
    }
  }

  /// A sample of `data` from `time` for `duration`, both in ticks.
  unsafe fn new_sample(data: &[u8], time: i64, duration: i64) -> windows::core::Result<IMFSample> {
    // SAFETY: the buffer, `data.len()` bytes long, is locked while `data` is copied into it.
    unsafe {
      let buffer: IMFMediaBuffer = MFCreateMemoryBuffer(data.len() as u32)?;
      let mut target: *mut u8 = std::ptr::null_mut();

      buffer.Lock(&mut target, None, None)?;
      std::ptr::copy_nonoverlapping(data.as_ptr(), target, data.len());
      buffer.Unlock()?;
      buffer.SetCurrentLength(data.len() as u32)?;

      let sample: IMFSample = MFCreateSample()?;

      sample.AddBuffer(&buffer)?;
      sample.SetSampleTime(time)?;
      sample.SetSampleDuration(duration)?;

      Ok(sample)
    }
  }
}

impl SinkResampling {
  /// `samples` from the input's frame `at`, brought to 48 kHz: what the resampler gives now, and the output frame it
  /// starts at.
  fn push(&mut self, samples: &[f32], at: u64) -> (Vec<f32>, u64) {
    let rate: u32 = self.resampler.get_input_rate();

    if self.next != Some(at) {
      self.resampler.restart();
      self.base = (u128::from(at) * u128::from(RecordResampler::RATE) / u128::from(rate)) as u64;
      self.given = 0;
    }

    let mut given: Vec<f32> = Vec::new();
    let starts_at: u64 = self.base + self.given;

    self.resampler.push(samples, &mut given);
    self.next = Some(at + (samples.len() / 2) as u64);
    self.given += (given.len() / 2) as u64;

    (given, starts_at)
  }
}
