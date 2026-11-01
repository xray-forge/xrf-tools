use glam::{UVec4, Vec4};
use xrf_renderer_core::{PassParameters, StorageArray};

use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_surface::StaticSurface;

/// What the impostors' draw reads: the impostors, their corners, the cull's terms and list of those drawn, and the
/// surfaces they wear.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct StaticImpostorParameters {
  #[storage]
  pub impostors: StorageArray<StaticImpostor>,
  /// Two a corner: its position and hemisphere term, then its atlas coordinate and sun term.
  #[storage]
  pub corners: StorageArray<Vec4>,
  #[storage]
  pub terms: StorageArray<UVec4>,
  #[storage]
  pub impostor_list: StorageArray<u32>,
  #[storage]
  pub surfaces: StorageArray<StaticSurface>,
}
