use glam::{Mat4, Vec3, Vec4};

use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;

/// The sky's colour overhead and at the horizon, until the weather draws one.
const SKY_ZENITH: Vec3 = Vec3::new(0.13, 0.15, 0.19);
const SKY_HORIZON: Vec3 = Vec3::new(0.32, 0.34, 0.37);

/// The lighting as `shaders/common/lighting.wgsl` declares it: one viewport's, since the sun is given in its view space.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightingUniform {
  pub to_sun: Vec4,
  /// The sun's colour, then its specular weight.
  pub sun: Vec4,
  pub ambient: Vec4,
  pub environment: Vec4,
  pub sky_irradiance: Vec4,
  pub sky_zenith: Vec4,
  pub sky_horizon: Vec4,
  /// The settings' tonemap scale, and ones where the scene is lit, the exposure adapts and the occlusion darkens.
  pub params: Vec4,
}

impl LightingUniform {
  pub fn new(lighting: &RenderLighting, view: Mat4, options: &RenderViewOptions, is_adapting: bool) -> Self {
    // The direction the light travels, turned to face the sun and into view space.
    let to_sun: Vec3 = view
      .transform_vector3(-lighting.get_sun_direction())
      .normalize_or_zero();

    Self {
      to_sun: to_sun.extend(0.0),
      sun: lighting.sun_color.extend(lighting.get_sun_specular()),
      ambient: lighting.get_ambient().extend(0.0),
      environment: lighting.get_environment().extend(0.0),
      sky_irradiance: lighting.sky_irradiance.extend(0.0),
      sky_zenith: SKY_ZENITH.extend(1.0),
      sky_horizon: SKY_HORIZON.extend(1.0),
      params: Vec4::new(
        options.tonemap_scale,
        options.is_lit as u32 as f32,
        is_adapting as u32 as f32,
        options.ambient_occlusion.is_enabled as u32 as f32,
      ),
    }
  }
}
