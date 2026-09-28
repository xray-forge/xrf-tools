use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::XrfResult;
use xrf_level::LevelGeomSource;
use xrf_ogf::OgfGeometryContainerChunk;

use crate::pack::visual_conversion::reverse_triangle_winding;
use crate::pack::visual_index_window::VisualIndexWindow;

/// The run of `level.geom` indices one drawable names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SectorIndexRange {
  pub(crate) buffer: u32,
  pub(crate) base: u32,
  pub(crate) count: u32,
}

impl SectorIndexRange {
  /// The run a geometry container declares.
  pub(crate) const fn of(container: &OgfGeometryContainerChunk) -> Self {
    Self {
      base: container.index_base,
      buffer: container.index_buffer_id,
      count: container.index_count,
    }
  }

  /// Reads one window of the run, checked against the run and against the `vertex_count` vertices it names, moved
  /// onto the vertices packed from `onto` and wound for renderer space.
  ///
  /// # Errors
  ///
  /// Returns an error when the window leaves the run or the buffer, or an index names a vertex past `vertex_count`.
  pub(crate) fn read_window<T: ByteOrder, D: ChunkDataSource>(
    &self,
    source: &LevelGeomSource<D>,
    window: VisualIndexWindow,
    vertex_count: u32,
    onto: u32,
  ) -> XrfResult<Vec<u32>> {
    window.check(self.count)?;

    let indices: Vec<u16> = source.read_indices::<T>(self.buffer, window.start_at(self.base)?, window.count)?;

    VisualIndexWindow::check_vertices(&indices, vertex_count)?;

    let mut rebased: Vec<u32> = indices.iter().map(|index| onto + u32::from(*index)).collect();

    reverse_triangle_winding(&mut rebased);

    Ok(rebased)
  }
}
