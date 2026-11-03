/// A linear gain moved to each new target over a fixed number of frames, so a change never clicks.
///
/// A gain below zero or not a number is taken as silence, so a bad volume can never put noise on the device.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GainRamp {
  value: f32,
  target: f32,
  step: f32,
  frames_left: u32,
}

impl GainRamp {
  pub(crate) fn new(value: f32) -> Self {
    let value: f32 = Self::to_gain(value);

    Self {
      value,
      target: value,
      step: 0.0,
      frames_left: 0,
    }
  }

  /// Moves towards `target` over `frames` frames, from wherever the ramp is now; zero frames jumps.
  pub(crate) fn set_target(&mut self, target: f32, frames: u32) {
    let target: f32 = Self::to_gain(target);

    self.target = target;

    if frames == 0 {
      self.value = target;
      self.frames_left = 0;
    } else {
      self.step = (target - self.value) / frames as f32;
      self.frames_left = frames;
    }
  }

  pub(crate) fn get_target(&self) -> f32 {
    self.target
  }

  fn to_gain(gain: f32) -> f32 {
    if gain.is_finite() { gain.max(0.0) } else { 0.0 }
  }

  /// Whether the ramp has reached its target.
  pub(crate) fn is_settled(&self) -> bool {
    self.frames_left == 0
  }

  /// The gain for the next frame.
  #[inline]
  pub(crate) fn next(&mut self) -> f32 {
    if self.frames_left > 0 {
      self.frames_left -= 1;
      self.value = if self.frames_left == 0 {
        self.target
      } else {
        self.value + self.step
      };
    }

    self.value
  }
}
