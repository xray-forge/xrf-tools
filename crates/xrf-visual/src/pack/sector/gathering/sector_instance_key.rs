use xrf_ogf::OgfGeometryContainerChunk;

use crate::pack::sector::gathering::sector_index_range::SectorIndexRange;
use crate::pack::sector::gathering::sector_vertex_range::SectorVertexRange;

/// What makes two placed copies of a mesh one instanced draw: the geometry, and the surface dressing it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SectorInstanceKey {
  pub(crate) vertices: SectorVertexRange,
  pub(crate) indices: SectorIndexRange,
  pub(crate) shader_id: u16,
}

impl SectorInstanceKey {
  /// The key a placed drawable comes to.
  pub(crate) const fn of(container: &OgfGeometryContainerChunk, shader_id: u16) -> Self {
    Self {
      indices: SectorIndexRange::of(container),
      shader_id,
      vertices: SectorVertexRange::of(container),
    }
  }
}
