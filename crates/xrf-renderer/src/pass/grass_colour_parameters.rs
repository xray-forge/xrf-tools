use glam::Vec4;
use xrf_renderer_core::{PassParameters, StorageArray, StorageArrayMut};

use crate::pass::grass_model_record::GrassModelRecord;

/// What the grass's colour pass reads and writes: the level's detail models, and each one's colour where its tufts
/// stand and the share of its texture they cover.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct GrassColourParameters {
  #[storage]
  pub grass_models: StorageArray<GrassModelRecord>,
  #[storage]
  pub grass_colours: StorageArrayMut<Vec4>,
}
