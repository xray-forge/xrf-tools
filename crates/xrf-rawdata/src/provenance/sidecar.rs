use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use xrf_error::{XrfError, XrfResult};

/// The convention every provenance sidecar follows (decision 22): JSON beside the file it describes, named after it
/// with `.xrf.json` added, and sources named by their SHA-256.
pub(crate) struct Sidecar;

impl Sidecar {
  /// What a sidecar's name adds to its file's: `<name>.object.xrf.json`, `<name>.level.xrf.json`.
  pub(crate) const SUFFIX: &'static str = ".xrf.json";

  /// Where the sidecar of the file at `path` stands: `<path>.xrf.json`, beside it.
  pub(crate) fn get_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();

    name.push(Self::SUFFIX);

    PathBuf::from(name)
  }

  /// The SHA-256 of `bytes`, lowercase hex, as a sidecar names a source by.
  pub(crate) fn get_sha256_hex(bytes: &[u8]) -> String {
    Self::format_hex(&Sha256::digest(bytes))
  }

  /// Bytes as a sidecar keeps them, lowercase hex.
  pub(crate) fn format_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
  }

  /// A sidecar's text.
  ///
  /// # Errors
  ///
  /// Returns an error when the value does not serialise, which a provenance always does.
  pub(crate) fn format_json<T: Serialize>(value: &T) -> XrfResult<String> {
    serde_json::to_string_pretty(value)
      .map_err(|error| XrfError::new_serialization_error(format!("Provenance did not serialise: {error}")))
  }
}
