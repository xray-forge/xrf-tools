use glam::Vec4;
use xrf_renderer_core::{PassParameters, StorageArray, UniformBinding};

use crate::feature::scene::grass::draw::grass_wind_uniform::GrassWindUniform;
use crate::feature::scene::grass::level::grass_model_record::GrassModelRecord;

/// What the grass's draw reads past the textures (`grass/grass`): the items the planting sorted by model, the models
/// and the sway.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct GrassDrawParameters {
  #[storage]
  pub sorted: StorageArray<Vec4>,
  #[storage]
  pub models: StorageArray<GrassModelRecord>,
  #[uniform]
  pub wind: UniformBinding<GrassWindUniform>,
}
