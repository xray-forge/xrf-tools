use std::fs;
use std::path::PathBuf;

use xrf_archive::ArchiveProject;

use super::fixtures::{Entry, create_project, create_temporary_directory};
use crate::ArchiveUnpacker;

#[test]
fn unpack_preserves_empty_files_and_directories() {
  let directory: PathBuf = create_temporary_directory("empty-tree");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\empty\\", b""),
      Entry::stored("configs\\empty.ltx", b""),
    ],
  );
  let out: PathBuf = directory.join("out");

  ArchiveUnpacker::unpack(&project, &out).expect("unpack");

  assert!(out.join("configs").join("empty").is_dir());
  assert_eq!(
    fs::metadata(out.join("configs").join("empty.ltx"))
      .expect("empty file")
      .len(),
    0
  );
}

/// A Unix filename is bytes, not text, so a destination can be perfectly valid and still not be valid
/// Unicode. Summarising a finished run used to `to_str().unwrap()` both halves, turning a completed
/// extraction into a panic; the result renders lossily instead and stays a success.
#[test]
#[cfg(unix)]
fn unpack_renders_a_summary_for_paths_that_are_not_valid_unicode() {
  use std::collections::HashMap;
  use std::ffi::OsStr;
  use std::os::unix::ffi::OsStrExt;

  use xrf_archive::ArchiveDescriptor;

  use crate::ArchiveUnpackResult;

  let directory: PathBuf = create_temporary_directory("non-utf8");
  let root: String = directory.display().to_string();

  let mut project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);

  // The fixture builds no volume records, and the summary names them, so one has to be added here.
  project.archives.push(ArchiveDescriptor {
    created_at: None,
    modified_at: None,
    files: HashMap::new(),
    output_root_path: PathBuf::new(),
    path: directory.join(OsStr::from_bytes(b"broken\xff.db0")),
  });

  let out: PathBuf = directory.join(OsStr::from_bytes(b"out\xff"));
  let result: ArchiveUnpackResult = ArchiveUnpacker::unpack(&project, &out).expect("unpack");

  // The write really did complete: the panic used to happen after this file was already on disk.
  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );

  assert_eq!(result.destination, format!("{root}/out\u{FFFD}"));
  assert_eq!(result.archives, vec![format!("{root}/broken\u{FFFD}.db0")]);
}
