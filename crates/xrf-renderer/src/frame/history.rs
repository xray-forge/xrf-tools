/// What a feature keeps from frame to frame as two alike: the side this frame writes and the side it reads, which the
/// frame before wrote, swapped once the frame is submitted.
pub struct History<T> {
  sides: [T; 2],
  /// The side this frame writes.
  index: usize,
  /// Whether the side this frame reads holds a frame.
  is_valid: bool,
  /// Frames swapped since it was made.
  frame: u32,
}

impl<T> History<T> {
  /// Two sides made by `make`, given each side's index; neither holds a frame yet.
  pub fn new(mut make: impl FnMut(usize) -> T) -> Self {
    Self {
      sides: [make(0), make(1)],
      index: 0,
      is_valid: false,
      frame: 0,
    }
  }

  /// The history `slot` holds where it still fits (`is_fitting`), made afresh by `make` otherwise.
  pub fn fit(
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
  pub fn get_written(&self) -> &T {
    &self.sides[self.index]
  }

  /// The side this frame reads, which the frame before wrote.
  pub fn get_read(&self) -> &T {
    &self.sides[1 - self.index]
  }

  /// Both sides, the written first.
  pub fn get_sides(&self) -> [&T; 2] {
    [self.get_written(), self.get_read()]
  }

  /// Whether the side this frame reads holds a frame.
  pub fn is_valid(&self) -> bool {
    self.is_valid
  }

  /// Frames swapped since it was made.
  pub fn get_frame(&self) -> u32 {
    self.frame
  }

  /// The side written becomes the side the next frame reads.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
    self.frame = self.frame.wrapping_add(1);
  }

  /// Leaves the next frame nothing to read.
  pub fn invalidate(&mut self) {
    self.is_valid = false;
  }
}
