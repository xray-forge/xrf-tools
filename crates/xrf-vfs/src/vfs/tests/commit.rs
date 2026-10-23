//! Changing what a snapshot holds: a transaction committed to disk, or changes already there applied.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::vfs::tests::fake_source::{FakeArchiveSource, directory};
use crate::{
  XrayAsset, XrayCachePolicy, XrayChange, XrayChangeKind, XrayChangeSet, XrayInput, XrayInputKind, XrayInputRecord,
  XrayLogicalPath, XrayLookupScope, XrayMountId, XrayRevision, XraySearch, XraySearchKey, XrayTransaction, XrayVfs,
  XrayVfsHost,
};

const SYSTEM_LTX: &str = "configs\\system.ltx";

/// A search over every mount of `vfs`.
fn search(vfs: XrayVfs) -> XraySearch {
  XraySearch::new(vfs).with_step("all", XrayLookupScope::all())
}

/// Commits `transaction` against the snapshot `search` reads, answering a search over the snapshot it leaves.
fn commit(search: &XraySearch, transaction: XrayTransaction) -> (XraySearch, XrayChangeSet) {
  let (next, changes): (XrayVfs, XrayChangeSet) =
    search.get_vfs().commit(&transaction).expect("the transaction commits");

  (self::search(next), changes)
}

fn text(search: &XraySearch, path: &str) -> String {
  String::from_utf8(search.read(path).expect("the path reads")).expect("the test file is text")
}

/// Every file name in a directory, staged and set-aside leftovers included.
fn list_names(directory: &Path) -> Vec<String> {
  let mut names: Vec<String> = fs::read_dir(directory)
    .expect("the directory lists")
    .map(|entry| entry.expect("an entry").file_name().to_string_lossy().into_owned())
    .collect();

  names.sort();

  names
}

#[test]
fn overwrites_a_loose_file_in_place_and_moves_its_revision() {
  let root: PathBuf = directory("commit_overwrite", &["configs/system.ltx", "configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", &root).expect("the tree mounts");
  let before: XraySearch = search(vfs);

  let (after, changes): (XraySearch, XrayChangeSet) = commit(
    &before,
    XrayTransaction::new(&before).with_write(SYSTEM_LTX, "edited").unwrap(),
  );

  assert_eq!(fs::read(root.join("configs/system.ltx")).unwrap(), b"edited");
  assert_eq!(text(&after, SYSTEM_LTX), "edited");
  assert_eq!(changes.get_revision(), XrayRevision::MOUNTED.next());
  assert_eq!(changes.get_changes(), &[XrayChange::modified(id, SYSTEM_LTX).unwrap()]);
  assert_eq!(after.get_vfs().get_revision(), changes.get_revision());
  assert_eq!(
    list_names(&root.join("configs")),
    ["game.ltx", "system.ltx"],
    "nothing is left beside the file"
  );

  let file = |path: &str| {
    XrayInput::new(
      XraySearchKey::of(after.get_steps()),
      XrayInputKind::File(XrayLogicalPath::new(path).unwrap()),
    )
  };

  assert_eq!(
    after.get_vfs().answer_input(&file(SYSTEM_LTX)).revision,
    changes.get_revision()
  );
  assert_eq!(
    after.get_vfs().answer_input(&file("configs\\game.ltx")).revision,
    XrayRevision::MOUNTED
  );
  assert_eq!(
    before.get_vfs().answer_input(&file(SYSTEM_LTX)).revision,
    XrayRevision::MOUNTED,
    "the snapshot committed against stays what it was"
  );
}

#[test]
fn creates_a_path_only_archived_in_the_first_writable_mount_the_search_reaches() {
  let front: PathBuf = directory("commit_create_front", &["configs/other.ltx"]);
  let back: PathBuf = directory("commit_create_back", &["configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  let front_id: XrayMountId = vfs.mount_directory("", &front).expect("the front tree mounts");
  vfs.mount_directory("", &back).expect("the back tree mounts");
  vfs
    .mount("", Box::new(FakeArchiveSource::new("configs", &[SYSTEM_LTX])))
    .expect("the archive mounts");

  let before: XraySearch = search(vfs);

  let (after, changes): (XraySearch, XrayChangeSet) = commit(
    &before,
    XrayTransaction::new(&before)
      .with_write(SYSTEM_LTX, "overridden")
      .unwrap(),
  );

  assert_eq!(fs::read(front.join("configs/system.ltx")).unwrap(), b"overridden");
  assert_eq!(text(&after, SYSTEM_LTX), "overridden", "the created file wins at once");
  assert_eq!(
    text(&before, SYSTEM_LTX),
    "configs",
    "the snapshot before still answers with the archive"
  );
  assert_eq!(
    changes.get_changes(),
    &[XrayChange::created(front_id, SYSTEM_LTX).unwrap()]
  );
  assert_eq!(
    after
      .list_shadowing_entries()
      .into_iter()
      .find(|entry| entry.get_logical_path() == SYSTEM_LTX)
      .map(|entry| entry.shadowed.len()),
    Some(1),
    "the archived copy is still reportable behind it"
  );
}

#[test]
fn a_created_file_moves_every_directory_above_it_and_no_other() {
  let root: PathBuf = directory("commit_directories", &["configs/system.ltx", "textures/a.dds"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");

  let before: XraySearch = search(vfs);
  let (after, changes): (XraySearch, XrayChangeSet) = commit(
    &before,
    XrayTransaction::new(&before)
      .with_write("configs\\weathers\\default.ltx", "weather")
      .unwrap(),
  );

  let listing = |directory: Option<&str>| {
    XrayInput::new(
      XraySearchKey::of(after.get_steps()),
      XrayInputKind::Listing(directory.map(|directory| XrayLogicalPath::new(directory).unwrap())),
    )
  };

  for directory in [None, Some("configs"), Some("configs\\weathers")] {
    assert_eq!(
      after.get_vfs().answer_input(&listing(directory)).revision,
      changes.get_revision(),
      "{directory:?} lists something new"
    );
  }

  assert_eq!(
    after.get_vfs().answer_input(&listing(Some("textures"))).revision,
    XrayRevision::MOUNTED
  );
  assert_eq!(
    after.list_children("configs\\weathers").unwrap().files.len(),
    1,
    "the next snapshot indexes the created file"
  );
}

#[test]
fn deletes_a_loose_file_and_drops_it_from_the_next_snapshot() {
  let root: PathBuf = directory("commit_delete", &["configs/system.ltx", "configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", &root).expect("the tree mounts");
  let before: XraySearch = search(vfs);

  let (after, changes): (XraySearch, XrayChangeSet) =
    commit(&before, XrayTransaction::new(&before).with_delete(SYSTEM_LTX).unwrap());

  assert!(!root.join("configs/system.ltx").exists());
  assert!(after.find(SYSTEM_LTX).unwrap().is_none());
  assert!(
    before.find(SYSTEM_LTX).unwrap().is_some(),
    "the snapshot before keeps its index"
  );
  assert_eq!(changes.get_changes()[0].get_kind(), XrayChangeKind::Deleted);
  assert_eq!(changes.get_changes()[0].get_mount(), id);
  assert_eq!(list_names(&root.join("configs")), ["game.ltx"]);
}

#[test]
fn a_failing_write_leaves_every_file_as_it_was() {
  let root: PathBuf = directory("commit_all_or_nothing", &["configs/system.ltx", "configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");

  let before: XraySearch = search(vfs);
  // A file cannot hold a directory, so the second write fails after the first is staged.
  let transaction: XrayTransaction = XrayTransaction::new(&before)
    .with_write(SYSTEM_LTX, "edited")
    .unwrap()
    .with_write("configs\\game.ltx\\inner.ltx", "unwritable")
    .unwrap();

  assert!(before.get_vfs().commit(&transaction).is_err());
  assert_eq!(
    fs::read(root.join("configs/system.ltx")).unwrap(),
    b"commit_all_or_nothing"
  );
  assert_eq!(
    list_names(&root.join("configs")),
    ["game.ltx", "system.ltx"],
    "nothing staged is left behind"
  );
}

#[test]
fn refuses_a_write_no_writable_mount_can_hold() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount("", Box::new(FakeArchiveSource::new("configs", &[SYSTEM_LTX])))
    .expect("the archive mounts");

  let search: XraySearch = search(vfs);
  let error: String = search
    .get_vfs()
    .commit(&XrayTransaction::new(&search).with_write(SYSTEM_LTX, "edited").unwrap())
    .expect_err("nothing can hold the write")
    .to_string();

  assert!(error.contains("no writable loose mount"), "{error}");
}

#[test]
fn refuses_to_delete_an_archived_path() {
  let root: PathBuf = directory("commit_delete_archived", &["configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");
  vfs
    .mount("", Box::new(FakeArchiveSource::new("configs", &[SYSTEM_LTX])))
    .expect("the archive mounts");

  let search: XraySearch = search(vfs);

  assert!(
    search
      .get_vfs()
      .commit(&XrayTransaction::new(&search).with_delete(SYSTEM_LTX).unwrap())
      .is_err()
  );
}

#[test]
fn refuses_a_write_outside_the_writable_mount_base() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory("configs\\weapons", directory("commit_base", &["ak74.ltx"]))
    .expect("the subtree mounts");

  let search: XraySearch = search(vfs);

  assert!(
    search
      .get_vfs()
      .commit(
        &XrayTransaction::new(&search)
          .with_write("textures\\a.dds", "bytes")
          .unwrap()
      )
      .is_err()
  );
}

#[test]
fn refuses_a_transaction_whose_reads_moved() {
  let root: PathBuf = directory("commit_stale", &["configs/system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");

  let record: XrayInputRecord = XrayInputRecord::new();
  let before: XraySearch = search(vfs).with_recording(&record);

  before.read(SYSTEM_LTX).expect("the author reads the file");

  let (after, _) = commit(
    &before,
    XrayTransaction::new(&before)
      .with_write(SYSTEM_LTX, "someone else")
      .unwrap(),
  );

  let error: String = after
    .get_vfs()
    .commit(
      &XrayTransaction::new(&before)
        .with_write(SYSTEM_LTX, "the author")
        .unwrap()
        .with_expected(&record),
    )
    .expect_err("what the author read moved")
    .to_string();

  assert!(error.contains("stale"), "{error}");
  assert_eq!(text(&after, SYSTEM_LTX), "someone else");
}

#[test]
fn expects_only_what_a_filtered_record_keeps() {
  let root: PathBuf = directory("commit_expected_filtered", &["configs/system.ltx", "configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");

  let record: XrayInputRecord = XrayInputRecord::new();
  let before: XraySearch = search(vfs).with_recording(&record);

  before.read(SYSTEM_LTX).expect("the author reads the file it writes");
  before.read("configs\\game.ltx").expect("and another beside it");

  let (after, _) = commit(
    &before,
    XrayTransaction::new(&before)
      .with_write("configs\\game.ltx", "someone else")
      .unwrap(),
  );
  let written: XrayLogicalPath = XrayLogicalPath::new(SYSTEM_LTX).unwrap();
  let expected: XrayInputRecord =
    record.filtered(|input| matches!(input.get_kind(), XrayInputKind::File(path) if *path == written));

  assert_eq!(expected.len(), 1);

  after
    .get_vfs()
    .commit(
      &XrayTransaction::new(&before)
        .with_write(SYSTEM_LTX, "the author")
        .unwrap()
        .with_expected(&expected),
    )
    .expect("what the author writes did not move, whatever else did");
  assert!(
    !record.is_current_in(after.get_vfs()),
    "the whole record would have refused it"
  );
}

#[test]
fn applies_a_file_written_behind_the_snapshot() {
  let root: PathBuf = directory("apply_created", &["configs/system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", &root).expect("the tree mounts");

  fs::write(root.join("configs/weather.ltx"), b"weather").expect("a file written outside the VFS");

  let before: XraySearch = search(vfs);

  assert!(before.find("configs\\weather.ltx").unwrap().is_none());

  let (next, changes): (XrayVfs, XrayChangeSet) = before
    .get_vfs()
    .apply(vec![XrayChange::created(id, "configs\\weather.ltx").unwrap()])
    .expect("the change applies");

  assert_eq!(text(&search(next), "configs\\weather.ltx"), "weather");
  assert_eq!(changes.get_revision(), XrayRevision::MOUNTED.next());
}

#[test]
fn an_empty_transaction_moves_no_revision() {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount_directory("", directory("commit_empty", &["configs/system.ltx"]))
    .expect("the tree mounts");

  let search: XraySearch = search(vfs);
  let (_, changes): (XraySearch, XrayChangeSet) = commit(&search, XrayTransaction::new(&search));

  assert!(changes.is_empty());
  assert_eq!(changes.get_revision(), XrayRevision::MOUNTED);
}

#[test]
fn refuses_to_apply_a_change_to_an_archive() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount("", Box::new(FakeArchiveSource::new("configs", &[SYSTEM_LTX])))
    .expect("the archive mounts");

  assert!(
    vfs
      .apply(vec![XrayChange::created(id, "configs\\game.ltx").unwrap()])
      .is_err()
  );
}

#[test]
fn a_change_keeps_what_was_retained_of_every_other_file() {
  let root: PathBuf = directory("commit_retention", &["configs/system.ltx", "configs/game.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::configs());

  vfs.mount_directory("", &root).expect("the tree mounts");

  let before: XraySearch = search(vfs);
  let parses: AtomicUsize = AtomicUsize::new(0);
  let parse = |search: &XraySearch, path: &str| -> Arc<String> {
    let asset: XrayAsset = search.find(path).unwrap().expect("the path resolves");

    search
      .read_parsed(&asset, |bytes| {
        parses.fetch_add(1, Ordering::Relaxed);

        Ok(String::from_utf8_lossy(&bytes).into_owned())
      })
      .expect("the path parses")
  };

  parse(&before, SYSTEM_LTX);
  parse(&before, "configs\\game.ltx");

  let (after, _) = commit(
    &before,
    XrayTransaction::new(&before)
      .with_write("configs\\game.ltx", "edited")
      .unwrap()
      .with_write("configs\\weather.ltx", "created")
      .unwrap(),
  );

  assert_eq!(*parse(&after, SYSTEM_LTX), "commit_retention");
  assert_eq!(parses.load(Ordering::Relaxed), 2, "the unchanged file is served");
  assert_eq!(*parse(&after, "configs\\game.ltx"), "edited");
  assert_eq!(parses.load(Ordering::Relaxed), 3, "the changed one is parsed again");
}

#[test]
fn a_host_commit_publishes_the_next_snapshot() {
  let root: PathBuf = directory("host_commit", &["configs/system.ltx"]);
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");

  let host: XrayVfsHost = XrayVfsHost::new(vfs);
  let before: XraySearch = search_in(&host);

  let changes: XrayChangeSet = host
    .commit(
      &XrayTransaction::new(&before)
        .with_write("configs\\game.ltx", "game")
        .unwrap(),
    )
    .expect("the host commits");

  assert_eq!(host.get_vfs().get_revision(), changes.get_revision());
  assert_eq!(text(&search_in(&host), "configs\\game.ltx"), "game");
  assert!(
    before.find("configs\\game.ltx").unwrap().is_none(),
    "a search keeps its snapshot"
  );
}

fn search_in(host: &XrayVfsHost) -> XraySearch {
  XraySearch::new(host.get_vfs()).with_step("all", XrayLookupScope::all())
}
