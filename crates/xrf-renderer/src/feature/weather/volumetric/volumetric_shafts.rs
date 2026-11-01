use xrf_renderer_core::GraphTexture;

use crate::feature::weather::volumetric::froxels::froxels::Froxels;

/// What the volumetric lighting answers for the sun shafts' composite: the shadowed lights' shafts, none where none
/// draws; whether the froxels stand in for both shafts; and the froxels, where they draw.
#[derive(Clone, Copy)]
pub struct VolumetricShafts {
  pub light_volumes: Option<GraphTexture>,
  pub is_froxeled: bool,
  pub froxels: Option<Froxels>,
}
