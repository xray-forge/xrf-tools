use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
#[cfg(windows)]
use std::time::Duration;

use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_renderer::RenderBurst;

#[cfg(windows)]
use crate::viewer::burst::BurstSound;
use crate::viewer::burst::{BurstSoundChunk, BurstVideoOutcome};
use crate::viewer::recording::writer::MediaProbe;
#[cfg(windows)]
use crate::viewer::recording::writer::{RecordFormat, RecordSample, RecordSoundFormat, RecordSummary, RecordWriter};

/// Writes a burst as `burst.mp4` in its folder as it comes: frame `k` shows from `k × step` for one step, with the
/// sound from the first frame's start, so each frame plays with what its camera heard.
///
/// The writer waits on neither stream: sound can trail by seconds while a level's walls build, so each frame goes in
/// after whatever sound has arrived. An odd-sized frame loses its last column or row, which H.264 rejects.
pub(crate) struct BurstVideo {
  file: PathBuf,
  step: f64,
  /// The sound's sender, held until the sound takes it, and its receiver.
  sound: Option<Sender<BurstSoundChunk>>,
  heard: Receiver<BurstSoundChunk>,
  is_heard: bool,
  /// The first frame's size, and the writer it opened.
  size: Option<(u32, u32)>,
  #[cfg(windows)]
  writer: Option<RecordWriter>,
}

impl BurstVideo {
  pub(crate) const FILE: &'static str = "burst.mp4";

  /// A video of `burst`'s frames in `folder`.
  ///
  /// # Errors
  ///
  /// Returns the probe's reason when no video can be written here.
  pub(crate) fn new(burst: &RenderBurst, probe: &MediaProbe, folder: &Path) -> TauriResult<Self> {
    probe.load()?;

    let (sound, heard) = channel();

    Ok(Self {
      file: folder.join(Self::FILE),
      step: f64::from(burst.step),
      sound: Some(sound),
      heard,
      is_heard: false,
      size: None,
      #[cfg(windows)]
      writer: None,
    })
  }

  /// The sender for the burst's rendered sound, taken once; a video whose sound is not taken before its first frame is
  /// silent.
  pub(crate) fn take_sound(&mut self) -> Option<Sender<BurstSoundChunk>> {
    let sound: Option<Sender<BurstSoundChunk>> = self.sound.take();

    self.is_heard |= sound.is_some();

    sound
  }

  /// Write frame `index` after the sound received so far; the first frame opens the file at its size.
  ///
  /// # Errors
  ///
  /// Returns an error when the frame is too small or differs in size from the first, or the video cannot be written.
  pub(crate) fn write_frame(&mut self, index: u32, picture: &RgbaImage) -> TauriResult {
    #[cfg(windows)]
    {
      self.write_frame_windows(index, picture)
    }

    #[cfg(not(windows))]
    {
      let _ = (index, picture, &self.heard, self.size, self.step, self.is_heard);

      Err(MediaProbe::MEDIA_FOUNDATION.load().err().unwrap_or_default())
    }
  }

  /// Write the rest of the sound once its thread has ended, and close the file.
  ///
  /// # Errors
  ///
  /// Returns an error when no frame came, or the video cannot be finished.
  pub(crate) fn finish(self) -> TauriResult<BurstVideoOutcome> {
    #[cfg(windows)]
    {
      self.finish_windows()
    }

    #[cfg(not(windows))]
    {
      Err(MediaProbe::MEDIA_FOUNDATION.load().err().unwrap_or_default())
    }
  }
}

#[cfg(windows)]
impl BurstVideo {
  fn write_frame_windows(&mut self, index: u32, picture: &RgbaImage) -> TauriResult {
    let size: (u32, u32) = picture.dimensions();
    let (width, height) = (size.0 & !1, size.1 & !1);

    match self.size {
      None => {
        if width == 0 || height == 0 {
          return Err(format!(
            "The burst's {}×{} frames are too small for a video",
            size.0, size.1
          ));
        }

        // A sound not taken by now never comes; dropping its sender keeps `finish` from waiting on it.
        self.sound = None;
        self.size = Some(size);
        self.writer = Some(RecordWriter::start(
          self.file.clone(),
          RecordFormat {
            width,
            height,
            frame: self.get_time(1),
            sound: self.is_heard.then_some(RecordSoundFormat {
              rate: BurstSound::RATE,
              channels: 2,
            }),
          },
        )?);
      }
      Some(first) if first != size => {
        return Err(format!(
          "The burst's frame {index} is {}×{}, its first {}×{}",
          size.0, size.1, first.0, first.1
        ));
      }
      Some(_) => {}
    }

    let (time, end) = (self.get_time(index), self.get_time(index + 1));
    let writer: &mut RecordWriter = self.writer.as_mut().ok_or("The burst's video stopped")?;

    // Sound received so far goes in before the frame, interleaving the two where it can.
    while let Ok(chunk) = self.heard.try_recv() {
      Self::write_sound(writer, chunk)?;
    }

    writer.write(RecordSample::Frame {
      bgra: Self::copy_bgra(picture, width, height),
      time,
      duration: end - time,
    })
  }

  fn finish_windows(mut self) -> TauriResult<BurstVideoOutcome> {
    let mut writer: RecordWriter = self.writer.take().ok_or("The burst drew no frame to make a video of")?;
    let (width, height) = self.size.map_or((0, 0), |(width, height)| (width & !1, height & !1));

    // The sound rendered after the last frame; the loop ends when the sound's thread drops its sender.
    for chunk in &self.heard {
      Self::write_sound(&mut writer, chunk)?;
    }

    let summary: RecordSummary = writer.finish()?;

    Ok(BurstVideoOutcome {
      file: self.file,
      frames: summary.frames,
      duration: summary.duration.as_secs_f64(),
      width,
      height,
      encoder: summary.encoder,
    })
  }

  fn write_sound(writer: &mut RecordWriter, chunk: BurstSoundChunk) -> TauriResult {
    writer.write(RecordSample::Sound {
      samples: chunk.samples,
      at: chunk.at,
    })
  }

  fn get_time(&self, index: u32) -> Duration {
    Duration::from_secs_f64(f64::from(index) * self.step)
  }

  /// `picture`'s top-left `width` by `height` as BGRA. Rows are copied whole, then red and blue swapped in place, a
  /// loop the compiler vectorizes; a fullscreen frame is 29 MB.
  pub(crate) fn copy_bgra(picture: &RgbaImage, width: u32, height: u32) -> Vec<u8> {
    let (row, kept) = (picture.width() as usize * 4, width as usize * 4);
    let mut bgra: Vec<u8> = Vec::with_capacity(kept * height as usize);

    for pixels in picture.as_raw().chunks_exact(row).take(height as usize) {
      bgra.extend_from_slice(&pixels[..kept]);
    }

    for pixel in bgra.as_chunks_mut::<4>().0 {
      pixel.swap(0, 2);
    }

    bgra
  }
}
