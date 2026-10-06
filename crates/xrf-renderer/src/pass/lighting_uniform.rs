use glam::{Mat4, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_fog_settings::RenderFogSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::bitmask_search::BitmaskSearch;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::reflection_trace::ReflectionTrace;

/// The lighting the shaders read as their `Lighting`: one viewport's, since the sun is given in its view space.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Lighting")]
pub struct LightingUniform {
  /// Towards the sun, in view space.
  pub to_sun: Vec4,
  /// The sun's colour, then its specular weight.
  pub sun: Vec4,
  pub ambient: Vec4,
  pub environment: Vec4,
  /// The irradiance's stand-in, then one once both cubes are up.
  pub sky_irradiance: Vec4,
  /// The fog's colour, then one where it fogs.
  pub fog_color: Vec4,
  /// `fog_params.x` and `.w`, then one where far geometry fades into the sky's haze above the fold, then one where the sky is drawn.
  pub fog: Vec4,
  /// `sky_color`, then the blend between the keyframes' skies.
  pub sky: Vec4,
  /// The sky's rotation, the clouds', the clouds' clock.
  pub sky_params: Vec4,
  /// The clouds' colour and cover; nothing where they are hidden.
  pub clouds: Vec4,
  /// One for Anomaly's shading, then the rain's density.
  pub engine: Vec4,
  /// How much light the occlusion's creases bounce back (VBAO's alone), then ones where the scene is
  /// lit, the exposure adapts and the occlusion darkens.
  pub params: Vec4,
  /// `L_ambient`, `L_hemi_color` and `L_sun_color` as a forward pass binds them: the weather's own, neither doubled
  /// nor scaled by the console.
  pub forward_ambient: Vec4,
  pub forward_hemi: Vec4,
  pub forward_sun: Vec4,
  /// The sun's sprite colour times how far it has faded in, then half its side as a share of the distance it stands
  /// at; zero where none is drawn.
  pub sun_sprite: Vec4,
  /// The sun shafts' density, then the steps along a ray `accum_volumetric_sun` takes at the chosen quality; nothing
  /// where they are not drawn.
  pub shafts: Vec4,
  /// One while flora is lit as foliage, how much sunlight it passes through from behind, and how much of the sun's
  /// colour that light keeps.
  pub flora: Vec4,
  /// One where the indirect light is gathered and added.
  pub indirect: Vec4,
  /// One where the reflections are traced and blended in, the frame's pixels a traced pixel stands for each way, and
  /// their intensity.
  pub reflections: Vec4,
  /// The enhanced fog's height, density and sun colour, then one where it is drawn; nothing where the engine's alone is.
  pub height_fog: Vec4,
}

impl LightingUniform {
  pub fn new(lighting: &RenderLighting, view: Mat4, options: &RenderViewOptions, frame: &LightingFrame) -> Self {
    // The direction the light travels, turned to face the sun and into view space.
    let to_sun: Vec3 = view
      .transform_vector3(-lighting.get_sun_direction())
      .normalize_or_zero();
    let fog = lighting.fog.filter(|_| options.show.is_fogged);
    let (offset, scale) = fog.map_or((0.0, 0.0), |fog| fog.get_params());
    let flag = |is: bool| is as u32 as f32;
    let occlusion: &RenderAmbientOcclusionSettings = &options.features.ambient_occlusion;
    let height_fog: &RenderFogSettings = &options.features.fog;
    // Searched only where the view is lit and solid.
    let is_occluded: bool = occlusion.is_enabled && options.mode.is_lit && !options.mode.is_wireframe;

    Self {
      to_sun: to_sun.extend(0.0),
      sun: lighting
        .get_sun_color(&options.features.light_scales)
        .extend(lighting.get_sun_specular(&options.features.light_scales)),
      ambient: lighting.get_ambient(&options.features.light_scales).extend(0.0),
      environment: lighting.get_environment(&options.features.light_scales).extend(0.0),
      sky_irradiance: lighting.sky_irradiance.extend(flag(frame.is_irradiance_up)),
      fog_color: fog.map_or(Vec3::ZERO, |fog| fog.color).extend(flag(fog.is_some())),
      fog: Vec4::new(
        offset,
        scale,
        flag(options.show.is_sky_hazed),
        flag(options.show.is_sky_visible),
      ),
      sky: lighting.sky.color.extend(frame.sky_blend),
      sky_params: Vec4::new(
        lighting.sky.rotation,
        lighting.sky.clouds.rotation,
        frame.clouds_time,
        0.0,
      ),
      clouds: if options.show.is_clouded {
        lighting.sky.clouds.color
      } else {
        Vec4::ZERO
      },
      engine: Vec4::new(
        flag(lighting.engine == XrayEngine::Extended),
        lighting.rain.map_or(0.0, |rain| rain.density),
        0.0,
        0.0,
      ),
      params: Vec4::new(
        if is_occluded && occlusion.is_vbao() {
          occlusion.vbao.bounce.clamp(0.0, 1.0)
        } else {
          0.0
        },
        flag(options.mode.is_lit),
        flag(frame.is_adapting),
        flag(is_occluded),
      ),
      forward_ambient: lighting.ambient_color.extend(0.0),
      forward_hemi: lighting.hemisphere_color.extend(0.0),
      forward_sun: lighting.sun_color.extend(0.0),
      sun_sprite: frame.sun_sprite,
      shafts: if options.show.is_sun_shafted && options.mode.is_lit {
        Vec4::new(
          lighting.get_sun_shafts(&options.features.sun_shafts),
          options.features.sun_shafts.quality.get_steps(lighting.engine) as f32,
          0.0,
          0.0,
        )
      } else {
        Vec4::ZERO
      },
      flora: Vec4::new(
        flag(options.features.grass.foliage.is_enhanced()),
        options.features.grass.foliage.sss_intensity,
        options.features.grass.foliage.sss_color,
        0.0,
      ),
      indirect: Vec4::new(
        flag(BitmaskSearch::new(options).is_some_and(|search| search.is_lit)),
        0.0,
        0.0,
        0.0,
      ),
      reflections: ReflectionTrace::new(options).map_or(Vec4::ZERO, |trace| {
        Vec4::new(1.0, trace.get_ratio() as f32, trace.intensity, 0.0)
      }),
      height_fog: if fog.is_some() && options.mode.is_lit && height_fog.is_enhanced() {
        Vec4::new(
          height_fog.height.max(0.01),
          height_fog.density,
          height_fog.sun_color,
          1.0,
        )
      } else {
        Vec4::ZERO
      },
    }
  }
}
