//! Covers what a project may and may not do with a config that has no file on disk.
//!
//! The distinction these pin down is the one an installation forces: a formatting *verdict* needs only content, so it must
//! answer for an archived config, while *rewriting* one has nothing to write to and must refuse. Before this, both went
//! through the same physical-path check and a single archived config aborted the whole check.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayAssetContainer, XrayAssetSource, XrayLookupScope, XraySourceKind, XrayVfs};

use crate::project::{LtxProject, LtxProjectFormatResult};

/// A read-only source whose entries have no filesystem path, as archive volume entries do not.
///
/// In memory rather than a packed `.db` fixture: what matters here is only that `physical_path()` answers `None`, and
/// `xrf-ltx` cannot depend on the packer without a crate cycle.
#[derive(Debug)]
struct ArchivedConfigs {
  entries: HashMap<String, Vec<u8>>,
  root: PathBuf,
}

impl ArchivedConfigs {
  fn new(entries: &[(&str, &str)]) -> Self {
    Self {
      entries: entries
        .iter()
        .map(|(path, contents)| (path.to_string(), contents.as_bytes().to_vec()))
        .collect(),
      root: PathBuf::from("C:\\install\\db\\configs"),
    }
  }
}

impl XrayAssetSource for ArchivedConfigs {
  fn get_label(&self) -> &str {
    "configs"
  }

  fn get_kind(&self) -> XraySourceKind {
    XraySourceKind::Archive
  }

  fn is_writable(&self) -> bool {
    false
  }

  fn get_root_path(&self) -> &Path {
    &self.root
  }

  fn contains(&self, path: &str) -> bool {
    self.entries.contains_key(path)
  }

  fn locate(&self, path: &str) -> Option<XrayAssetContainer> {
    self.entries.contains_key(path).then(|| XrayAssetContainer::Archive {
      path: self.root.clone(),
    })
  }

  fn read(&self, path: &str) -> XrfResult<Vec<u8>> {
    self
      .entries
      .get(path)
      .cloned()
      .ok_or_else(|| XrfError::new_asset_error(format!("no entry {path}")))
  }

  fn write(&self, _path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_asset_error("archive is read only"))
  }

  fn create(&self, _path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_asset_error("archive is read only"))
  }

  fn list_entries<'a>(&'a self, _prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a> {
    Box::new(self.entries.keys().cloned())
  }

  fn get_size(&self, path: &str) -> Option<u64> {
    self.entries.get(path).map(|bytes| bytes.len() as u64)
  }
}

/// Opens a project over archived configs alone, so nothing in it has a file on disk.
fn archived_project(entries: &[(&str, &str)]) -> LtxProject {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount("", Box::new(ArchivedConfigs::new(entries)))
    .expect("archived configs mount");

  LtxProject::open_at_scope_opt(
    PathBuf::from("C:\\install\\gamedata\\configs"),
    vfs,
    XrayLookupScope::all(),
    Default::default(),
  )
  .expect("project opens over archived configs")
}

#[test]
fn checks_formatting_of_configs_that_have_no_file() {
  let project: LtxProject = archived_project(&[
    ("system.ltx", "[a]\r\nkey = value\r\n"),
    ("creatures\\actor.ltx", "[b]\nkey=value\n"),
  ]);

  let result: LtxProjectFormatResult = project
    .check_format_all_files()
    .expect("archived configs are checkable");

  assert_eq!(result.total_files, 2);
  assert_eq!(result.valid_files, 1);
  assert_eq!(result.invalid_files, 1);
  // Named by engine identity, the only honest answer for a config with no file to point at.
  assert_eq!(result.to_format, vec![PathBuf::from("creatures\\actor.ltx")]);
}

#[test]
fn refuses_to_rewrite_configs_that_have_no_file() {
  let project: LtxProject = archived_project(&[("system.ltx", "[a]\nkey=value\n")]);

  let error: XrfError = project
    .format_all_files()
    .expect_err("rewriting an archived config has nothing to write to");

  assert!(
    error.to_string().contains("system.ltx"),
    "the refusal names the config it cannot rewrite, got: {error}"
  );
}
