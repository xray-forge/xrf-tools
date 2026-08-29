use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::{is_component_prefix, to_host_relative};
use crate::source::xray_asset_source::label_from_path;
use crate::source::{DirectoryAssetIndex, XrayAssetIndex};
use crate::{XrayAssetContainer, XrayAssetSource, XrayPathCollision, XraySourceKind};

/// A directory of loose files, indexed once at mount time.
///
/// Two files inside it normalizing to one logical path are reported through [`XrayAssetSource::get_collisions`] rather than
/// refused, since only shadowing *between* mounts has a priority order to appeal to.
#[derive(Debug)]
pub struct XrayDirectorySource {
  label: String,
  index: XrayAssetIndex,
}

impl XrayDirectorySource {
  /// Walks and indexes a directory of loose assets.
  ///
  /// # Errors
  ///
  /// Returns an error when traversal fails or a path is not a valid X-Ray logical path.
  pub fn read(root: impl AsRef<Path>) -> XrfResult<Self> {
    Self::read_ignoring(root, &[])
  }

  /// Walks and indexes a directory, omitting logical prefixes.
  ///
  /// Ignoring belongs to one source rather than to a whole VFS: skipping `textures\wip` in an override tree while keeping it
  /// in the installation underneath is a real thing to want, and a single list could not express it.
  ///
  /// # Errors
  ///
  /// Returns an error when traversal fails, or a prefix or path is not a valid X-Ray logical path.
  pub fn read_ignoring(root: impl AsRef<Path>, ignored: &[String]) -> XrfResult<Self> {
    let root: &Path = root.as_ref();

    Ok(Self {
      index: XrayAssetIndex::new(DirectoryAssetIndex::read(root)?, ignored)?,
      label: label_from_path(root),
    })
  }
}

impl XrayAssetSource for XrayDirectorySource {
  fn get_label(&self) -> &str {
    &self.label
  }

  fn get_kind(&self) -> XraySourceKind {
    XraySourceKind::Directory
  }

  fn is_writable(&self) -> bool {
    true
  }

  fn get_root_path(&self) -> &Path {
    self.index.root()
  }

  fn contains(&self, path: &str) -> bool {
    self.index.find(path).ok().flatten().is_some()
  }

  fn locate(&self, path: &str) -> Option<XrayAssetContainer> {
    self
      .index
      .find(path)
      .ok()
      .flatten()
      .map(|asset| XrayAssetContainer::Directory {
        relative_path: asset.relative_path().to_path_buf(),
        root: self.index.root().to_path_buf(),
      })
  }

  fn read(&self, path: &str) -> XrfResult<Vec<u8>> {
    let Some(asset) = self.index.find(path)? else {
      // Absent, not unreadable: the distinction lets a caller fall back rather than fail.
      return Err(XrfError::new_not_found_error(format!(
        "no asset '{path}' under root {}",
        format_path(self.get_root_path())
      )));
    };

    let absolute: PathBuf = asset.absolute_path();

    fs::read(&absolute)
      .map_err(|error| XrfError::new_asset_error(format!("failed to read '{}': {error}", format_path(&absolute))))
  }

  /// Overwrites an indexed entry without creating new files.
  ///
  /// Refusing absent paths prevents the in-memory index from becoming stale.
  fn write(&self, path: &str, bytes: &[u8]) -> XrfResult<()> {
    let Some(absolute) = self.index.find(path).ok().flatten().map(|asset| asset.absolute_path()) else {
      return Err(XrfError::new_asset_error(format!(
        "no asset '{path}' under root {} to write",
        format_path(self.get_root_path())
      )));
    };

    fs::write(&absolute, bytes)
      .map_err(|error| XrfError::new_asset_error(format!("failed to write '{}': {error}", format_path(&absolute))))
  }

  /// Creates a loose file absent from the mount-time index, including any missing parent directories.
  ///
  /// The mount-time index is unchanged.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is already indexed or its directories or file cannot be created.
  fn create(&self, path: &str, bytes: &[u8]) -> XrfResult<()> {
    if self.contains(path) {
      return Err(XrfError::new_asset_error(format!(
        "asset '{path}' already exists under root {}",
        format_path(self.get_root_path())
      )));
    }

    let absolute: PathBuf = self.get_root_path().join(to_host_relative(path));

    if let Some(parent) = absolute.parent() {
      fs::create_dir_all(parent)
        .map_err(|error| XrfError::new_asset_error(format!("failed to create '{}': {error}", format_path(parent))))?;
    }

    fs::write(&absolute, bytes)
      .map_err(|error| XrfError::new_asset_error(format!("failed to create '{}': {error}", format_path(&absolute))))
  }

  fn list_entries<'a>(&'a self, prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a> {
    Box::new(
      self
        .index
        .assets()
        .map(|asset| asset.logical_path().to_string())
        .filter(move |path| prefix.is_none_or(|prefix| is_component_prefix(path, prefix))),
    )
  }

  fn get_size(&self, path: &str) -> Option<u64> {
    self
      .index
      .find(path)
      .ok()
      .flatten()
      .and_then(|asset| fs::metadata(asset.absolute_path()).ok())
      .map(|metadata| metadata.len())
  }

  fn get_collisions(&self) -> &[XrayPathCollision] {
    self.index.collisions()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::XrayAssetSource;
  use crate::source::{XrayDirectorySource, XraySourceKind};

  fn source(name: &str, files: &[&str]) -> XrayDirectorySource {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_directory_source/{name}"));

    let _ = fs::remove_dir_all(&root);

    for file in files {
      let path: PathBuf = root.join(file);

      fs::create_dir_all(path.parent().expect("file sits in a directory")).expect("test tree is creatable");
      fs::write(&path, b"payload").expect("test file is writable");
    }

    XrayDirectorySource::read(&root).expect("root indexes")
  }

  #[test]
  fn reports_itself_as_a_writable_directory() {
    let source: XrayDirectorySource = source("writable", &["textures/wpn/wpn_ak74.dds"]);

    assert_eq!(source.get_kind(), XraySourceKind::Directory);
    assert!(source.is_writable());
  }

  #[test]
  fn contains_and_reads_by_logical_path() {
    let source: XrayDirectorySource = source("reads", &["textures/wpn/wpn_ak74.dds"]);

    assert!(source.contains("textures\\wpn\\wpn_ak74.dds"));
    assert!(!source.contains("textures\\wpn\\wpn_val.dds"));
    assert_eq!(source.read("textures\\wpn\\wpn_ak74.dds").unwrap(), b"payload");
  }

  #[test]
  fn refuses_to_read_or_write_a_path_it_does_not_hold() {
    let source: XrayDirectorySource = source("absent", &["textures/wpn/wpn_ak74.dds"]);

    assert!(source.read("textures\\wpn\\wpn_val.dds").is_err());
    assert!(source.write("textures\\wpn\\wpn_val.dds", b"new").is_err());
  }

  #[test]
  fn writes_over_an_entry_it_holds() {
    let source: XrayDirectorySource = source("writes", &["configs\\system.ltx"]);

    source
      .write("configs\\system.ltx", b"formatted")
      .expect("write succeeds");

    assert_eq!(source.read("configs\\system.ltx").unwrap(), b"formatted");
  }

  #[test]
  fn enumerates_every_entry_and_narrows_by_prefix() {
    let source: XrayDirectorySource = source(
      "enumerates",
      &[
        "textures/wpn/wpn_ak74.dds",
        "configs/system.ltx",
        "configs/weapons/ak74.ltx",
      ],
    );

    let all: Vec<String> = source.list_entries(None).collect();
    let configs: Vec<String> = source.list_entries(Some("configs")).collect();

    assert_eq!(all.len(), 3);
    assert_eq!(configs.len(), 2);
    assert!(configs.iter().all(|path| path.starts_with("configs\\")));
  }

  #[test]
  fn a_prefix_matches_on_component_boundaries_only() {
    // `configs_backup` must not be swept up by a `configs` prefix, or a scoped operation would touch a sibling tree.
    let source: XrayDirectorySource = source("boundaries", &["configs/system.ltx", "configs_backup/system.ltx"]);

    assert_eq!(source.list_entries(Some("configs")).count(), 1);
  }
}
