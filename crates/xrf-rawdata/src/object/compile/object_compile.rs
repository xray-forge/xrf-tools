use crate::object::compile::object_compile_options::TangentMethod;

/// An editable object compiled into an OGF: its bytes, and what the export made of the geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectCompile {
  pub ogf: Vec<u8>,
  pub children: usize,
  pub vertices: usize,
  pub faces: usize,
  /// Faces the export left out: degenerate once packed, or, in a skeleton, repeating one already kept.
  pub dropped_faces: usize,
  /// The tangent method the vertices were given, or `None` for a static visual, which stores no tangents.
  pub tangents: Option<TangentMethod>,
}
