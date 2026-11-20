/// Where a visual's vertices or indices were packed: the buffer, and the first of them in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelGeomRange {
  pub buffer: u32,
  pub start: u32,
}
