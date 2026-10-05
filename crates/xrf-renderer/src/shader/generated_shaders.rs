use xrf_error::XrfResult;
use xrf_renderer_core::{ShaderBindings, ShaderDeclarations};

use crate::pass::lighting_uniform::LightingUniform;
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

  structs
    .declare::<LightingUniform>()
    .declare::<WaterUniform>()
    .declare::<WaterBlurUniform>();
  water
    .add::<WaterDepthParameters>()?
    .add::<WaterReflectionParameters<'_>>()?
    .add::<WaterSurfaceParameters<'_>>()?;
  water_blur.add::<WaterBlurParameters<'_>>()?;

  Ok(vec![
    ("generated/structs", structs.to_wgsl()),
    ("generated/static/water", format!("{STRUCTS_IMPORT}{}", water.to_wgsl())),
    (
      "generated/frame/water_blur",
      format!("{STRUCTS_IMPORT}{}", water_blur.to_wgsl()),
    ),
  ])
}
