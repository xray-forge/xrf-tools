use glam::{Vec2, Vec4};

use crate::feature::weather::wet::wet_uniform::WetUniform;
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::view_time::ViewTime;

/// How far gusts have drifted across the puddles and their calm waves have scrolled: each frame's seconds at that
/// frame's gust and rain, summed in `f64` rather than one clock times this frame's speed, so a wind turning or a rain
/// thickening late in a level changes how fast they move from then on, never where they stand, and they run on one
/// clock whether the rain falls or not.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WetDrift {
  /// Metres the gusts have drifted along renderer `x` and `z`.
  gusts: [f64; 2],
  /// How far the calm waves have scrolled, in their map's repeats.
  waves: f64,
}

impl WetDrift {
  /// The calm waves' scroll a second, and what the rain's density adds to it, 1.5 times over and held to one.
  const WAVE_PACE: (f64, f64, f64) = (0.01, 0.008, 1.5);

  /// Moves on by the seconds since the view's last frame, never back, the gusts blowing `gust` (`WetUniform::get_gust`)
  /// under a rain of `density`.
  pub fn advance(&mut self, time: ViewTime, gust: Vec2, density: f32) {
    let step: f64 = time.get_step().map_or(0.0, |step| step.max(0.0));
    let (pace, rain_pace, rain_share): (f64, f64, f64) = Self::WAVE_PACE;

    self.gusts[0] += step * f64::from(gust.x);
    self.gusts[1] += step * f64::from(gust.y);
    self.waves += step * (pace + (f64::from(density) * rain_share).clamp(0.0, 1.0) * rain_pace);
  }

  /// The drift as `WetUniform::drift` holds it: the gusts' within `WetUniform::GUST_REPEAT`, the waves' within one.
  pub fn get(&self) -> Vec4 {
    Vec4::new(
      LevelTime::wrap(self.gusts[0], WetUniform::GUST_REPEAT),
      LevelTime::wrap(self.gusts[1], WetUniform::GUST_REPEAT),
      LevelTime::wrap(self.waves, 1.0),
      0.0,
    )
  }
}
