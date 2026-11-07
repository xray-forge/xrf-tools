use crate::pack::buffer::visual_index_fault::VisualIndexFault;
use crate::pack::buffer::visual_index_window::VisualIndexWindow;

/// A mesh's whole run of indices beside the vertices they name, checked against them once.
///
/// Every window of a mesh lies within its run, so a run naming only vertices the mesh has vouches for each window, which
/// is then checked for its bounds alone. A progressive mesh stores dozens of windows, each nearly the whole run, and
/// reading each one's indices again was most of what packing a model cost.
pub(crate) struct VisualIndexRun<'a> {
  indices: &'a [u16],
  vertex_count: u32,
  /// Whether every index names one of the vertices; a run that fails has each window's own indices read instead.
  is_sound: bool,
}

impl<'a> VisualIndexRun<'a> {
  /// The run `indices`, naming vertices of a mesh holding `vertex_count` of them.
  pub(crate) fn new(indices: &'a [u16], vertex_count: u32) -> Self {
    Self {
      indices,
      vertex_count,
      is_sound: VisualIndexWindow::check_vertices(indices, vertex_count).is_ok(),
    }
  }

  /// Checks a window starts on a triangle, lies within the run and names only vertices the mesh has.
  pub(crate) fn check(&self, window: VisualIndexWindow) -> Result<(), VisualIndexFault> {
    match self.is_sound {
      true => window.check(u32::try_from(self.indices.len()).unwrap_or(u32::MAX)),
      false => window.select(self.indices, self.vertex_count).map(|_| ()),
    }
  }
}
