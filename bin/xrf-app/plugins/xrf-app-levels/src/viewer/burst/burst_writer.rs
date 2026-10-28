use std::path::PathBuf;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use image::RgbaImage;
use xrf_app_core::TauriResult;
use xrf_app_render::CaptureFile;
use xrf_renderer::{BurstEvent, BurstMarks, RenderBurst, RenderCapture};

use crate::viewer::burst::BurstOutcome;

/// A frame's samples summed as they come back: the scene's seconds it was drawn at, its colour's sums at its size and
/// how many samples of its marks held each of `BurstWriter::MARKS`, at theirs.
struct BurstSum {
  index: u32,
  time: f32,
  size: (u32, u32),
  colour: Vec<u32>,
  marks_size: (u32, u32),
  marks: Vec<[u16; 3]>,
  samples: u32,
}

/// A burst's frames written into its folder as they come back: `frame-NNN.png` each frame read back, or for a
/// reference `reference-NNN.png` each frame's samples averaged; and beside each `marks-NNN.png`, the share of its
/// samples each texel of the G-buffer was a plant (red), grass (green) or moved by itself (blue), at the size the scene
/// was drawn at.
pub struct BurstWriter {
  folder: PathBuf,
  /// Samples each frame is drawn at, one but for a reference, and what its frames' files are named.
  samples: u32,
  kind: &'static str,
  sum: Option<BurstSum>,
  outcome: BurstOutcome,
}

impl BurstWriter {
  /// The marks written, each to its channel of the marks' image.
  const MARKS: [u8; 3] = [BurstMarks::PLANT, BurstMarks::GRASS, BurstMarks::MOVED];

  /// How long the renderer may tell nothing of a burst before it counts as gone: a frame of the slowest view, read
  /// back, comes within a few seconds.
  const SILENCE: Duration = Duration::from_secs(60);

  /// A writer of `burst` into `folder`, which is there.
  pub fn new(folder: PathBuf, burst: &RenderBurst) -> Self {
    Self {
      outcome: BurstOutcome {
        folder: folder.clone(),
        ..BurstOutcome::default()
      },
      folder,
      samples: burst.samples.unwrap_or(1),
      kind: if burst.samples.is_some() { "reference" } else { "frame" },
      sum: None,
    }
  }

  /// Writes each frame `events` tells of as it comes, until the burst finishes; answers what was written.
  ///
  /// # Errors
  ///
  /// Returns an error when the burst stops, the renderer tells nothing for `SILENCE`, its frames come out of order, or
  /// one cannot be written.
  pub fn receive(mut self, events: &Receiver<BurstEvent>) -> TauriResult<BurstOutcome> {
    loop {
      match events.recv_timeout(Self::SILENCE) {
        Ok(BurstEvent::Shot {
          index,
          sample,
          time,
          picture,
          marks,
        }) => self.take((index, sample.unwrap_or(0), time), &picture, &marks)?,
        Ok(BurstEvent::Finished) => return Ok(self.outcome),
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

  /// Adds a frame's sample to its sum, the first starting it, and writes the frame once its last is in.
  fn take(
    &mut self,
    (index, sample, time): (u32, u32, f32),
    picture: &RenderCapture,
    marks: &BurstMarks,
  ) -> TauriResult {
    let expected: (u32, u32) = self
      .sum
      .as_ref()
      .map_or_else(|| (self.outcome.times.len() as u32, 0), |sum| (sum.index, sum.samples));

    if (index, sample) != expected {
      return Err(format!(
        "The burst's frame {index} sample {sample} came back where frame {} sample {} was due",
        expected.0, expected.1
      ));
    }

    let sum: &mut BurstSum = self.sum.get_or_insert_with(|| BurstSum {
      index,
      time,
      size: (picture.width, picture.height),
      colour: vec![0; picture.pixels.len()],
      marks_size: (marks.width, marks.height),
      marks: vec![[0; 3]; marks.marks.len()],
      samples: 0,
    });

    if sum.size != (picture.width, picture.height) || sum.marks_size != (marks.width, marks.height) {
      return Err(format!("The burst's frame {index} changed size between its samples"));
    }

    for (total, value) in sum.colour.iter_mut().zip(&picture.pixels) {
      *total += u32::from(*value);
    }

    for (counts, texel) in sum.marks.iter_mut().zip(&marks.marks) {
      for (count, mark) in counts.iter_mut().zip(Self::MARKS) {
        *count += u16::from(texel & mark != 0);
      }
    }

    sum.samples += 1;

    match self.sum.take_if(|sum| sum.samples == self.samples) {
      Some(sum) => self.write(sum),
      None => Ok(()),
    }
  }

  /// Writes a frame's mean over its samples and its marks' shares, and notes it in the outcome.
  fn write(&mut self, sum: BurstSum) -> TauriResult {
    let BurstSum {
      index,
      time,
      size: (width, height),
      colour,
      marks_size: (marks_width, marks_height),
      marks,
      samples,
    } = sum;
    let mean = |total: u32| ((total + samples / 2) / samples) as u8;
    let picture: RgbaImage = RgbaImage::from_raw(width, height, colour.into_iter().map(mean).collect())
      .ok_or_else(|| format!("The burst's frame {index} does not fill its size"))?;
    let shares: Vec<u8> = marks
      .into_iter()
      .flat_map(|[plant, grass, moved]| {
        [plant, grass, moved, samples as u16].map(|count| ((u32::from(count) * 255 + samples / 2) / samples) as u8)
      })
      .collect();
    let marks: RgbaImage = RgbaImage::from_raw(marks_width, marks_height, shares)
      .ok_or_else(|| format!("The burst's frame {index} marks do not fill their size"))?;
    CaptureFile::write_png(
      &picture,
      &self.folder.join(format!("{}-{index:03}.png", self.kind)),
      "burst frame",
    )?;
    CaptureFile::write_png(
      &marks,
      &self.folder.join(format!("marks-{index:03}.png")),
      "burst marks",
    )?;

    self.outcome.times.push(time);
    (self.outcome.width, self.outcome.height) = (width, height);
    (self.outcome.marks_width, self.outcome.marks_height) = (marks_width, marks_height);

    Ok(())
  }
}
