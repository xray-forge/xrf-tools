use crate::contract::layer::render_layer_mesh::RenderLayerMesh;

/// What a window's layer draws over everything else of it, its meshes in order: a developer tool's interface, its
/// console and stats. A host sends a whole layer each time it changes; the latest replaces the one before.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderLayer {
  pub meshes: Vec<RenderLayerMesh>,
}
