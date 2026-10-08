use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, ShaderBindings, ShaderDeclarations};

use crate::pass::ambient_occlusion_parameters::AmbientOcclusionParameters;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::bloom_parameters::BloomParameters;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::contact_shadow_parameters::ContactShadowParameters;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::enhanced_water_parameters::EnhancedWaterParameters;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::exposure_head::ExposureHead;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::exposure_state::ExposureState;
use crate::pass::exposure_uniform::ExposureUniform;
use crate::pass::flare_draw_parameters::FlareDrawParameters;
use crate::pass::flare_measure_parameters::FlareMeasureParameters;
use crate::pass::flare_texture_parameters::FlareTextureParameters;
use crate::pass::flare_uniform::FlareUniform;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::light_record::LightRecord;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::particle_parameters::ParticleParameters;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::pass::particle_vertex::ParticleVertex;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::shadow_uniform::ShadowUniform;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::smaa_parameters::SmaaParameters;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_impostor_parameters::StaticImpostorParameters;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::temporal_parameters::TemporalParameters;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_parameters::ThunderParameters;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::vbao_parameters::VbaoParameters;
use crate::pass::vbao_uniform::VbaoUniform;
use crate::pass::water_blur_parameters::WaterBlurParameters;
use crate::pass::water_blur_uniform::WaterBlurUniform;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_reflection_parameters::WaterReflectionParameters;
use crate::pass::water_surface_parameters::WaterSurfaceParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::pass::wet_apply_parameters::WetApplyParameters;
use crate::pass::wet_patch_parameters::WetPatchParameters;
use crate::pass::wet_uniform::WetUniform;

/// What each module of bindings opens with, for the structs its uniforms hold.
const STRUCTS_IMPORT: &str = "#import \"generated/structs\"\n\n";

/// The modules under `shaders/generated/`, written from Rust declarations, by name: the structs shaders share with
/// Rust, and each module's bindings, gathered from the parameters of every pass drawing with it, importing the structs.
///
/// # Errors
///
/// Returns an error when passes sharing a module declare one of its bindings otherwise.
pub fn list_generated_shaders() -> XrfResult<Vec<(&'static str, String)>> {
  let mut structs: ShaderDeclarations = ShaderDeclarations::new();
  let mut water: ShaderBindings = ShaderBindings::new();
  let mut water_enhanced: ShaderBindings = ShaderBindings::new();
  let mut water_reflection: ShaderBindings = ShaderBindings::new();
  let mut water_blur: ShaderBindings = ShaderBindings::new();
  let mut present: ShaderBindings = ShaderBindings::new();
  let mut overlay: ShaderBindings = ShaderBindings::new();
  let mut upscale: ShaderBindings = ShaderBindings::new();
  let mut bloom: ShaderBindings = ShaderBindings::new();
  let mut fxaa: ShaderBindings = ShaderBindings::new();
  let mut smaa: ShaderBindings = ShaderBindings::new();
  let mut sky: ShaderBindings = ShaderBindings::new();
  let mut particles: ShaderBindings = ShaderBindings::new();
  let mut contact_shadows: ShaderBindings = ShaderBindings::new();
  let mut sun: ShaderBindings = ShaderBindings::new();
  let mut light_binning: ShaderBindings = ShaderBindings::new();
  let mut lights: ShaderBindings = ShaderBindings::new();
  let mut ambient_occlusion: ShaderBindings = ShaderBindings::new();
  let mut vbao: ShaderBindings = ShaderBindings::new();
  let mut reflections: ShaderBindings = ShaderBindings::new();
  let mut combine: ShaderBindings = ShaderBindings::new();
  let mut composited: ShaderBindings = ShaderBindings::new();
  let mut sky_haze: ShaderBindings = ShaderBindings::new();
  let mut sun_shafts: ShaderBindings = ShaderBindings::new();
  let mut exposure: ShaderBindings = ShaderBindings::new();
  let mut flare: ShaderBindings = ShaderBindings::new();
  let mut flare_visibility: ShaderBindings = ShaderBindings::new();
  let mut rain: ShaderBindings = ShaderBindings::new();
  let mut wet_patch: ShaderBindings = ShaderBindings::new();
  let mut wet_apply: ShaderBindings = ShaderBindings::new();
  let mut thunder: ShaderBindings = ShaderBindings::new();
  let mut temporal: ShaderBindings = ShaderBindings::new();
  let mut pyramid: ShaderBindings = ShaderBindings::new();
  let mut static_cull: ShaderBindings = ShaderBindings::new();
  let mut static_draw: ShaderBindings = ShaderBindings::new();
  let mut static_impostor: ShaderBindings = ShaderBindings::new();

  structs
    .declare::<LightingUniform>()
    .declare::<WaterUniform>()
    .declare::<EnhancedWaterUniform>()
    .declare::<WaterBlurUniform>()
    .declare::<PresentUniform>()
    .declare::<UpscaleUniform>()
    .declare::<BloomUniform>()
    .declare::<ParticleVertex>()
    .declare::<ParticleSurfaceRecord>()
    .declare::<ShadowUniform>()
    .declare::<ContactShadowUniform>()
    .declare::<LightsUniform>()
    .declare::<LightRecord>()
    .declare::<AmbientOcclusionUniform>()
    .declare::<VbaoUniform>()
    .declare::<ReflectionUniform>()
    .declare::<ExposureUniform>()
    .declare::<ExposureState>()
    .declare::<ExposureHead>()
    .declare::<FlareUniform>()
    .declare::<RainUniform>()
    .declare::<WetUniform>()
    .declare::<ThunderUniform>()
    .declare::<TemporalUniform>()
    .declare::<FsrUniform>();
  StaticCullParameters::declare(&mut structs);
  StaticDrawParameters::declare(&mut structs);
  StaticImpostorParameters::declare(&mut structs);
  water
    .add::<WaterDepthParameters>()?
    .add::<WaterSurfaceParameters<'_>>()?;
  water_enhanced.add::<EnhancedWaterParameters<'_>>()?;
  water_reflection.add::<WaterReflectionParameters<'_>>()?;
  water_blur.add::<WaterBlurParameters<'_>>()?;
  present.add::<PresentParameters<'_>>()?;
  overlay.add::<OverlayParameters>()?;
  upscale.add::<UpscaleParameters>()?;
  bloom.add::<BloomParameters<'_>>()?;
  fxaa.add::<FxaaParameters<'_>>()?;
  smaa.add::<SmaaParameters<'_>>()?;
  sky.add::<SkyParameters<'_>>()?;
  particles.add::<ParticleParameters<'_>>()?;
  contact_shadows.add::<ContactShadowParameters>()?;
  sun.add::<SunParameters<'_>>()?;
  light_binning.add::<LightBinningParameters>()?;
  lights.add::<LightsParameters<'_>>()?;
  ambient_occlusion.add::<AmbientOcclusionParameters>()?;
  vbao.add::<VbaoParameters>()?;
  reflections.add::<ReflectionParameters<'_>>()?;
  combine.add::<CombineParameters<'_>>()?;
  composited.add::<CompositedParameters<'_>>()?;
  sky_haze.add::<SkyHazeParameters>()?;
  sun_shafts.add::<SunShaftsParameters>()?;
  exposure.add::<ExposureParameters>()?;
  flare
    .add::<FlareDrawParameters<'_>>()?
    .add::<FlareTextureParameters>()?;
  flare_visibility.add::<FlareMeasureParameters>()?;
  rain.add::<RainParameters<'_>>()?;
  wet_patch.add::<WetPatchParameters<'_>>()?;
  wet_apply.add::<WetApplyParameters>()?;
  thunder.add::<ThunderParameters<'_>>()?;
  temporal.add::<TemporalParameters<'_>>()?;
  pyramid
    .add::<PyramidDepthParameters>()?
    .add::<PyramidLevelParameters>()?;
  static_cull.add::<StaticCullParameters>()?;
  static_draw.add::<StaticDrawParameters>()?;
  static_impostor.add::<StaticImpostorParameters>()?;

  Ok(vec![
    ("generated/structs", structs.to_wgsl()),
    ("generated/static/water", format!("{STRUCTS_IMPORT}{}", water.to_wgsl())),
    (
      "generated/static/water_enhanced",
      format!("{STRUCTS_IMPORT}{}", water_enhanced.to_wgsl()),
    ),
    (
      "generated/static/water_reflection",
      format!("{STRUCTS_IMPORT}{}", water_reflection.to_wgsl()),
    ),
    (
      "generated/frame/present",
      format!("{STRUCTS_IMPORT}{}", present.to_wgsl()),
    ),
    (
      "generated/frame/overlay",
      format!("{STRUCTS_IMPORT}{}", overlay.to_wgsl()),
    ),
    (
      "generated/frame/upscale",
      format!("{STRUCTS_IMPORT}{}", upscale.to_wgsl()),
    ),
    ("generated/frame/bloom", format!("{STRUCTS_IMPORT}{}", bloom.to_wgsl())),
    ("generated/frame/fxaa", format!("{STRUCTS_IMPORT}{}", fxaa.to_wgsl())),
    ("generated/frame/smaa", format!("{STRUCTS_IMPORT}{}", smaa.to_wgsl())),
    ("generated/common/sky", format!("{STRUCTS_IMPORT}{}", sky.to_wgsl())),
    (
      "generated/frame/particles",
      format!("{STRUCTS_IMPORT}{}", particles.to_wgsl()),
    ),
    (
      "generated/frame/contact_shadows",
      format!("{STRUCTS_IMPORT}{}", contact_shadows.to_wgsl()),
    ),
    ("generated/frame/sun", format!("{STRUCTS_IMPORT}{}", sun.to_wgsl())),
    (
      "generated/frame/light_binning",
      format!("{STRUCTS_IMPORT}{}", light_binning.to_wgsl()),
    ),
    (
      "generated/frame/lights",
      format!("{STRUCTS_IMPORT}{}", lights.to_wgsl()),
    ),
    (
      "generated/frame/ambient_occlusion",
      format!("{STRUCTS_IMPORT}{}", ambient_occlusion.to_wgsl()),
    ),
    ("generated/frame/vbao", format!("{STRUCTS_IMPORT}{}", vbao.to_wgsl())),
    (
      "generated/frame/reflections",
      format!("{STRUCTS_IMPORT}{}", reflections.to_wgsl()),
    ),
    (
      "generated/frame/combine",
      format!("{STRUCTS_IMPORT}{}", combine.to_wgsl()),
    ),
    (
      "generated/static/composited",
      format!("{STRUCTS_IMPORT}{}", composited.to_wgsl()),
    ),
    (
      "generated/frame/sky_haze",
      format!("{STRUCTS_IMPORT}{}", sky_haze.to_wgsl()),
    ),
    (
      "generated/frame/sun_shafts",
      format!("{STRUCTS_IMPORT}{}", sun_shafts.to_wgsl()),
    ),
    (
      "generated/frame/exposure",
      format!("{STRUCTS_IMPORT}{}", exposure.to_wgsl()),
    ),
    ("generated/frame/flare", format!("{STRUCTS_IMPORT}{}", flare.to_wgsl())),
    (
      "generated/frame/flare_visibility",
      format!("{STRUCTS_IMPORT}{}", flare_visibility.to_wgsl()),
    ),
    ("generated/frame/rain", format!("{STRUCTS_IMPORT}{}", rain.to_wgsl())),
    (
      "generated/frame/wet_patch",
      format!("{STRUCTS_IMPORT}{}", wet_patch.to_wgsl()),
    ),
    (
      "generated/frame/wet_apply",
      format!("{STRUCTS_IMPORT}{}", wet_apply.to_wgsl()),
    ),
    (
      "generated/frame/thunder",
      format!("{STRUCTS_IMPORT}{}", thunder.to_wgsl()),
    ),
    (
      "generated/frame/temporal",
      format!("{STRUCTS_IMPORT}{}", temporal.to_wgsl()),
    ),
    (
      "generated/frame/pyramid",
      format!("{STRUCTS_IMPORT}{}", pyramid.to_wgsl()),
    ),
    (
      "generated/static/cull",
      format!("{STRUCTS_IMPORT}{}", static_cull.to_wgsl()),
    ),
    (
      "generated/static/draw",
      format!("{STRUCTS_IMPORT}{}", static_draw.to_wgsl()),
    ),
    (
      "generated/static/impostor",
      format!("{STRUCTS_IMPORT}{}", static_impostor.to_wgsl()),
    ),
    (
      "generated/frame/water_blur",
      format!("{STRUCTS_IMPORT}{}", water_blur.to_wgsl()),
    ),
  ])
}
