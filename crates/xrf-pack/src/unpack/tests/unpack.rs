use std::fs;
use std::fs::File;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject, ArchiveVolumeReaders};
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
  let result: ArchiveUnpackResult =
    ArchiveUnpacker::unpack_opt(&project, &out, ArchiveUnpackOptions::default().with_concurrency(ONE)).expect("unpack");

  // The write really did complete: the panic used to happen after this file was already on disk.
  assert_eq!(
    fs::read_to_string(out.join("configs").join("system.ltx")).expect("written file"),
    "[section]"
  );

  assert_eq!(result.destination, format!("{root}/out\u{FFFD}"));
  assert_eq!(result.archives, vec![format!("{root}/broken\u{FFFD}.db0")]);
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

/// A descriptor belonging to some other set is refused rather than read out of whichever volume happens to be open.
///
/// The readers open exactly what their own project's entries name, so an entry from elsewhere has no volume here.
/// Serving it anyway would mean reading whatever its offset landed in, which is how one archive's bytes end up
/// written out under another archive's name.
#[test]
fn readers_refuse_an_entry_from_another_set() {
  let directory: PathBuf = create_temporary_directory("readers-foreign-entry");
  let mine_at: PathBuf = directory.join("mine");
  let theirs_at: PathBuf = directory.join("theirs");

  fs::create_dir_all(&mine_at).expect("one set");
  fs::create_dir_all(&theirs_at).expect("another set");

  let mine: ArchiveProject = create_project(&mine_at, &[Entry::stored("configs\\system.ltx", b"[section]")]);
  let theirs: ArchiveProject = create_project(&theirs_at, &[Entry::stored("configs\\other.ltx", b"[other]")]);

  let readers: ArchiveVolumeReaders = ArchiveVolumeReaders::open(&mine).expect("volumes open");
  let foreign: &ArchiveFileDescriptor = theirs.files.values().next().expect("an entry");
  let mut target: File = File::create(directory.join("out.bin")).expect("target file");

  let error: XrfError = readers
    .write_descriptor_contents(&mut target, foreign)
    .expect_err("an entry from another set has no volume here");

  assert!(matches!(error, XrfError::Read { .. }), "{error}");
}
