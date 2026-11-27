/// A rectangle of a layer texture's pixels, from `left` and `top` up to but not including `right` and `bottom`; empty
/// where either side has no length.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LayerRegion {
  pub left: u32,
  pub top: u32,
  pub right: u32,
  pub bottom: u32,
}

impl LayerRegion {
  /// The region `width` by `height` pixels from `left` and `top`.
  pub(crate) fn new(left: u32, top: u32, width: u32, height: u32) -> Self {
    Self {
      left,
      top,
      right: left + width,
      bottom: top + height,
    }
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.right <= self.left || self.bottom <= self.top
  }

  pub(crate) fn get_width(&self) -> u32 {
    self.right.saturating_sub(self.left)
  }

  pub(crate) fn get_height(&self) -> u32 {
    self.bottom.saturating_sub(self.top)
  }

  /// The smallest region covering both, an empty one adding nothing.
  pub(crate) fn union(self, other: Self) -> Self {
    if self.is_empty() {
      return other;
    }

    if other.is_empty() {
      return self;
    }

    Self {
      left: self.left.min(other.left),
      top: self.top.min(other.top),
      right: self.right.max(other.right),
      bottom: self.bottom.max(other.bottom),
    }
  }
}
