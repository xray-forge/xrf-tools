use glam::Vec4;

use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::light_unit::PHYSICAL_LIGHT_UNIT;
use crate::lighting::light_units::LightUnits;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::lighting_frame::LightingFrame;

/// The share of the frame's exposure the game's pictures (the sky, the clouds, the fog's colour) are shown at, so they
/// show as the engine's curve shows them at the authored levels whatever lifts the frame's light.
///
/// The game paints its sky and fog at their own levels against the ground's. The authored levels' exposure shows them
/// as the engine's display does, the whole of it. The physical levels lift a night's land towards being readable, many
/// stops past the authored law, which would blow a sky's eight-bit night picture out into its steps: the pictures are
/// shown at the exposure the authored law gives the frame's land in the authored levels' units instead. A transform's
/// own lift and look are undone where the sky enters the scene (`picture_for_display` in `common/display_transform`),
/// so the share is the same under every display.
///
/// The share is set against the scale the frame's light is written at, the pre-exposure read back with the metering it
/// comes from. At the authored levels the pictures follow the land's exposure as it adapts, so the share is of the
/// frame's exposure. At the physical levels they have an exposure of their own, the law's: shown at their share of the
/// pre-exposure (`is_pre_exposed`), a frame's exposure snapping ahead of its read-back copy (the first frame, a cut)
/// cannot carry the sky with it for the frames until the copy is back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PictureShares {
  pub share: f32,
  /// Whether the share is of the pre-exposure rather than of the frame's exposure.
  pub is_pre_exposed: bool,
}

impl PictureShares {
  /// The whole exposure, under the engine's light and wherever nothing has been metered.
  pub const WHOLE: Self = Self {
    share: 1.0,
    is_pre_exposed: false,
  };

  /// The share for a view's `options`, the weather's `lighting` and what its `frame` metered.
  pub fn of(options: &RenderViewOptions, lighting: &RenderLighting, frame: &LightingFrame) -> Self {
    if !options.is_light_physical() {
      return Self::WHOLE;
    }

    let current: f32 = frame.light_units.weather * frame.pre_exposure.current;
    let is_levels_physical: bool = options.features.light_level == RenderLightLevel::Physical;
    let reference: f32 = match (options.features.light_level, frame.metering.land) {
      (RenderLightLevel::Physical, Some(land)) => {
        // The land as the authored levels would light it: their unit, their key light's gain, in place of the world's.
        let gain: f32 = LightUnits::get_key_gain(lighting.get_sun_color(&options.features.light_scales));
        let authored: f32 = land + (PHYSICAL_LIGHT_UNIT * gain / frame.light_units.weather).log2();

        PHYSICAL_LIGHT_UNIT * ExposureHistogramUniform::get_law_scale(options.features.exposure.compensation, authored)
      }
      _ => current,
    };
    let share: f32 = reference / current.max(f32::MIN_POSITIVE);

    Self {
      share: if share.is_finite() { share } else { 1.0 },
      is_pre_exposed: is_levels_physical,
    }
  }

  /// As `LightingUniform::pictures` holds it: the share, then one where it is of the pre-exposure.
  pub fn to_vec4(self) -> Vec4 {
    Vec4::new(self.share, f32::from(u8::from(self.is_pre_exposed)), 0.0, 0.0)
  }
}
