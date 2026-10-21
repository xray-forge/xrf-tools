use glam::{Vec3, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_light_level::RenderLightLevel;

/// The sky's irradiance as order-2 spherical harmonics, as `shaders/frame/sky_irradiance.wgsl` writes it: nine
/// coefficients, each in a vector's colour, of the irradiance over π in the engine's light units, already convolved by
/// the cosine; the first's `w` one once the sky was projected, nothing before; the second to fourth's `w` the authored
/// hemisphere's first coefficient (`hemisphere_environment`, the look's hemisphere scale in it), its level.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "SkyHarmonics")]
pub struct SkyHarmonics {
  pub coefficients: [Vec4; 9],
}

impl SkyHarmonics {
  /// The share of the authored hemisphere's level under which the sky's own picture is taken for dark: a night sky
  /// drawn black, whose shape says nothing.
  const DARK_SKY_SHARE: f32 = 1e-4;
  /// The share of the sky's luminance a channel of its picture is held to when the authored levels take the authored
  /// hemisphere's colour, so a channel the picture all but lacks is not lifted from nothing.
  const LEAST_CHANNEL_SHARE: f32 = 0.05;

  /// The luminance of the sky's irradiance over π a level surface facing up takes, in the engine's units, by the look's
  /// hemisphere scale (`hemi`); none before the sky was projected. The nine harmonics at the zenith
  /// (`common/sky_harmonics`, y up): the first, the second by y, and the third band's two that do not vanish there.
  pub fn get_zenith_irradiance(&self, hemi: f32) -> f32 {
    let [c0, c1, _, _, _, _, c6, _, c8] = self.coefficients;

    if c0.w < 0.5 {
      return 0.0;
    }

    let zenith: Vec3 =
      c0.truncate() * 0.282_095 + c1.truncate() * 0.488_603 - c6.truncate() * 0.315_392 - c8.truncate() * 0.546_274;

    (zenith * hemi).dot(Vec3::new(0.2126, 0.7152, 0.0722)).max(0.0)
  }

  /// The coefficients the lighting carries at the light's `level`, the look's hemisphere scale (`hemi`) in them. At
  /// the physical levels the sky's own; at the authored levels the sky's shape at the authored hemisphere's level and
  /// colour, each channel's own, as the game lights its hemisphere: by day the two levels stand within a few percent,
  /// but a night sky's picture lights a fraction of the hemisphere the game authors and leaves the side away from the
  /// moon black, and a sky's picture tints the shade it lights its own colour, a day's blue grey where the game's
  /// hemisphere is warmer. A sky too dark to have a shape lights evenly at that level.
  pub fn at_level(&self, level: RenderLightLevel, hemi: f32) -> [Vec4; 9] {
    let luminance = |colour: Vec3| colour.dot(Vec3::new(0.2126, 0.7152, 0.0722));
    let [sky, authored]: [Vec3; 2] = [
      self.coefficients[0].truncate() * hemi,
      Vec3::new(self.coefficients[1].w, self.coefficients[2].w, self.coefficients[3].w),
    ];
    let scale: Vec3 = match level {
      RenderLightLevel::Physical => Vec3::splat(hemi),
      RenderLightLevel::Authored if luminance(sky) > luminance(authored) * Self::DARK_SKY_SHARE => {
        hemi * authored / sky.max(Vec3::splat(luminance(sky) * Self::LEAST_CHANNEL_SHARE))
      }
      RenderLightLevel::Authored => {
        return std::array::from_fn(|term| {
          let colour: Vec3 = if term == 0 { authored } else { Vec3::ZERO };

          colour.extend(self.coefficients[term].w)
        });
      }
    };

    self
      .coefficients
      .map(|coefficient| (coefficient.truncate() * scale).extend(coefficient.w))
  }
}
