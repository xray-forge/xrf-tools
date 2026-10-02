use serde::{Deserialize, Serialize};

/// A rectangle of a window's client area, in device pixels from its top left corner.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderRect {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
}

impl RenderRect {
  /// Whether the rectangle covers any pixel at all.
  pub fn is_empty(&self) -> bool {
    self.width == 0 || self.height == 0
  }

  /// The part of the rectangle inside a surface of the given size, or `None` where nothing of it is.
  pub fn clip(&self, width: u32, height: u32) -> Option<RenderRect> {
    let left: i64 = (self.x as i64).max(0);
    let top: i64 = (self.y as i64).max(0);
    let right: i64 = (self.x as i64 + self.width as i64).min(width as i64);
    let bottom: i64 = (self.y as i64 + self.height as i64).min(height as i64);

    (right > left && bottom > top).then(|| RenderRect {
      x: left as i32,
      y: top as i32,
      width: (right - left) as u32,
      height: (bottom - top) as u32,
    })
  }
}
