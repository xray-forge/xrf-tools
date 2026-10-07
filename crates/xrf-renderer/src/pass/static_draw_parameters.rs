use glam::{UVec2, Vec4};
use xrf_renderer_core::{PassParameters, StorageArray, UniformBinding};

use crate::pass::wind_uniform::WindUniform;
use crate::scene::static_scene::static_cluster::StaticCluster;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_surface::StaticSurface;

/// What a static draw pulls its vertices through: a view's visible list, the clusters, slots and places it names, one
/// layout's vertex words through the shared indices, the surfaces, the wind, and skinned models' links and bones.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct StaticDrawParameters {
  #[storage]
  pub clusters: StorageArray<StaticCluster>,
  #[storage]
  pub slots: StorageArray<StaticSlot>,
  #[storage]
  pub places: StorageArray<StaticPlace>,
  #[storage]
  pub surfaces: StorageArray<StaticSurface>,
  #[storage]
  pub indices: StorageArray<u32>,
  #[storage]
  pub lists: StorageArray<UVec2>,
  #[storage]
  pub words: StorageArray<u32>,
  #[uniform]
  pub wind: UniformBinding<WindUniform>,
  /// Skinned models' links, two words a vertex: four bones' indices as bytes, then their weights as bytes.
  #[storage]
  pub skins: StorageArray<u32>,
  /// Skinned objects' bone matrices, three rows a bone, from its bind to where it stands.
  #[storage]
  pub bones: StorageArray<Vec4>,
}
