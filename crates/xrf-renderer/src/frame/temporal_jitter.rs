use glam::Vec2;

/// Places a pixel's sample cycles through while unscaled: `ffxFsr2GetJitterPhaseCount`'s base.
const BASE_PHASES: f32 = 8.0;

/// Where each frame's samples sit within their pixels while a temporal mode resolves them: Halton (2, 3), as
/// `ffxFsr2GetJitterOffset` places them.
#[derive(Clone, Copy, Debug, Default)]
pub struct TemporalJitter {
  phase: u32,
}

impl TemporalJitter {
  /// The next frame's offset, in drawn pixels about each pixel's centre, `y` down, cycling through
  /// `ffxFsr2GetJitterPhaseCount`'s places for a viewport `ratio` times the drawing's side: eight times its square.
  pub fn next(&mut self, ratio: f32) -> Vec2 {
    let phases: u32 = Self::get_phases(ratio);

    self.phase = (self.phase + 1) % phases;

    Vec2::new(halton(self.phase + 1, 2) - 0.5, halton(self.phase + 1, 3) - 0.5)
  }

  /// How many places the samples cycle through for a viewport `ratio` times the drawing's side.
  pub fn get_phases(ratio: f32) -> u32 {
    ((BASE_PHASES * ratio * ratio) as u32).max(1)
  }
}

/// The Halton sequence's value at an index in a base, in `[0, 1)`.
pub fn halton(index: u32, base: u32) -> f32 {
  let mut fraction: f32 = 1.0;
  let mut result: f32 = 0.0;
  let mut rest: u32 = index;

  while rest > 0 {
    fraction /= base as f32;
    result += fraction * (rest % base) as f32;
    rest /= base;
  }

  result
}
