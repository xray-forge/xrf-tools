/// How precisely a compiled vertex stores its normal, which decides how far apart two normals may come back and still
/// be one: a smoothed normal shared across a seam, or a two-sided copy's normal turned around.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NormalPrecision {
  /// Three floats, as an OGF's vertices store a normal: the export sums a fan's face normals once per corner, starting
  /// from a different face each time, so one smoothed normal can differ in its last bits. A hard edge differs by far
  /// more.
  Float,
  /// A byte a component, `(n + 1) * 127.5` truncated, as a level's vertices store a normal (`D3DCOLOR`). A mirrored
  /// copy's inverted normal truncates to one step off the exact negation: over vanilla CoP's 22 levels at full detail,
  /// 11,692,886 mirrored corners stand one step from their mirror's negation and 36 at none. Corners sharing a position
  /// either agree exactly (2,599,789) or differ by over 20 steps (8,370,887), but for 5,372 a step apart.
  Byte,
}

impl NormalPrecision {
  /// Component difference within which two float normals are one.
  const FLOAT_TOLERANCE: f32 = 1e-3;

  /// One quantisation step, `1 / 127.5`, and the float error its decoding leaves.
  const BYTE_TOLERANCE: f32 = 1.0 / 127.5 + 1e-5;

  /// How far, component by component, two normals stored at this precision may differ and still be one.
  pub(crate) const fn get_tolerance(self) -> f32 {
    match self {
      Self::Float => Self::FLOAT_TOLERANCE,
      Self::Byte => Self::BYTE_TOLERANCE,
    }
  }

  /// The tolerance of comparing normals of `self` and `other` precision: the coarser one's.
  pub(crate) fn get_tolerance_with(self, other: Self) -> f32 {
    self.get_tolerance().max(other.get_tolerance())
  }
}
