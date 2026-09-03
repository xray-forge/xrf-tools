//! What the configuration's rules put in the archive and keep out of it, and what an excluded subtree is spared.
//!
//! The rules themselves are unit tested beside the configuration that owns them. These pack real trees, which is
//! where a rule can be caught reaching a neighbour it shares a prefix with, or reading a subtree it excluded.

use std::path::PathBuf;

use xrf_archive::ArchiveProject;
use xrf_error::XrfError;

use crate::pack::config::ArchivePackDirectory;
use crate::pack::tests::fixtures::{BINARY, CONFIG, allow_directory, create_config, deny_directory, open, pack, read};
use crate::pack::{ArchivePackResult, ArchivePacker};

#[test]
fn leaves_out_what_the_engine_rebuilds() {
  let (result, destination) = pack(
    "leaves_out_what_the_engine_rebuilds",
    &[
      ("configs\\system.ltx", CONFIG),
      ("readme.txt", b"notes"),
      ("textures\\lod\\lod_wall.dds", BINARY),
    ],
    |_| {},
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_total, 1, "only the config is packed");
  assert_eq!(result.files_skipped, 2);
  assert!(project.files.contains_key("configs\\system.ltx"));
  assert!(!project.files.contains_key("readme.txt"));
}

#[test]
fn an_excluded_directory_does_not_reach_the_neighbour_it_shares_a_prefix_with() {
  let (result, destination) = pack(
    "an_excluded_directory_does_not_reach_the_neighbour_it_shares_a_prefix_with",
    &[
      ("configs\\system.ltx", CONFIG),
      ("configs\\weapons\\w_ak74.ltx", CONFIG),
      ("configs_backup\\system.ltx", CONFIG),
    ],
    |config| {
      // Spelled unlike the tree on disk, because the engine resolves a name either way.
      config.exclude_directories = vec![ArchivePackDirectory {
        path: String::from("Configs"),
        is_recursive: true,
      }];
    },
  );
  let project: ArchiveProject = open(&destination);

  assert_eq!(result.files_total, 1, "only the backup tree survives the rule");
  assert_eq!(read(&project, "configs_backup\\system.ltx"), CONFIG);

  for name in ["configs\\", "configs\\system.ltx", "configs\\weapons\\w_ak74.ltx"] {
    assert!(!project.files.contains_key(name), "'{name}' is excluded");
  }
}

/// A packed volume set is read as a complete build of what the configuration selected, so a source subtree the walk
/// cannot enumerate has to end the run. Filtered out, packing reported success over an archive silently missing
/// everything below the unreadable directory.
#[test]
fn refuses_to_pack_a_source_subtree_it_cannot_read() {
  let (config, destination) = create_config(
    "refuses_to_pack_a_source_subtree_it_cannot_read",
    &[
      ("configs\\system.ltx", CONFIG),
      ("scripts\\locked\\hidden.script", CONFIG),
    ],
  );
  let locked: PathBuf = config.source.join("scripts").join("locked");

  if !deny_directory(&locked) {
    return;
  }

  let result: Result<ArchivePackResult, XrfError> = ArchivePacker::pack(&config);

  // Restored before asserting, so a failed expectation still leaves a tree the next run can remove.
  allow_directory(&locked);

  assert!(
    result.is_err(),
    "an unreadable subtree is a packing failure, not an omission"
  );
  assert!(
    !destination.exists(),
    "and selection fails before anything is published"
  );
}

/// The other half of that rule: a subtree a recursive exclusion drops holds nothing that could be selected, so it is
/// never read at all and does not have to be readable for the rest of the tree to pack.
#[test]
fn does_not_read_a_recursively_excluded_subtree() {
  let (mut config, destination) = create_config(
    "does_not_read_a_recursively_excluded_subtree",
    &[
      ("configs\\system.ltx", CONFIG),
      ("scripts\\locked\\hidden.script", CONFIG),
    ],
  );
  let locked: PathBuf = config.source.join("scripts").join("locked");

  config.exclude_directories = vec![ArchivePackDirectory {
    path: String::from("scripts"),
    is_recursive: true,
  }];

  if !deny_directory(&locked) {
    return;
  }

  let result: Result<ArchivePackResult, XrfError> = ArchivePacker::pack(&config);

  allow_directory(&locked);

  assert_eq!(
    result.expect("the excluded subtree is never read").files_total,
    1,
    "only the configuration outside the rule is packed"
  );
  assert!(open(&destination).files.contains_key("configs\\system.ltx"));
}
