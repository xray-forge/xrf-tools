/// What a graph buffer is: its size. Its usage is not stated; the graph gathers it from the accesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphBufferDescriptor {
  pub label: &'static str,
  pub size: u64,
}

impl GraphBufferDescriptor {
  pub fn new(label: &'static str, size: u64) -> Self {
    Self { label, size }
  }
}
