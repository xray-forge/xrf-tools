use std::sync::Arc;

use crate::decode::decoded_sound::DecodedSound;
use crate::decode::sound_channels::SoundChannels;
use crate::voice::cursor::stereo_frame::StereoFrame;

/// A voice's place in a sound decoded whole.
#[derive(Debug)]
pub(crate) struct DecodedCursor {
  sound: Arc<DecodedSound>,
  /// The source frame the next output frame is read at, between frames when the rates differ.
  position: f64,
  is_looped: bool,
}

impl DecodedCursor {
  pub(crate) fn new(sound: Arc<DecodedSound>, start_frame: u64, is_looped: bool) -> Self {
    let position: f64 = start_frame.min(sound.get_frames_count() as u64) as f64;

    Self {
      sound,
      position,
      is_looped,
    }
  }

  pub(crate) fn get_sample_rate(&self) -> u32 {
    self.sound.get_sample_rate()
  }

  pub(crate) fn get_channels(&self) -> SoundChannels {
    self.sound.get_channels()
  }

  /// The source frame playing now.
  pub(crate) fn get_frame(&self) -> u64 {
    self.position as u64
  }

  /// Writes the next frames, `step` source frames apart, into `out` in the sound's own channels, leaving frames past a
  /// sound played once as they were. Answers `false` once the sound has played to its end without looping.
  // todo: resample block-wise, with a fast path for a sound at the output rate and a mono one interpolated once rather
  // than in both channels, and skip a voice whose gain has settled at zero: 1024 voices all playing take about 40% of a
  // buffer's time here, a silent one as much as a heard one, too much for Extended's 1024 targets beside the rest of a
  // frame's work.
  pub(crate) fn render(&mut self, out: &mut [f32], step: f64) -> bool {
    let samples: &[f32] = self.sound.get_samples();
    let channels: usize = self.sound.get_channels().get_count();
    let frames: usize = self.sound.get_frames_count();
    let length: f64 = frames as f64;

    if frames == 0 || self.position >= length {
      return false;
    }

    for at in 0..out.len() / channels {
      let index: usize = self.position as usize;
      let t: f32 = (self.position - index as f64) as f32;
      let frame: StereoFrame = if index >= 1 && index + 2 < frames {
        StereoFrame::hermite(
          StereoFrame::read(samples, channels, index - 1),
          StereoFrame::read(samples, channels, index),
          StereoFrame::read(samples, channels, index + 1),
          StereoFrame::read(samples, channels, index + 2),
          t,
        )
      } else {
        StereoFrame::hermite(
          self.read_around(index, -1),
          self.read_around(index, 0),
          self.read_around(index, 1),
          self.read_around(index, 2),
          t,
        )
      };

      frame.write(out, channels, at);
      self.position += step;

      if self.position >= length {
        if !self.is_looped {
          return false;
        }

        self.position %= length;
      }
    }

    true
  }

  /// The frame `offset` from `index`, across the loop seam for a looped sound and silence past either end otherwise.
  fn read_around(&self, index: usize, offset: isize) -> StereoFrame {
    let frames: isize = self.sound.get_frames_count() as isize;
    let at: isize = index as isize + offset;
    let at: Option<isize> = if (0..frames).contains(&at) {
      Some(at)
    } else if self.is_looped {
      Some(at.rem_euclid(frames))
    } else {
      None
    };

    at.map_or(StereoFrame::SILENCE, |at| {
      StereoFrame::read(
        self.sound.get_samples(),
        self.sound.get_channels().get_count(),
        at as usize,
      )
    })
  }
}
