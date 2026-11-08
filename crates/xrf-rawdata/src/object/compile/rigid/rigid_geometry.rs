use crate::object::compile::rigid::rigid_split::RigidSplit;

/// What a static object's meshes become on the way to an OGF: its children, in the order the export writes them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RigidGeometry {
  pub(crate) splits: Vec<RigidSplit>,
  /// Faces the export leaves out, as the editor reports them: degenerate ones, whose corners stand within `EPS` of
  /// each other or pack onto one vertex.
  pub(crate) dropped_faces: usize,
}
