use std::f32::consts::TAU;

use glam::Vec4;

use crate::lighting::grass_swing::GrassSwing;
use crate::pass::grass_wind_uniform::GrassWindUniform;

/// `wind_strength_factor` on a still level: with no gusts the weather's noise is zero, so it is a half.
const STRENGTH: f32 = 0.5;

/// What a frame's step is taken as where the time ran back or leapt past a second, as `hw_Render` takes it.
const STEP_FALLBACK: f32 = 0.03;

/// The first wave's direction through the level, over a turn (`CDetailManager::hw_Render`).
const FIRST_WAVE: [f32; 3] = [1.0 / 5.0, 1.0 / 7.0, 1.0 / 3.0];

/// The second wave's, the same components in another order.
const SECOND_WAVE: [f32; 3] = [1.0 / 3.0, 1.0 / 7.0, 1.0 / 5.0];

/// The grass's sway as `CDetailManager::hw_Render` builds it each frame: two winds turning at their own rates and a wave
/// running through the level, the normal and fast swings mixed by the wind's strength, each advanced by the frame's
/// step so a change of strength turns them no faster at once.
#[derive(Debug, Default)]
pub struct GrassWind {
  /// `m_time_rot_1`, `m_time_rot_2` and `m_time_pos`.
  first_turn: f32,
  second_turn: f32,
  phase: f32,
  /// `m_global_time_old`.
  time: Option<f32>,
}

impl GrassWind {
  /// The sway at a time, in seconds; still where the wind does not blow.
  pub fn advance(&mut self, time: f32, is_windy: bool) -> GrassWindUniform {
    let elapsed: f32 = time - self.time.unwrap_or(time);
    let step: f32 = if (0.0..=1.0).contains(&elapsed) {
      elapsed
    } else {
      STEP_FALLBACK
    };

    self.time = Some(time);

    if !is_windy {
      return GrassWindUniform::default();
    }

    let swing: GrassSwing = GrassSwing::NORMAL.mix(&GrassSwing::FAST, STRENGTH);
    let turning = |seconds: f32| if seconds > 0.0 { TAU * step / seconds } else { 0.0 };

    self.first_turn += turning(swing.rot1);
    self.second_turn += turning(swing.rot2);
    self.phase += step * swing.speed;

    let wave = |direction: [f32; 3]| Vec4::new(direction[0], direction[1], direction[2], self.phase) / TAU;

    GrassWindUniform {
      wind_1: Vec4::new(self.first_turn.sin(), 0.0, self.first_turn.cos(), 0.0) * swing.amp1,
      wind_2: Vec4::new(self.second_turn.sin(), 0.0, self.second_turn.cos(), 0.0) * swing.amp2,
      wave_1: wave(FIRST_WAVE),
      wave_2: wave(SECOND_WAVE),
    }
  }
}
