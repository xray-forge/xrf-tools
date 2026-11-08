use std::f32::consts::TAU;

use glam::Vec4;

use crate::feature::scene::grass::draw::grass_wind_uniform::GrassWindUniform;
use crate::lighting::wind::foliage_motion::FoliageMotion;
use crate::lighting::wind::grass_swing::GrassSwing;
use crate::scene::level::view::view_time::ViewTime;

/// What a frame's step is taken as where the time ran back or leapt past a second, as `hw_Render` takes it.
const STEP_FALLBACK: f32 = 0.03;

/// The first wave's direction through the level, over a turn (`CDetailManager::hw_Render`).
const FIRST_WAVE: [f32; 3] = [1.0 / 5.0, 1.0 / 7.0, 1.0 / 3.0];

/// The second wave's, the same components in another order.
const SECOND_WAVE: [f32; 3] = [1.0 / 3.0, 1.0 / 7.0, 1.0 / 5.0];

/// The grass's sway as `CDetailManager::hw_Render` builds it each frame: two winds turning at their own rates and a
/// wave running through the level, the normal and fast swings mixed by `wind_strength_factor`, each advanced by the
/// frame's step so a change of strength turns them no faster at once. Each is kept within a turn, as only its sine,
/// cosine or `cyclic` is read, so it moves as finely after a day as in a level's first minute.
#[derive(Debug, Default)]
pub(crate) struct GrassWind {
  /// `m_time_rot_1`, `m_time_rot_2` and `m_time_pos`.
  first_turn: f32,
  second_turn: f32,
  phase: f32,
  /// The sway the last frame drew with.
  last: Option<GrassWindUniform>,
}

impl GrassWind {
  /// The sway at a time at the wind's strength; still where the grass is held still.
  pub(crate) fn advance(&mut self, time: ViewTime, motion: FoliageMotion, strength: f32) -> GrassWindUniform {
    let wind: GrassWindUniform = self.turn(time, motion, strength).following(self.last.as_ref());

    self.last = Some(wind);

    wind
  }

  fn turn(&mut self, time: ViewTime, motion: FoliageMotion, strength: f32) -> GrassWindUniform {
    // The step since the view's last frame, `m_global_time_old`; one back or past a second is the fallback, as
    // `hw_Render` takes either.
    let step: f32 = match time.get_step() {
      None => 0.0,
      Some(step) if (0.0..=1.0).contains(&step) => step as f32,
      Some(_) => STEP_FALLBACK,
    };

    if motion == FoliageMotion::Still {
      return GrassWindUniform::default();
    }

    let swing: GrassSwing = GrassSwing::NORMAL.mix(&GrassSwing::FAST, strength);
    let turning = |seconds: f32| if seconds > 0.0 { TAU * step / seconds } else { 0.0 };

    self.first_turn = (self.first_turn + turning(swing.rot1)).rem_euclid(TAU);
    self.second_turn = (self.second_turn + turning(swing.rot2)).rem_euclid(TAU);
    // The wave's phase over a turn, which `cyclic` repeats over once it is taken as turns below.
    self.phase = (self.phase + step * swing.speed).rem_euclid(TAU);

    let wave = |direction: [f32; 3]| Vec4::new(direction[0], direction[1], direction[2], self.phase) / TAU;

    GrassWindUniform {
      wind_1: Vec4::new(self.first_turn.sin(), 0.0, self.first_turn.cos(), 0.0) * swing.amp1,
      wind_2: Vec4::new(self.second_turn.sin(), 0.0, self.second_turn.cos(), 0.0) * swing.amp2,
      wave_1: wave(FIRST_WAVE),
      wave_2: wave(SECOND_WAVE),
      ..Default::default()
    }
  }
}
