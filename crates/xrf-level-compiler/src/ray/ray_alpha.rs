/// How much of a shadow ray an alpha texel lets through, by the alpha `a` it reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RayAlpha {
  /// `1 - a`: xrDO_Light's and xrAI's.
  Linear,
  /// `1 - a²`: xrLC's.
  Squared,
}
