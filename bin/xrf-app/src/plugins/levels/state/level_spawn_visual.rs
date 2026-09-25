use std::sync::Arc;

use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{VisualPackage, VisualRestPose};

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// A visual a spawned object stands as: packed, posed as it stands still, and dressed.
pub struct LevelSpawnVisual {
  pub package: VisualPackage,
  /// Its bones as the object stands: the `idle` cycle's first frame, or the bind pose; `None` for a visual without
  /// bones.
  pub rest: Option<Arc<VisualRestPose>>,
  /// How the renderer draws each submesh, in their order.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// What each texture a submesh binds resolved to.
  pub textures: Vec<LevelTextureReference>,
}
