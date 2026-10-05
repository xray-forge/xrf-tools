/// A buffer a frame graph knows: one it makes for the frame or one imported from outside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GraphBuffer {
  pub(crate) index: u32,
}

impl GraphBuffer {
  pub(crate) fn new(index: usize) -> Self {
    Self { index: index as u32 }
  }

  pub(crate) fn get_index(self) -> usize {
    self.index as usize
  }
}
