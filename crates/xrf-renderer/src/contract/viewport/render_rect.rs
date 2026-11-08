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
  /// The part of the rectangle inside a surface of the given size, or `None` where nothing of it is.
  pub fn clip(&self, width: u32, height: u32) -> Option<Self> {
    let left: i64 = i64::from(self.x).max(0);
    let top: i64 = i64::from(self.y).max(0);
    let right: i64 = (i64::from(self.x) + i64::from(self.width)).min(i64::from(width));
    let bottom: i64 = (i64::from(self.y) + i64::from(self.height)).min(i64::from(height));

    (right > left && bottom > top).then(|| Self {
      x: left as i32,
      y: top as i32,
      width: (right - left) as u32,
      height: (bottom - top) as u32,
    })
  }
}
