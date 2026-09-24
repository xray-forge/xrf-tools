/// One of the engine's slide windows of a progressive mesh: where in its indices it starts, and triangles it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SectorWindow {
  pub(crate) offset: u32,
  pub(crate) triangles: u32,
}

impl SectorWindow {
  /// Indices it draws.
  pub(crate) const fn get_index_count(&self) -> u32 {
    self.triangles * 3
  }
}
