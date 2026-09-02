use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;

use super::fixtures::{Entry, create_project, create_temporary_directory};
use crate::{ArchiveUnpackOptions, ArchiveUnpacker};

/// One worker: the smallest pool that runs at all, and the sequential unpack the crate no longer spells separately.
const ONE: NonZeroUsize = NonZeroUsize::new(1).expect("a non-zero worker count");

/// More entries than workers, which is what an ordinary run looks like.
const TWO: NonZeroUsize = NonZeroUsize::new(2).expect("a non-zero worker count");

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

  ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).expect("unpack");

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
  use std::ffi::OsStr;
  use std::os::unix::ffi::OsStrExt;

  use crate::ArchiveUnpackResult;

  let directory: PathBuf = create_temporary_directory("non-utf8");
  let root: String = directory.display().to_string();

  // The volume really lives under a name that is not valid Unicode, rather than a record pointing at one: a project
  // holds only volumes it read, and unpacking opens every one of them before it writes anything.
  let volumes: PathBuf = directory.join(OsStr::from_bytes(b"vol\xff"));

  fs::create_dir_all(&volumes).expect("a directory whose name is not valid Unicode");

  let project: ArchiveProject = create_project(&volumes, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let out: PathBuf = directory.join(OsStr::from_bytes(b"out\xff"));
  let result: ArchiveUnpackResult =
    ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).expect("unpack");

  // The write really did complete: the panic used to happen after this file was already on disk.
  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );

  assert_eq!(result.destination, format!("{root}/out\u{FFFD}"));
  assert_eq!(result.archives, vec![format!("{root}/vol\u{FFFD}/files.db0")]);
}

/// A plain test, deliberately not an async one: unpacking drives its own pool and must not need an ambient executor.
/// While this ran on a Tokio join set the same call panicked with "there is no reactor running" outside a runtime, and
/// inside one it put every blocking write onto an executor worker.
///
/// One worker is also the smallest pool that runs at all. Zero used to be expressible, and an archive holding a single
/// file then never finished; `NonZeroUsize` at the boundary is what makes that call unwritable.
#[test]
fn unpack_writes_a_file_on_a_single_worker_without_a_runtime() {
  let directory: PathBuf = create_temporary_directory("parallel-one-permit");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let out: PathBuf = directory.join("out");

  ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).expect("unpack");

  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );
}

/// The ordinary parallel run: more entries than workers, a directory row with nothing to write, and a compressed entry
/// a worker has to decompress.
#[test]
fn unpack_writes_every_entry_across_several_workers() {
  const COMPRESSIBLE: &[u8] = b"[alife]\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

  let directory: PathBuf = create_temporary_directory("parallel-several-permits");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\empty\\", b""),
      Entry::stored("configs\\system.ltx", b"[section]"),
      Entry::compressed("configs\\alife.ltx", COMPRESSIBLE),
    ],
  );
  let out: PathBuf = directory.join("out");

  ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(TWO)).expect("unpack");

  assert!(out.join("configs").join("empty").is_dir());
  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );
  assert_eq!(
    fs::read(out.join("configs").join("alife.ltx")).expect("written file"),
    COMPRESSIBLE
  );
}

/// An entry naming a volume its project does not hold is refused rather than read out of another one.
///
/// Attribution is a position in the set, so this is the one way it can be wrong: a descriptor that came from a
/// different project. Serving it would mean reading whichever volume that position happened to land on.
#[test]
fn an_entry_naming_a_volume_outside_its_project_is_refused() {
  let directory: PathBuf = create_temporary_directory("entry-outside-project");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let stray: ArchiveFileDescriptor = project
    .files
    .values()
    .next()
    .expect("an entry")
    .clone()
    .in_volume(project.archives.len() as u32);

  let error: XrfError = project
    .get_volume_of(&stray)
    .expect_err("a position past the set has no volume");

  assert!(matches!(error, XrfError::Read { .. }), "{error}");
}
