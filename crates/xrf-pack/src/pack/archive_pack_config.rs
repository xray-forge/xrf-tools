use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::Ltx;

/// Largest volume the engine will open, and the default (`XRP_MAX_SIZE` in `xrCompress.h`).
pub const VOLUME_SIZE_MAX: u64 = 1900 * xrf_utils::BYTES_PER_MEGABYTE;

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
/// included directory recurses into subdirectories, while an excluded one matches by prefix rather than exactly.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
  pub max_volume_size: u64,
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
      volume_extension: ArchiveVolumeExtension::default(),
    }
  }

  /// Apply an xrCompress configuration file.
  ///
  /// Reads the dialect `ProcessLTX` accepts: `[options] exclude_exts`, `[include_files]` as bare names,
  /// `[include_folders]` and `[exclude_folders]` as `path = <bool>`, and a `[header]` copied verbatim.
  pub fn with_ltx_file<P: AsRef<Path>>(self, path: P) -> XrfResult<Self> {
    self.with_ltx(&Ltx::read_from_file_full(path)?)
  }

  pub fn with_ltx(mut self, ltx: &Ltx) -> XrfResult<Self> {
    if let Some(section) = ltx.section("options")
      && let Some(extensions) = section.get("exclude_exts")
    {
      self.exclude_extensions = extensions
        .split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(String::from)
        .collect();
    }

    if let Some(section) = ltx.section("include_files") {
      // Files are listed as bare names, so only the key carries meaning.
      self.include_files = section.iter().map(|(name, _)| String::from(name)).collect();
    }

    if let Some(section) = ltx.section("include_folders") {
      self.include_directories = section.iter().map(Self::directory_from_entry).collect();
    }

    if let Some(section) = ltx.section("exclude_folders") {
      self.exclude_directories = section.iter().map(Self::directory_from_entry).collect();
    }

    if let Some(section) = ltx.section("header") {
      let mut header: String = String::from("[header]\r\n");

      for (key, value) in section.iter() {
        header.push_str(key);
        header.push_str(" = ");
        header.push_str(value);
        header.push_str("\r\n");
      }

      self.header = Some(header);
    }

    Ok(self)
  }

  /// Clamp an explicit volume size the way `SetMaxVolumeSize` does, refusing zero outright.
  pub fn with_max_volume_size(mut self, size: u64) -> XrfResult<Self> {
    if size == 0 {
      return Err(XrfError::new_invalid_error(
        "Archive volume size must be greater than zero".to_string(),
      ));
    }

    self.max_volume_size = size.min(VOLUME_SIZE_MAX);

    Ok(self)
  }

  /// Reject volume sizes that cannot produce an engine-mountable archive.
  pub(crate) fn validate_for_packing(&self) -> XrfResult {
    if self.max_volume_size == 0 {
      return Err(XrfError::new_invalid_error(
        "Archive volume size must be greater than zero".to_string(),
      ));
    }

    if self.max_volume_size > VOLUME_SIZE_MAX {
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

  fn directory_from_entry((path, value): (&str, &str)) -> ArchivePackDirectory {
    ArchivePackDirectory {
      // `.\` names the source root itself, which is more readable as an empty prefix.
      path: if path == ".\\" || path == "./" {
        String::new()
      } else {
        path.replace('/', "\\").trim_end_matches('\\').to_string()
      },
      is_recursive: matches!(value, "true" | "on" | "yes" | "1"),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_ltx::Ltx;

  use super::{ArchivePackConfig, ArchivePackMode, ArchiveVolumeExtension, VOLUME_SIZE_MAX};

  fn config_from_ltx(source: &str) -> ArchivePackConfig {
    ArchivePackConfig::new("gamedata", "db", "configs")
      .with_ltx(&Ltx::read_from_str(source).expect("ltx parses"))
      .expect("ltx applies")
  }

  #[test]
  fn defaults_match_the_engine() {
    let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs");

    assert_eq!(config.mode, ArchivePackMode::Compress);
    assert_eq!(config.max_volume_size, VOLUME_SIZE_MAX);
    assert_eq!(config.volume_extension, ArchiveVolumeExtension::Db);
    assert!(config.is_with_skip_list, "the vanilla skip list is on by default");
  }

  #[test]
  fn reads_the_xrcompress_dialect() {
    let config: ArchivePackConfig = config_from_ltx(
      "[options]\nexclude_exts = *.txt, *.json\n\n\
       [include_files]\ngamemtl.xr\nshaders.xr\n\n\
       [include_folders]\nconfigs = true\nspawns = false\n\n\
       [exclude_folders]\nlevels\\build = true\n",
    );

    assert_eq!(config.exclude_extensions, vec!["*.txt", "*.json"]);
    assert_eq!(config.include_files, vec!["gamemtl.xr", "shaders.xr"]);
    assert_eq!(config.include_directories.len(), 2);
    assert_eq!(config.include_directories[0].path, "configs");
    assert!(config.include_directories[0].is_recursive);
    assert!(!config.include_directories[1].is_recursive);
    assert_eq!(config.exclude_directories[0].path, "levels\\build");
  }

  #[test]
  fn reads_the_source_root_as_an_empty_prefix() {
    let config: ArchivePackConfig = config_from_ltx("[include_folders]\n.\\ = false\n");

    assert_eq!(config.include_directories[0].path, "");
  }

  #[test]
  fn keeps_the_header_verbatim_for_the_engine_to_parse() {
    let config: ArchivePackConfig =
      config_from_ltx("[header]\nauto_load = true\nentry_point = $fs_root$\\gamedata\\\n");
    let header: &str = config.header.as_deref().expect("header is carried");

    assert!(header.starts_with("[header]\r\n"), "the section names itself");
    assert!(header.contains("auto_load = true\r\n"));
    assert!(header.contains("entry_point = $fs_root$\\gamedata\\\r\n"));
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
  fn clamps_an_oversized_volume_and_refuses_an_empty_one() {
    let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "configs")
      .with_max_volume_size(VOLUME_SIZE_MAX * 4)
      .expect("oversized size clamps");

    assert_eq!(config.max_volume_size, VOLUME_SIZE_MAX);
    assert!(
      ArchivePackConfig::new("gamedata", "db", "configs")
        .with_max_volume_size(0)
        .is_err()
    );
  }
}
