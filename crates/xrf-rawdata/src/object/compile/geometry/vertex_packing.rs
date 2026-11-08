/// How an export packs a split's corners into vertices (see [`super::vertex_packer::VertexPacker`]): the grid of
/// candidates it lays over the object's box, and whether a face packing onto the vertices of one already added is
/// dropped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VertexPacking {
  /// Cells along each axis, past the first.
  pub(crate) cells: [u32; 3],
  /// How far the grid's box reaches past the object's on every side, before the `EPS` both exports add.
  pub(crate) growth: f32,
  /// Whether a face whose corners pack onto the vertices of a face already added, in any winding, is dropped.
  pub(crate) drops_repeated_faces: bool,
}

impl VertexPacking {
  /// `CSkeletonCollectorPacked`: `clpSMX`, `clpSMY` and `clpSMZ` cells over the object's box grown by
  /// `g_EpsSkelPositionDelta`, a repeated face dropped.
  pub(crate) const SKELETON: Self = Self {
    cells: [28, 16, 28],
    growth: 1e-3,
    drops_repeated_faces: true,
  };

  /// `CObjectOGFCollectorPacked`: `clpOGFMX`, `clpOGFMY` and `clpOGFMZ` cells over the object's box, a face dropped
  /// only where two of its corners pack onto one vertex.
  pub(crate) const RIGID: Self = Self {
    cells: [48, 16, 48],
    growth: 0.0,
    drops_repeated_faces: false,
  };
}
