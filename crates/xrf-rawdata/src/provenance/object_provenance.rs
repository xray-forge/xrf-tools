use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use xrf_error::{XrfError, XrfResult};

use crate::object::compile::object_compile_options::TangentMethod;
use crate::object::decompile::object_inference::ObjectInference;

/// The sidecar written beside a decompiled `.object`, so it is never taken for an authored one: where it came from,
/// what made it and when, and what in it was inferred rather than read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
  /// The tangent method the source's basis was made with, which a compile of the object follows unless told otherwise
  /// (see [`ObjectCompileOptions::following`](crate::ObjectCompileOptions::following)). Sidecars written before it was
  /// recorded read as the Actor Editor's.
  #[serde(default)]
  pub tangents: TangentMethod,
}

impl ObjectProvenance {
  /// What the sidecar's name adds to its object's, `<name>.object.xrf.json`.
  pub const SIDECAR_SUFFIX: &'static str = ".xrf.json";

  /// The provenance of an object decompiled now from `source_bytes`, read from `source`, with what the decompile
  /// inferred and the tangent method it found the source made with.
  pub fn new(
    source: impl Into<String>,
    source_bytes: &[u8],
    tool: impl Into<String>,
    tool_version: impl Into<String>,
    inferred: Vec<ObjectInference>,
    tangents: TangentMethod,
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
      tangents,
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

  /// Where the sidecar of the object at `object` stands: `<object>.xrf.json`, beside it.
  pub fn sidecar_path(object: &Path) -> PathBuf {
    let mut name = object.as_os_str().to_owned();

    name.push(Self::SIDECAR_SUFFIX);

    PathBuf::from(name)
  }

  /// The provenance of the object at `object`, read from its sidecar, or `None` when it has none, as an authored object
  /// has none.
  ///
  /// # Errors
  ///
  /// Returns an error when a sidecar stands there but cannot be read or is not one.
  pub fn read_beside(object: &Path) -> XrfResult<Option<Self>> {
    let sidecar: PathBuf = Self::sidecar_path(object);
    let text: String = match std::fs::read_to_string(&sidecar) {
      Ok(text) => text,
      Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
      Err(error) => {
        return Err(XrfError::new_read_error(format!(
          "{} was not read: {error}",
          sidecar.display()
        )));
      }
    };

    serde_json::from_str(&text).map(Some).map_err(|error| {
      XrfError::new_parsing_error(format!("{} is not a provenance sidecar: {error}", sidecar.display()))
    })
  }
}
