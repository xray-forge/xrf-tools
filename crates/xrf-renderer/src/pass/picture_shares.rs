use glam::Vec4;

use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::light_unit::PHYSICAL_LIGHT_UNIT;
use crate::lighting::light_units::LightUnits;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::display_mode::DisplayMode;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::exposure_metering::ExposureMetering;
use crate::pass::lighting_frame::LightingFrame;

/// The shares of the frame's exposure the game's pictures are shown at, so they show as the engine's curve shows them at
/// the authored levels whatever lifts the frame's light: the sky's and the clouds', then the fog's colour's.
///
/// The game paints its sky and fog at their own levels against the ground's. The authored levels' exposure shows them
/// as the engine's display does, one each. The physical levels lift a night's land towards being readable, many stops
/// past the authored law, which would blow a sky's eight-bit night picture out into its steps: both pictures are shown
/// at the exposure the authored law gives the frame's land in the authored levels' units instead. A transform anchored
/// on the land (`DisplayMode::get_anchor_stops`) lifts the sky by the land's stops, which AgX's toe makes several at
/// dusk: the sky's share trades them for its own at its metered level and bright end (`DisplayMode::get_sky_lift`),
/// where the fog's keeps the land's, as it veils the land.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PictureShares {
  pub sky: f32,
  pub fog: f32,
}

impl PictureShares {
  /// The whole exposure, under the engine's light and wherever nothing has been metered.
  pub const WHOLE: Self = Self { sky: 1.0, fog: 1.0 };

  /// The shares for a view's `options`, the weather's `lighting` and what its `frame` metered.
  pub fn of(options: &RenderViewOptions, lighting: &RenderLighting, frame: &LightingFrame) -> Self {
    if !options.is_light_physical() {
      return Self::WHOLE;
    }

    let current: f32 = frame.light_units.weather * frame.pre_exposure.current;
    let reference: f32 = match (options.features.light_level, frame.metering.land) {
      (RenderLightLevel::Physical, Some(land)) => {
        // The land as the authored levels would light it: their unit, their key light's gain, in place of the world's.
        let gain: f32 = LightUnits::get_key_gain(lighting.get_sun_color(&options.features.light_scales));
        let authored: f32 = land + (PHYSICAL_LIGHT_UNIT * gain / frame.light_units.weather).log2();

        PHYSICAL_LIGHT_UNIT * ExposureHistogramUniform::get_law_scale(options.features.exposure.compensation, authored)
      }
      _ => current,
    };
    let fog: f32 = reference / current.max(f32::MIN_POSITIVE);
    let mode: DisplayMode = DisplayMode::of(options);
    let contrast: f32 = options.features.grading.contrast;
    // The grading raises the light the shares scale to the contrast's power, so the stops traded are divided by it.
    let metering: ExposureMetering = frame.metering;
    let stops: f32 = metering.sky.map_or(0.0, |sky| {
      let sky_lift: f32 = mode.get_sky_lift(
        (
          sky.exp2() * reference,
          metering.sky_bright.map(|bright| bright.exp2() * reference),
        ),
        contrast,
      );

      (sky_lift - mode.get_lift((metering.key, metering.bright), contrast)) / contrast.max(0.05)
    });
    let shares: Self = Self {
      sky: fog * stops.exp2(),
      fog,
    };

    if shares.sky.is_finite() && shares.fog.is_finite() {
      shares
    } else {
      Self::WHOLE
    }
  }

  /// As `LightingUniform::pictures` holds them.
  pub fn to_vec4(self) -> Vec4 {
    Vec4::new(self.sky, self.fog, 0.0, 0.0)
  }
}
