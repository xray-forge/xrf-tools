use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::combine_pass::CombinePass;
use crate::pass::composited_pass::CompositedPass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::exposure_pass::ExposurePass;
use crate::pass::flare_pass::FlarePass;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::fxaa_pass::FxaaPass;
use crate::pass::grass_pass::GrassPass;
use crate::pass::lights_pass::LightsPass;
use crate::pass::material_table::MaterialTable;
use crate::pass::overlay_pass::OverlayPass;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::present_pass::PresentPass;
use crate::pass::rain_pass::RainPass;
use crate::pass::sky_bindings::SkyBindings;
use crate::pass::sky_haze_pass::SkyHazePass;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::sun_pass::SunPass;
use crate::pass::sun_shafts_pass::SunShaftsPass;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::thunder_pass::ThunderPass;
use crate::pass::upscale_pass::UpscalePass;
use crate::pass::water_pass::WaterPass;
use crate::pass::wet_pass::WetPass;

/// The passes a level's frame runs, borrowed from the GPU state together.
#[derive(Clone, Copy)]
pub struct LevelPasses<'a> {
  pub cull: &'a StaticCullPass,
  pub gbuffer: &'a StaticGBufferPass,
  pub shadow: &'a StaticShadowPass,
  pub pyramid: &'a DepthPyramidPass,
  pub sun: &'a SunPass,
  pub ambient_occlusion: &'a AmbientOcclusionPass,
  pub lights: &'a LightsPass,
  pub combine: &'a CombinePass,
  pub sky_haze: &'a SkyHazePass,
  pub sky: &'a SkyBindings,
  pub water: &'a WaterPass,
  pub composited: &'a CompositedPass,
  pub particles: &'a ParticlePass,
  pub grass: &'a GrassPass,
  pub rain: &'a RainPass,
  pub wet: &'a WetPass,
  pub thunder: &'a ThunderPass,
  pub flares: &'a FlarePass,
  pub sun_shafts: &'a SunShaftsPass,
  pub temporal: &'a TemporalPass,
  pub fsr: &'a FsrPass,
  pub fxaa: &'a FxaaPass,
  /// None where its lookup textures could not be read, and the frame smooths as FXAA does.
  pub smaa: Option<&'a SmaaPass>,
  pub upscale: &'a UpscalePass,
  pub exposure: &'a ExposurePass,
  pub present: &'a PresentPass,
  pub overlay: &'a OverlayPass,
  pub table: &'a MaterialTable,
}
