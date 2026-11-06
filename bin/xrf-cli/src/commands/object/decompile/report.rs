use serde::Serialize;
use xrf_rawdata::ObjectInference;

/// What `object decompile` wrote, as its JSON report says it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectDecompileReport {
  pub(crate) source: String,
  pub(crate) destination: String,
  pub(crate) sidecar: String,
  pub(crate) surfaces: usize,
  pub(crate) points: usize,
  pub(crate) faces: usize,
  pub(crate) two_sided_faces: usize,
  pub(crate) bones: usize,
  pub(crate) inferred: Vec<ObjectInference>,
}
