use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::path::validate_host_file_name;

/// Largest volume the engine will open, and the default (`XRP_MAX_SIZE` in `xrCompress.h`).
pub const VOLUME_SIZE_MAX: u64 = 1900 * xrf_utils::BYTES_PER_MEGABYTE;

/// Smallest volume size a caller may ask for.
///
/// A floor on what a person requests, not on what the format allows: every volume repeats the header text and the
/// directory rows of what it holds, so a cap of a few kilobytes spends most of a set restating itself, and the engine
/// mounts each volume it produces. `ArchiveVolumeLayout` still refuses a cap the actual header and table cannot open
/// within, which is the only limit derived from content, and a test setting `max_volume_size` directly stays free to
/// exercise a split at any size.
pub const VOLUME_SIZE_MIN: u64 = xrf_utils::BYTES_PER_MEGABYTE;

/// Ceiling no configuration may pass, whatever it allows.
///
/// Nothing about the archive format stops here — this is a typo guard. A volume size is entered in megabytes, so one
/// stray digit asks for a volume larger than the disk, and a run that only discovers it after walking the source has
/// already cost what it took to get there. Far enough above `VOLUME_SIZE_MAX` that no fork raising `XRP_MAX_SIZE` has
/// any reason to reach it, and low enough that a mistyped number lands past it.
pub const VOLUME_SIZE_HARD_MAX: u64 = 32 * xrf_utils::BYTES_PER_GIGABYTE;

/// Where a packed `gamedata` tree mounts, which is what nearly every archive is.
pub const DEFAULT_ENTRY_POINT: &str = "$fs_root$\\gamedata\\";

/// Header written unless a configuration supplies its own.
///
/// An archive without a header is not merely unmounted: unless it is named `xdb`, the loader assumes it
/// is an encrypted Shadow of Chernobyl archive and decrypts it into nonsense
/// (`xray-16/src/xrCore/LocatorAPI.cpp`). Defaulting to a mountable header makes the harmless case the
/// easy one, and a configuration that names a different entry point still replaces it.
pub fn default_header() -> String {
  format!("[header]\r\nauto_load = true\r\nentry_point = {DEFAULT_ENTRY_POINT}\r\n")
}

/// How file payloads are stored in the archive.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchivePackMode {
  /// Compress what the engine expects to be compressed and store the rest.
  #[default]
  Compress,
  /// Store everything, the `-store` flag of xrCompress.
  Store,
}

/// Extension the produced volumes carry, which also decides how the engine treats a missing header.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveVolumeExtension {
  #[default]
  Db,
  Xdb,
}

impl ArchiveVolumeExtension {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Db => "db",
      Self::Xdb => "xdb",
    }
  }
}

/// One `[include_folders]` or `[exclude_folders]` entry.
///
/// The section names keep the engine's spelling because they are the xrCompress dialect; everything this crate names
/// itself says `directory`.
///
/// The boolean has a different meaning on each side, which is an xrCompress quirk worth stating: an
/// included directory recurses into subdirectories, while an excluded one covers everything below itself rather than
/// only the directory it names. Either way the path is matched on complete components, without case.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePackDirectory {
  pub path: String,
  pub is_recursive: bool,
}

/// Everything needed to pack one archive volume set.
///
/// Built from defaults, then optionally from an xrCompress LTX, then from explicit parameters, so a
/// command line and a form can layer over the same config file in the same order.
///
/// Also the wire contract the desktop editor holds: it is read from a configuration file, edited in
/// place, packed, and written back, so all three surfaces speak one shape.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePackConfig {
  /// Root the archived names are relative to, normally a `gamedata` directory.
  pub source: PathBuf,
  pub destination: PathBuf,
  /// Base name of the volumes, which become `<name>.db0`, `<name>.db1` and so on.
  ///
  /// One host file name, never a path: it is joined to `destination`, and packing refuses anything that would
  /// resolve elsewhere.
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
  /// Let `max_volume_size` exceed `VOLUME_SIZE_MAX`, for an engine fork that raised `XRP_MAX_SIZE`.
  ///
  /// Defaulted rather than required, because this shape is also a configuration file on disk: one written before the
  /// field existed reads back as the safe answer instead of failing to parse.
  #[serde(default)]
  pub is_with_oversized_volumes: bool,
  pub volume_extension: ArchiveVolumeExtension,
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
      header: Some(default_header()),
      mode: ArchivePackMode::default(),
      max_volume_size: VOLUME_SIZE_MAX,
      is_with_oversized_volumes: false,
      volume_extension: ArchiveVolumeExtension::default(),
    }
  }

  /// Take an explicit volume size, refusing one outside the range the engine can mount.
  ///
  /// `SetMaxVolumeSize` warns and clamps to `XRP_MAX_SIZE` instead. Clamping is wrong for a request typed by a
  /// person: they asked for volumes of one size, and a set silently split at another is not what they would have
  /// asked for had they known. The number came from a command line or a form, so the answer belongs where it was
  /// entered.
  ///
  /// The upper bound is the one [`Self::with_oversized_volumes`] lifts, so a caller wanting both applies that first.
  /// In the other order this refuses, naming it — `validate_for_packing` is the gate that cannot be ordered wrong.
  ///
  /// # Errors
  ///
  /// Returns an invalid error below `VOLUME_SIZE_MIN`, or above `VOLUME_SIZE_MAX` unless oversized volumes are
  /// allowed.
  pub fn with_max_volume_size(mut self, size: u64) -> XrfResult<Self> {
    if size < VOLUME_SIZE_MIN {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must be at least {VOLUME_SIZE_MIN} bytes, got {size}"
      )));
    }

    if size > VOLUME_SIZE_HARD_MAX {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must not exceed {VOLUME_SIZE_HARD_MAX} bytes, got {size}. Nothing lifts this bound; \
         check the number for a stray digit."
      )));
    }

    if size > VOLUME_SIZE_MAX && !self.is_with_oversized_volumes {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must not exceed {VOLUME_SIZE_MAX} bytes, got {size}. No unmodified engine mounts \
         a larger volume; allow oversized volumes to pack one anyway."
      )));
    }

    self.max_volume_size = size;

    Ok(self)
  }

  /// Allow a volume size past `VOLUME_SIZE_MAX`, which only an engine fork that raised `XRP_MAX_SIZE` can open.
  ///
  /// The archive stays well-formed — nothing about the format stops at 1900 MB. What stops there is `xrCompress` and
  /// the loader, so a set packed this way is not a S.T.A.L.K.E.R. archive any shipped build will mount, and the
  /// failure surfaces at load time in the engine rather than here. Off by default, and no configuration file turns it
  /// on by accident: `with_ltx` does not read it, because the xrCompress dialect has no such key.
  ///
  /// `VOLUME_SIZE_HARD_MAX` still stands: this lifts an engine limit, not the guard against a mistyped number.
  pub fn with_oversized_volumes(mut self, is_allowed: bool) -> Self {
    self.is_with_oversized_volumes = is_allowed;

    self
  }

  /// Reject a configuration that cannot produce an engine-mountable archive, or that would publish outside the
  /// destination the caller was shown.
  ///
  /// Runs before any filesystem operation, because both publication paths join the name to `destination`: the volume
  /// each `File::create` opens, and the rename a lone volume ends under.
  pub(crate) fn validate_for_packing(&self) -> XrfResult {
    validate_host_file_name(&self.name, "Archive name")?;

    if self.max_volume_size == 0 {
      return Err(XrfError::new_invalid_error(
        "Archive volume size must be greater than zero".to_string(),
      ));
    }

    if self.max_volume_size > VOLUME_SIZE_HARD_MAX {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must not exceed {VOLUME_SIZE_HARD_MAX} bytes"
      )));
    }

    if self.max_volume_size > VOLUME_SIZE_MAX && !self.is_with_oversized_volumes {
      return Err(XrfError::new_invalid_error(format!(
        "Archive volume size must not exceed {VOLUME_SIZE_MAX} bytes"
      )));
    }

    Ok(())
  }

  /// Name of one volume of a set, as `<base>.db<index>`.
  pub fn volume_name(&self, index: usize) -> String {
    format!("{}.{}{index}", self.name, self.volume_extension.as_str())
  }

  /// Name of a set that turned out to hold one volume, as `<base>.db`.
  ///
  /// The engine mounts by scanning for any extension starting with `db` or `xdb`, so the index is never
  /// required. Dropping it for a lone volume is what the shipped games do, and it reads better than a
  /// `0` that never has a sibling.
  pub fn single_volume_name(&self) -> String {
    format!("{}.{}", self.name, self.volume_extension.as_str())
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::{
    ArchivePackConfig, ArchivePackMode, ArchiveVolumeExtension, VOLUME_SIZE_HARD_MAX, VOLUME_SIZE_MAX, VOLUME_SIZE_MIN,
  };

  #[test]
  fn defaults_match_the_engine() {
    let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

    assert_eq!(config.mode, ArchivePackMode::Compress);
    assert_eq!(config.max_volume_size, VOLUME_SIZE_MAX);
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
    let oversized: u64 = VOLUME_SIZE_MAX + 1;

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
        .with_max_volume_size(VOLUME_SIZE_MIN - 1)
        .is_err()
    );
  }

  #[test]
  fn keeps_the_typo_guard_above_everything_the_fork_flag_lifts() {
    let mistyped: u64 = VOLUME_SIZE_HARD_MAX + 1;
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
    config.max_volume_size = VOLUME_SIZE_HARD_MAX;

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
    // Neither bound is clamped: an accepted size is the size that was asked for.
    for size in [VOLUME_SIZE_MIN, VOLUME_SIZE_MAX, VOLUME_SIZE_MIN + 1] {
      let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs")
        .with_max_volume_size(size)
        .expect("a size within range is taken as given");

      assert_eq!(config.max_volume_size, size);
    }

    for size in [0, 1, VOLUME_SIZE_MIN - 1, VOLUME_SIZE_MAX + 1, VOLUME_SIZE_MAX * 4] {
      assert!(
        ArchivePackConfig::new("gamedata", "db", "configs")
          .with_max_volume_size(size)
          .is_err(),
        "{size} bytes is refused rather than clamped"
      );
    }
  }
}
