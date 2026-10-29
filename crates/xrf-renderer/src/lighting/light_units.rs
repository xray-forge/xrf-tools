use glam::Vec3;

use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::daylight::Daylight;
use crate::lighting::light_unit::PHYSICAL_LIGHT_UNIT;
use crate::lighting::luminance::luminance;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::{authored_colour, authored_light};
use crate::pass::sky_harmonics::SkyHarmonics;

/// The cd/m² a light of one in the engine's units stands for, the weather's and the local lights' apart, and the gain
/// the lights take: one each under the engine's light; at the authored levels the physical light's unit for both, every
/// light by the key light's gain; at the anchored levels the unit with no gain, each light decoded where it is anchored
/// (`anchored_light`); at the physical levels the weather's set so its sky lights a level surface as the
/// world's does (`Daylight`), and the local lights' brought to real lamps'.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightUnits {
  /// The sky's, the clouds', the hemisphere's, the ambient's and the fog's, and at the authored levels the sun's.
  pub weather: f32,
  /// The lamps' and the self-lit surfaces'.
  pub local: f32,
  /// What the lights (the weather's sun, ambient and hemisphere colour and the lamps, kept at their strength by
  /// `authored_light`) take on top of their unit: the key light's gain at the authored levels, else one. The pictures
  /// (the sky, the clouds, the fog, the self-lit surfaces) are decoded as they are shown and take none.
  pub light_gain: f32,
}

impl LightUnits {
  /// The engine's light, whose values have no units.
  pub const NONE: Self = Self {
    weather: 1.0,
    local: 1.0,
    light_gain: 1.0,
  };
  /// The local lights' share of the unit at the physical levels: an engine lamp's 130 000 to 500 000 cd, from its range
  /// (`LIGHT_ENERGY_SHARE`), brought to a real lamp's 130 to 500.
  pub const PHYSICAL_LOCAL: f32 = 1e-3;
  /// The least gain the weather's key light gives: the sRGB encoding's slope at black, under which a darkening key
  /// would take the night's fill with it.
  pub const LEAST_KEY_GAIN: f32 = 1.0 / 12.92;

  /// The units a view's light is in under `lighting`, its sky's irradiance `harmonics` as last projected and the
  /// world's light where the levels are physical.
  pub fn of(
    options: &RenderViewOptions,
    lighting: &RenderLighting,
    harmonics: &SkyHarmonics,
    daylight: Option<&Daylight>,
  ) -> Self {
    if !options.is_light_physical() {
      return Self::NONE;
    }

    match (options.features.light_level, daylight) {
      (RenderLightLevel::Physical, Some(daylight)) => {
        let zenith: f32 = harmonics.get_zenith_irradiance(options.features.light_scales.hemi);

        Self {
          // A sky not yet projected lights at the unit until it is.
          weather: if zenith > 0.0 {
            daylight.sky / (std::f32::consts::PI * zenith)
          } else {
            PHYSICAL_LIGHT_UNIT
          },
          local: PHYSICAL_LIGHT_UNIT * Self::PHYSICAL_LOCAL,
          light_gain: 1.0,
        }
      }
      (RenderLightLevel::Anchored, _) => Self {
        weather: PHYSICAL_LIGHT_UNIT,
        local: PHYSICAL_LIGHT_UNIT,
        light_gain: 1.0,
      },
      _ => Self {
        weather: PHYSICAL_LIGHT_UNIT,
        local: PHYSICAL_LIGHT_UNIT,
        light_gain: Self::get_key_gain(lighting.get_sun_color(&options.features.light_scales)),
      },
    }
  }

  /// The gain the authored levels give the lights: what decoding the key light (the weather's sun, by night its moon)
  /// does to its luminance, decoded over kept. The lights keep their strengths against each other, as the game
  /// balanced them (`authored_light`), so a night keeps the fill the game gives it and a lamp its pool, and the day's
  /// range follows the key light's decode, as the engine's display shows it: the key light itself stands at its
  /// decoded luminance, and a moon twenty times dimmer than the sun in display values shows hundreds of times darker.
  /// A key gone dark, between the sun's last keyframe and the moon's first, gives no less than `LEAST_KEY_GAIN`.
  pub fn get_key_gain(key: Vec3) -> f32 {
    let kept: f32 = luminance(authored_light(key, true));

    if kept <= 0.0 {
      return Self::LEAST_KEY_GAIN;
    }

    (luminance(authored_colour(key, true)) / kept).max(Self::LEAST_KEY_GAIN)
  }
}
