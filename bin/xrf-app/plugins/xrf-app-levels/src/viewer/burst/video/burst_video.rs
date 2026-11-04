use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_renderer::RenderBurst;

#[cfg(windows)]
use crate::viewer::burst::BurstSound;
use crate::viewer::burst::{BurstSoundOutcome, BurstVideoOutcome};
use crate::viewer::record::MediaProbe;
#[cfg(windows)]
use crate::viewer::record::{RecordFormat, RecordSample, RecordSoundFormat, RecordSummary, RecordWriter};

/// A burst written as one video once its frames and sound are in, `burst.mp4` beside them: frame `k` shown from
/// `k × step` for a step, the sound from the first frame's start, so each frame plays with what its camera heard. A
/// frame of an odd size loses its last column or row, which H.264 does not take.
pub struct BurstVideo {
  step: f64,
}

impl BurstVideo {
  /// The video's file in the burst's folder.
  pub const FILE: &'static str = "burst.mp4";

  /// A video of `burst`'s frames, where `probe` finds what writes one.
  ///
  /// # Errors
  ///
  /// Returns why no video can be written here: the probe's answer.
  pub fn new(burst: &RenderBurst, probe: &MediaProbe) -> TauriResult<Self> {
    probe.load()?;

    Ok(Self {
      step: f64::from(burst.step),
    })
  }

  /// Writes the burst's `frames`, its files in the order drawn, and its `sound` where it has one, into `folder`.
  ///
  /// # Errors
  ///
  /// Returns an error when there is no frame, one cannot be read or differs in size, the sound does not match the
  /// frames, or the video cannot be written.
  pub fn write(
    &self,
    frames: &[PathBuf],
    sound: Option<&BurstSoundOutcome>,
    folder: &Path,
  ) -> TauriResult<BurstVideoOutcome> {
    #[cfg(windows)]
    {
      self.encode(frames, sound, folder)
    }

    #[cfg(not(windows))]
    {
      let _ = (frames, sound, folder, self.step);

      Err(MediaProbe::MEDIA_FOUNDATION.load().err().unwrap_or_default())
    }
  }
}

#[cfg(windows)]
impl BurstVideo {
  fn encode(
    &self,
    frames: &[PathBuf],
    sound: Option<&BurstSoundOutcome>,
    folder: &Path,
  ) -> TauriResult<BurstVideoOutcome> {
    let file: PathBuf = folder.join(Self::FILE);
    let first: RgbaImage = Self::read_frame(frames.first().ok_or("The burst wrote no frame to make a video of")?)?;
    let size: (u32, u32) = first.dimensions();
    let (width, height) = (size.0 & !1, size.1 & !1);

    if width == 0 || height == 0 {
      return Err(format!(
        "The burst's {}×{} frames are too small for a video",
        size.0, size.1
      ));
    }

    let samples: Option<Vec<f32>> = sound.map(|sound| BurstSound::read_wave(&sound.file)).transpose()?;

    if let Some(sound) = sound
      && sound.offsets.len() != frames.len()
    {
      return Err(format!(
        "The burst's sound starts {} frames where it drew {}",
        sound.offsets.len(),
        frames.len()
      ));
    }

    let at = |index: usize| Duration::from_secs_f64(index as f64 * self.step);
    let mut writer: RecordWriter = RecordWriter::start(
      file.clone(),
      RecordFormat {
        width,
        height,
        frame: at(1),
        sound: sound.map(|sound| RecordSoundFormat {
          rate: sound.rate,
          channels: sound.channels,
        }),
      },
    )?;
    let mut first: Option<RgbaImage> = Some(first);

    for (index, path) in frames.iter().enumerate() {
      // Each frame's sound goes just before it, so the two arrive interleaved.
      if let (Some(sound), Some(samples)) = (sound, &samples) {
        let channels: usize = usize::from(sound.channels);
        let frames_held: u64 = (samples.len() / channels) as u64;
        let from: u64 = sound.offsets[index].min(frames_held);
        let to: u64 = sound
          .offsets
          .get(index + 1)
          .map_or(frames_held, |&to| to.min(frames_held));

        writer.write(RecordSample::Sound {
          samples: samples[from as usize * channels..to.max(from) as usize * channels].to_vec(),
          at: from,
        })?;
      }

      let picture: RgbaImage = match first.take() {
        Some(picture) => picture,
        None => Self::read_frame(path)?,
      };

      if picture.dimensions() != size {
        return Err(format!(
          "The burst's frame {} is {}×{}, its first {}×{}",
          path.display(),
          picture.width(),
          picture.height(),
          size.0,
          size.1
        ));
      }

      writer.write(RecordSample::Frame {
        bgra: Self::to_bgra(&picture, width, height),
        time: at(index),
        duration: at(index + 1) - at(index),
      })?;
    }

    let summary: RecordSummary = writer.finish()?;

    Ok(BurstVideoOutcome {
      file,
      frames: summary.frames,
      duration: summary.duration.as_secs_f64(),
      width,
      height,
      encoder: summary.encoder,
    })
  }

  fn read_frame(path: &Path) -> TauriResult<RgbaImage> {
    Ok(
      image::open(path)
        .map_err(|error| format!("The burst's frame {} cannot be read: {error}", path.display()))?
        .into_rgba8(),
    )
  }

  /// `picture`'s top left `width` by `height`, blue first.
  fn to_bgra(picture: &RgbaImage, width: u32, height: u32) -> Vec<u8> {
    picture
      .rows()
      .take(height as usize)
      .flat_map(|row| {
        row.take(width as usize).flat_map(|pixel| {
          let [red, green, blue, alpha] = pixel.0;

          [blue, green, red, alpha]
        })
      })
      .collect()
  }
}
