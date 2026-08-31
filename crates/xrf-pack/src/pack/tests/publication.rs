//! What packing refuses to publish over, and what a run that did not finish leaves behind.
//!
//! Two halves of one rule. A destination already holding the set is refused unless the caller forces it, and because
//! of that refusal every volume of the set in a destination afterwards belongs to the run that just failed there — so
//! it is taken back rather than left as residue.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use xrf_error::XrfError;
use xrf_job::{JobHandle, JobOutcome, JobProgress, ProgressSink};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::archive_pack_options::{ArchivePackOptions, PACK_PHASE_WRITE};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_packer::ArchivePacker;
use crate::pack::tests::fixtures::CONFIG;

/// A source tree of its own, packed as `packed` into the scope's shared destination.
///
/// Unlike the shared fixture, the destination is left exactly as the test found it: what a second run does to a
/// destination the first one filled is the whole subject here.
fn create_run(scope: &str, run: &str, files: &[(&str, &[u8])]) -> ArchivePackConfig {
  let source: PathBuf = build_absolute_generated_test_resource_path(&format!("{scope}/{run}"));

  let _ = fs::remove_dir_all(&source);

  for (name, contents) in files {
    let path: PathBuf = source.join(name.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("source directory");
    fs::write(&path, contents).expect("source file");
  }

  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, destination_of(scope), "packed");

  config.include_directories = vec![ArchivePackDirectory {
    path: String::new(),
    is_recursive: true,
  }];

  config
}

/// The one destination every run of a scope publishes into, emptied before the scope's first run.
fn destination_of(scope: &str) -> PathBuf {
  build_absolute_generated_test_resource_path(&format!("{scope}/db"))
}

fn clear_destination(scope: &str) -> PathBuf {
  let destination: PathBuf = destination_of(scope);

  let _ = fs::remove_dir_all(&destination);

  destination
}

/// File names in a destination, sorted, so an assertion reads as what is on disk.
fn list_files(destination: &Path) -> Vec<String> {
  let Ok(entries) = fs::read_dir(destination) else {
    return Vec::new();
  };

  let mut names: Vec<String> = entries
    .map(|entry| {
      entry
        .expect("directory entry")
        .file_name()
        .to_string_lossy()
        .into_owned()
    })
    .collect();

  names.sort();

  names
}

/// Stops the run it watches as soon as one entry has been written.
///
/// Reported to at a zero interval, so the write phase's first advance arrives here rather than being coalesced away.
/// Waiting for that advance is what makes the cancellation land inside the write loop, with a volume already created,
/// instead of at the boundary before it where there would be nothing to clean up.
#[derive(Default)]
struct CancelOnFirstEntry {
  job: std::sync::OnceLock<JobHandle>,
  is_cancelled: AtomicBool,
}

impl ProgressSink for CancelOnFirstEntry {
  fn report(&self, progress: &JobProgress) {
    let is_writing: bool = progress
      .levels
      .iter()
      .any(|level| level.id == PACK_PHASE_WRITE && level.completed > 0);

    if is_writing && !self.is_cancelled.swap(true, Ordering::Relaxed) {
      self.job.get().expect("the sink is given its job").cancel();
    }
  }
}

#[test]
fn refuses_a_destination_that_already_holds_the_set() {
  let scope: &str = "refuses_a_destination_that_already_holds_the_set";
  let destination: PathBuf = clear_destination(scope);

  ArchivePacker::pack(&create_run(scope, "first", &[("configs\\first.ltx", CONFIG)])).expect("the first run publishes");

  let published: Vec<u8> = fs::read(destination.join("packed.db")).expect("the first run's volume");
  let error: XrfError = ArchivePacker::pack(&create_run(scope, "second", &[("configs\\second.ltx", CONFIG)]))
    .expect_err("a second run over the same set is refused");

  assert!(matches!(error, XrfError::Invalid { .. }));
  assert!(error.to_string().contains("packed.db"), "the refusal names the set");

  // The point of refusing: the archive the caller already had is still the one on disk, byte for byte.
  assert_eq!(list_files(&destination), vec!["packed.db"]);
  assert_eq!(fs::read(destination.join("packed.db")).expect("still there"), published);
}

#[test]
fn a_forced_run_replaces_the_set_it_was_pointed_at() {
  let scope: &str = "a_forced_run_replaces_the_set_it_was_pointed_at";
  let destination: PathBuf = clear_destination(scope);

  ArchivePacker::pack(&create_run(scope, "first", &[("configs\\first.ltx", CONFIG)])).expect("the first run publishes");

  let published: Vec<u8> = fs::read(destination.join("packed.db")).expect("the first run's volume");
  let result: ArchivePackResult = ArchivePacker::pack_opt(
    &create_run(scope, "second", &[("configs\\second.ltx", CONFIG)]),
    ArchivePackOptions::default().with_force(true),
  )
  .expect("a forced run publishes over it");

  assert_eq!(result.volumes, vec![destination.join("packed.db")]);
  assert_ne!(
    fs::read(destination.join("packed.db")).expect("the replacement"),
    published,
    "the forced run wrote its own set over the previous one"
  );
}

#[test]
fn a_second_named_set_in_one_destination_is_not_a_conflict() {
  // Packing `gamedata` and `textures` into one output folder is ordinary, and neither publishes the other's names.
  let scope: &str = "a_second_named_set_in_one_destination_is_not_a_conflict";
  let destination: PathBuf = clear_destination(scope);

  let mut other: ArchivePackConfig = create_run(scope, "other", &[("configs\\other.ltx", CONFIG)]);

  other.name = String::from("textures");

  ArchivePacker::pack(&create_run(scope, "first", &[("configs\\first.ltx", CONFIG)])).expect("the first set publishes");
  ArchivePacker::pack(&other).expect("a differently named set publishes beside it");

  assert_eq!(list_files(&destination), vec!["packed.db", "textures.db"]);
}

#[test]
fn a_name_it_cannot_encode_leaves_the_destination_as_it_found_it() {
  // The first name encodes and is written; the second cannot be spelled in the encoding the engine reads, and by then
  // a volume exists at its published name. Nothing of it may survive the refusal.
  let scope: &str = "a_name_it_cannot_encode_leaves_the_destination_as_it_found_it";
  let destination: PathBuf = clear_destination(scope);

  let error: XrfError = ArchivePacker::pack(&create_run(
    scope,
    "run",
    &[("configs\\a.ltx", CONFIG), ("configs\\ロゴ.ltx", CONFIG)],
  ))
  .expect_err("a name the engine cannot read is refused");

  assert!(matches!(error, XrfError::Encoding { .. }));
  assert_eq!(list_files(&destination), Vec::<String>::new());
}

#[test]
fn a_cap_no_volume_could_hold_leaves_the_destination_as_it_found_it() {
  // The reproduction packing under `--max-size 1` records: one entry lands, the next fits in no volume of that size,
  // and the run stops with a closed volume and an open one already on disk.
  let scope: &str = "a_cap_no_volume_could_hold_leaves_the_destination_as_it_found_it";
  let destination: PathBuf = clear_destination(scope);

  let mut config: ArchivePackConfig = create_run(
    scope,
    "run",
    &[("textures\\a.dds", &[b'a'; 128]), ("textures\\z.dds", &[b'z'; 8192])],
  );

  config.max_volume_size = 2048;

  let error: XrfError = ArchivePacker::pack(&config).expect_err("an entry no volume could hold is refused");

  assert!(matches!(error, XrfError::Invalid { .. }));
  assert_eq!(list_files(&destination), Vec::<String>::new());
}

#[test]
fn a_cancelled_run_leaves_the_destination_as_it_found_it() {
  let scope: &str = "a_cancelled_run_leaves_the_destination_as_it_found_it";
  let destination: PathBuf = clear_destination(scope);

  let files: Vec<(String, Vec<u8>)> = (0..8)
    .map(|index: u8| (format!("configs\\file_{index}.ltx"), vec![b'a' + index; 512]))
    .collect();
  let borrowed: Vec<(&str, &[u8])> = files
    .iter()
    .map(|(name, contents)| (name.as_str(), contents.as_slice()))
    .collect();

  let sink: Arc<CancelOnFirstEntry> = Arc::new(CancelOnFirstEntry::default());
  let job: JobHandle = JobHandle::with_interval(sink.clone(), Duration::ZERO);

  assert!(sink.job.set(job.clone()).is_ok(), "the sink is given its job once");

  let result: ArchivePackResult = ArchivePacker::pack_opt(
    &create_run(scope, "run", &borrowed),
    ArchivePackOptions::default().with_job(job),
  )
  .expect("a stopped run reports rather than fails");

  assert_eq!(result.outcome, JobOutcome::Cancelled);

  // Both lists are empty because there is nothing to name: the volume the run had opened is gone with it.
  assert!(result.volumes.is_empty());
  assert!(result.volumes_opened.is_empty());
  assert_eq!(list_files(&destination), Vec::<String>::new());
}

#[test]
fn the_set_is_every_volume_of_that_name_whatever_its_index_or_case() {
  let scope: &str = "the_set_is_every_volume_of_that_name_whatever_its_index_or_case";
  let destination: PathBuf = clear_destination(scope);

  fs::create_dir_all(&destination).expect("destination");

  for name in [
    "packed.db",
    "packed.db0",
    "packed.db10",
    "PACKED.DB2",
    // Neither the same set nor the same name: an extension packing never writes, a longer base name, and a
    // suffix that is not an index.
    "packed.xdb0",
    "packedx.db0",
    "other.db1",
    "packed.db0x",
  ] {
    fs::write(destination.join(name), b"volume").expect("volume");
  }

  // A directory named like a volume is not one, and removing it on a failed run would take whatever it holds.
  fs::create_dir_all(destination.join("packed.db9")).expect("directory");

  let config: ArchivePackConfig = ArchivePackConfig::new("gamedata", &destination, "packed");
  let mut published: Vec<String> = ArchivePacker::list_published_volumes(&config)
    .expect("the destination lists")
    .iter()
    .map(|volume| volume.file_name().expect("named").to_string_lossy().into_owned())
    .collect();

  published.sort();

  assert_eq!(published, vec!["PACKED.DB2", "packed.db", "packed.db0", "packed.db10"]);
}
