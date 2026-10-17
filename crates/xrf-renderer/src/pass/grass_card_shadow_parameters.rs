use xrf_renderer_core::{PassParameters, StorageArray};

use crate::pass::grass_model_record::GrassModelRecord;

/// What the grass cards' draw into a shadow's map reads past the textures (`grass/card_shadow`): the cards' corners,
/// their triangles' records and the models.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct GrassCardShadowParameters {
  #[storage]
  pub card_vertices: StorageArray<u32>,
  #[storage]
  pub card_triangles: StorageArray<u32>,
  #[storage]
  pub models: StorageArray<GrassModelRecord>,
}
