use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::pack::config::{ArchivePackConfig, ArchivePackMode, ArchiveVolumeExtension, default_header};
use crate::patch::config::ArchivePatchScope;

/// Everything one patch run compares, and what it publishes the difference as.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchConfig {
  /// Roots the release being patched is mounted from, in the order the engine would mount them.
  pub base: Vec<PathBuf>,
  /// Roots the new build is mounted from, in the same order sense as [`Self::base`].
  pub target: Vec<PathBuf>,
  pub destination: PathBuf,
  /// Base name of the volumes, which become `<name>.db0`, `<name>.db1` and so on.
  pub name: String,
  /// Logical prefixes the comparison is restricted to, or the whole of both worlds when empty.
  pub include: Vec<String>,
  /// Logical prefixes dropped from the comparison, applied after [`Self::include`].
  pub ignore: Vec<String>,
  /// Extension patterns that keep a file out of the comparison, such as `*.txt`, matched with the dot.
  pub exclude_extensions: Vec<String>,
  /// Verbatim `[header]` text written as chunk 666, defaulting to the mountable gamedata header.
  pub header: Option<String>,
  pub mode: ArchivePackMode,
  pub max_volume_size: u64,
  pub is_with_oversized_volumes: bool,
  pub volume_extension: ArchiveVolumeExtension,
}

impl ArchivePatchConfig {
  /// A comparison of two root sets, published to `destination` under `name`.
  pub fn new<D: AsRef<Path>>(base: Vec<PathBuf>, target: Vec<PathBuf>, destination: D, name: &str) -> Self {
    Self {
      base,
      target,
      destination: destination.as_ref().into(),
      name: name.into(),
      include: Vec::new(),
      ignore: Vec::new(),
      exclude_extensions: Vec::new(),
      header: Some(default_header()),
      mode: ArchivePackMode::default(),
      max_volume_size: crate::VOLUME_SIZE_MAX,
      is_with_oversized_volumes: false,
      volume_extension: ArchiveVolumeExtension::default(),
    }
  }

  /// The publication half, as the configuration the volume writer already speaks.
  ///
  /// `source` is the first target root. Nothing in the write path reads it — the walk is the only consumer of a pack
  /// configuration's source, and a patch has no walk — but the field has to hold something, and the tree the payloads
  /// actually come from is the one honest answer available.
  pub(crate) fn to_publication(&self) -> ArchivePackConfig {
    ArchivePackConfig {
      header: self.header.clone(),
      is_with_oversized_volumes: self.is_with_oversized_volumes,
      max_volume_size: self.max_volume_size,
      mode: self.mode,
      volume_extension: self.volume_extension,
      ..ArchivePackConfig::new(
        self.target.first().cloned().unwrap_or_default(),
        &self.destination,
        &self.name,
      )
    }
  }

  /// What the comparison is allowed to look at.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a prefix that is not a name the engine can address.
  pub(crate) fn to_scope(&self) -> XrfResult<ArchivePatchScope> {
    ArchivePatchScope::new(&self.include, &self.ignore, &self.exclude_extensions)
  }

  /// Reject a configuration that cannot produce a comparison, before anything is mounted.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for an empty root set on either side, for a scope that is not addressable, and for
  /// anything the publication half already refuses.
  pub(crate) fn validate_for_patching(&self) -> XrfResult<()> {
    for (roots, side) in [(&self.base, "base"), (&self.target, "target")] {
      if roots.is_empty() {
        return Err(XrfError::new_invalid_error(format!(
          "A patch compares two worlds and no {side} root was given"
        )));
      }
    }

    self.to_scope()?;
    self.to_publication().validate_for_packing()
  }
}
