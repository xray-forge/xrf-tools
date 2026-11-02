use glam::Vec3;

/// `u_diffuse2s`'s gloss factor (`r2_types.h`).
const GLOSS_FACTOR: f32 = 4.0;

/// What a light of a colour contributes to specular, `u_diffuse2s` of it: its mean, softened below one. Where it is
/// linear (`is_linear`, the physical levels' decoded light: `RenderViewOptions::is_specular_linear`) its mean alone, so
/// its specular follows it as its diffuse does.
pub fn to_light_specular(color: Vec3, is_linear: bool) -> f32 {
  let mean: f32 = color.element_sum() / 3.0;

  GLOSS_FACTOR
    * if is_linear || mean >= 1.0 {
      mean
    } else {
      mean.powf(2.0 / 3.0)
    }
}
