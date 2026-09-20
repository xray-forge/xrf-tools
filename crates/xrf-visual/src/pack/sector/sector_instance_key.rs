use xrf_ogf::OgfGeometryContainerChunk;

use crate::pack::sector::sector_vertex_range::SectorVertexRange;

/// What makes two placed copies of a mesh one instanced draw: the geometry, and the surface dressing it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SectorInstanceKey {
  pub(crate) vertices: SectorVertexRange,
  pub(crate) index_buffer: u32,
  pub(crate) index_base: u32,
  pub(crate) index_count: u32,
  pub(crate) shader_id: u16,
}

impl SectorInstanceKey {
  /// The key a placed drawable comes to.
  pub(crate) const fn of(container: &OgfGeometryContainerChunk, shader_id: u16) -> Self {
    Self {
      index_base: container.index_base,
      index_buffer: container.index_buffer_id,
      index_count: container.index_count,
      shader_id,
      vertices: SectorVertexRange::of(container),
    }
  }
}
