use serde::Serialize;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::VisualDescription;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// One visual spawned objects are drawn as: what its pack says, the pose it stands in, and what dresses it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnModelDescription {
  /// The visual as the objects name it, which its bytes are read by.
  pub name: String,
  pub description: VisualDescription,
  /// Twelve floats a bone, basis then translation, in model and renderer space: the pose it stands in, the `idle`
  /// cycle's first frame or the bind pose. `None` for a visual without bones.
  pub rest: Option<Vec<f32>>,
  /// How the renderer draws each submesh, in their order.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// What each texture a submesh binds resolved to.
  pub textures: Vec<LevelTextureReference>,
}
