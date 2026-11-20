use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use xrf_error::{XrfError, XrfResult};

/// The convention every provenance sidecar follows: JSON beside the file it describes, named after it
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

  /// The file the sidecar at `path` describes, its name without the suffix in any case; none where `path` names no
  /// sidecar.
  pub(crate) fn get_described(path: &Path) -> Option<PathBuf> {
    let name: &str = path.file_name()?.to_str()?;
    let stem: usize = name.len().checked_sub(Self::SUFFIX.len())?;

    (stem > 0 && name.get(stem..)?.eq_ignore_ascii_case(Self::SUFFIX)).then(|| path.with_file_name(&name[..stem]))
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
