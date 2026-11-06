use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sha2::{Digest, Sha256};
use xrf_error::{XrfError, XrfResult};

use crate::object::decompile::object_inference::ObjectInference;

/// The sidecar written beside a decompiled `.object`, so it is never taken for an authored one: where it came from,
/// what made it and when, and what in it was inferred rather than read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectProvenance {
  /// The compiled file it was decompiled from, as the tool was given it.
  pub source: String,
  /// The source's SHA-256, lowercase hex, which says whether a later source is the same file.
  pub source_sha256: String,
  pub tool: String,
  pub tool_version: String,
  /// Seconds since the Unix epoch.
  pub decompiled_at: u64,
  pub inferred: Vec<ObjectInference>,
}

impl ObjectProvenance {
  /// What the sidecar's name adds to its object's, `<name>.object.xrf.json`.
  pub const SIDECAR_SUFFIX: &'static str = ".xrf.json";

  /// The provenance of an object decompiled now from `source_bytes`, read from `source`.
  pub fn new(
    source: impl Into<String>,
    source_bytes: &[u8],
    tool: impl Into<String>,
    tool_version: impl Into<String>,
    inferred: Vec<ObjectInference>,
  ) -> Self {
    Self {
      source: source.into(),
      source_sha256: Sha256::digest(source_bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect(),
      tool: tool.into(),
      tool_version: tool_version.into(),
      decompiled_at: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs()),
      inferred,
    }
  }

  /// The sidecar's text.
  ///
  /// # Errors
  ///
  /// Returns an error when the provenance does not serialise, which it always does.
  pub fn to_json(&self) -> XrfResult<String> {
    serde_json::to_string_pretty(self)
      .map_err(|error| XrfError::new_serialization_error(format!("Provenance did not serialise: {error}")))
  }
}
