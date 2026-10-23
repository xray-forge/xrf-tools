use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use rayon::prelude::*;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{
  XrayInputKind, XrayInputRecord, XrayLookupScope, XrayMountMode, XrayRevision, XrayRoots, XraySearch, XrayTransaction,
  XrayVfs,
};

use crate::core::assets::AssetMountState;
use crate::core::session::{
  Session, SessionChange, SessionDerived, SessionId, SessionPart, SessionParts, SessionRestore, SessionRootChange,
  SessionSnapshot,
};

#[test]
fn a_failed_replacement_keeps_the_committed_value_readable() {
  let session: Session<String> = Session::new("test");
  let first: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.commit_open(first, String::from("first")).unwrap();
  session.begin_open(SessionId::new()).unwrap();

  assert_eq!(session.require(first).unwrap().as_str(), "first");
}

#[test]
fn a_late_open_cannot_replace_a_newer_open_or_undo_its_close() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.begin_open(second).unwrap();
  session.commit_open(second, "second").unwrap();

  assert!(session.commit_open(first, "first").is_err());

  session.close(&[second]).unwrap();

  assert!(session.commit_open(first, "first").is_err());
  assert!(session.get().unwrap().is_none());
}

#[test]
fn closing_a_pending_open_prevents_its_publication() {
  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();
  session.close(&[id]).unwrap();

  assert!(session.commit_open(id, "late").is_err());
}

#[test]
fn an_old_teardown_cannot_close_a_new_owners_session_or_opening() {
  let session: Session<&str> = Session::new("test");
  let old: SessionId = SessionId::new();
  let new: SessionId = SessionId::new();

  session.begin_open(old).unwrap();
  session.commit_open(old, "old").unwrap();
  session.begin_open(new).unwrap();
  session.close(&[old]).unwrap();
  session.commit_open(new, "new").unwrap();
  session.close(&[old]).unwrap();

  assert_eq!(**session.require(new).unwrap(), "new");
  assert!(session.require(old).is_err());
}

#[test]
fn an_accepted_snapshot_outlives_close() {
  let session: Session<String> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();

  let held: Arc<SessionSnapshot<String>> = session.commit_open(id, String::from("before")).unwrap();

  session.close(&[id]).unwrap();

  assert_eq!(held.as_str(), "before");
  assert!(session.get().unwrap().is_none());
}

#[test]
fn restoration_and_open_serialize_the_same_identity_revision_and_value() {
  let session: Session<Vec<u8>> = Session::new("wire test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();

  let opened = session.commit_open(id, vec![1, 2, 3]).unwrap();
  let restored = SessionRestore::from(session.get().unwrap());
  let expected = serde_json::json!({ "sessionId": id, "revision": 0, "value": [1, 2, 3] });

  assert_eq!(serde_json::to_value(&opened).unwrap(), expected);
  assert_eq!(serde_json::to_value(&restored).unwrap(), expected);

  session.close(&[id]).unwrap();

  assert_eq!(
    serde_json::to_value(SessionRestore::from(session.get().unwrap())).unwrap(),
    serde_json::Value::Null
  );
}

#[test]
fn reloading_checks_the_previous_identity_before_reserving_publication() {
  let session: Session<&str> = Session::new("reload test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.commit_open(first, "first").unwrap();
  session.begin_reload(second, first).unwrap();

  assert_eq!(**session.require(first).unwrap(), "first");
  assert!(session.begin_reload(SessionId::new(), SessionId::new()).is_err());

  session.commit_open(second, "reloaded").unwrap();

  assert!(session.require(first).is_err());
}

// Long work asks before it starts as well as before it publishes: a sector pack that has queued behind others is
// usually answering a question nobody is asking any more, and packing anyway is how a queue grows faster than it
// drains.
#[test]
fn a_superseded_opening_can_be_told_before_its_work_is_done() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  session.begin_open(first).unwrap();

  assert!(session.require_opening(first).is_ok());

  session.begin_open(second).unwrap();

  assert!(session.require_opening(first).is_err());
  assert!(session.require_opening(second).is_ok());
}

#[test]
fn an_opening_that_was_closed_or_published_is_no_longer_the_one_to_do_work_for() {
  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();
  session.commit_open(id, "value").unwrap();

  assert!(
    session.require_opening(id).is_err(),
    "a published opening is finished, not pending"
  );

  let closed: SessionId = SessionId::new();

  session.begin_open(closed).unwrap();
  session.close(&[closed]).unwrap();

  assert!(session.require_opening(closed).is_err());
}

#[test]
fn the_early_check_and_the_commit_refuse_a_superseded_opening_the_same_way() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.begin_open(SessionId::new()).unwrap();

  let early: String = session.require_opening(first).unwrap_err();
  let late: String = session.commit_open(first, "first").unwrap_err();

  assert_eq!(early, late, "a caller cannot tell the two apart and should not have to");
}

/// A part each test builds, or keeps, by its own count, behind a plain lock.
struct CountedPart;

impl SessionPart for CountedPart {
  type Value = Arc<u32>;

  const NAME: &'static str = "counted";

  const IS_POOLED: bool = false;
}

/// A pooled part asking for another as it builds.
struct DependentPart;

impl SessionPart for DependentPart {
  type Value = u32;

  const NAME: &'static str = "dependent";
}

/// A part built over Rayon while the pool's workers wait on it, pooled as parts are by default.
struct PooledPart;

impl SessionPart for PooledPart {
  type Value = u64;

  const NAME: &'static str = "pooled";
}

#[test]
fn a_part_is_built_once_and_shared_by_every_ask() {
  let parts: SessionParts = SessionParts::new();
  let builds: AtomicU32 = AtomicU32::new(0);

  assert_eq!(
    parts.get::<CountedPart>(),
    None,
    "asking whether it was built builds nothing"
  );

  let first: Arc<u32> = parts.get_or_init::<CountedPart>(|| Arc::new(builds.fetch_add(1, Ordering::SeqCst) + 7));
  let second: Arc<u32> = parts.get_or_init::<CountedPart>(|| Arc::new(builds.fetch_add(1, Ordering::SeqCst)));

  assert!(Arc::ptr_eq(&first, &second));
  assert_eq!(builds.load(Ordering::SeqCst), 1);
  assert_eq!(parts.get::<CountedPart>().as_deref(), Some(&7));
}

#[test]
fn a_part_the_opening_read_is_kept_rather_than_built() {
  let parts: SessionParts = SessionParts::new()
    .with::<CountedPart>(Arc::new(3), XrayInputRecord::new())
    .with::<PooledPart>(5, XrayInputRecord::new());

  assert_eq!(
    *parts.get_or_init::<CountedPart>(|| unreachable!("the part was kept")),
    3
  );
  assert_eq!(parts.get::<PooledPart>(), Some(5));
  assert_eq!(
    parts.get_or_init::<PooledPart>(|| unreachable!("the pooled part was kept")),
    5
  );
}

#[test]
fn a_part_may_ask_for_another_as_it_builds() {
  let parts: SessionParts = SessionParts::new();
  let value: u32 = parts.get_or_init::<DependentPart>(|| *parts.get_or_init::<CountedPart>(|| Arc::new(2)) * 10);

  assert_eq!(value, 20);
}

#[test]
fn a_pooled_part_built_over_rayon_does_not_deadlock_the_workers_waiting_on_it() {
  let parts: SessionParts = SessionParts::new();
  let sums: Vec<u64> = (0..64)
    .into_par_iter()
    .map(|_| parts.get_or_init::<PooledPart>(|| (0..10_000u64).into_par_iter().sum()))
    .collect();

  assert!(sums.iter().all(|sum| *sum == 49_995_000));
}

/// A loose tree of `files` mounted as the whole of a VFS.
fn mounted(name: &str, files: &[&str]) -> XrayVfs {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("session_parts/{name}"));

  let _ = fs::remove_dir_all(&root);

  for file in files {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("directory created");
    fs::write(&path, b"payload").expect("file written");
  }

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("the tree mounts");

  vfs
}

fn search_of(vfs: XrayVfs) -> XraySearch {
  XraySearch::new(vfs).with_step("root", XrayLookupScope::all())
}

/// Finds a file, as a build reading it would.
fn read(search: &XraySearch, logical_path: &str) {
  search.find(logical_path).expect("the lookup succeeds");
}

/// The files a record holds, sorted, each with whether a source answered for it.
fn list_files(inputs: &XrayInputRecord) -> Vec<(String, bool)> {
  let mut files: Vec<(String, bool)> = inputs
    .list()
    .into_iter()
    .filter_map(|(input, answer)| match input.get_kind() {
      XrayInputKind::File(path) => Some((path.as_str().to_string(), answer.source.is_some())),
      XrayInputKind::Listing(_) => None,
    })
    .collect();

  files.sort();
  files
}

fn files(paths: &[(&str, bool)]) -> Vec<(String, bool)> {
  paths
    .iter()
    .map(|(path, is_found)| (path.to_string(), *is_found))
    .collect()
}

#[test]
fn a_part_records_what_its_build_reads() {
  let vfs: XrayVfs = mounted("records", &["configs/system.ltx"]);
  let parts: SessionParts = SessionParts::new();

  parts.get_or_record::<CountedPart>(&search_of(vfs), |search| {
    read(search, "configs\\system.ltx");
    read(search, "configs\\absent.ltx");

    Arc::new(1)
  });

  assert_eq!(
    list_files(&parts.get_inputs::<CountedPart>().expect("the part was asked for")),
    files(&[("configs\\absent.ltx", false), ("configs\\system.ltx", true)])
  );
}

#[test]
fn a_part_asked_for_as_another_builds_adds_its_inputs_to_the_askers() {
  let vfs: XrayVfs = mounted("nested", &["configs/system.ltx", "configs/game.ltx"]);
  let search: XraySearch = search_of(vfs);
  let build_counted = |search: &XraySearch| {
    read(search, "configs\\game.ltx");

    Arc::new(2)
  };

  for is_built_first in [false, true] {
    let parts: SessionParts = SessionParts::new();

    if is_built_first {
      parts.get_or_record::<CountedPart>(&search, build_counted);
    }

    let value: u32 = parts.get_or_record::<DependentPart>(&search, |search| {
      read(search, "configs\\system.ltx");

      *parts.get_or_record::<CountedPart>(search, build_counted) * 10
    });

    assert_eq!(value, 20);
    assert_eq!(
      list_files(&parts.get_inputs::<DependentPart>().expect("the asker was asked for")),
      files(&[("configs\\game.ltx", true), ("configs\\system.ltx", true)]),
      "the asker read the part built {}",
      if is_built_first { "before" } else { "for it" }
    );
    assert_eq!(
      list_files(&parts.get_inputs::<CountedPart>().expect("the part was asked for")),
      files(&[("configs\\game.ltx", true)])
    );
  }
}

#[test]
fn a_pooled_part_records_what_its_build_reads() {
  let vfs: XrayVfs = mounted("pooled", &["configs/system.ltx"]);
  let search: XraySearch = search_of(vfs);
  let parts: SessionParts = SessionParts::new();
  let sums: Vec<u64> = (0..64)
    .into_par_iter()
    .map(|_| {
      parts.get_or_record::<PooledPart>(&search, |search| {
        read(search, "configs\\system.ltx");

        (0..10_000u64).into_par_iter().sum()
      })
    })
    .collect();

  assert!(sums.iter().all(|sum| *sum == 49_995_000));
  assert_eq!(
    list_files(&parts.get_inputs::<PooledPart>().expect("the part was asked for")),
    files(&[("configs\\system.ltx", true)])
  );
}

#[test]
fn a_part_read_a_key_at_a_time_records_each_read_as_its_own() {
  let vfs: XrayVfs = mounted("keyed", &["meshes/a.ogf", "meshes/b.ogf"]);
  let search: XraySearch = search_of(vfs);
  let parts: SessionParts = SessionParts::new();

  parts.get_or_init::<CountedPart>(|| Arc::new(0));
  parts.record::<CountedPart, _>(&search, |search| read(search, "meshes\\a.ogf"));
  parts.record::<CountedPart, _>(&search, |search| read(search, "meshes\\b.ogf"));

  assert_eq!(
    list_files(&parts.get_inputs::<CountedPart>().expect("the part was asked for")),
    files(&[("meshes\\a.ogf", true), ("meshes\\b.ogf", true)])
  );
}

#[test]
fn a_snapshot_keeps_what_its_value_and_its_kept_parts_were_read_from() {
  let vfs: XrayVfs = mounted("kept", &["levels/l01/level", "spawns/all.spawn"]);
  let root: XrayInputRecord = XrayInputRecord::new();
  let spawn: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search_of(vfs).with_recording(&root);

  read(&search, "levels\\l01\\level");
  read(&search.clone().with_recording(&spawn), "spawns\\all.spawn");
  search.record_inputs(&spawn);

  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();

  let opened: Arc<SessionSnapshot<&str>> = session
    .commit_open_with(
      id,
      "level",
      root,
      SessionParts::new().with::<CountedPart>(Arc::new(1), spawn),
    )
    .unwrap();

  assert_eq!(
    list_files(&opened.inputs),
    files(&[("levels\\l01\\level", true), ("spawns\\all.spawn", true)])
  );
  assert_eq!(
    list_files(&opened.parts.get_inputs::<CountedPart>().expect("the part was kept")),
    files(&[("spawns\\all.spawn", true)])
  );
  assert_eq!(
    opened.parts.list_inputs().len(),
    1,
    "a kept part is listed with its inputs"
  );
}

/// A loose tree of `files` behind an asset state, with the search an open over it takes.
fn opened_tree(name: &str, files: &[&str]) -> (AssetMountState, XraySearch) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("session_follow/{name}"));

  let _ = fs::remove_dir_all(&root);

  for file in files {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("directory created");
    fs::write(&path, b"payload").expect("file written");
  }

  let assets: AssetMountState = AssetMountState::new();
  let search: XraySearch = assets
    .search(&XrayRoots::one(root, XrayMountMode::Directory))
    .expect("the tree mounts");

  (assets, search)
}

/// Opens `value` in `session` as an open reading `paths` through `search` would.
fn open_reading(
  session: &Session<&'static str>,
  search: &XraySearch,
  value: &'static str,
  paths: &[&str],
) -> Arc<SessionSnapshot<&'static str>> {
  let inputs: XrayInputRecord = XrayInputRecord::new();
  let recording: XraySearch = search.clone().with_recording(&inputs);

  for path in paths {
    read(&recording, path);
  }

  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();
  session
    .commit_open_with(id, value, inputs, SessionParts::new())
    .unwrap()
}

/// Commits new bytes for one file, as a save does.
fn write(assets: &AssetMountState, search: &XraySearch, path: &str) {
  assets
    .commit(&XrayTransaction::new(search).with_write(path, "edited").unwrap())
    .expect("the write commits");
}

#[test]
fn a_commit_moves_a_session_along_sharing_its_value_and_the_parts_it_did_not_touch() {
  let (assets, search) = opened_tree(
    "kept",
    &["configs/system.ltx", "configs/game.ltx", "configs/weather.ltx"],
  );
  let session: Session<&str> = Session::new("test");
  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "level", &["configs\\system.ltx"]);

  opened.parts.get_or_record::<CountedPart>(&search, |search| {
    read(search, "configs\\system.ltx");

    Arc::new(1)
  });
  opened.parts.get_or_record::<PooledPart>(&search, |search| {
    read(search, "configs\\game.ltx");

    2
  });
  // Asked for, never built: what it would read is not known, so it is neither carried nor reported.
  opened
    .parts
    .record::<DependentPart, _>(&search, |search| read(search, "configs\\weather.ltx"));

  write(&assets, &search, "configs\\game.ltx");

  let change: SessionChange = session.follow(&assets).unwrap().expect("the session moved");
  let next: Arc<SessionSnapshot<&str>> = session.get().unwrap().unwrap();

  assert_eq!(
    change,
    SessionChange {
      session_id: opened.session_id,
      revision: assets.get_vfs().get_revision(),
      root: SessionRootChange::Kept,
      stale_parts: vec!["pooled"],
    }
  );
  assert!(
    Arc::ptr_eq(&next.value, &opened.value),
    "the value is shared, not copied"
  );
  assert_eq!(next.revision, change.revision);
  assert_eq!(next.parts.get::<CountedPart>().as_deref(), Some(&1));
  assert_eq!(next.parts.get::<PooledPart>(), None);
  assert!(next.parts.get_inputs::<DependentPart>().is_none());
  assert!(
    session.follow(&assets).unwrap().is_none(),
    "a session at the project's revision stays where it is"
  );
}

#[test]
fn a_commit_to_what_the_value_read_derives_it_again_starting_without_parts() {
  let (assets, search) = opened_tree("derived", &["configs/system.ltx"]);
  let session: Session<&str> =
    Session::new("test").with_derivation(|_, _| Ok(SessionDerived::new("derived", XrayInputRecord::new())));
  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "opened", &["configs\\system.ltx"]);

  opened.parts.get_or_init::<CountedPart>(|| Arc::new(1));

  write(&assets, &search, "configs\\system.ltx");

  let change: SessionChange = session.follow(&assets).unwrap().expect("the session moved");
  let next: Arc<SessionSnapshot<&str>> = session.get().unwrap().unwrap();

  assert_eq!(change.root, SessionRootChange::Derived);
  assert_eq!(change.stale_parts, vec!["counted"]);
  assert_eq!(**next, "derived");
  assert_eq!(next.session_id, opened.session_id);
  assert_eq!(next.revision, change.revision);
  assert_eq!(next.parts.get::<CountedPart>(), None);
}

#[test]
fn a_moved_value_nothing_derives_is_kept_and_reported_stale() {
  let (assets, search) = opened_tree("stale", &["configs/system.ltx"]);
  let session: Session<&str> = Session::new("test");
  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "opened", &["configs\\system.ltx"]);

  write(&assets, &search, "configs\\system.ltx");

  let change: SessionChange = session.follow(&assets).unwrap().expect("the session is reported");

  assert_eq!(change.root, SessionRootChange::Stale);
  assert_eq!(change.revision, XrayRevision::MOUNTED, "it stays at its own revision");
  assert!(Arc::ptr_eq(&session.get().unwrap().unwrap(), &opened));
}

#[test]
fn a_failed_derivation_is_reported_and_leaves_the_snapshot_as_it_was() {
  let (assets, search) = opened_tree("failed", &["configs/system.ltx"]);
  let session: Session<&str> = Session::new("test").with_derivation(|_, _| Err(String::from("unreadable")));
  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "opened", &["configs\\system.ltx"]);

  write(&assets, &search, "configs\\system.ltx");

  assert_eq!(
    session.follow(&assets).unwrap().expect("the failure is reported").root,
    SessionRootChange::Failed {
      reason: String::from("unreadable")
    }
  );
  assert!(Arc::ptr_eq(&session.get().unwrap().unwrap(), &opened));
}

#[test]
fn a_commit_moves_every_followed_session_before_it_returns() {
  let (assets, search) = opened_tree("followed", &["configs/system.ltx", "configs/game.ltx"]);
  let followed: Session<&str> = Session::new("followed");
  let unfollowed: Session<&str> = Session::new("unfollowed");
  let closed: Session<&str> = Session::new("closed");

  assets.follow(followed.clone());
  assets.follow(closed.clone());

  open_reading(&followed, &search, "followed", &["configs\\system.ltx"]);
  open_reading(&unfollowed, &search, "unfollowed", &["configs\\system.ltx"]);

  write(&assets, &search, "configs\\game.ltx");

  assert_eq!(
    followed.get().unwrap().unwrap().revision,
    assets.get_vfs().get_revision()
  );
  assert_eq!(unfollowed.get().unwrap().unwrap().revision, XrayRevision::MOUNTED);
  assert!(
    closed.get().unwrap().is_none(),
    "a session with nothing open stays empty"
  );
}
