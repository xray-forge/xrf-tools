use xrf_material::XraySurfaceDescriptor;
use xrf_visual::VisualPackage;

use crate::host::render_model_skeleton::RenderModelSkeleton;

/// One spawned visual as its objects stand: packed and posed, with how each of its submeshes is drawn.
#[derive(Debug)]
pub struct RenderSpawnModel {
  /// Its name, as [`crate::RenderLevelSpawn::visuals`] names it.
  pub name: String,
  /// Posed as its objects stand, its skin baked away; or, with [`Self::skeleton`], still carrying its skin.
  pub package: VisualPackage,
  /// How each submesh is drawn, in their order.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// The skeleton its skin hangs from, for a model the renderer poses itself; `None` for one posed already.
  pub skeleton: Option<RenderModelSkeleton>,
}
