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
  ///
  /// It works in runs: a sound at the output rate is copied; elsewhere the frames whose four neighbours lie inside the
  /// sound are interpolated without a bounds check, a mono sound once rather than in both channels, and only the few at
  /// its edges read around them.
  pub(crate) fn render(&mut self, out: &mut [f32], step: f64) -> bool {
    let channels: usize = self.sound.get_channels().get_count();
    let frames: usize = self.sound.get_frames_count();
    let wanted: usize = out.len() / channels;
    let mut at: usize = 0;

    if frames == 0 || self.position >= frames as f64 {
      return false;
    }

    while at < wanted {
      at += if step == 1.0 && self.position.fract() == 0.0 {
        self.copy(&mut out[at * channels..wanted * channels], channels)
      } else {
        self.interpolate(&mut out[at * channels..wanted * channels], channels, step)
      };

      if !self.wrap() {
        return false;
      }
    }

    true
  }

  /// Moves on `frames` output frames, `step` source frames apart, rendering nothing: a voice no one hears. Answers
  /// `false` once the sound has played to its end without looping.
  pub(crate) fn skip(&mut self, frames: usize, step: f64) -> bool {
    if self.sound.get_frames_count() == 0 {
      return false;
    }

    self.position += frames as f64 * step;

    self.wrap()
  }

  /// Brings a position past the end back into a looped sound, answering whether the sound plays on.
  fn wrap(&mut self) -> bool {
    let length: f64 = self.sound.get_frames_count() as f64;

    if self.position < length {
      return true;
    }

    if !self.is_looped {
      return false;
    }

    self.position %= length;

    true
  }

  /// Copies frames from the playing one at the output rate, up to the sound's end, answering how many.
  fn copy(&mut self, out: &mut [f32], channels: usize) -> usize {
    let index: usize = self.position as usize;
    let count: usize = (self.sound.get_frames_count() - index).min(out.len() / channels);

    out[..count * channels].copy_from_slice(&self.sound.get_samples()[index * channels..(index + count) * channels]);
    self.position += count as f64;

    count
  }

  /// Interpolates frames from the playing one, answering how many: a run of inner frames where there is one, else
  /// one frame at the sound's edge, at least one, and none past its end.
  fn interpolate(&mut self, out: &mut [f32], channels: usize, step: f64) -> usize {
    let samples: &[f32] = self.sound.get_samples();
    let frames: usize = self.sound.get_frames_count();
    let wanted: usize = out.len() / channels;
    // The last place whose frame and its next two lie inside the sound, so `index + 2 < frames`.
    let inner_end: f64 = frames.saturating_sub(2) as f64;
    let inner: usize = if self.position >= 1.0 && self.position < inner_end {
      (((inner_end - self.position) / step) as usize).min(wanted)
    } else {
      0
    };

    if inner == 0 {
      let index: usize = self.position as usize;
      let t: f32 = (self.position - index as f64) as f32;

      StereoFrame::hermite(
        self.read_around(index, -1),
        self.read_around(index, 0),
        self.read_around(index, 1),
        self.read_around(index, 2),
        t,
      )
      .write(out, channels, 0);
      self.position += step;

      return 1;
    }

    let mut position: f64 = self.position;

    if channels == 1 {
      for slot in &mut out[..inner] {
        let index: usize = position as usize;
        let window: &[f32] = &samples[index - 1..index + 3];

        *slot = StereoFrame::hermite_sample(
          window[0],
          window[1],
          window[2],
          window[3],
          (position - index as f64) as f32,
        );
        position += step;
      }
    } else {
      for pair in out[..inner * 2].as_chunks_mut::<2>().0 {
        let index: usize = position as usize;
        let t: f32 = (position - index as f64) as f32;
        let window: &[f32] = &samples[(index - 1) * 2..(index + 3) * 2];

        pair[0] = StereoFrame::hermite_sample(window[0], window[2], window[4], window[6], t);
        pair[1] = StereoFrame::hermite_sample(window[1], window[3], window[5], window[7], t);
        position += step;
      }
    }

    self.position = position;

    inner
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
