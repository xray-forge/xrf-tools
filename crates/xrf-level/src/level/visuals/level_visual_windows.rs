use xrf_ogf::OgfSlideWindow;

use crate::geom::window::level_geom_slide_window::LevelGeomSlideWindow;

/// A progressive visual's slide windows, its levels of detail finest first, wherever it keeps them: a static
/// progressive mesh in its own `OGF_SWIDATA`, a progressive tree in the table of `level.geom` its container names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LevelVisualWindows<'a> {
  Own(&'a [OgfSlideWindow]),
  Table(&'a [LevelGeomSlideWindow]),
}

impl LevelVisualWindows<'_> {
  /// Each window, finest first.
  pub fn iter(&self) -> impl Iterator<Item = LevelGeomSlideWindow> + '_ {
    let (own, table): (&[OgfSlideWindow], &[LevelGeomSlideWindow]) = match self {
      Self::Own(windows) => (windows, &[]),
      Self::Table(windows) => (&[], windows),
    };

    own
      .iter()
      .map(|window| LevelGeomSlideWindow {
        offset: window.offset,
        triangles: window.num_tris,
        vertices: window.num_verts,
      })
      .chain(table.iter().copied())
  }

  /// The window drawing the visual's full detail, its first.
  pub fn get_full_detail(&self) -> Option<LevelGeomSlideWindow> {
    self.iter().next()
  }

  pub fn len(&self) -> usize {
    match self {
      Self::Own(windows) => windows.len(),
      Self::Table(windows) => windows.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}
