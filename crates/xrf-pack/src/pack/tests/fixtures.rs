//! Source trees, packed sets, and the assertions the pack tests are written against.
//!
//! Everything is built under the per-process scratch tree, scoped per test, so two tests never share a destination.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveProject;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_utils::format_path;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_packer::ArchivePacker;

/// A configuration fragment large enough that compressing it actually pays off.
pub(crate) const CONFIG: &[u8] =
  b"[section]\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

/// Bytes with no structure to exploit, standing in for a texture or a mesh.
pub(crate) const BINARY: &[u8] = &[0x44, 0x44, 0x53, 0x20, 0x01, 0x02, 0x03, 0xfe, 0xff, 0x00];

/// Build a source tree under this test's scratch directory and return its root.
pub(crate) fn create_source(scope: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/gamedata"));

  let _ = fs::remove_dir_all(&root);

  for (name, contents) in files {
    let path: PathBuf = root.join(name.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("source directory");
    fs::write(&path, contents).expect("source file");
  }

  root
}

/// A configuration packing a whole freshly built tree into this scope's destination.
///
/// Returned unpacked so a test can refuse it, which is what the checks that must leave no destination behind need.
pub(crate) fn create_config(scope: &str, files: &[(&str, &[u8])]) -> (ArchivePackConfig, PathBuf) {
  let source: PathBuf = create_source(scope, files);
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "packed");

  // Every test packs the whole tree unless it says otherwise.
  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];

  (config, destination)
}

/// Pack a freshly built source tree, returning what was written and where.
pub(crate) fn pack(
  scope: &str,
  files: &[(&str, &[u8])],
  configure: impl FnOnce(&mut ArchivePackConfig),
) -> (ArchivePackResult, PathBuf) {
  let (mut config, destination) = create_config(scope, files);

  configure(&mut config);

  let result: ArchivePackResult = ArchivePacker::pack(&config).expect("source tree packs");

  (result, destination)
}

pub(crate) fn open(destination: &Path) -> ArchiveProject {
  ArchiveProject::new(destination).expect("written archive opens")
}

pub(crate) fn read(project: &ArchiveProject, name: &str) -> Vec<u8> {
  project
    .read_file_bytes(name)
    .unwrap_or_else(|error| panic!("archive holds '{name}': {error}"))
}

/// Assert every produced volume file is within the cap its configuration advertised.
///
/// File lengths, not the writer's own accounting: the cap covers the descriptor chunk appended at close, which is
/// what a check against reported sizes or entry counts would miss.
pub(crate) fn assert_volumes_within(result: &ArchivePackResult, max_volume_size: u64) {
  for volume in &result.volumes {
    let length: u64 = fs::metadata(volume).expect("volume metadata").len();

    assert!(
      length <= max_volume_size,
      "'{}' is {length} bytes, past the configured maximum of {max_volume_size}",
      format_path(volume)
    );
  }
}

/// Distinct payloads of one size, since identical ones would alias onto a single copy and never split.
pub(crate) fn distinct_files(count: u8, extension: &str, size: usize) -> Vec<(String, Vec<u8>)> {
  (0..count)
    .map(|index| (format!("textures\\tile_{index}.{extension}"), vec![b'a' + index; size]))
    .collect()
}

pub(crate) fn borrow_files(files: &[(String, Vec<u8>)]) -> Vec<(&str, &[u8])> {
  files
    .iter()
    .map(|(name, contents)| (name.as_str(), contents.as_slice()))
    .collect()
}
