use xrf_math::Vector3d;

/// One node as xrAI works on it (`vertex`, `xrAI/compiler.h:17`): its four links, by node index in `build.aimap`'s
/// order, the unit normal of its plane, where it stands, and its covers in each of the four directions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AiVertex {
  /// Left, forward, right, back; [`Self::NO_LINK`] for none.
  pub links: [u32; 4],
  pub normal: Vector3d,
  pub position: Vector3d,
  /// Cover from 1.5 m up, 0 to 1.
  pub high_cover: [f32; 4],
  /// Cover from 0.6 m up, 0 to 1.
  pub low_cover: [f32; 4],
}

impl AiVertex {
  /// xrAI's `InvalidNode`, a three-byte link's widest value.
  pub(crate) const NO_LINK: u32 = (1 << 24) - 1;
}
