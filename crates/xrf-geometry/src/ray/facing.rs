/// Which sides of a triangle a ray test meets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
  /// Only the side `(p1 - p0) x (p2 - p0)` points toward, the side a ray running against that normal meets: the
  /// engine's `OPT_CULL`.
  Front,
  /// Either side.
  Both,
}
