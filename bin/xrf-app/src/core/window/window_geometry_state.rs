use std::mem;

use crate::core::window::window_geometry::WindowGeometry;

/// What the main window is at, and what the disk is owed about it.
#[derive(Debug)]
pub struct WindowGeometryState {
  geometry: WindowGeometry,
  written: Option<WindowGeometry>,
  is_settling: bool,
  is_saved: bool,
}

impl WindowGeometryState {
  pub fn new(geometry: WindowGeometry) -> Self {
    Self {
      geometry,
      written: None,
      is_settling: false,
      is_saved: true,
    }
  }

  /// Where the window was when it was last taken.
  pub fn get_geometry(&self) -> WindowGeometry {
    self.geometry
  }

  /// Take where the window is now.
  pub fn record(&mut self, geometry: WindowGeometry) {
    self.geometry = geometry;
  }

  /// Whether this record is the one that has to start a settle delay, rather than joining one under way.
  pub fn begin_settling(&mut self) -> bool {
    !mem::replace(&mut self.is_settling, true)
  }

  /// The rectangle worth writing, which a window nobody has measured and a rectangle already written are not.
  pub fn take_unwritten(&mut self) -> Option<WindowGeometry> {
    self.is_settling = false;

    if !self.geometry.is_measured() || self.written == Some(self.geometry) {
      return None;
    }

    self.written = Some(self.geometry);
    self.is_saved = false;

    Some(self.geometry)
  }

  /// Whether a write is still owed a save, answered once per write.
  pub fn take_unsaved(&mut self) -> bool {
    !mem::replace(&mut self.is_saved, true)
  }
}
