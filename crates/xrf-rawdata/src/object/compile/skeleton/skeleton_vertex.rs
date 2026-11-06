use xrf_math::{Vector2d, Vector3d};

/// One bone a skeleton vertex follows, and how strongly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkeletonVertexBone {
  /// The bone's index in the object's bones.
  pub bone: u16,
  pub weight: f32,
}

/// One vertex of a skeleton split, `SSkelVert` as `CExportSkeleton::PrepareGeometry` builds it.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  pub texture_coordinate: Vector2d,
  /// The bones it follows by ascending index, their weights normalised. A vertex of a face whose corners all follow
  /// one bone lists that bone twice, the copy weightless, as the export pads it; it then packs apart from the same
  /// corner of a face that blends, which lists the bone once.
  pub bones: Vec<SkeletonVertexBone>,
}
