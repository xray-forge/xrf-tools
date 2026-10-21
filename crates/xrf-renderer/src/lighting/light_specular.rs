use glam::Vec3;

/// `u_diffuse2s`'s gloss factor (`r2_types.h`).
const GLOSS_FACTOR: f32 = 4.0;

/// What a light of a colour contributes to specular, `u_diffuse2s` of it: its mean, softened below one. Under the
/// physical light (`is_physical`, the colour decoded) its mean alone, so its specular follows it as its diffuse does.
pub fn to_light_specular(color: Vec3, is_physical: bool) -> f32 {
  let mean: f32 = color.element_sum() / 3.0;

  GLOSS_FACTOR
    * if is_physical || mean >= 1.0 {
      mean
    } else {
      mean.powf(2.0 / 3.0)
    }
}
