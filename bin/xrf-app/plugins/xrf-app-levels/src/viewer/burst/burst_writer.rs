use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use image::RgbaImage;
use rayon::prelude::*;
use xrf_app_core::TauriResult;
use xrf_renderer::{BurstEvent, BurstMarks, RenderBurst, RenderCapture};

use crate::viewer::burst::{BurstFiles, BurstOutcome, BurstSound, BurstVideo};

/// A frame's samples summed as they come back: the scene's seconds it was drawn at, its colour's sums at its size and
/// how many samples of its marks held each of `BurstWriter::MARKS`, at theirs.
struct BurstSum {
  index: u32,
  time: f64,
  size: BurstSize,
  colour: Vec<u32>,
  marks_size: BurstSize,
  marks: Vec<[u16; 3]>,
  samples: u32,
}

/// A picture's pixels across and down.
#[derive(Clone, Copy, PartialEq, Eq)]
struct BurstSize {
  width: u32,
  height: u32,
}

/// One sample of one frame, as the renderer numbers them.
#[derive(Clone, Copy, PartialEq, Eq)]
struct BurstSample {
  index: u32,
  sample: u32,
}

/// Writes a burst's frames into its folder as they come back: `frame-NNN.png`, or for a reference `reference-NNN.png`
/// averaging its samples; beside each, `marks-NNN.png` at the scene's drawn size, holding the share of samples each
/// G-buffer texel was a plant (red), grass (green) or self-moved (blue). At the end it writes `sound.wav` and
/// `burst.mp4` when asked for.
///
/// The renderer stalls while the writer holds its in-flight frames, so summing and PNG encoding run on the pool
/// ([`BurstFiles`]).
pub(crate) struct BurstWriter {
  folder: PathBuf,
  /// Samples each frame is drawn at, one but for a reference, and what its frames' files are named.
  samples: u32,
  kind: &'static str,
  sum: Option<BurstSum>,
  files: BurstFiles,
  sound: Option<BurstSound>,
  video: Option<BurstVideo>,
  outcome: BurstOutcome,
}

impl BurstWriter {
  /// The marks written, each to its channel of the marks' image.
  const MARKS: [u8; 3] = [BurstMarks::PLANT, BurstMarks::GRASS, BurstMarks::MOVED];

  /// How long the renderer may tell nothing of a burst before it counts as gone: a frame of the slowest view, read
  /// back, comes within a few seconds.
  const SILENCE: Duration = Duration::from_secs(60);

  /// A writer of `burst` into `folder`, which is there and is the burst's own: a failed burst removes it.
  pub(crate) fn new(folder: PathBuf, burst: &RenderBurst) -> Self {
    Self {
      outcome: BurstOutcome {
        folder: folder.clone(),
        ..BurstOutcome::default()
      },
      folder,
      samples: burst.samples.unwrap_or(1),
      kind: if burst.samples.is_some() { "reference" } else { "frame" },
      sum: None,
      files: BurstFiles::new(),
      sound: None,
      video: None,
    }
  }

  pub(crate) fn with_sound(self, sound: BurstSound) -> Self {
    Self {
      sound: Some(sound),
      ..self
    }
  }

  pub(crate) fn with_video(self, video: BurstVideo) -> Self {
    Self {
      video: Some(video),
      ..self
    }
  }

  /// Write each frame from `events` until the burst finishes, then its sound and video, and wait for its pictures. A
  /// burst that fails removes its folder, so neither a partial burst nor its name is left behind.
  ///
  /// # Errors
  ///
  /// Returns an error when the burst stops, the renderer is silent for `SILENCE`, frames arrive out of order, or a
  /// picture, the sound or the video cannot be written.
  pub(crate) fn receive(mut self, events: &Receiver<BurstEvent>) -> TauriResult<BurstOutcome> {
    let written: TauriResult = self.write_all(events);
    // Every picture under way lands before the folder is removed, which a write still going would refill or hold open.
    let pictures: TauriResult = self.files.finish();

    match written.and(pictures) {
      Ok(()) => Ok(self.outcome),
      Err(error) => {
        // The video's file is closed as it is dropped, which a folder holding it open could not be removed past.
        drop(self.video.take());
        drop(self.sound.take());

        if let Err(removal) = fs::remove_dir_all(&self.folder) {
          log::warn!(
            "Could not remove the failed burst's folder '{}': {removal}",
            self.folder.display()
          );
        }

        Err(error)
      }
    }
  }

  /// Write each frame from `events` until the burst finishes, then its sound and video.
  fn write_all(&mut self, events: &Receiver<BurstEvent>) -> TauriResult {
    loop {
      match events.recv_timeout(Self::SILENCE) {
        Ok(BurstEvent::Shot {
          index,
          sample,
          time,
          picture,
          marks,
        }) => self.take(index, sample.unwrap_or(0), time, picture, &marks)?,
        Ok(BurstEvent::Finished) => {
          // The sound's thread ends first, having sent the video the rest of its sound.
          if let Some(sound) = self.sound.take() {
            self.outcome.sound = Some(sound.write(&self.folder)?);
          }

          if let Some(video) = self.video.take() {
            self.outcome.video = Some(video.finish()?);
          }

          return Ok(());
        }
        Ok(BurstEvent::Stopped { error }) => return Err(format!("The burst stopped: {error}")),
        Err(RecvTimeoutError::Timeout) => {
          return Err(format!(
            "The renderer told nothing of the burst for {} s",
            Self::SILENCE.as_secs()
          ));
        }
        Err(RecvTimeoutError::Disconnected) => {
          return Err(String::from("The renderer stopped before the burst finished"));
        }
      }
    }
  }

  /// Take sample `sample` of frame `index`, drawn at scene `time` in seconds. A single-sample frame is written at once;
  /// a reference's samples are summed and written once the last is in.
  fn take(&mut self, index: u32, sample: u32, time: f64, picture: RenderCapture, marks: &BurstMarks) -> TauriResult {
    let due: BurstSample = self.sum.as_ref().map_or_else(
      || BurstSample {
        index: self.outcome.times.len() as u32,
        sample: 0,
      },
      |sum| BurstSample {
        index: sum.index,
        sample: sum.samples,
      },
    );

    if (BurstSample { index, sample }) != due {
      return Err(format!(
        "The burst's frame {index} sample {sample} came back where frame {} sample {} was due",
        due.index, due.sample
      ));
    }

    let size: BurstSize = BurstSize {
      width: picture.width,
      height: picture.height,
    };
    let marks_size: BurstSize = BurstSize {
      width: marks.width,
      height: marks.height,
    };

    if self.samples == 1 {
      let picture: RgbaImage = RgbaImage::from_raw(size.width, size.height, picture.pixels)
        .ok_or_else(|| format!("The burst's frame {index} does not fill its size"))?;
      let shares: RgbaImage = Self::draw_shares(index, marks_size, 1, &Self::count_once(&marks.marks))?;

      return self.write(index, time, picture, shares);
    }

    let sum: &mut BurstSum = self.sum.get_or_insert_with(|| BurstSum {
      index,
      time,
      size,
      colour: vec![0; picture.pixels.len()],
      marks_size,
      marks: vec![[0; 3]; marks.marks.len()],
      samples: 0,
    });

    if sum.size != size || sum.marks_size != marks_size {
      return Err(format!("The burst's frame {index} changed size between its samples"));
    }

    sum
      .colour
      .par_iter_mut()
      .zip(picture.pixels.par_iter())
      .for_each(|(total, value)| *total += u32::from(*value));
    sum
      .marks
      .par_iter_mut()
      .zip(marks.marks.par_iter())
      .for_each(|(counts, texel)| {
        for (count, mark) in counts.iter_mut().zip(Self::MARKS) {
          *count += u16::from(texel & mark != 0);
        }
      });
    sum.samples += 1;

    match self.sum.take_if(|sum| sum.samples == self.samples) {
      Some(BurstSum {
        index,
        time,
        size,
        colour,
        marks_size,
        marks,
        samples,
      }) => {
        let mean: Vec<u8> = colour
          .par_iter()
          .map(|total| ((total + samples / 2) / samples) as u8)
          .collect();
        let picture: RgbaImage = RgbaImage::from_raw(size.width, size.height, mean)
          .ok_or_else(|| format!("The burst's frame {index} does not fill its size"))?;
        let shares: RgbaImage = Self::draw_shares(index, marks_size, samples, &marks)?;

        self.write(index, time, picture, shares)
      }
      None => Ok(()),
    }
  }

  /// Write frame `index`'s picture and marks, hand the picture to the video, and note it in the outcome.
  fn write(&mut self, index: u32, time: f64, picture: RgbaImage, shares: RgbaImage) -> TauriResult {
    if let Some(video) = &mut self.video {
      video.write_frame(index, &picture)?;
    }

    self.outcome.times.push(time);
    (self.outcome.width, self.outcome.height) = picture.dimensions();
    (self.outcome.marks_width, self.outcome.marks_height) = shares.dimensions();
    self.files.write(
      picture,
      self.folder.join(format!("{}-{index:03}.png", self.kind)),
      "burst frame",
    )?;
    self
      .files
      .write(shares, self.folder.join(format!("marks-{index:03}.png")), "burst marks")
  }

  /// Per texel, which of `MARKS` a single sample holds, as counts of 0 or 1.
  fn count_once(marks: &[u8]) -> Vec<[u16; 3]> {
    marks
      .par_iter()
      .map(|texel| Self::MARKS.map(|mark| u16::from(texel & mark != 0)))
      .collect()
  }

  /// The marks image: each mark's share of `samples` in its channel, alpha opaque.
  fn draw_shares(index: u32, size: BurstSize, samples: u32, counts: &[[u16; 3]]) -> TauriResult<RgbaImage> {
    let mut shares: Vec<u8> = vec![0; counts.len() * 4];

    shares
      .par_chunks_exact_mut(4)
      .zip(counts.par_iter())
      .for_each(|(texel, [plant, grass, moved])| {
        let share = |count: u16| ((u32::from(count) * 255 + samples / 2) / samples) as u8;

        texel.copy_from_slice(&[share(*plant), share(*grass), share(*moved), 255]);
      });

    RgbaImage::from_raw(size.width, size.height, shares)
      .ok_or_else(|| format!("The burst's frame {index} marks do not fill their size"))
  }
}
