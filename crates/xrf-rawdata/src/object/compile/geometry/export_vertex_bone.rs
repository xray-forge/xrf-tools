/// One bone a skeleton's vertex follows, and how strongly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportVertexBone {
  /// The bone's index in the object's bones.
  pub bone: u16,
  pub weight: f32,
}
