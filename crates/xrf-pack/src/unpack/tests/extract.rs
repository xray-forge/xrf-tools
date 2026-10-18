use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use xrf_archive::ArchiveProject;
use xrf_job::{ExecutionRequest, JobHandle, JobOutcome};

use super::fixtures::{Entry, create_project, create_temporary_directory};
use crate::{ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveUnpacker};

/// More entries than workers, which is what an ordinary run looks like.
const TWO: NonZeroUsize = NonZeroUsize::new(2).expect("a non-zero worker count");

#[test]
fn extract_directory_writes_every_file_under_the_prefix() {
  let directory: PathBuf = create_temporary_directory("directory");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\gameplay\\dialogs.xml", b"<game_dialogs/>"),
      Entry::stored("configs\\system.ltx", b"[section]"),
      Entry::stored("meshes\\actor.ogf", b"ogf"),
    ],
  );

  let out: PathBuf = directory.join("out");
  let result: ArchiveExtractDirectoryResult =
    ArchiveUnpacker::extract_directory(&project, "configs", &out).expect("extraction");

  assert_eq!(result.extracted_count, 2);
  // The prefix is stripped: the user chose the destination for that directory already.
  assert_eq!(
    fs::read_to_string(out.join("gameplay").join("dialogs.xml")).expect("nested file"),
    "<game_dialogs/>"
  );
  assert_eq!(
    fs::read_to_string(out.join("system.ltx")).expect("root file"),
    "[section]"
  );
  assert!(!out.join("actor.ogf").exists(), "must not reach outside the prefix");
}

#[test]
fn extract_directory_preserves_empty_files_and_skips_directory_records() {
  let directory: PathBuf = create_temporary_directory("empty");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\gameplay\\", b""),
      Entry::stored("configs\\gameplay\\dialogs.xml", b"<game_dialogs/>"),
      Entry::stored("configs\\empty.ltx", b""),
    ],
  );

  let out: PathBuf = directory.join("out");
  let result: ArchiveExtractDirectoryResult =
    ArchiveUnpacker::extract_directory(&project, "configs", &out).expect("extraction");

  assert_eq!(result.extracted_count, 2);
  assert!(out.join("gameplay").join("dialogs.xml").exists());
  assert_eq!(fs::metadata(out.join("empty.ltx")).expect("empty file").len(), 0);
}

#[test]
fn extract_directory_takes_the_whole_archive_for_an_empty_prefix() {
  let directory: PathBuf = create_temporary_directory("root");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\system.ltx", b"[section]"),
      Entry::stored("meshes\\actor.ogf", b"ogf"),
    ],
  );

  let out: PathBuf = directory.join("out");
  let result: ArchiveExtractDirectoryResult =
    ArchiveUnpacker::extract_directory(&project, "", &out).expect("extraction");

  assert_eq!(result.extracted_count, 2);
  assert!(out.join("configs").join("system.ltx").exists());
  assert!(out.join("meshes").join("actor.ogf").exists());
}

#[test]
fn extract_file_writes_to_the_exact_path_it_is_given() {
  let directory: PathBuf = create_temporary_directory("single");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);

  let target: PathBuf = directory.join("chosen").join("renamed.ltx");

  ArchiveUnpacker::extract_file(&project, "configs\\system.ltx", &target).expect("extraction");

  assert_eq!(fs::read_to_string(&target).expect("written file"), "[section]");
}

#[test]
fn extract_file_refuses_a_directory_record() {
  let directory: PathBuf = create_temporary_directory("directory-record");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\", b"")]);

  assert!(ArchiveUnpacker::extract_file(&project, "configs\\", directory.join("out")).is_err());
}

#[test]
fn read_file_bytes_returns_the_stored_contents() {
  let directory: PathBuf = create_temporary_directory("bytes");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\system.ltx", b"[section]"),
      Entry::stored("textures\\wall.dds", b"\x44\x44\x53\x20not-a-real-dds"),
    ],
  );

  // Reads by offset, so the second entry must not bleed into the first.
  assert_eq!(
    project.read_file_bytes("configs\\system.ltx").expect("bytes"),
    b"[section]"
  );
  assert_eq!(
    project.read_file_bytes("textures\\wall.dds").expect("bytes"),
    b"\x44\x44\x53\x20not-a-real-dds"
  );
}

#[test]
fn read_file_bytes_reports_an_unknown_name() {
  let directory: PathBuf = create_temporary_directory("bytes-missing");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);

  assert!(project.read_file_bytes("configs\\other.ltx").is_err());
}

/// Compressible enough that lzo actually shrinks it, so `size_compressed` really differs.
const COMPRESSIBLE: &[u8] = b"[section]\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\nvalue = 1\n";

#[test]
fn read_file_bytes_decompresses_a_compressed_entry() {
  let directory: PathBuf = create_temporary_directory("compressed-bytes");
  let project: ArchiveProject = create_project(&directory, &[Entry::compressed("configs\\system.ltx", COMPRESSIBLE)]);

  let descriptor = project.files.get("configs\\system.ltx").expect("descriptor");

  assert!(
    descriptor.size_compressed < descriptor.size_real,
    "the fixture has to be genuinely compressed for this to test anything"
  );

  // Also verifies the crc, which is computed over the decompressed bytes.
  assert_eq!(
    project.read_file_bytes("configs\\system.ltx").expect("bytes"),
    COMPRESSIBLE
  );
}

#[test]
fn read_file_as_string_accepts_a_compressed_entry() {
  let directory: PathBuf = create_temporary_directory("compressed-string");
  let project: ArchiveProject = create_project(&directory, &[Entry::compressed("configs\\system.ltx", COMPRESSIBLE)]);

  let result = project.read_file_as_string("configs\\system.ltx").expect("read");

  // Previously refused outright, because reading raw bytes at the offset would have produced rubbish.
  assert_eq!(result.content, String::from_utf8_lossy(COMPRESSIBLE));
  assert_eq!(result.size, COMPRESSIBLE.len() as u32);
}

#[test]
fn read_file_as_string_decodes_windows_1251_text() {
  // Archive text is Windows-1251; these bytes spell `Прицел`, which a lossy UTF-8 read used to turn into replacement
  // characters in the archive explorer.
  const CYRILLIC_W1251: &[u8] = b"[wpn]\r\nname = \xCF\xF0\xE8\xF6\xE5\xEB\r\n";

  let directory: PathBuf = create_temporary_directory("w1251-string");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\weapon.ltx", CYRILLIC_W1251)]);

  let result = project.read_file_as_string("configs\\weapon.ltx").expect("read");

  assert_eq!(result.content, "[wpn]\r\nname = Прицел\r\n");
  assert!(!result.content.contains('\u{FFFD}'), "no replacement characters");
}

#[test]
fn extract_file_writes_a_compressed_entry_decompressed() {
  let directory: PathBuf = create_temporary_directory("compressed-extract");
  let project: ArchiveProject = create_project(&directory, &[Entry::compressed("configs\\system.ltx", COMPRESSIBLE)]);

  let target: PathBuf = directory.join("out").join("system.ltx");

  ArchiveUnpacker::extract_file(&project, "configs\\system.ltx", &target).expect("extraction");

  // What lands on disk is the file, not the archive's compressed image of it.
  assert_eq!(fs::read(&target).expect("written file"), COMPRESSIBLE);
}

/// The ordinary parallel run: more entries than workers, a directory record with nothing to write, and a compressed
/// entry a worker has to decompress.
#[test]
fn extract_directory_writes_every_entry_across_several_workers() {
  let directory: PathBuf = create_temporary_directory("directory-parallel");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\gameplay\\", b""),
      Entry::stored("configs\\gameplay\\dialogs.xml", b"<game_dialogs/>"),
      Entry::stored("configs\\system.ltx", b"[section]"),
      Entry::compressed("configs\\alife.ltx", COMPRESSIBLE),
      Entry::stored("meshes\\actor.ogf", b"ogf"),
    ],
  );

  let out: PathBuf = directory.join("out");
  let result: ArchiveExtractDirectoryResult = ExecutionRequest::Workers(TWO)
    .resolve()
    .install(|| ArchiveUnpacker::extract_directory(&project, "configs", &out))
    .expect("the pool starts")
    .expect("extraction");

  assert_eq!(result.outcome, JobOutcome::Completed);
  assert_eq!(result.extracted_count, 3, "the directory record is not a file");
  assert_eq!(
    result.size,
    (b"<game_dialogs/>".len() + b"[section]".len() + COMPRESSIBLE.len()) as u64
  );
  assert_eq!(
    fs::read_to_string(out.join("gameplay").join("dialogs.xml")).expect("nested file"),
    "<game_dialogs/>"
  );
  assert_eq!(
    fs::read(out.join("alife.ltx")).expect("decompressed file"),
    COMPRESSIBLE
  );
  assert!(!out.join("actor.ogf").exists(), "must not reach outside the prefix");
}

/// A run asked to stop before its first entry writes nothing and reports that as a cancellation, not a failure.
#[test]
fn extract_directory_stops_between_entries_when_cancelled() {
  let directory: PathBuf = create_temporary_directory("directory-cancelled");
  let project: ArchiveProject = create_project(
    &directory,
    &[
      Entry::stored("configs\\system.ltx", b"[section]"),
      Entry::stored("configs\\game.ltx", b"[game]"),
    ],
  );

  let job: JobHandle = JobHandle::inert();

  job.cancel();

  let out: PathBuf = directory.join("out");
  let result: ArchiveExtractDirectoryResult = ArchiveUnpacker::extract_directory_opt(
    &project,
    "configs",
    &out,
    ArchiveExtractOptions::default().with_job(job),
  )
  .expect("a cancelled run still answers");

  assert_eq!(result.outcome, JobOutcome::Cancelled);
  assert_eq!(result.extracted_count, 0);
  assert!(!out.join("system.ltx").exists());
}

#[test]
fn extract_directory_reports_a_prefix_that_matches_nothing() {
  let directory: PathBuf = create_temporary_directory("missing");
  let project: ArchiveProject = create_project(&directory, &[Entry::stored("configs\\system.ltx", b"[section]")]);

  assert!(ArchiveUnpacker::extract_directory(&project, "meshes", directory.join("out")).is_err());
}
