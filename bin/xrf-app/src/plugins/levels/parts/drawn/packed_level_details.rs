use xrf_material::XraySurfaceDescriptor;
use xrf_visual::DetailsPackage;

use crate::plugins::levels::state::LevelTextureReference;

/// A level's grass, packed and dressed.
pub struct PackedLevelDetails {
  pub package: DetailsPackage,
  pub surfaces: Vec<XraySurfaceDescriptor>,
  pub textures: Vec<LevelTextureReference>,
}
