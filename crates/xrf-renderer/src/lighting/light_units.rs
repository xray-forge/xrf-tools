use glam::Vec3;

use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::light_unit::PHYSICAL_LIGHT_UNIT;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::authored_colour;

/// The cd/m² a light of one in the engine's units stands for, the weather's and the local lights' apart: one each under
/// the engine's light; the physical light's unit for both at the authored levels; at the physical levels the weather's
/// dimmed with its sky's tint towards a real night and the local lights' brought to real lamps'.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightUnits {
  /// The sun's, the sky's, the hemisphere's and the fog's.
  pub weather: f32,
  /// The lamps' and the self-lit surfaces'.
  pub local: f32,
}

impl LightUnits {
  /// The engine's light, whose values have no units.
  pub const NONE: Self = Self {
    weather: 1.0,
    local: 1.0,
  };
  /// The local lights' share of the unit at the physical levels: an engine lamp's 70 000 to 270 000 cd, from its range
  /// (`LIGHT_THIRD_SHARE`), brought to a real lamp's 70 to 270.
  pub const PHYSICAL_LOCAL: f32 = 1e-3;
  /// The luminance of the sky's tint at Anomaly's clear noon (0.65, decoded), at and above which the weather's light
  /// stands at the unit.
  pub const DAYLIGHT_SKY: f32 = 0.38;
  /// How steeply the weather's light falls with its sky's tint below daylight: the authored full moon (0.09 at a sky of
  /// 0.2) comes to 0.27 lx, a real one's, and dusk (a sky of 0.4-0.5) to a thirtieth of the day.
  pub const NIGHT_POWER: f32 = 3.3;

  /// The units a view's light is in under `lighting`.
  pub fn of(options: &RenderViewOptions, lighting: &RenderLighting) -> Self {
    if !options.is_light_physical() {
      return Self::NONE;
    }

    match options.features.light_level {
      RenderLightLevel::Authored => Self {
        weather: PHYSICAL_LIGHT_UNIT,
        local: PHYSICAL_LIGHT_UNIT,
      },
      RenderLightLevel::Physical => Self {
        weather: PHYSICAL_LIGHT_UNIT * Self::get_night_share(lighting.sky.color),
        local: PHYSICAL_LIGHT_UNIT * Self::PHYSICAL_LOCAL,
      },
    }
  }

  /// The share of the day's light a weather's sky tint leaves its light at the physical levels.
  pub fn get_night_share(sky: Vec3) -> f32 {
    let tint: f32 = authored_colour(sky, true).dot(Vec3::new(0.2126, 0.7152, 0.0722));

    (tint / Self::DAYLIGHT_SKY).min(1.0).powf(Self::NIGHT_POWER)
  }
}
