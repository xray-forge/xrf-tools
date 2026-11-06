use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;

/// What a skeleton object's meshes become on the way to an OGF: its children, in the order the export writes them.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonGeometry {
  pub splits: Vec<SkeletonSplit>,
  /// Faces the export leaves out, as the Actor Editor reports them: degenerate ones, whose corners stand within
  /// `EPS` of each other, and duplicates, whose corners packed into the vertices of a face already added.
  pub dropped_faces: usize,
}
