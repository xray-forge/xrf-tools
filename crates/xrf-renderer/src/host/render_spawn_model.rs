use xrf_material::XraySurfaceDescriptor;
use xrf_visual::VisualPackage;

/// One spawned visual as its objects stand: packed and posed, with how each of its submeshes is drawn.
#[derive(Debug)]
pub struct RenderSpawnModel {
  /// Its name, as [`crate::RenderLevelSpawn::visuals`] names it.
  pub name: String,
  /// Posed as its objects stand, its skin baked away.
  pub package: VisualPackage,
  /// How each submesh is drawn, in their order.
  pub surfaces: Vec<XraySurfaceDescriptor>,
}
