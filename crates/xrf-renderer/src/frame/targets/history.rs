/// What a feature keeps from frame to frame as two alike: the side this frame writes and the side it reads, which the
/// frame before wrote, swapped once the frame is submitted.
pub(crate) struct History<T> {
  sides: [T; 2],
  /// The side this frame writes.
  index: usize,
  /// Whether the side this frame reads holds a frame.
  is_valid: bool,
  /// Frames swapped since it was made or restarted.
  frame: u32,
}

impl<T> History<T> {
  /// Two sides made by `make`, given each side's index; neither holds a frame yet.
  pub(crate) fn new(mut make: impl FnMut(usize) -> T) -> Self {
    Self {
      sides: [make(0), make(1)],
      index: 0,
      is_valid: false,
      frame: 0,
    }
  }

  /// The history `slot` holds where it still fits (`is_fitting`), made afresh by `make` otherwise.
  pub(crate) fn fit(
    slot: &mut Option<Self>,
    is_fitting: impl FnOnce(&Self) -> bool,
    make: impl FnMut(usize) -> T,
  ) -> &mut Self {
    if slot.as_ref().is_some_and(|history| !is_fitting(history)) {
      *slot = None;
    }

    slot.get_or_insert_with(|| Self::new(make))
  }

  /// The side this frame writes.
  pub(crate) fn get_written(&self) -> &T {
    &self.sides[self.index]
  }

  /// The side this frame reads, which the frame before wrote.
  pub(crate) fn get_read(&self) -> &T {
    &self.sides[1 - self.index]
  }

  /// Both sides, the written first.
  pub(crate) fn get_sides(&self) -> [&T; 2] {
    [self.get_written(), self.get_read()]
  }

  /// Whether the side this frame reads holds a frame.
  pub(crate) fn is_valid(&self) -> bool {
    self.is_valid
  }

  /// Frames swapped since it was made or restarted.
  pub(crate) fn get_frame(&self) -> u32 {
    self.frame
  }

  /// The side written becomes the side the next frame reads.
  pub(crate) fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
    self.frame = self.frame.wrapping_add(1);
  }

  /// Leaves the next frame nothing to read.
  pub(crate) fn invalidate(&mut self) {
    self.is_valid = false;
  }

  /// Leaves the next frame nothing to read and counts its frames from none again, as if just made.
  pub(crate) fn restart(&mut self) {
    self.is_valid = false;
    self.frame = 0;
  }
}
