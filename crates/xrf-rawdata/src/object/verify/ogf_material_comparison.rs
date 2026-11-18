/// How a compiled texture and shader's geometry compares with the reference's.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OgfMaterialComparison {
  /// Compiled vertices one of the reference's stands in for.
  pub(crate) vertices_matched: usize,
  /// Compiled vertices that would match but for their normal.
  pub(crate) vertices_normal_only: usize,
  /// Reference faces the compile draws too.
  pub(crate) faces_matched: usize,
  /// Reference faces with two corners at one position.
  pub(crate) reference_flat_faces: usize,
  /// Reference faces the compile does not draw, and faces it draws the reference does not.
  pub(crate) faces_missing: usize,
  pub(crate) faces_extra: usize,
}
