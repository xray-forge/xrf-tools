use serde::Serialize;

use crate::object::compile::tangent_method::TangentMethod;

/// What an object's compile into a file wrote ([`ObjectCompiler::compile_to_path`]).
///
/// [`ObjectCompiler::compile_to_path`]: crate::ObjectCompiler::compile_to_path
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectCompileReport {
  pub source: String,
  pub destination: String,
  pub children: usize,
  pub vertices: usize,
  pub faces: usize,
  /// Faces the export drops: degenerate once packed, or, in a skeleton, repeating one already kept.
  pub dropped_faces: usize,
  pub bones: usize,
  pub bytes: usize,
  /// The tangent method the vertices were given, `actorEditor` or `mikkTSpace`, or `null` for a static visual or a
  /// detail model, which store no tangents.
  pub tangents: Option<TangentMethod>,
}
