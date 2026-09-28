use xrf_ogf::OgfGeometryContainerChunk;

/// The range of `level.geom` one drawable names, which is what two drawables share geometry by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SectorVertexRange {
  pub(crate) buffer: u32,
  pub(crate) base: u32,
  pub(crate) count: u32,
}

impl SectorVertexRange {
  /// The range a geometry container declares.
  pub(crate) const fn of(container: &OgfGeometryContainerChunk) -> Self {
    Self {
      base: container.vertex_base,
      buffer: container.vertex_buffer_id,
      count: container.vertex_count,
    }
  }
}
