use glam::Vec4;
use xrf_renderer_core::{PassParameters, StorageArray, UniformBinding};

use crate::pass::grass_model_record::GrassModelRecord;
use crate::pass::grass_wind_uniform::GrassWindUniform;

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
