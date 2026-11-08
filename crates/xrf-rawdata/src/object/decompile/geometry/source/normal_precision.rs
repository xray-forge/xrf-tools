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
  /// No normal at all, as a detail model's vertices (`fvfVertexIn`: a position and a texture coordinate): nothing tells
  /// a hard edge from a soft one or a face from its mirror, and `EDetail::Update` drops a mirrored copy anyway.
  Unstored,
}

impl NormalPrecision {
  /// Component difference within which two float normals are one.
  const FLOAT_TOLERANCE: f32 = 1e-3;

  /// One quantisation step, `1 / 127.5`, and the float error its decoding leaves.
  const BYTE_TOLERANCE: f32 = 1.0 / 127.5 + 1e-5;

  /// How far, component by component, two normals of `self` and `other` precision may differ and still be one: the
  /// coarser one's tolerance, or `None` where either stores no normal to compare.
  pub(crate) fn get_tolerance_with(self, other: Self) -> Option<f32> {
    match (self, other) {
      (Self::Unstored, _) | (_, Self::Unstored) => None,
      (Self::Byte, _) | (_, Self::Byte) => Some(Self::BYTE_TOLERANCE),
      (Self::Float, Self::Float) => Some(Self::FLOAT_TOLERANCE),
    }
  }
}
