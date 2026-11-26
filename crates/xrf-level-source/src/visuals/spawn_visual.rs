use std::sync::Arc;

use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{VisualPackage, VisualRestPose};

use crate::parts::LevelTextureReference;

/// A spawned object's visual: packed, posed at rest, and dressed.
pub struct SpawnVisual {
  pub package: VisualPackage,
  /// The `idle` cycle's first frame, or the bind pose; `None` for a visual without bones.
  pub rest: Option<Arc<VisualRestPose>>,
  /// One per submesh, in order.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// The resolved textures its surfaces bind: each base with its bump pair and detail.
  pub textures: Vec<LevelTextureReference>,
}
