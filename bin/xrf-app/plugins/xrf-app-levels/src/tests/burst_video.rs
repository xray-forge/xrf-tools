//! A burst's video: its frames and its sound written as one MP4 through Media Foundation, read back through its Source
//! Reader, and refused where Media Foundation does not load.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use image::{Rgba, RgbaImage};
use windows::Win32::Media::MediaFoundation::{
  IMFSample, IMFSourceReader, MF_SOURCE_READER_FIRST_AUDIO_STREAM, MF_SOURCE_READER_FIRST_VIDEO_STREAM,
  MF_SOURCE_READERF_ENDOFSTREAM, MFCreateSourceReaderFromURL,
};
use windows::core::{HSTRING, PCWSTR};
use xrf_renderer::RenderBurst;
use xrf_test_utils::GeneratedResources;

use crate::viewer::burst::{BurstSound, BurstSoundOutcome, BurstVideo, BurstVideoOutcome};
use crate::viewer::record::{
  MediaProbe, MediaSession, RecordFormat, RecordSample, RecordSoundFormat, RecordSummary, RecordWriter,
};

/// Half a second of frames 1/60 s apart.
const BURST: RenderBurst = RenderBurst {
  at: 600.0,
  step: 1.0 / 60.0,
  lead_in: 0,
  frames: 30,
  samples: None,
};

/// The sound's frames each burst frame lasts, at 48 kHz.
const FRAME_SOUND: u64 = 800;

/// Ticks of Media Foundation's clock a second.
const TICKS: i64 = 10_000_000;

/// A burst's folder, made empty.
fn folder(name: &str) -> PathBuf {
  let folder: PathBuf = GeneratedResources::path(&format!("burst_video/{name}"));

  let _ = fs::remove_dir_all(&folder);
  fs::create_dir_all(&folder).expect("the folder to be made");

  folder
}

/// What a stream of a file holds: its samples, the first's time, and the last's end, in ticks.
#[derive(Debug, Default)]
struct ReadStream {
  samples: u32,
  first: Option<i64>,
  end: i64,
}

/// Reads `stream` of `file` through to its end, as a player would, its samples left as they were written.
fn read_stream(file: &Path, stream: u32) -> ReadStream {
  let _session: MediaSession = MediaSession::start().expect("Media Foundation to start");
  let path: HSTRING = HSTRING::from(file.as_os_str());
  let mut read: ReadStream = ReadStream::default();

  // SAFETY: a reader of this function's own, released before the session shuts down.
  unsafe {
    let reader: IMFSourceReader = MFCreateSourceReaderFromURL(PCWSTR(path.as_ptr()), None).expect("the file to open");

    loop {
      let mut flags: u32 = 0;
      let mut time: i64 = 0;
      let mut sample: Option<IMFSample> = None;

      reader
        .ReadSample(stream, 0, None, Some(&mut flags), Some(&mut time), Some(&mut sample))
        .expect("a sample to be read");

      if let Some(sample) = sample {
        read.samples += 1;
        read.first.get_or_insert(time);
        read.end = read
          .end
          .max(time + sample.GetSampleDuration().expect("the sample's duration"));
      }

      if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
        break;
      }
    }
  }

  read
}

/// A burst of `width` by `height` frames and its sound written as a video into the folder `name`, and its file read
/// back: each frame once, a step apart from zero, the sound beside them from the same start to the same end, an odd size
/// cut to the even one under it.
fn write_and_read(name: &str, width: u32, height: u32) {
  let folder: PathBuf = folder(name);
  let frames: Vec<PathBuf> = (0..BURST.frames)
    .map(|index| {
      let file: PathBuf = folder.join(format!("frame-{index:03}.png"));
      let shade: u8 = (index * 8) as u8;

      RgbaImage::from_pixel(width, height, Rgba([shade, 255 - shade, 64, 255]))
        .save(&file)
        .expect("the frame to be written");

      file
    })
    .collect();
  let samples: Vec<f32> = (0..u64::from(BURST.frames) * FRAME_SOUND * 2)
    .map(|sample| ((sample / 2) as f32 * 0.05).sin() * 0.5)
    .collect();
  let sound: BurstSoundOutcome = BurstSoundOutcome {
    file: folder.join("sound.wav"),
    rate: BurstSound::RATE,
    channels: 2,
    offsets: (0..u64::from(BURST.frames)).map(|index| index * FRAME_SOUND).collect(),
  };

  fs::write(&sound.file, BurstSound::to_wave(&samples)).expect("the sound to be written");

  assert_eq!(
    BurstSound::read_wave(&sound.file).expect("the sound to be read"),
    samples
  );

  let outcome: BurstVideoOutcome = BurstVideo::new(&BURST, &MediaProbe::MEDIA_FOUNDATION)
    .expect("Media Foundation to load")
    .write(&frames, Some(&sound), &folder)
    .expect("the video to be written");
  let video: ReadStream = read_stream(&outcome.file, MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32);
  let audio: ReadStream = read_stream(&outcome.file, MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32);
  let burst_end: i64 = i64::from(BURST.frames) * TICKS / 60;

  assert_eq!(outcome.file, folder.join("burst.mp4"));
  assert_eq!((outcome.width, outcome.height), (width & !1, height & !1));
  assert_eq!(outcome.frames, BURST.frames);
  assert!((outcome.duration - 0.5).abs() < 1e-6, "{outcome:?}");
  assert!(!outcome.encoder.is_empty());
  assert_eq!(video.samples, BURST.frames, "{video:?}");
  // A frame the encoder reorders would start the picture a frame late, behind the sound.
  assert_eq!(
    (video.first, audio.first),
    (Some(0), Some(0)),
    "{outcome:?} {video:?} {audio:?}"
  );
  assert!((video.end - burst_end).abs() <= 1, "{video:?}");
  // AAC holds 1024 frames a packet, so the sound ends within one of the picture.
  assert!((audio.end - burst_end).abs() <= 1024 * TICKS / 48_000, "{audio:?}");
}

// Through Microsoft's software encoder, which takes a size too small for a GPU's.
#[test]
fn writes_a_small_bursts_frames_and_sound_as_one_video() {
  write_and_read("small", 129, 73);
}

// Through the GPU's encoder where it has one.
#[test]
fn writes_a_bursts_frames_and_sound_as_one_video() {
  write_and_read("larger", 321, 181);
}

// The viewer's widest burst at its default step, 3730×1930 frames 1/120 s apart, past what H.264's highest common
// level names a frame rate for, is written as given.
#[test]
fn writes_the_viewers_widest_burst_at_its_default_step() {
  let file: PathBuf = folder("widest").join("burst.mp4");
  let (width, height): (u32, u32) = (3730, 1930);
  let step: Duration = Duration::from_secs_f64(1.0 / 120.0);
  let mut writer: RecordWriter = RecordWriter::start(
    file.clone(),
    RecordFormat {
      width,
      height,
      frame: step,
      sound: None,
    },
  )
  .expect("the writer to take the size and rate");

  for index in 0..12u32 {
    writer
      .write(RecordSample::Frame {
        bgra: vec![(index * 20) as u8; width as usize * height as usize * 4],
        time: step * index,
        duration: step,
      })
      .expect("the frame to be written");
  }

  let summary: RecordSummary = writer.finish().expect("the video to be finished");
  let video: ReadStream = read_stream(&file, MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32);

  assert_eq!(summary.frames, 12, "{summary:?}");
  assert_eq!((video.samples, video.first), (12, Some(0)), "{summary:?} {video:?}");
  assert!((video.end - TICKS / 10).abs() <= 1, "{summary:?} {video:?}");
}

// Sound from a device at a rate AAC does not take is brought to 48 kHz on its own clock: a block missing between two
// does not slide what comes after, and the sound ends with the picture.
#[test]
fn writes_sound_from_a_rate_aac_does_not_take() {
  let file: PathBuf = folder("resampled").join("burst.mp4");
  let (width, height): (u32, u32) = (320, 180);
  let step: Duration = Duration::from_secs_f64(1.0 / 60.0);
  let mut writer: RecordWriter = RecordWriter::start(
    file.clone(),
    RecordFormat {
      width,
      height,
      frame: step,
      sound: Some(RecordSoundFormat {
        rate: 96_000,
        channels: 2,
      }),
    },
  )
  .expect("the writer to take the sound's rate");

  for index in 0..30u32 {
    // Half a second of 96 kHz in blocks of 256 frames, one skipped as a full tap drops it.
    for block in (index * 1600 / 256)..((index + 1) * 1600 / 256) {
      if block != 40 {
        writer
          .write(RecordSample::Sound {
            samples: vec![0.25; 512],
            at: u64::from(block) * 256,
          })
          .expect("the sound to be written");
      }
    }

    writer
      .write(RecordSample::Frame {
        bgra: vec![128; width as usize * height as usize * 4],
        time: step * index,
        duration: step,
      })
      .expect("the frame to be written");
  }

  writer.finish().expect("the video to be finished");

  let audio: ReadStream = read_stream(&file, MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32);

  assert_eq!(audio.first, Some(0), "{audio:?}");
  assert!((audio.end - TICKS / 2).abs() <= 1024 * TICKS / 48_000, "{audio:?}");
}

// Where Media Foundation does not load, a video is refused before anything draws, naming what is missing.
#[test]
fn refuses_a_video_where_media_foundation_does_not_load() {
  let error: String = BurstVideo::new(&BURST, &MediaProbe::new(&["xrf-no-such-library.dll"]))
    .err()
    .expect("the video to be refused");

  assert!(error.contains("xrf-no-such-library.dll"), "{error}");
  assert!(error.contains("Media Feature Pack"), "{error}");
}
