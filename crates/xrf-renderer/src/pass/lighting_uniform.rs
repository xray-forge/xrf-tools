use glam::{Mat4, Vec3, Vec4};
use xrf_engine_target::XrayEngine;

use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::lighting_frame::LightingFrame;

/// The lighting as `shaders/common/lighting.wgsl` declares it: one viewport's, since the sun is given in its view space.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightingUniform {
  pub to_sun: Vec4,
  /// The sun's colour, then its specular weight.
  pub sun: Vec4,
  pub ambient: Vec4,
  pub environment: Vec4,
  /// The irradiance's stand-in, then one once both cubes are up.
  pub sky_irradiance: Vec4,
  /// The fog's colour, then one where it fogs.
  pub fog_color: Vec4,
  /// `fog_params.x` and `.w`, then one where the sky's haze takes the fog, then one where the sky is drawn.
  pub fog: Vec4,
  /// `sky_color`, then the blend between the keyframes' skies.
  pub sky: Vec4,
  /// The sky's rotation, the clouds', the clouds' clock.
  pub sky_params: Vec4,
  /// The clouds' colour and cover; nothing where they are hidden.
  pub clouds: Vec4,
  /// One for Anomaly's shading, then the rain's density.
  pub engine: Vec4,
  /// The settings' tonemap scale, and ones where the scene is lit, the exposure adapts and the occlusion darkens.
  pub params: Vec4,
  /// `L_ambient` and `L_hemi_color` as a forward pass binds them: the weather's own, neither doubled nor scaled.
  pub forward_ambient: Vec4,
  pub forward_hemi: Vec4,
  /// The sun's sprite colour times how far it has faded in, then half its side as a share of the distance it stands
  /// at; zero where none is drawn.
  pub sun_sprite: Vec4,
}

impl LightingUniform {
  pub fn new(lighting: &RenderLighting, view: Mat4, options: &RenderViewOptions, frame: &LightingFrame) -> Self {
    // The direction the light travels, turned to face the sun and into view space.
    let to_sun: Vec3 = view
      .transform_vector3(-lighting.get_sun_direction())
      .normalize_or_zero();
    let fog = lighting.fog.filter(|_| options.is_fogged);
    let (offset, scale) = fog.map_or((0.0, 0.0), |fog| fog.get_params());
    let flag = |is: bool| is as u32 as f32;

    Self {
      to_sun: to_sun.extend(0.0),
      sun: lighting
        .get_sun_color(&options.light_scales)
        .extend(lighting.get_sun_specular(&options.light_scales)),
      ambient: lighting.get_ambient(&options.light_scales).extend(0.0),
      environment: lighting.get_environment(&options.light_scales).extend(0.0),
      sky_irradiance: lighting.sky_irradiance.extend(flag(frame.is_irradiance_up)),
      fog_color: fog.map_or(Vec3::ZERO, |fog| fog.color).extend(flag(fog.is_some())),
      fog: Vec4::new(offset, scale, flag(options.is_sky_hazed), flag(options.is_sky_visible)),
      sky: lighting.sky.color.extend(frame.sky_blend),
      sky_params: Vec4::new(
        lighting.sky.rotation,
        lighting.sky.clouds.rotation,
        frame.clouds_time,
        0.0,
      ),
      clouds: if options.is_clouded {
        lighting.sky.clouds.color
      } else {
        Vec4::ZERO
      },
      engine: Vec4::new(
        flag(lighting.engine == XrayEngine::Extended),
        lighting
          .rain
          .filter(|_| options.is_rainy)
          .map_or(0.0, |rain| rain.density),
        0.0,
        0.0,
      ),
      params: Vec4::new(
        options.tonemap_scale,
        flag(options.is_lit),
        flag(frame.is_adapting),
        flag(options.ambient_occlusion.is_enabled),
      ),
      forward_ambient: lighting.ambient_color.extend(0.0),
      forward_hemi: lighting.hemisphere_color.extend(0.0),
      sun_sprite: frame.sun_sprite,
    }
  }
}
