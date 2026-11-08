use crate::object::compile::object_compile_options::TangentMethod;

/// An editable object compiled: the file's bytes, an OGF or a detail model, and what the export made of the geometry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectCompile {
  pub bytes: Vec<u8>,
  pub children: usize,
  pub vertices: usize,
  pub faces: usize,
  /// Faces the export left out: degenerate once packed, or, in a skeleton or a detail model, repeating one already
  /// kept.
  pub dropped_faces: usize,
  /// The tangent method the vertices were given, or `None` for a static visual or a detail model, which store no
  /// tangents.
  pub tangents: Option<TangentMethod>,
}
