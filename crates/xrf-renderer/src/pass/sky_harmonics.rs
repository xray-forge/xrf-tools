use glam::{Vec3, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_light_level::RenderLightLevel;
use crate::lighting::luminance::luminance;
use crate::lighting::srgb::anchored_light;

/// The sky's irradiance as order-2 spherical harmonics, as `shaders/frame/sky_irradiance.wgsl` writes it: nine
/// coefficients, each in a vector's colour, of the irradiance over π in the engine's light units, already convolved by
/// the cosine; the first's `w` one once the sky was projected, nothing before; the second to fourth's `w` the authored
/// hemisphere's first coefficient (`hemisphere_environment`: the engine's own square of the hemisphere colour and the
/// cubes, its hue decoded per channel, the look's hemisphere scale in it), its level; the fifth to seventh's `w` what
/// that hemisphere gives a level surface facing up, its lookup there.
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
  /// hemisphere scale (`hemi`); none before the sky was projected.
  pub fn get_zenith_irradiance(&self, hemi: f32) -> f32 {
    if !self.is_projected() {
      return 0.0;
    }

    luminance(Self::get_zenith(&self.coefficients) * hemi).max(0.0)
  }

  /// What the authored hemisphere gives a level surface facing up, in the engine's units, as the irradiance pass looked
  /// it up (`hemisphere_environment`); none before the sky was projected.
  pub fn get_authored_zenith(&self) -> Option<Vec3> {
    self.is_projected().then(|| self.get_authored_terms().1)
  }

  /// The authored hemisphere's first coefficient and what it gives a level surface facing up, from the `w`s the
  /// irradiance pass wrote them into.
  fn get_authored_terms(&self) -> (Vec3, Vec3) {
    let [_, c1, c2, c3, c4, c5, c6, _, _] = self.coefficients;

    (Vec3::new(c1.w, c2.w, c3.w), Vec3::new(c4.w, c5.w, c6.w))
  }

  /// Whether the sky was projected.
  fn is_projected(&self) -> bool {
    self.coefficients[0].w > 0.5
  }

  /// The irradiance over π `coefficients` give a level surface facing up, per channel: the nine harmonics at the
  /// zenith (`common/sky_harmonics`, y up), the first, the second by y, and the third band's two that do not vanish
  /// there.
  fn get_zenith(coefficients: &[Vec4; 9]) -> Vec3 {
    let [c0, c1, _, _, _, _, c6, _, c8] = *coefficients;

    c0.truncate() * 0.282_095 + c1.truncate() * 0.488_603 - c6.truncate() * 0.315_392 - c8.truncate() * 0.546_274
  }

  /// The coefficients the lighting carries at the light's `level`, the look's hemisphere scale (`hemi`) in them. At
  /// the physical levels the sky's own; at the authored levels the sky's shape, each channel's own, scaled so a level
  /// surface facing up takes what the authored hemisphere gives it, as the game lights its hemisphere: a night sky's
  /// picture lights a fraction of the hemisphere the game authors and leaves the side away from the moon black, and a
  /// sky's picture tints the shade it lights its own colour, a day's blue grey where the game's hemisphere is warmer.
  /// Matched at the zenith, as HDRP levels a sky by its upper hemisphere's illuminance, not on the sphere's mean, at
  /// which a top-heavy cloudy sky lights level ground a quarter over the game's. A sky too dark to have a shape lights
  /// evenly at the authored hemisphere's level. At the anchored levels as at the authored, the hemisphere anchored over
  /// the weather's `ambient` (`anchored_light`), both in the engine's units.
  pub fn at_level(&self, level: RenderLightLevel, (hemi, ambient): (f32, Vec3)) -> [Vec4; 9] {
    let (sky, zenith): (Vec3, Vec3) = (
      self.coefficients[0].truncate() * hemi,
      Self::get_zenith(&self.coefficients) * hemi,
    );
    let (authored, authored_zenith): (Vec3, Vec3) = self.get_authored_terms();
    // Each channel's share of the hemisphere's light it keeps anchored, its zenith's.
    let kept: Vec3 = match level {
      RenderLightLevel::Anchored => {
        anchored_light(authored_zenith, ambient) / authored_zenith.max(Vec3::splat(f32::MIN_POSITIVE))
      }
      RenderLightLevel::Authored | RenderLightLevel::Physical => Vec3::ONE,
    };
    let (authored, authored_zenith): (Vec3, Vec3) = (authored * kept, authored_zenith * kept);
    let scale: Vec3 = match level {
      RenderLightLevel::Physical => Vec3::splat(hemi),
      RenderLightLevel::Authored | RenderLightLevel::Anchored
        if luminance(sky) > luminance(authored) * Self::DARK_SKY_SHARE
          && luminance(zenith) > luminance(authored_zenith) * Self::DARK_SKY_SHARE =>
      {
        hemi * authored_zenith / zenith.max(Vec3::splat(luminance(zenith) * Self::LEAST_CHANNEL_SHARE))
      }
      RenderLightLevel::Authored | RenderLightLevel::Anchored => {
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
