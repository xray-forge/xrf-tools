use std::mem;

use crate::window::geometry::window_geometry::WindowGeometry;

/// What the main window is at, and what the disk is owed about it.
#[derive(Debug)]
pub(super) struct WindowGeometryState {
  geometry: WindowGeometry,
  written: Option<WindowGeometry>,
  is_settling: bool,
  is_saved: bool,
}

impl WindowGeometryState {
  /// The state of a window last at `geometry`.
  pub(super) fn new(geometry: WindowGeometry) -> Self {
    Self {
      geometry,
      written: None,
      is_settling: false,
      is_saved: true,
    }
  }

  /// Where the window was when it was last taken.
  pub(super) fn get_geometry(&self) -> WindowGeometry {
    self.geometry
  }

  /// Take where the window is now.
  pub(super) fn record(&mut self, geometry: WindowGeometry) {
    self.geometry = geometry;
  }

  /// Whether this record is the one that has to start a settle delay, rather than joining one under way.
  pub(super) fn begin_settling(&mut self) -> bool {
    !mem::replace(&mut self.is_settling, true)
  }

  /// The rectangle worth writing, which a window nobody has measured and a rectangle already written are not.
  pub(super) fn take_unwritten(&mut self) -> Option<WindowGeometry> {
    self.is_settling = false;

    if !self.geometry.is_measured() || self.written == Some(self.geometry) {
      return None;
    }

    self.written = Some(self.geometry);
    self.is_saved = false;

    Some(self.geometry)
  }

  /// Whether a write is still owed a save, answered once per write.
  pub(super) fn take_unsaved(&mut self) -> bool {
    !mem::replace(&mut self.is_saved, true)
  }
}
