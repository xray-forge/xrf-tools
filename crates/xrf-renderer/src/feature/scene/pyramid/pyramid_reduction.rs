/// Which depth a depth pyramid's texel keeps of the four below it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PyramidReduction {
  /// The farthest, which an occlusion cull tests against.
  Farthest,
  /// The nearest, which the reflections' rays step over.
  Nearest,
}
