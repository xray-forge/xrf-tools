//! Names crossing from the host tree into an archive: what survives, and what has to be refused rather than mangled.
//!
//! Folding two authored names into one engine name is registration's own concern and is tested beside
//! `archive_pack_name_table`. These are the names an archive cannot carry at all.

use xrf_archive::ArchiveProject;

use crate::pack::ArchivePacker;
use crate::pack::tests::fixtures::{CONFIG, create_config, open, pack, read};

#[test]
fn keeps_names_the_engine_can_read() {
  let name: &str = "configs\\текст\\диалог.ltx";
  let (_, destination) = pack("keeps_names_the_engine_can_read", &[(name, CONFIG)], |_| {});
  let project: ArchiveProject = open(&destination);

  // The reader decodes names as windows-1251, so a Cyrillic name only survives if it was written so.
  assert_eq!(read(&project, name), CONFIG);
}

#[test]
fn refuses_a_name_it_cannot_encode() {
  let (config, _) = create_config("refuses_a_name_it_cannot_encode", &[("configs\\ロゴ.ltx", CONFIG)]);

  // Silently mangling a name would produce an archive the engine cannot resolve by that name.
  assert!(ArchivePacker::pack(&config).is_err());
}

/// A Unix filename is bytes, not text, so a source file can be perfectly valid and have no archive name at all. The
/// writer already refuses a name it cannot encode as windows-1251; this one never reached it, because a host path
/// that is not valid Unicode produced no name to refuse.
#[test]
#[cfg(target_os = "linux")]
fn refuses_a_source_file_whose_host_name_is_not_valid_unicode() {
  use std::ffi::OsStr;
  use std::os::unix::ffi::OsStrExt;

  let (config, destination) = create_config(
    "refuses_a_source_file_whose_host_name_is_not_valid_unicode",
    &[("configs\\system.ltx", CONFIG)],
  );

  std::fs::write(
    config.source.join("configs").join(OsStr::from_bytes(b"broken\xff.ltx")),
    CONFIG,
  )
  .expect("source file");

  assert!(
    ArchivePacker::pack(&config).is_err(),
    "a file with no archive name is reported rather than dropped"
  );
  assert!(!destination.exists(), "and nothing is published");
}
