use glam::Vec4;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::SectorSurface;

/// One submesh of a packed model: its run of the model's indices, the clusters it is cut into, and what it wears.
#[derive(Clone, Debug)]
pub struct StaticModelPart {
  /// The clusters of its run, as `(first index, triangles, sphere)`, the index counted in the model's indices.
  pub clusters: Vec<(u32, u32, Vec4)>,
  /// Its shader and base texture, as a sector names a surface.
  pub surface: SectorSurface,
  pub descriptor: Option<XraySurfaceDescriptor>,
}
