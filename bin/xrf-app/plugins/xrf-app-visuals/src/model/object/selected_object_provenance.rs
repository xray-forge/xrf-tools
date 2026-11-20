use serde::Serialize;
use xrf_rawdata::{ObjectInference, ObjectProvenance};

/// Where a decompiled object came from, as its sidecar says, so it is never taken for an authored one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedObjectProvenance {
  /// The compiled file it was decompiled from, as the tool was given it.
  pub source: String,
  pub source_sha256: String,
  pub tool: String,
  pub tool_version: String,
  /// Seconds since the Unix epoch.
  pub decompiled_at: u32,
  /// What the decompiler inferred or defaulted rather than read.
  pub inferred: Vec<ObjectInference>,
}

impl From<&ObjectProvenance> for SelectedObjectProvenance {
  fn from(provenance: &ObjectProvenance) -> Self {
    Self {
      source: provenance.source.clone(),
      source_sha256: provenance.source_sha256.clone(),
      tool: provenance.tool.clone(),
      tool_version: provenance.tool_version.clone(),
      decompiled_at: u32::try_from(provenance.decompiled_at).unwrap_or(u32::MAX),
      inferred: provenance.inferred.clone(),
    }
  }
}
