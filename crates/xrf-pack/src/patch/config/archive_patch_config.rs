use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::pack::config::{ArchivePackConfig, ArchivePackMode, ArchiveVolumeExtension, default_header};
use crate::patch::config::ArchivePatchScope;
use crate::patch::world::ArchivePatchRole;

/// Comparison roots, entry filters, and patch volume settings.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchConfig {
  /// Root of the release to patch. Installations use `fsgame.ltx` mount order; later declarations win.
  pub base: PathBuf,
  /// Root the new build is mounted from.
  pub target: PathBuf,
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
  /// A comparison of two roots, published to `destination` under `name`.
  pub fn new<B: AsRef<Path>, T: AsRef<Path>, D: AsRef<Path>>(base: B, target: T, destination: D, name: &str) -> Self {
    Self {
      base: base.as_ref().into(),
      target: target.as_ref().into(),
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

  /// Builds volume settings for publication. Payloads come from the mounted target, not a source walk.
  pub(crate) fn to_publication(&self) -> ArchivePackConfig {
    ArchivePackConfig {
      header: self.header.clone(),
      is_with_oversized_volumes: self.is_with_oversized_volumes,
      max_volume_size: self.max_volume_size,
      mode: self.mode,
      volume_extension: self.volume_extension,
      ..ArchivePackConfig::new(&self.target, &self.destination, &self.name)
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

  /// Validates comparison and volume settings before mounting.
  ///
  /// # Errors
  ///
  /// Rejects empty root paths, invalid logical prefixes, and invalid packing settings.
  pub(crate) fn validate_for_patching(&self) -> XrfResult<()> {
    for (root, role) in [
      (&self.base, ArchivePatchRole::Base),
      (&self.target, ArchivePatchRole::Target),
    ] {
      if root.as_os_str().is_empty() {
        return Err(XrfError::new_invalid_error(format!(
          "A patch compares two worlds and no {role} root was given"
        )));
      }
    }

    self.to_scope()?;
    self.to_publication().validate_for_packing()
  }
}
