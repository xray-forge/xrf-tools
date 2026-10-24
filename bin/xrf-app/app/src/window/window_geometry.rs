use serde::{Deserialize, Serialize};

/// Where the main window was last left, in the physical pixels the desktop addresses its monitors with.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowGeometry {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
  /// Scale factor of the monitor the rectangle was measured on, which decides how it is read on a different one.
  pub scale: f64,
  pub maximized: bool,
}

impl Default for WindowGeometry {
  /// The rectangle no window has: every field a partial file leaves out reads as unmeasured.
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
      scale: 0.0,
      maximized: false,
    }
  }
}

impl WindowGeometry {
  /// Whether the rectangle describes a window at all, which a hand-edited or truncated file need not.
  pub fn is_measured(&self) -> bool {
    self.width > 0 && self.height > 0 && self.scale.is_finite() && self.scale > 0.0
  }
}
