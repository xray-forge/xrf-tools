use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::HostFileName;

use crate::pack::config::{ArchivePackDirectory, ArchivePackHeader, ArchivePackMode, ArchiveVolumeExtension};

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
  /// Verbatim `[header]` text written as chunk 666.
  pub header: Option<String>,
  pub mode: ArchivePackMode,
  /// Hard maximum for a produced volume file, counting every byte it holds: chunk headers, header text, payloads as
  /// they are actually stored, and the descriptor table written last. Packing refuses a cap it cannot keep rather
  /// than exceeding it, which is stricter than the target xrCompress tests before each file and routinely overshoots.
  pub max_volume_size: u64,
  /// Let `max_volume_size` exceed [`Self::VOLUME_SIZE_MAX`], for an engine fork that raised `XRP_MAX_SIZE`.
  #[serde(default)]
  pub is_with_oversized_volumes: bool,
  pub volume_extension: ArchiveVolumeExtension,
}

impl ArchivePackConfig {
  /// Largest volume the engine will open, and the default (`XRP_MAX_SIZE` in `xrCompress.h`).
  pub const VOLUME_SIZE_MAX: u64 = 1900 * xrf_utils::BYTES_PER_MEGABYTE;

  /// Smallest volume size a caller may ask for.
  pub const VOLUME_SIZE_MIN: u64 = xrf_utils::BYTES_PER_MEGABYTE;

  /// Ceiling no configuration may pass, whatever it allows.
  pub const VOLUME_SIZE_HARD_MAX: u64 = 32 * xrf_utils::BYTES_PER_GIGABYTE;

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
      header: Some(ArchivePackHeader::default_text()),
      mode: ArchivePackMode::default(),
      max_volume_size: Self::VOLUME_SIZE_MAX,
      is_with_oversized_volumes: false,
      volume_extension: ArchiveVolumeExtension::default(),
    }
  }

  /// Take an explicit volume size, refusing one outside the range the engine can mount.
  ///
  /// # Errors
  ///
  /// Returns an invalid error below [`Self::VOLUME_SIZE_MIN`], or above [`Self::VOLUME_SIZE_MAX`] unless oversized
  /// volumes are allowed.
  pub fn with_max_volume_size(mut self, size: u64) -> XrfResult<Self> {
    Self::check_volume_size(size, self.is_with_oversized_volumes)?;

    self.max_volume_size = size;

    Ok(self)
  }

  /// Refuses a volume size a person asks for outside the range the engine can mount: under
  /// [`Self::VOLUME_SIZE_MIN`], which is more likely a unit mistaken than a set meant to hold a few kilobytes a volume,
  /// or past what [`Self::check_volume_bounds`] refuses.
  pub(crate) fn check_volume_size(size: u64, is_oversized_allowed: bool) -> XrfResult {
    if size < Self::VOLUME_SIZE_MIN {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must be at least {} bytes, got {size}",
        Self::VOLUME_SIZE_MIN
      )));
    }

    Self::check_volume_bounds(size, is_oversized_allowed)
  }

  /// Refuses a volume size no set can be packed at: none, past [`Self::VOLUME_SIZE_MAX`] unless oversized volumes are
  /// allowed, and past [`Self::VOLUME_SIZE_HARD_MAX`] ever.
  fn check_volume_bounds(size: u64, is_oversized_allowed: bool) -> XrfResult {
    if size == 0 {
      return Err(XrfError::new_invalid_error(
        "Archive volume size must be greater than zero",
      ));
    }

    if size > Self::VOLUME_SIZE_HARD_MAX {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must not exceed {} bytes, got {size}. Nothing lifts this bound; check the number for a \
         stray digit.",
        Self::VOLUME_SIZE_HARD_MAX
      )));
    }

    if size > Self::VOLUME_SIZE_MAX && !is_oversized_allowed {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must not exceed {} bytes, got {size}. No unmodified engine mounts a larger volume; allow \
         oversized volumes to pack one anyway.",
        Self::VOLUME_SIZE_MAX
      )));
    }

    Ok(())
  }

  /// Allow a volume size past [`Self::VOLUME_SIZE_MAX`], which only an engine fork that raised `XRP_MAX_SIZE` can
  /// open.
  ///
  /// [`Self::VOLUME_SIZE_HARD_MAX`] still stands: this lifts an engine limit, not the guard against a mistyped number.
  pub fn with_oversized_volumes(mut self, is_allowed: bool) -> Self {
    self.is_with_oversized_volumes = is_allowed;

    self
  }

  /// Reject a configuration that cannot produce an engine-mountable archive, or that would publish outside the
  /// destination the caller was shown.
  pub(crate) fn validate_for_packing(&self) -> XrfResult {
    HostFileName::new(&self.name, "Archive name")?;

    // A header is optional; one that exists and cannot be loaded is not. Judged here rather than at each surface, so
    // a configuration file reaches the same answer a flag does.
    if let Some(header) = self.header.as_deref() {
      ArchivePackHeader::require_loadable(header)?;
    }

    // The minimum is a guard on what a person types, kept by the setters; a configuration packs at any size the
    // engine can mount.
    Self::check_volume_bounds(self.max_volume_size, self.is_with_oversized_volumes)
  }

  /// Name of one volume of a set, as `<base>.db<index>`.
  pub fn volume_name(&self, index: usize) -> String {
    format!("{}.{}{index}", self.name, self.volume_extension.as_str())
  }

  /// Name of a set that turned out to hold one volume, as `<base>.db`.
  pub fn single_volume_name(&self) -> String {
    format!("{}.{}", self.name, self.volume_extension.as_str())
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::{ArchivePackConfig, ArchivePackMode, ArchiveVolumeExtension};

  #[test]
  fn defaults_match_the_engine() {
    let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

    assert_eq!(config.mode, ArchivePackMode::Compress);
    assert_eq!(config.max_volume_size, ArchivePackConfig::VOLUME_SIZE_MAX);
    assert_eq!(config.volume_extension, ArchiveVolumeExtension::Db);
    assert!(config.is_with_skip_list, "the vanilla skip list is on by default");
  }

  #[test]
  fn names_volumes_the_way_the_engine_mounts_them() {
    let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "levels");

    assert_eq!(config.volume_name(0), "levels.db0");
    assert_eq!(config.volume_name(12), "levels.db12");

    config.volume_extension = ArchiveVolumeExtension::Xdb;

    assert_eq!(config.volume_name(0), "levels.xdb0");
  }

  #[test]
  fn packs_an_oversized_volume_only_when_that_is_asked_for_explicitly() {
    let oversized: u64 = ArchivePackConfig::VOLUME_SIZE_MAX + 1;

    // The escape hatch is applied first, and the size it lifts the bound for is then taken as given.
    let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs")
      .with_oversized_volumes(true)
      .with_max_volume_size(oversized)
      .expect("an allowed oversized size is taken as given");

    assert_eq!(config.max_volume_size, oversized);
    assert!(config.validate_for_packing().is_ok(), "the gate honours the same flag");

    // The other order refuses, and says what would have allowed it.
    let error: XrfError = ArchivePackConfig::new("gamedata", "db", "configs")
      .with_max_volume_size(oversized)
      .expect_err("the bound still stands until it is lifted");

    assert!(error.to_string().contains("oversized volumes"), "{error}");

    // Lifting the bound is not lifting the floor: only the maximum is a fork's business.
    assert!(
      ArchivePackConfig::new("gamedata", "db", "configs")
        .with_oversized_volumes(true)
        .with_max_volume_size(ArchivePackConfig::VOLUME_SIZE_MIN - 1)
        .is_err()
    );
  }

  #[test]
  fn keeps_the_typo_guard_above_everything_the_fork_flag_lifts() {
    let mistyped: u64 = ArchivePackConfig::VOLUME_SIZE_HARD_MAX + 1;
    let error: XrfError = ArchivePackConfig::new("gamedata", "db", "configs")
      .with_oversized_volumes(true)
      .with_max_volume_size(mistyped)
      .expect_err("the hard bound is not the one the flag lifts");

    assert!(error.to_string().contains("stray digit"), "{error}");

    // Nor can the field be set past it directly: the gate before any write says the same thing.
    let mut config: ArchivePackConfig =
      ArchivePackConfig::new("gamedata", "db", "configs").with_oversized_volumes(true);

    config.max_volume_size = mistyped;

    assert!(config.validate_for_packing().is_err());

    // The bound itself is allowed, so the refusal is a ceiling rather than an off-by-one.
    config.max_volume_size = ArchivePackConfig::VOLUME_SIZE_HARD_MAX;

    assert!(config.validate_for_packing().is_ok());
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

    assert!(!config.is_with_oversized_volumes);
  }

  #[test]
  fn refuses_a_volume_size_outside_the_range_the_engine_mounts() {
    let (min, max): (u64, u64) = (ArchivePackConfig::VOLUME_SIZE_MIN, ArchivePackConfig::VOLUME_SIZE_MAX);

    // Neither bound is clamped: an accepted size is the size that was asked for.
    for size in [min, max, min + 1] {
      let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs")
        .with_max_volume_size(size)
        .expect("a size within range is taken as given");

      assert_eq!(config.max_volume_size, size);
    }

    for size in [0, 1, min - 1, max + 1, max * 4] {
      assert!(
        ArchivePackConfig::new("gamedata", "db", "configs")
          .with_max_volume_size(size)
          .is_err(),
        "{size} bytes is refused rather than clamped"
      );
    }
  }
}
