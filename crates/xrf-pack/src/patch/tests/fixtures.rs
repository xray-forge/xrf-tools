//! Loose and archived comparison fixtures in per-test scratch directories.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveProject;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::pack::config::ArchivePackConfig;
use crate::pack::{ArchivePackResult, ArchivePacker};
use crate::patch::compare::ArchivePatchChange;
use crate::patch::config::ArchivePatchConfig;
use crate::patch::{ArchivePatchResult, ArchivePatcher};

/// A configuration fragment large enough that compressing it pays off.
pub(crate) const CONFIG: &[u8] =
  b"[section]\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

/// The same fragment with one value changed, so a pair differs without changing length.
pub(crate) const CONFIG_EDITED: &[u8] =
  b"[section]\nvalue = 1\nvalue = 1\nvalue = 2\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

/// A longer fragment, so a pair differs in size and never needs a checksum.
pub(crate) const CONFIG_LONGER: &[u8] =
  b"[section]\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

/// Bytes with no structure to exploit, standing in for a texture.
pub(crate) const BINARY: &[u8] = &[0x44, 0x44, 0x53, 0x20, 0x01, 0x02, 0x03, 0xfe, 0xff, 0x00];

/// Build a loose tree under this test's scratch directory and return its root.
pub(crate) fn create_tree(scope: &str, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/{name}"));

  let _ = fs::remove_dir_all(&root);

  for (entry, contents) in files {
    let path: PathBuf = root.join(entry.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("tree directory");
    fs::write(&path, contents).expect("tree file");
  }

  fs::create_dir_all(&root).expect("tree root");

  root
}

/// Packs a scratch tree and returns the archive directory.
pub(crate) fn create_volumes(scope: &str, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let source: PathBuf = create_tree(scope, &format!("{name}-source"), files);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/{name}"));

  let _ = fs::remove_dir_all(&destination);

  let packed: ArchivePackResult =
    ArchivePacker::pack(&ArchivePackConfig::new(&source, &destination, name)).expect("the fixture packs");

  assert!(!packed.volumes.is_empty(), "the fixture published a volume set");

  destination
}

/// Where a run under this scope publishes.
pub(crate) fn destination(scope: &str) -> PathBuf {
  let path: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/published"));

  let _ = fs::remove_dir_all(&path);

  path
}

/// Compares two roots and publishes with default options.
pub(crate) fn patch(base: &Path, target: &Path, into: &Path) -> ArchivePatchResult {
  ArchivePatcher::patch(&config(base, target, into)).expect("the comparison runs")
}

/// Compares two roots without publishing, using default options.
pub(crate) fn compare(base: &Path, target: &Path, into: &Path) -> ArchivePatchResult {
  ArchivePatcher::compare(&config(base, target, into)).expect("the comparison runs")
}

/// The configuration a test's two roots and destination make, read as an overlay.
pub(crate) fn config(base: &Path, target: &Path, into: &Path) -> ArchivePatchConfig {
  ArchivePatchConfig::new(base, into, "patch").with_target(target)
}

/// One installation supplying both sides: its volumes against its own loose tree.
pub(crate) fn split_config(install: &Path, into: &Path) -> ArchivePatchConfig {
  ArchivePatchConfig::new(install, into, "patch")
}

/// Compares one installation against its own loose tree without publishing.
pub(crate) fn compare_split(install: &Path, into: &Path) -> ArchivePatchResult {
  ArchivePatcher::compare(&split_config(install, into)).expect("the comparison runs")
}

/// Compares one installation against its own loose tree and publishes.
pub(crate) fn patch_split(install: &Path, into: &Path) -> ArchivePatchResult {
  ArchivePatcher::patch(&split_config(install, into)).expect("the comparison runs")
}

/// Engine names of the entries a published patch actually holds, in table order.
pub(crate) fn published_names(destination: &Path) -> Vec<String> {
  let project: ArchiveProject = ArchiveProject::new_shallow(destination).expect("the patch reads back");
  let mut names: Vec<String> = project
    .files
    .values()
    .filter(|descriptor| !descriptor.is_directory)
    .map(|descriptor| descriptor.name.to_string())
    .collect();

  names.sort();

  names
}

/// The payload one published entry reads back as.
pub(crate) fn published_bytes(destination: &Path, name: &str) -> Vec<u8> {
  let project: ArchiveProject = ArchiveProject::new_shallow(destination).expect("the patch reads back");

  project.read_file_bytes(name).expect("the entry reads back")
}

/// The names one class of a comparison reported.
pub(crate) fn names_of(changes: &[ArchivePatchChange]) -> Vec<&str> {
  changes.iter().map(|change| change.name.as_str()).collect()
}

/// The entries every suite starts from, before its own edits.
pub(crate) const BASE_FILES: &[(&str, &[u8])] = &[
  ("configs\\system.ltx", CONFIG),
  ("configs\\weapons\\ak74.ltx", CONFIG),
  ("textures\\wall.dds", BINARY),
];

/// An installation: a volume set under `db\`, a loose `gamedata\`, and the `fsgame.ltx` declaring both.
///
/// Declaration order matters and is the engine's own: `$arch_dir$` before `$game_data$`, so the loose tree wins. That
/// ordering is exactly what makes an installation unable to compare against itself, and what the split reads apart.
pub(crate) fn create_installation(
  scope: &str,
  name: &str,
  archived: &[(&str, &[u8])],
  loose: &[(&str, &[u8])],
) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/{name}"));

  let _ = fs::remove_dir_all(&root);

  let packed: PathBuf = create_volumes(scope, &format!("{name}-db"), archived);

  fs::create_dir_all(root.join("db")).expect("installation db directory");

  for volume in fs::read_dir(&packed).expect("the packed fixture lists") {
    let volume: fs::DirEntry = volume.expect("a packed volume");

    fs::copy(volume.path(), root.join("db").join(volume.file_name())).expect("volume copied into the installation");
  }

  for (entry, contents) in loose {
    let path: PathBuf = root.join("gamedata").join(entry.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("loose directory");
    fs::write(&path, contents).expect("loose file");
  }

  fs::create_dir_all(root.join("gamedata")).expect("loose root");
  fs::write(
    root.join("fsgame.ltx"),
    concat!(
      "$app_data_root$ = true|  false| $fs_root$|  _appdata_\\\n",
      "$arch_dir$      = false| false| $fs_root$|  db\\\n",
      "$game_data$     = false| true|  $fs_root$|  gamedata\\\n"
    ),
  )
  .expect("fsgame declared");

  root
}
