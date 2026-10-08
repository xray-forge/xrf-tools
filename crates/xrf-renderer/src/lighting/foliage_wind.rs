use std::f32::consts::FRAC_PI_2;

use glam::Vec4;

use crate::contract::render_foliage_settings::RenderFoliageSettings;
use crate::lighting::render_wind::RenderWind;
use crate::pass::foliage_wind_values::FoliageWindValues;

/// How far the enhanced foliage's flow fields have drifted, summed frame by frame at each frame's wind, so a change of
/// wind changes how fast they move from then on, never where they stand.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FoliageWind {
  /// Drifted across the engine's ground in `xy`, the trunks' clock in `z`, seconds in `w`.
  anim: Vec4,
  previous: Vec4,
  /// The clock the last frame was drawn at.
  clock: Option<f32>,
}

impl FoliageWind {
  /// The share of its strongest a wind's velocity is taken for.
  const VELOCITY_SHARE: f32 = 0.001;
  /// The least velocity the fields drift at, so a near calm does not crawl in slow motion.
  const LEAST_VELOCITY: f32 = 200.0;

  /// Moves the fields on by the seconds since the last frame at this frame's wind, still while `is_windy` is off;
  /// `wetness`, from nothing to one, bends the leaves' normals harder.
  pub fn advance(
    &mut self,
    time: f32,
    settings: &RenderFoliageSettings,
    (wind, is_windy): (RenderWind, bool),
    wetness: f32,
  ) -> FoliageWindValues {
    let elapsed: f32 = self.clock.map_or(0.0, |clock| (time - clock).max(0.0));
    let velocity: f32 = wind
      .velocity
      .max(settings.min_speed / Self::VELOCITY_SHARE)
      .max(Self::LEAST_VELOCITY)
      * Self::VELOCITY_SHARE;
    let turn: f32 = -wind.direction + FRAC_PI_2;
    let (sine, cosine): (f32, f32) = turn.sin_cos();

    self.clock = Some(time);
    self.previous = self.anim;

    if is_windy {
      self.anim += Vec4::new(
        velocity * cosine,
        velocity * sine,
        (velocity * 1.33).clamp(0.0, 1.0),
        1.0,
      ) * elapsed;
    }

    let speed: f32 = if is_windy {
      settings
        .min_speed
        .max((wind.velocity * Self::VELOCITY_SHARE).clamp(0.0, 1.0))
    } else {
      0.0
    };

    FoliageWindValues {
      wind: Vec4::new(cosine, sine, speed, (speed * 1.66).sqrt().clamp(0.0, 1.0)),
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
        settings.is_enhanced() as u32 as f32,
      ),
      anim: self.anim,
      previous_anim: self.previous,
      flora: Vec4::new(0.0, 0.0, wetness, settings.is_enhanced() as u32 as f32),
    }
  }
}
