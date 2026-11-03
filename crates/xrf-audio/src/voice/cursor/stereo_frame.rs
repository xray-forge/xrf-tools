use std::ops::{Add, Mul, Sub};

/// One frame of one or two channels: a mono frame carries its sample in both.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StereoFrame {
  pub(crate) left: f32,
  pub(crate) right: f32,
}

impl StereoFrame {
  pub(crate) const SILENCE: Self = Self { left: 0.0, right: 0.0 };

  /// Frame `index` of interleaved `samples` of one or two channels, a mono sample heard in both.
  pub(crate) fn read(samples: &[f32], channels: usize, index: usize) -> Self {
    if channels == 1 {
      let sample: f32 = samples[index];

      Self {
        left: sample,
        right: sample,
      }
    } else {
      Self {
        left: samples[index * 2],
        right: samples[index * 2 + 1],
      }
    }
  }

  /// Writes the frame as frame `index` of interleaved `samples` of one or two channels.
  #[inline]
  pub(crate) fn write(self, samples: &mut [f32], channels: usize, index: usize) {
    if channels == 1 {
      samples[index] = self.left;
    } else {
      samples[index * 2] = self.left;
      samples[index * 2 + 1] = self.right;
    }
  }

  /// [`hermite`] in each channel.
  pub(crate) fn hermite(previous: Self, current: Self, next: Self, after: Self, t: f32) -> Self {
    Self {
      left: hermite(previous.left, current.left, next.left, after.left, t),
      right: hermite(previous.right, current.right, next.right, after.right, t),
    }
  }
}

/// The four-point, third-order Hermite curve through `current` and `next` at `t` in `[0, 1)`, steered by the samples
/// either side: smooth enough for the engines' 22 and 44.1 kHz sounds on a 48 kHz device, cheap enough per voice.
#[inline]
pub(crate) fn hermite(previous: f32, current: f32, next: f32, after: f32, t: f32) -> f32 {
  let slope: f32 = (next - previous) * 0.5;
  let curve: f32 = previous - current * 2.5 + next * 2.0 - after * 0.5;
  let cubic: f32 = (after - previous) * 0.5 + (current - next) * 1.5;

  ((cubic * t + curve) * t + slope) * t + current
}

impl Add for StereoFrame {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      left: self.left + other.left,
      right: self.right + other.right,
    }
  }
}

impl Sub for StereoFrame {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      left: self.left - other.left,
      right: self.right - other.right,
    }
  }
}

impl Mul<f32> for StereoFrame {
  type Output = Self;

  fn mul(self, gain: f32) -> Self {
    Self {
      left: self.left * gain,
      right: self.right * gain,
    }
  }
}
