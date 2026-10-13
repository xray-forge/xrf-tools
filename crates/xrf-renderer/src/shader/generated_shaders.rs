use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, ShaderBindings, ShaderDeclarations};

use crate::pass::ambient_occlusion_parameters::AmbientOcclusionParameters;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::bloom_parameters::BloomParameters;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::contact_shadow_parameters::ContactShadowParameters;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::cover_pyramid_depth_parameters::CoverPyramidDepthParameters;
use crate::pass::cover_pyramid_level_parameters::CoverPyramidLevelParameters;
use crate::pass::dlss_reactive_parameters::DlssReactiveParameters;
use crate::pass::enhanced_bloom_parameters::EnhancedBloomParameters;
use crate::pass::enhanced_bloom_uniform::EnhancedBloomUniform;
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
use crate::pass::fog_scattering_parameters::FogScatteringParameters;
use crate::pass::fog_scattering_uniform::FogScatteringUniform;
use crate::pass::froxel_inject_parameters::FroxelInjectParameters;
use crate::pass::froxel_integrate_parameters::FroxelIntegrateParameters;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::pass::glass_copy_dispatch_parameters::GlassCopyDispatchParameters;
use crate::pass::glass_copy_parameters::GlassCopyParameters;
use crate::pass::glass_parameters::GlassParameters;
use crate::pass::glass_uniform::GlassUniform;
use crate::pass::grass_colour_parameters::GrassColourParameters;
use crate::pass::grass_model_record::GrassModelRecord;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::indirect_trace_parameters::IndirectTraceParameters;
use crate::pass::indirect_trace_uniform::IndirectTraceUniform;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::light_contact_parameters::LightContactParameters;
use crate::pass::light_record::LightRecord;
use crate::pass::light_shadowing_parameters::LightShadowingParameters;
use crate::pass::light_trace_uniform::LightTraceUniform;
use crate::pass::light_tracing_parameters::LightTracingParameters;
use crate::pass::light_visibility_parameters::LightVisibilityParameters;
use crate::pass::light_volume_parameters::LightVolumeParameters;
use crate::pass::light_volume_record::LightVolumeRecord;
use crate::pass::light_volumes_uniform::LightVolumesUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::lowest_heights_parameters::LowestHeightsParameters;
use crate::pass::overhead_shift_parameters::OverheadShiftParameters;
use crate::pass::overhead_shift_uniform::OverheadShiftUniform;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::particle_parameters::ParticleParameters;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::pass::particle_vertex::ParticleVertex;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::puddle_keep_parameters::PuddleKeepParameters;
use crate::pass::puddle_sites_parameters::PuddleSitesParameters;
use crate::pass::puddle_sites_uniform::PuddleSitesUniform;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::ray_guides_parameters::RayGuidesParameters;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::reflection_scene_parameters::ReflectionSceneParameters;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::shadow_tile_restore_parameters::ShadowTileRestoreParameters;
use crate::pass::shadow_uniform::ShadowUniform;
use crate::pass::sky_behind_parameters::SkyBehindParameters;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_motion_parameters::SkyMotionParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::pass::sky_prefilter_uniform::SkyPrefilterUniform;
use crate::pass::smaa_parameters::SmaaParameters;
use crate::pass::smoothing_uniform::SmoothingUniform;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_impostor_parameters::StaticImpostorParameters;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::sun_trace_parameters::SunTraceParameters;
use crate::pass::sun_trace_uniform::SunTraceUniform;
use crate::pass::surroundings_parameters::SurroundingsParameters;
use crate::pass::surroundings_uniform::SurroundingsUniform;
use crate::pass::temporal_parameters::TemporalParameters;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_parameters::ThunderParameters;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::trace_grass_uniform::TraceGrassUniform;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::trace_request::TraceRequest;
use crate::pass::trace_request_parameters::TraceRequestParameters;
use crate::pass::trace_sway_parameters::TraceSwayParameters;
use crate::pass::trace_sway_slot::TraceSwaySlot;
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
  let mut enhanced_bloom: ShaderBindings = ShaderBindings::new();
  let mut fog_scattering: ShaderBindings = ShaderBindings::new();
  let mut fxaa: ShaderBindings = ShaderBindings::new();
  let mut smaa: ShaderBindings = ShaderBindings::new();
  let mut sky: ShaderBindings = ShaderBindings::new();
  let mut scene_trace: ShaderBindings = ShaderBindings::new();
  let mut particles: ShaderBindings = ShaderBindings::new();
  let mut contact_shadows: ShaderBindings = ShaderBindings::new();
  let mut sun: ShaderBindings = ShaderBindings::new();
  let mut sun_trace: ShaderBindings = ShaderBindings::new();
  let mut sky_motion: ShaderBindings = ShaderBindings::new();
  let mut light_binning: ShaderBindings = ShaderBindings::new();
  let mut sky_behind: ShaderBindings = ShaderBindings::new();
  let mut ray_guides: ShaderBindings = ShaderBindings::new();
  let mut dlss_reactive: ShaderBindings = ShaderBindings::new();
  let mut grass_occlusion: ShaderBindings = ShaderBindings::new();
  let mut lights: ShaderBindings = ShaderBindings::new();
  let mut light_contact: ShaderBindings = ShaderBindings::new();
  let mut light_shadowing: ShaderBindings = ShaderBindings::new();
  let mut light_tracing: ShaderBindings = ShaderBindings::new();
  let mut light_visibility: ShaderBindings = ShaderBindings::new();
  let mut ambient_occlusion: ShaderBindings = ShaderBindings::new();
  let mut vbao: ShaderBindings = ShaderBindings::new();
  let mut reflections: ShaderBindings = ShaderBindings::new();
  let mut reflection_scene: ShaderBindings = ShaderBindings::new();
  let mut indirect_trace: ShaderBindings = ShaderBindings::new();
  let mut trace_requests: ShaderBindings = ShaderBindings::new();
  let mut grass_colours: ShaderBindings = ShaderBindings::new();
  let mut combine: ShaderBindings = ShaderBindings::new();
  let mut composited: ShaderBindings = ShaderBindings::new();
  let mut glass: ShaderBindings = ShaderBindings::new();
  let mut glass_copy_dispatch: ShaderBindings = ShaderBindings::new();
  let mut glass_copy: ShaderBindings = ShaderBindings::new();
  let mut sky_haze: ShaderBindings = ShaderBindings::new();
  let mut sun_shafts: ShaderBindings = ShaderBindings::new();
  let mut light_volumes: ShaderBindings = ShaderBindings::new();
  let mut froxel_inject: ShaderBindings = ShaderBindings::new();
  let mut froxel_integrate: ShaderBindings = ShaderBindings::new();
  let mut exposure: ShaderBindings = ShaderBindings::new();
  let mut flare: ShaderBindings = ShaderBindings::new();
  let mut flare_visibility: ShaderBindings = ShaderBindings::new();
  let mut rain: ShaderBindings = ShaderBindings::new();
  let mut wet_patch: ShaderBindings = ShaderBindings::new();
  let mut wet_apply: ShaderBindings = ShaderBindings::new();
  let mut thunder: ShaderBindings = ShaderBindings::new();
  let mut temporal: ShaderBindings = ShaderBindings::new();
  let mut pyramid: ShaderBindings = ShaderBindings::new();
  let mut shadow_tile_restore: ShaderBindings = ShaderBindings::new();
  let mut overhead_shift: ShaderBindings = ShaderBindings::new();
  let mut puddle_sites: ShaderBindings = ShaderBindings::new();
  let mut sky_prefilter: ShaderBindings = ShaderBindings::new();
  let mut surroundings: ShaderBindings = ShaderBindings::new();
  let mut static_cull: ShaderBindings = ShaderBindings::new();
  let mut static_draw: ShaderBindings = ShaderBindings::new();
  let mut trace_sway: ShaderBindings = ShaderBindings::new();
  let mut static_impostor: ShaderBindings = ShaderBindings::new();

  structs
    .declare::<CameraUniform>()
    .declare::<BackdropUniform>()
    .declare::<LightingUniform>()
    .declare::<TraceSwaySlot>()
    .declare::<GlassUniform>()
    .declare::<SmoothingUniform>()
    .declare::<FroxelUniform>()
    .declare::<GrassModelRecord>()
    .declare::<TraceGrassUniform>()
    .declare::<TraceRequest>()
    .declare::<WaterUniform>()
    .declare::<EnhancedWaterUniform>()
    .declare::<WaterBlurUniform>()
    .declare::<PresentUniform>()
    .declare::<UpscaleUniform>()
    .declare::<BloomUniform>()
    .declare::<EnhancedBloomUniform>()
    .declare::<FogScatteringUniform>()
    .declare::<PuddleSitesUniform>()
    .declare::<SkyPrefilterUniform>()
    .declare::<OverheadShiftUniform>()
    .declare::<SurroundingsUniform>()
    .declare::<ParticleVertex>()
    .declare::<ParticleSurfaceRecord>()
    .declare::<ShadowUniform>()
    .declare::<ContactShadowUniform>()
    .declare::<SunTraceUniform>()
    .declare::<LightTraceUniform>()
    .declare::<IndirectTraceUniform>()
    .declare::<LightsUniform>()
    .declare::<LightRecord>()
    .declare::<LightVolumeRecord>()
    .declare::<LightVolumesUniform>()
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
  TraceParameters::declare(&mut structs);
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
  enhanced_bloom.add::<EnhancedBloomParameters<'_>>()?;
  fog_scattering.add::<FogScatteringParameters<'_>>()?;
  fxaa.add::<FxaaParameters<'_>>()?;
  smaa.add::<SmaaParameters<'_>>()?;
  sky.add::<SkyParameters<'_>>()?;
  scene_trace.add::<TraceParameters<'_>>()?;
  particles.add::<ParticleParameters<'_>>()?;
  contact_shadows.add::<ContactShadowParameters>()?;
  sun.add::<SunParameters<'_>>()?;
  sun_trace.add::<SunTraceParameters>()?;
  sky_motion.add::<SkyMotionParameters>()?;
  light_binning.add::<LightBinningParameters>()?;
  sky_behind.add::<SkyBehindParameters>()?;
  ray_guides.add::<RayGuidesParameters>()?;
  dlss_reactive.add::<DlssReactiveParameters>()?;
  grass_occlusion.add::<GrassOcclusionParameters>()?;
  lights.add::<LightsParameters<'_>>()?;
  light_contact.add::<LightContactParameters>()?;
  light_shadowing.add::<LightShadowingParameters<'_>>()?;
  light_tracing.add::<LightTracingParameters>()?;
  light_visibility.add::<LightVisibilityParameters>()?;
  ambient_occlusion.add::<AmbientOcclusionParameters>()?;
  vbao.add::<VbaoParameters>()?;
  reflections.add::<ReflectionParameters<'_>>()?;
  trace_requests.add::<TraceRequestParameters>()?;
  grass_colours.add::<GrassColourParameters>()?;
  reflection_scene.add::<ReflectionSceneParameters<'_>>()?;
  indirect_trace.add::<IndirectTraceParameters<'_>>()?;
  combine.add::<CombineParameters<'_>>()?;
  composited.add::<CompositedParameters<'_>>()?;
  glass.add::<GlassParameters<'_>>()?;
  glass_copy_dispatch.add::<GlassCopyDispatchParameters>()?;
  glass_copy.add::<GlassCopyParameters>()?;
  sky_haze.add::<SkyHazeParameters>()?;
  sun_shafts.add::<SunShaftsParameters>()?;
  light_volumes.add::<LightVolumeParameters>()?;
  froxel_inject.add::<FroxelInjectParameters<'_>>()?;
  froxel_integrate.add::<FroxelIntegrateParameters>()?;
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
  shadow_tile_restore.add::<ShadowTileRestoreParameters>()?;
  overhead_shift.add::<OverheadShiftParameters>()?;
  pyramid
    .add::<PyramidDepthParameters>()?
    .add::<PyramidLevelParameters>()?
    .add::<CoverPyramidDepthParameters>()?
    .add::<CoverPyramidLevelParameters>()?;
  puddle_sites
    .add::<LowestHeightsParameters>()?
    .add::<PuddleSitesParameters>()?
    .add::<PuddleKeepParameters<'_>>()?;
  sky_prefilter.add::<SkyPrefilterParameters<'_>>()?;
  surroundings.add::<SurroundingsParameters<'_>>()?;
  static_cull.add::<StaticCullParameters>()?;
  static_draw.add::<StaticDrawParameters>()?;
  trace_sway.add::<TraceSwayParameters>()?;
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
    (
      "generated/frame/enhanced_bloom",
      format!("{STRUCTS_IMPORT}{}", enhanced_bloom.to_wgsl()),
    ),
    (
      "generated/frame/fog_scattering",
      format!("{STRUCTS_IMPORT}{}", fog_scattering.to_wgsl()),
    ),
    ("generated/frame/fxaa", format!("{STRUCTS_IMPORT}{}", fxaa.to_wgsl())),
    ("generated/frame/smaa", format!("{STRUCTS_IMPORT}{}", smaa.to_wgsl())),
    ("generated/common/sky", format!("{STRUCTS_IMPORT}{}", sky.to_wgsl())),
    (
      "generated/common/scene_trace",
      format!("{STRUCTS_IMPORT}{}", scene_trace.to_wgsl()),
    ),
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
      "generated/frame/sky_motion",
      format!("{STRUCTS_IMPORT}{}", sky_motion.to_wgsl()),
    ),
    (
      "generated/frame/sun_trace",
      format!("{STRUCTS_IMPORT}{}", sun_trace.to_wgsl()),
    ),
    (
      "generated/grass/occlusion",
      format!("{STRUCTS_IMPORT}{}", grass_occlusion.to_wgsl()),
    ),
    (
      "generated/frame/sky_behind",
      format!("{STRUCTS_IMPORT}{}", sky_behind.to_wgsl()),
    ),
    (
      "generated/frame/ray_guides",
      format!("{STRUCTS_IMPORT}{}", ray_guides.to_wgsl()),
    ),
    (
      "generated/frame/dlss_reactive",
      format!("{STRUCTS_IMPORT}{}", dlss_reactive.to_wgsl()),
    ),
    (
      "generated/frame/light_binning",
      format!("{STRUCTS_IMPORT}{}", light_binning.to_wgsl()),
    ),
    (
      "generated/frame/lights",
      format!("{STRUCTS_IMPORT}{}", lights.to_wgsl()),
    ),
    (
      "generated/frame/light_contact",
      format!("{STRUCTS_IMPORT}{}", light_contact.to_wgsl()),
    ),
    (
      "generated/frame/light_shadowing",
      format!("{STRUCTS_IMPORT}{}", light_shadowing.to_wgsl()),
    ),
    (
      "generated/frame/light_tracing",
      format!("{STRUCTS_IMPORT}{}", light_tracing.to_wgsl()),
    ),
    (
      "generated/frame/light_visibility",
      format!("{STRUCTS_IMPORT}{}", light_visibility.to_wgsl()),
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
      "generated/frame/reflection_scene",
      format!("{STRUCTS_IMPORT}{}", reflection_scene.to_wgsl()),
    ),
    (
      "generated/frame/indirect_trace",
      format!("{STRUCTS_IMPORT}{}", indirect_trace.to_wgsl()),
    ),
    (
      "generated/common/trace_requests",
      format!("{STRUCTS_IMPORT}{}", trace_requests.to_wgsl()),
    ),
    (
      "generated/grass/colours",
      format!("{STRUCTS_IMPORT}{}", grass_colours.to_wgsl()),
    ),
    (
      "generated/frame/combine",
      format!("{STRUCTS_IMPORT}{}", combine.to_wgsl()),
    ),
    (
      "generated/static/composited",
      format!("{STRUCTS_IMPORT}{}", composited.to_wgsl()),
    ),
    ("generated/static/glass", format!("{STRUCTS_IMPORT}{}", glass.to_wgsl())),
    (
      "generated/frame/glass_copy_dispatch",
      format!("{STRUCTS_IMPORT}{}", glass_copy_dispatch.to_wgsl()),
    ),
    (
      "generated/frame/glass_copy",
      format!("{STRUCTS_IMPORT}{}", glass_copy.to_wgsl()),
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
      "generated/frame/light_volumes",
      format!("{STRUCTS_IMPORT}{}", light_volumes.to_wgsl()),
    ),
    (
      "generated/frame/froxel_inject",
      format!("{STRUCTS_IMPORT}{}", froxel_inject.to_wgsl()),
    ),
    (
      "generated/frame/froxel_integrate",
      format!("{STRUCTS_IMPORT}{}", froxel_integrate.to_wgsl()),
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
      "generated/frame/shadow_tile_restore",
      format!("{STRUCTS_IMPORT}{}", shadow_tile_restore.to_wgsl()),
    ),
    (
      "generated/frame/overhead_shift",
      format!("{STRUCTS_IMPORT}{}", overhead_shift.to_wgsl()),
    ),
    (
      "generated/frame/puddle_sites",
      format!("{STRUCTS_IMPORT}{}", puddle_sites.to_wgsl()),
    ),
    (
      "generated/frame/sky_prefilter",
      format!("{STRUCTS_IMPORT}{}", sky_prefilter.to_wgsl()),
    ),
    (
      "generated/frame/surroundings",
      format!("{STRUCTS_IMPORT}{}", surroundings.to_wgsl()),
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
      "generated/static/trace_sway",
      format!("{STRUCTS_IMPORT}{}", trace_sway.to_wgsl()),
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
