use xrf_geometry::XrlcTracer;
use xrf_level::BuildPrjFile;

use crate::input::compiler_shaders::CompilerShaders;
use crate::light::source::light_sources::LightSources;
use crate::shadow::shadow_materials::ShadowMaterials;

/// What lighting a project's multi-use models and placements takes ([`LightModels::of`]): the models through their own
/// faces, the placements through the level's ray model.
///
/// [`LightModels::of`]: crate::light::vertex::light_models::LightModels::of
#[derive(Clone, Copy)]
pub struct LightModelsInput<'a> {
  pub project: &'a BuildPrjFile,
  pub shaders: &'a CompilerShaders,
  pub lights: &'a LightSources,
  /// The models' own faces' shadow materials, loaded over [`LightModels::get_model_faces`].
  ///
  /// [`LightModels::get_model_faces`]: crate::light::vertex::light_models::LightModels::get_model_faces
  pub model_shadows: &'a ShadowMaterials,
  /// Where each model's faces start among them.
  pub model_starts: &'a [usize],
  /// The level's ray model, as xrLC builds it, and what each of its faces lets through.
  pub tracer: &'a XrlcTracer,
  pub shadows: &'a ShadowMaterials,
  /// The samples each vertex takes ([`LightQuality::get_model_samples`](crate::LightQuality::get_model_samples)).
  pub samples: u32,
}
