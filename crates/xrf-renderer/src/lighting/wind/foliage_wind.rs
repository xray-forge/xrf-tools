use std::f32::consts::FRAC_PI_2;

use glam::{DVec4, Vec4};

use crate::contract::feature::scene::render_foliage_settings::RenderFoliageSettings;
use crate::feature::scene::wind::foliage_wind_values::FoliageWindValues;
use crate::lighting::wind::foliage_motion::FoliageMotion;
use crate::lighting::wind::render_wind::RenderWind;
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::view_time::ViewTime;

/// How far the enhanced foliage's flow fields have drifted, summed frame by frame at each frame's wind, so a change of
/// wind changes how fast they move from then on, never where they stand; summed in `f64`, so a frame's few milliseconds
/// still add to a day's sum.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FoliageWind {
  /// Drifted across the engine's ground in `xy`, the trunks' clock in `z`, seconds in `w`.
  anim: DVec4,
  previous: DVec4,
  /// The wind the last frame was drawn in.
  wind: Option<Vec4>,
}

impl FoliageWind {
  /// The share of its strongest a wind's velocity is taken for.
  const VELOCITY_SHARE: f32 = 0.001;
  /// The least velocity the fields drift at, so a near calm does not crawl in slow motion.
  const LEAST_VELOCITY: f32 = 200.0;

  /// Moves the fields on by the seconds since the view's last frame, never back, at this frame's `wind`, as far as `motion` lets them;
  /// `wetness`, from nothing to one, bends the leaves' normals harder.
  pub(crate) fn advance(
    &mut self,
    time: ViewTime,
    settings: &RenderFoliageSettings,
    wind: RenderWind,
    motion: FoliageMotion,
    wetness: f32,
  ) -> FoliageWindValues {
    let is_windy: bool = motion == FoliageMotion::Blowing;
    let elapsed: f64 = time.get_advance();
    let velocity: f32 = wind
      .velocity
      .max(settings.min_speed / Self::VELOCITY_SHARE)
      .max(Self::LEAST_VELOCITY)
      * Self::VELOCITY_SHARE;
    let turn: f32 = -wind.direction + FRAC_PI_2;
    let (sine, cosine): (f32, f32) = turn.sin_cos();

    self.previous = self.anim;

    if is_windy {
      self.anim += Vec4::new(
        velocity * cosine,
        velocity * sine,
        (velocity * 1.33).clamp(0.0, 1.0),
        1.0,
      )
      .as_dvec4()
        * elapsed;
    }

    let speed: f32 = if is_windy {
      settings
        .min_speed
        .max((wind.velocity * Self::VELOCITY_SHARE).clamp(0.0, 1.0))
    } else {
      0.0
    };

    let blowing: Vec4 = Vec4::new(cosine, sine, speed, (speed * 1.66).sqrt().clamp(0.0, 1.0));

    FoliageWindValues {
      wind: blowing,
      previous_wind: self.wind.replace(blowing).unwrap_or(blowing),
      grass: Vec4::new(
        settings.grass_speed,
        settings.grass_turbulence,
        settings.grass_push,
        settings.grass_wave,
      ),
      trees: Vec4::new(
        settings.trees_speed,
        settings.trees_trunk,
        settings.trees_bend,
        u32::from(settings.is_enhanced()) as f32,
      ),
      anim: Self::to_anim(self.anim, settings.trees_trunk),
      previous_anim: Self::to_anim(self.previous, settings.trees_trunk),
      flora: Vec4::new(0.0, 0.0, wetness, u32::from(settings.is_enhanced()) as f32),
    }
  }

  /// The drift as `shaders/common/foliage_wind.wgsl` reads it: the trunks' clock taken within the turn their swing
  /// repeats over at `trunk_speed`, `cos(phase) * sin(phase * 5)` with the phase the clock times that speed.
  fn to_anim(anim: DVec4, trunk_speed: f32) -> Vec4 {
    let swing: f64 = std::f64::consts::TAU / f64::from(trunk_speed);

    // todo: Keep the flow fields' drift in `xy` precise past some hours of wind: the gradient noise it moves through
    //  has no repeat to take it within, so it reaches the shader as an `f32` that steps once a frame's drift falls
    //  under its last place; a periodic lattice with each read's own wrapped offset would.
    Vec4::new(
      anim.x as f32,
      anim.y as f32,
      LevelTime::wrap(anim.z, swing),
      anim.w as f32,
    )
  }
}
