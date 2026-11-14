use glam::Vec3;

/// The light a point gathers, `base_color_c` (`xrLC_Light/base_color.h`): static colour, sky and sun.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightColor {
  pub rgb: Vec3,
  pub hemi: f32,
  pub sun: f32,
}
