use crate::pack::visual_index_fault::VisualIndexFault;

/// A run of a mesh's indices one draw reads, checked before any of it reaches a GPU: every packer's one way of reading
/// a window, so none draws past its indices or a vertex it was not given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VisualIndexWindow {
  /// Where in the mesh's indices it starts, on a triangle.
  pub(crate) offset: u32,
  /// Indices it reads, whole triangles.
  pub(crate) count: u32,
}

impl VisualIndexWindow {
  /// Indices a triangle takes.
  const TRIANGLE: u32 = 3;

  /// One of the engine's slide windows: `triangles` triangles from `offset`.
  pub(crate) const fn of_triangles(offset: u32, triangles: u16) -> Self {
    Self {
      count: triangles as u32 * Self::TRIANGLE,
      offset,
    }
  }

  /// Every whole triangle of a mesh of `index_count` indices.
  pub(crate) const fn whole(index_count: u32) -> Self {
    Self {
      count: index_count / Self::TRIANGLE * Self::TRIANGLE,
      offset: 0,
    }
  }

  /// Checks it starts on a triangle and lies within `available` indices.
  pub(crate) const fn check(self, available: u32) -> Result<(), VisualIndexFault> {
    if !self.offset.is_multiple_of(Self::TRIANGLE) {
      return Err(VisualIndexFault::Misaligned { offset: self.offset });
    }

    // Widened, so a window from the file cannot wrap its way back inside.
    if self.offset as u64 + self.count as u64 > available as u64 {
      return Err(VisualIndexFault::Outside {
        available,
        count: self.count,
        offset: self.offset,
      });
    }

    Ok(())
  }

  /// Where it starts among indices stored from `base`, which is what a buffer is asked to read from.
  pub(crate) const fn start_at(self, base: u32) -> Result<u32, VisualIndexFault> {
    match base.checked_add(self.offset) {
      Some(start) => Ok(start),
      None => Err(VisualIndexFault::Unaddressable {
        base,
        offset: self.offset,
      }),
    }
  }

  /// Its indices out of a mesh's own, checked against them and against the `vertex_count` vertices they name.
  pub(crate) fn select(self, indices: &[u16], vertex_count: u32) -> Result<&[u16], VisualIndexFault> {
    self.check(u32::try_from(indices.len()).unwrap_or(u32::MAX))?;

    let selected: &[u16] = &indices[self.offset as usize..(self.offset + self.count) as usize];

    Self::check_vertices(selected, vertex_count)?;

    Ok(selected)
  }

  /// Checks every index names one of `vertex_count` vertices.
  pub(crate) fn check_vertices(indices: &[u16], vertex_count: u32) -> Result<(), VisualIndexFault> {
    match indices.iter().find(|index| u32::from(**index) >= vertex_count) {
      Some(index) => Err(VisualIndexFault::Stray {
        index: *index,
        vertex_count,
      }),
      None => Ok(()),
    }
  }
}
