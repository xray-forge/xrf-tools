use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::{is_component_prefix, to_host_relative};
use crate::source::xray_asset_source::label_from_path;
use crate::source::{DirectoryAssetIndex, XrayAssetIndex, XrayFileStamp};
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

  /// The file indexed for a path, in the case it has on disk, or where a new one goes below the root.
  fn get_writable_path(&self, path: &str) -> Option<PathBuf> {
    match self.index.find(path).ok()? {
      Some(asset) => Some(asset.absolute_path()),
      None => Some(self.get_root_path().join(to_host_relative(path))),
    }
  }

  fn with_changes(&self, created: &[&str], deleted: &[&str]) -> XrfResult<Box<dyn XrayAssetSource>> {
    Ok(Box::new(Self {
      index: self.index.with_changes(created, deleted)?,
      label: self.label.clone(),
    }))
  }

  fn list_entries<'a>(&'a self, prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a> {
    let Some(prefix) = prefix else {
      return Box::new(self.index.assets().map(|asset| asset.logical_path().to_string()));
    };

    Box::new(
      self
        .index
        .assets_starting_with(prefix)
        .map(|asset| asset.logical_path().to_string())
        .filter(move |path| is_component_prefix(path, prefix)),
    )
  }

  /// Answers from the index, which already dropped the ignored prefixes and the files that fold onto one identity.
  fn count_entries(&self) -> usize {
    self.index.len()
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

  fn get_stamp(&self, path: &str) -> Option<XrayFileStamp> {
    let asset = self.index.find(path).ok()??;

    fs::metadata(asset.absolute_path())
      .ok()
      .map(|metadata| XrayFileStamp::of(&metadata))
  }

  fn get_collisions(&self) -> &[XrayPathCollision] {
    self.index.collisions()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_test_utils::GeneratedResources;

  use crate::XrayAssetSource;
  use crate::source::{XrayDirectorySource, XraySourceKind};

  fn source(name: &str, files: &[&str]) -> XrayDirectorySource {
    let root: PathBuf =
      GeneratedResources::create_dir(&format!("xray_directory_source/{name}")).expect("test tree is creatable");

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
  fn refuses_to_read_a_path_it_does_not_hold() {
    let source: XrayDirectorySource = source("absent", &["textures/wpn/wpn_ak74.dds"]);

    assert!(source.read("textures\\wpn\\wpn_val.dds").is_err());
  }

  #[test]
  fn writes_an_indexed_file_where_it_is_and_a_new_one_below_the_root() {
    let source: XrayDirectorySource = source("writable_paths", &["Configs/System.ltx"]);

    assert!(
      source
        .get_writable_path("configs\\system.ltx")
        .is_some_and(|path| path.ends_with("Configs/System.ltx")),
      "the file keeps the case it has on disk"
    );
    assert_eq!(
      source.get_writable_path("configs\\game.ltx"),
      Some(source.get_root_path().join("configs").join("game.ltx"))
    );
  }

  #[test]
  fn takes_created_and_deleted_files_without_walking_again() {
    let source: XrayDirectorySource = source("changes", &["configs/system.ltx", "configs/game.ltx"]);

    fs::write(source.get_root_path().join("configs/weather.ltx"), b"weather").expect("a file is created");

    let changed = source
      .with_changes(&["configs\\weather.ltx"], &["configs\\game.ltx"])
      .expect("the source takes the changes");
    let entries: Vec<String> = changed.list_entries(None).collect();

    assert_eq!(entries, ["configs\\system.ltx", "configs\\weather.ltx"]);
    assert_eq!(changed.read("configs\\weather.ltx").unwrap(), b"weather");
    assert_eq!(source.count_entries(), 2, "the source changed from stays as it was");
  }

  #[test]
  fn leaves_a_created_file_under_an_ignored_prefix_out() {
    let root: PathBuf =
      GeneratedResources::create_dir("xray_directory_source/ignored_changes").expect("the tree is creatable");

    fs::create_dir_all(root.join("configs")).expect("the tree is creatable");

    let source: XrayDirectorySource =
      XrayDirectorySource::read_ignoring(&root, &[String::from("configs\\wip")]).expect("the root indexes");
    let changed = source
      .with_changes(&["configs\\wip\\draft.ltx"], &[])
      .expect("the source takes the change");

    assert_eq!(changed.count_entries(), 0);
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
    // `configs-old` and `configs.ltx` sort between `configs` and `configs\`, inside the range a listing walks.
    let source: XrayDirectorySource = source(
      "boundaries",
      &[
        "configs/system.ltx",
        "configs_backup/system.ltx",
        "configs-old/system.ltx",
        "configs.ltx",
      ],
    );

    let configs: Vec<String> = source.list_entries(Some("configs")).collect();

    assert_eq!(configs, vec![String::from("configs\\system.ltx")]);
  }
}
