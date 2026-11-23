use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_utils::HostFileName;

use crate::pack::config::{ArchivePackDirectory, ArchiveVolumeSettings};

/// Everything needed to pack one archive volume set.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePackConfig {
  /// Root the archived names are relative to, normally a `gamedata` directory.
  pub source: PathBuf,
  pub destination: PathBuf,
  /// Base name of the volumes, which become `<name>.db0`, `<name>.db1` and so on.
  pub name: String,
  pub include_files: Vec<String>,
  pub include_directories: Vec<ArchivePackDirectory>,
  pub exclude_directories: Vec<ArchivePackDirectory>,
  /// Extension patterns from `[options] exclude_exts`, matched against the extension with its dot.
  pub exclude_extensions: Vec<String>,
  /// Apply the skip rules xrCompress hard-codes for editor and source leftovers.
  pub is_with_skip_list: bool,
  /// How the volumes are written.
  #[serde(flatten)]
  pub volume_settings: ArchiveVolumeSettings,
}

impl ArchivePackConfig {
  pub fn new<S: AsRef<Path>, D: AsRef<Path>>(source: S, destination: D, name: &str) -> Self {
    Self {
      source: source.as_ref().into(),
      destination: destination.as_ref().into(),
      name: name.into(),
      include_files: Vec::new(),
      include_directories: Vec::new(),
      exclude_directories: Vec::new(),
      exclude_extensions: Vec::new(),
      is_with_skip_list: true,
      volume_settings: ArchiveVolumeSettings::default(),
    }
  }

  /// Reject a configuration that cannot produce an engine-mountable archive, or that would publish outside the
  /// destination the caller was shown.
  pub(crate) fn validate_for_packing(&self) -> XrfResult {
    HostFileName::new(&self.name, "Archive name")?;

    self.volume_settings.validate()
  }

  /// Name of one volume of a set, as `<base>.db<index>`.
  pub fn volume_name(&self, index: usize) -> String {
    format!(
      "{}.{}{index}",
      self.name,
      self.volume_settings.volume_extension.as_str()
    )
  }

  /// Name of a set that turned out to hold one volume, as `<base>.db`.
  pub fn single_volume_name(&self) -> String {
    format!("{}.{}", self.name, self.volume_settings.volume_extension.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::{ArchivePackConfig, ArchiveVolumeSettings};
  use crate::pack::config::ArchiveVolumeExtension;

  #[test]
  fn defaults_match_the_engine() {
    let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

    assert!(config.is_with_skip_list, "the vanilla skip list is on by default");
    assert!(config.validate_for_packing().is_ok(), "a fresh configuration packs");
  }

  #[test]
  fn names_volumes_the_way_the_engine_mounts_them() {
    let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "levels");

    assert_eq!(config.volume_name(0), "levels.db0");
    assert_eq!(config.volume_name(12), "levels.db12");

    config.volume_settings.volume_extension = ArchiveVolumeExtension::Xdb;

    assert_eq!(config.volume_name(0), "levels.xdb0");
  }

  #[test]
  fn refuses_a_volume_size_no_set_can_be_written_at() {
    let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

    config.volume_settings.max_volume_size = ArchiveVolumeSettings::VOLUME_SIZE_HARD_MAX + 1;

    assert!(
      config.validate_for_packing().is_err(),
      "the volume settings are part of the gate"
    );
  }

  #[test]
  fn reads_a_configuration_written_before_the_oversized_flag_existed() {
    // The shape is a file on disk as well as a wire contract, so an older one must still load, safe side up.
    let config: ArchivePackConfig = serde_json::from_str(
      r#"{"source":"gamedata","destination":"db","name":"configs","includeFiles":[],"includeDirectories":[],
          "excludeDirectories":[],"excludeExtensions":[],"isWithSkipList":true,"header":null,"mode":"Compress",
          "maxVolumeSize":1024,"volumeExtension":"Db"}"#,
    )
    .expect("a configuration without the field parses");

    assert!(!config.volume_settings.is_with_oversized_volumes);
    assert_eq!(config.volume_settings.max_volume_size, 1024);
    assert_eq!(config.volume_settings.header, None);
  }

  #[test]
  fn writes_the_volume_settings_beside_its_own_fields() {
    // Flattened, so the wire names `maxVolumeSize` where it always did rather than under a nested object.
    let json: serde_json::Value =
      serde_json::to_value(ArchivePackConfig::new("gamedata", "db", "configs")).expect("a configuration serializes");

    assert_eq!(json["maxVolumeSize"], ArchiveVolumeSettings::VOLUME_SIZE_MAX);
    assert_eq!(json["mode"], "Compress");
    assert!(json.get("volumeSettings").is_none(), "{json}");
  }
}
