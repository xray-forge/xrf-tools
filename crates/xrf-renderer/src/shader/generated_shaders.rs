use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, ShaderBindings, ShaderDeclarations};

use crate::pass::bloom_parameters::BloomParameters;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::particle_parameters::ParticleParameters;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::pass::particle_vertex::ParticleVertex;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::smaa_parameters::SmaaParameters;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_impostor_parameters::StaticImpostorParameters;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::water_blur_parameters::WaterBlurParameters;
use crate::pass::water_blur_uniform::WaterBlurUniform;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_reflection_parameters::WaterReflectionParameters;
use crate::pass::water_surface_parameters::WaterSurfaceParameters;
use crate::pass::water_uniform::WaterUniform;

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
  let mut water_blur: ShaderBindings = ShaderBindings::new();
  let mut present: ShaderBindings = ShaderBindings::new();
  let mut overlay: ShaderBindings = ShaderBindings::new();
  let mut upscale: ShaderBindings = ShaderBindings::new();
  let mut bloom: ShaderBindings = ShaderBindings::new();
  let mut fxaa: ShaderBindings = ShaderBindings::new();
  let mut smaa: ShaderBindings = ShaderBindings::new();
  let mut sky: ShaderBindings = ShaderBindings::new();
  let mut particles: ShaderBindings = ShaderBindings::new();
  let mut static_cull: ShaderBindings = ShaderBindings::new();
  let mut static_draw: ShaderBindings = ShaderBindings::new();
  let mut static_impostor: ShaderBindings = ShaderBindings::new();

  structs
    .declare::<LightingUniform>()
    .declare::<WaterUniform>()
    .declare::<WaterBlurUniform>()
    .declare::<PresentUniform>()
    .declare::<UpscaleUniform>()
    .declare::<BloomUniform>()
    .declare::<ParticleVertex>()
    .declare::<ParticleSurfaceRecord>();
  StaticCullParameters::declare(&mut structs);
  StaticDrawParameters::declare(&mut structs);
  StaticImpostorParameters::declare(&mut structs);
  water
    .add::<WaterDepthParameters>()?
    .add::<WaterReflectionParameters<'_>>()?
    .add::<WaterSurfaceParameters<'_>>()?;
  water_blur.add::<WaterBlurParameters<'_>>()?;
  present.add::<PresentParameters<'_>>()?;
  overlay.add::<OverlayParameters>()?;
  upscale.add::<UpscaleParameters>()?;
  bloom.add::<BloomParameters<'_>>()?;
  fxaa.add::<FxaaParameters<'_>>()?;
  smaa.add::<SmaaParameters<'_>>()?;
  sky.add::<SkyParameters<'_>>()?;
  particles.add::<ParticleParameters<'_>>()?;
  static_cull.add::<StaticCullParameters>()?;
  static_draw.add::<StaticDrawParameters>()?;
  static_impostor.add::<StaticImpostorParameters>()?;

  Ok(vec![
    ("generated/structs", structs.to_wgsl()),
    ("generated/static/water", format!("{STRUCTS_IMPORT}{}", water.to_wgsl())),
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
