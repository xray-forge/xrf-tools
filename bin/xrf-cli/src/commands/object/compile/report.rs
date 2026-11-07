use serde::Serialize;
use xrf_rawdata::TangentMethod;

/// What `object compile` wrote, as its JSON report says it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectCompileReport {
  pub(crate) source: String,
  pub(crate) destination: String,
  pub(crate) children: usize,
  pub(crate) vertices: usize,
  pub(crate) faces: usize,
  /// Faces the export drops: degenerate once packed, or repeating one already kept.
  pub(crate) dropped_faces: usize,
  pub(crate) bones: usize,
  pub(crate) bytes: usize,
  /// The tangent method the vertices were given, `actorEditor` or `mikkTSpace`.
  pub(crate) tangents: TangentMethod,
}
