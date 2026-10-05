use glam::{UVec2, UVec4, Vec4};
use xrf_renderer_core::{GraphTexture, PassParameters, ShaderAtomicU32, StorageArray, StorageArrayMut, UniformBinding};

use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::scene::static_scene::static_cluster::StaticCluster;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_region::StaticRegion;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_slot::StaticSlot;

/// What a cull of the static scene reads and writes: the scene's records, the view's visible list and draw arguments,
/// and for the camera's two phases the candidates, the late arguments, the depth pyramid and the view it was reduced
/// through.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct StaticCullParameters {
  #[storage]
  pub clusters: StorageArray<StaticCluster>,
  #[storage]
  pub spheres: StorageArray<Vec4>,
  #[storage]
  pub slots: StorageArray<StaticSlot>,
  #[storage]
  pub places: StorageArray<StaticPlace>,
  #[storage]
  pub rows: StorageArray<StaticRow>,
  #[storage]
  pub regions: StorageArray<StaticRegion>,
  /// Each visible cluster and its place, in its batch's run.
  #[storage]
  pub lists: StorageArrayMut<UVec2>,
  /// Each batch's draw arguments, then the cull's counts and the impostors' draw.
  #[storage]
  pub args: StorageArrayMut<ShaderAtomicU32>,
  #[uniform]
  pub params: UniformBinding<StaticCullParams>,
  #[storage]
  pub candidates: StorageArrayMut<UVec2>,
  /// Each batch's late draw arguments, then the late phase's dispatch and the candidates' count.
  #[storage]
  pub late: StorageArrayMut<ShaderAtomicU32>,
  #[texture(d2, unfilterable)]
  pub pyramid: GraphTexture,
  #[uniform]
  pub occlusion: UniformBinding<StaticOcclusionUniform>,
  #[storage]
  pub impostors: StorageArray<StaticImpostor>,
  /// Each impostor's best facet, the next best, its fade and blend bytes, and what its level of detail draws.
  #[storage]
  pub terms: StorageArrayMut<UVec4>,
  #[storage]
  pub impostor_list: StorageArrayMut<u32>,
}
