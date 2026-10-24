use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rayon::prelude::*;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{
  XrayAsset, XrayInputKind, XrayInputRecord, XrayLookupScope, XrayMountMode, XrayRevision, XrayRoots, XraySearch,
  XrayTransaction, XrayVfs,
};

use crate::assets::{AssetChangeEvent, AssetChangeSubscriber, AssetMountState};
use crate::session::{
  Session, SessionChange, SessionDerived, SessionId, SessionPart, SessionParts, SessionRestore, SessionRootChange,
  SessionScopes, SessionSnapshot, Sessions,
};

#[test]
fn a_failed_replacement_keeps_the_committed_value_readable() {
  let session: Session<String> = Session::new("test");
  let first: SessionId = SessionId::generate();

  session.begin_open(first).unwrap();
  session.commit_open(first, String::from("first")).unwrap();
  session.begin_open(SessionId::generate()).unwrap();

  assert_eq!(session.require(first).unwrap().as_str(), "first");
}

#[test]
fn a_late_open_cannot_replace_a_newer_open_or_undo_its_close() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::generate();
  let second: SessionId = SessionId::generate();

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
  let id: SessionId = SessionId::generate();

  session.begin_open(id).unwrap();
  session.close(&[id]).unwrap();

  assert!(session.commit_open(id, "late").is_err());
}

#[test]
fn an_old_teardown_cannot_close_a_new_owners_session_or_opening() {
  let session: Session<&str> = Session::new("test");
  let old: SessionId = SessionId::generate();
  let new: SessionId = SessionId::generate();

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
  let id: SessionId = SessionId::generate();

  session.begin_open(id).unwrap();

  let held: Arc<SessionSnapshot<String>> = session.commit_open(id, String::from("before")).unwrap();

  session.close(&[id]).unwrap();

  assert_eq!(held.as_str(), "before");
  assert!(session.get().unwrap().is_none());
}

#[test]
fn restoration_and_open_serialize_the_same_identity_revision_and_value() {
  let session: Session<Vec<u8>> = Session::new("wire test");
  let id: SessionId = SessionId::generate();

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
  let first: SessionId = SessionId::generate();
  let second: SessionId = SessionId::generate();

  session.begin_open(first).unwrap();
  session.commit_open(first, "first").unwrap();
  session.begin_reload(second, first).unwrap();

  assert_eq!(**session.require(first).unwrap(), "first");
  assert!(
    session
      .begin_reload(SessionId::generate(), SessionId::generate())
      .is_err()
  );

  session.commit_open(second, "reloaded").unwrap();

  assert!(session.require(first).is_err());
}

// Long work asks before it starts as well as before it publishes: a sector pack that has queued behind others is
// usually answering a question nobody is asking any more, and packing anyway is how a queue grows faster than it
// drains.
#[test]
fn a_superseded_opening_can_be_told_before_its_work_is_done() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::generate();
  let second: SessionId = SessionId::generate();

  session.begin_open(first).unwrap();

  assert!(session.require_opening(first).is_ok());

  session.begin_open(second).unwrap();

  assert!(session.require_opening(first).is_err());
  assert!(session.require_opening(second).is_ok());
}

#[test]
fn an_opening_that_was_closed_or_published_is_no_longer_the_one_to_do_work_for() {
  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::generate();

  session.begin_open(id).unwrap();
  session.commit_open(id, "value").unwrap();

  assert!(
    session.require_opening(id).is_err(),
    "a published opening is finished, not pending"
  );

  let closed: SessionId = SessionId::generate();

  session.begin_open(closed).unwrap();
  session.close(&[closed]).unwrap();

  assert!(session.require_opening(closed).is_err());
}

#[test]
fn the_early_check_and_the_commit_refuse_a_superseded_opening_the_same_way() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::generate();

  session.begin_open(first).unwrap();
  session.begin_open(SessionId::generate()).unwrap();

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
  let id: SessionId = SessionId::generate();

  session.begin_open(id).unwrap();

  let opened: Arc<SessionSnapshot<&str>> = session
    .commit_open_with(
      id,
      "level",
      XrayRevision::MOUNTED,
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
  let (assets, _, search) = opened_roots(name, files);

  (assets, search)
}

/// The same, with the tree's root.
fn opened_roots(name: &str, files: &[&str]) -> (AssetMountState, PathBuf, XraySearch) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("session_follow/{name}"));

  let _ = fs::remove_dir_all(&root);

  for file in files {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("directory created");
    fs::write(&path, b"payload").expect("file written");
  }

  let assets: AssetMountState = AssetMountState::new();
  let search: XraySearch = assets.search(&roots_at(&root)).expect("the tree mounts");

  (assets, root, search)
}

fn roots_at(root: &Path) -> XrayRoots {
  XrayRoots::one(root.to_path_buf(), XrayMountMode::Directory)
}

/// Parses a config through `search`, counting the parses that ran.
fn parse_counted(search: &XraySearch, logical_path: &str, parses: &AtomicU32) -> String {
  let asset: XrayAsset = search
    .find(logical_path)
    .expect("the lookup succeeds")
    .expect("the config is there");

  search
    .read_parsed(&asset, |bytes| {
      parses.fetch_add(1, Ordering::Relaxed);

      Ok(String::from_utf8_lossy(&bytes).into_owned())
    })
    .expect("the config parses")
    .to_string()
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

  let id: SessionId = SessionId::generate();

  session.begin_open(id).unwrap();
  session
    .commit_open_with(id, value, search.get_vfs().get_revision(), inputs, SessionParts::new())
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
  // Asked for, never built: what it would read is not known, so it is not carried, and is named for whoever waits on it.
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
      stale_parts: vec!["dependent", "pooled"],
      is_view_renewed: false,
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
fn a_kept_change_renews_the_view_only_where_it_lets_a_part_the_view_is_built_from_go() {
  for (view_parts, is_renewed) in [(&["pooled"][..], true), (&["counted"][..], false)] {
    let (assets, search) = opened_tree("view", &["configs/system.ltx", "configs/game.ltx"]);
    let session: Session<&str> = Session::new("test").with_view_parts(view_parts);
    let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "level", &["configs\\system.ltx"]);

    opened.parts.get_or_init::<CountedPart>(|| Arc::new(1));
    opened.parts.get_or_record::<PooledPart>(&search, |search| {
      read(search, "configs\\game.ltx");

      2
    });

    write(&assets, &search, "configs\\game.ltx");

    let change: SessionChange = session.follow(&assets).unwrap().expect("the session moved");

    assert_eq!(change.root, SessionRootChange::Kept);
    assert_eq!(change.stale_parts, vec!["pooled"]);
    assert_eq!(change.is_view_renewed, is_renewed, "view parts {view_parts:?}");
  }
}

#[test]
fn an_answer_read_from_a_moved_snapshot_carries_the_revision_it_was_read_at() {
  let (assets, search) = opened_tree("answered", &["configs/system.ltx", "configs/game.ltx"]);
  let session: Session<&str> = Session::new("test");
  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "level", &["configs\\system.ltx"]);

  write(&assets, &search, "configs\\game.ltx");
  session.follow(&assets).unwrap().expect("the session moved");

  let next: Arc<SessionSnapshot<&str>> = session.get().unwrap().unwrap();

  assert_ne!(next.revision, XrayRevision::MOUNTED);
  assert_eq!(
    serde_json::to_value(next.answer(next.len())).unwrap(),
    serde_json::json!({ "sessionId": opened.session_id, "revision": next.revision, "value": 5 })
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
  assert!(change.is_view_renewed, "a value derived again renews any view of it");
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

  write(&assets, &search, "configs\\system.ltx");

  assert!(
    session.follow(&assets).unwrap().is_none(),
    "a snapshot reported stale is not reported again"
  );

  let reopened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "reopened", &["configs\\system.ltx"]);

  write(&assets, &search, "configs\\system.ltx");

  assert_eq!(
    session.follow(&assets).unwrap().map(|change| change.session_id),
    Some(reopened.session_id),
    "the snapshot replacing it is followed afresh"
  );
}

#[test]
fn a_carried_part_records_nothing_its_old_snapshot_reads_afterwards() {
  let (assets, search) = opened_tree("carried_record", &["configs/system.ltx", "configs/game.ltx"]);
  let session: Session<&str> = Session::new("test");
  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "opened", &[]);

  opened.parts.get_or_init::<CountedPart>(|| Arc::new(1));
  write(&assets, &search, "configs\\game.ltx");
  session.follow(&assets).unwrap().expect("the session moved");

  // A reader still on the old snapshot reads on into its part.
  opened
    .parts
    .record::<CountedPart, _>(&search, |search| read(search, "configs\\system.ltx"));

  let next: Arc<SessionSnapshot<&str>> = session.get().unwrap().unwrap();

  assert_eq!(
    next.parts.get::<CountedPart>().as_deref(),
    Some(&1),
    "the part is carried"
  );
  assert!(
    next
      .parts
      .get_inputs::<CountedPart>()
      .expect("the part is carried")
      .is_empty()
  );
}

#[test]
fn an_open_published_after_a_commit_it_did_not_read_catches_up() {
  let (assets, search) = opened_tree("caught_up", &["configs/system.ltx", "configs/game.ltx"]);
  let session: Session<&str> = Session::new("test");
  let log: ChangeLog = ChangeLog::default();

  assets.add_follower(session.clone());
  assets.add_subscriber(log.clone());

  // Read before the commit, published after its fan-out found nothing open.
  let inputs: XrayInputRecord = XrayInputRecord::new();

  read(&search.clone().with_recording(&inputs), "configs\\system.ltx");
  write(&assets, &search, "configs\\game.ltx");

  let id: SessionId = SessionId::generate();

  session.begin_open(id).unwrap();

  let opened: Arc<SessionSnapshot<&str>> = session
    .commit_open_with(
      id,
      "opened",
      search.get_vfs().get_revision(),
      inputs,
      SessionParts::new(),
    )
    .unwrap();

  assert_eq!(opened.revision, XrayRevision::MOUNTED, "stamped with what it read");

  let caught_up: Arc<SessionSnapshot<&str>> = session.catch_up(opened, &assets);

  assert_eq!(caught_up.revision, assets.get_vfs().get_revision());
  assert_eq!(
    log.list(),
    vec![SessionChange {
      session_id: id,
      revision: caught_up.revision,
      root: SessionRootChange::Kept,
      stale_parts: Vec::new(),
      is_view_renewed: false,
    }]
  );
  assert!(
    Arc::ptr_eq(&session.catch_up(Arc::clone(&caught_up), &assets), &caught_up),
    "a current open catches up with nothing"
  );
}

// Commits racing each other are heard in the order they were published, each event naming the sessions it moved at its
// own revision.
#[test]
fn concurrent_commits_are_heard_in_revision_order_with_their_own_sessions() {
  let files: Vec<String> = (0..8).map(|index| format!("configs/file_{index}.ltx")).collect();
  let names: Vec<&str> = files.iter().map(String::as_str).collect();
  let (assets, search) = opened_tree("concurrent", &names);
  let session: Session<&str> = Session::new("test");
  let log: ChangeLog = ChangeLog::default();

  assets.add_follower(session.clone());
  assets.add_subscriber(log.clone());
  open_reading(&session, &search, "opened", &[]);

  thread::scope(|scope| {
    for index in 0..8 {
      let (assets, search) = (&assets, &search);

      scope.spawn(move || write(assets, search, &format!("configs\\file_{index}.ltx")));
    }
  });

  let events: Vec<AssetChangeEvent> = log.taken.lock().unwrap().clone();
  let revisions: Vec<XrayRevision> = events.iter().map(|event| event.revision).collect();
  let mut sorted: Vec<XrayRevision> = revisions.clone();

  sorted.sort();

  assert_eq!(events.len(), 8);
  assert_eq!(revisions, sorted, "heard in the order published");
  assert!(
    events
      .iter()
      .flat_map(|event| event
        .sessions
        .iter()
        .map(move |change| (event.revision, change.revision)))
      .all(|(event, session)| event == session),
    "every session an event names moved to that event's revision"
  );
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
fn a_fresh_search_reads_a_file_changed_behind_the_app_again_and_moves_its_sessions() {
  let (assets, root, search) = opened_roots("changed_on_disk", &["configs/system.ltx", "configs/game.ltx"]);
  let session: Session<&str> =
    Session::new("test").with_derivation(|_, _| Ok(SessionDerived::new("derived", XrayInputRecord::new())));
  let parses: AtomicU32 = AtomicU32::new(0);

  assets.add_follower(session.clone());
  open_reading(&session, &search, "opened", &["configs\\system.ltx"]);
  parse_counted(&search, "configs\\system.ltx", &parses);
  parse_counted(&search, "configs\\game.ltx", &parses);

  let unchanged: XraySearch = assets.search_fresh(&roots_at(&root)).expect("the roots open afresh");

  assert_eq!(unchanged.get_vfs().get_revision(), XrayRevision::MOUNTED);
  assert_eq!(
    **session.get().unwrap().unwrap(),
    "opened",
    "nothing changed moves nothing"
  );

  fs::write(root.join("configs/system.ltx"), b"edited behind the app").expect("file rewritten");

  let fresh: XraySearch = assets.search_fresh(&roots_at(&root)).expect("the roots open afresh");
  let next: Arc<SessionSnapshot<&str>> = session.get().unwrap().unwrap();

  assert_ne!(fresh.get_vfs().get_revision(), XrayRevision::MOUNTED);
  assert_eq!(**next, "derived");
  assert_eq!(next.revision, fresh.get_vfs().get_revision());
  assert_eq!(
    parse_counted(&fresh, "configs\\system.ltx", &parses),
    "edited behind the app"
  );
  assert_eq!(parse_counted(&fresh, "configs\\game.ltx", &parses), "payload");
  assert_eq!(
    parses.load(Ordering::Relaxed),
    3,
    "the changed file is parsed again and the unchanged one is served"
  );
}

#[test]
fn a_commit_moves_every_followed_session_before_it_returns() {
  let (assets, search) = opened_tree("followed", &["configs/system.ltx", "configs/game.ltx"]);
  let followed: Session<&str> = Session::new("followed");
  let unfollowed: Session<&str> = Session::new("unfollowed");
  let closed: Session<&str> = Session::new("closed");

  assets.add_follower(followed.clone());
  assets.add_follower(closed.clone());

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

/// Each change event a subscriber took, with the revision its session stood at as it took it.
struct RecordingSubscriber {
  session: Session<&'static str>,
  taken: Arc<Mutex<Vec<(AssetChangeEvent, XrayRevision)>>>,
}

impl AssetChangeSubscriber for RecordingSubscriber {
  fn take_change(&self, _: &AssetMountState, event: &AssetChangeEvent) {
    let revision: XrayRevision = self.session.get().unwrap().unwrap().revision;

    self.taken.lock().unwrap().push((event.clone(), revision));
  }
}

/// Every change event a subscriber took, in order.
#[derive(Clone, Default)]
struct ChangeLog {
  taken: Arc<Mutex<Vec<AssetChangeEvent>>>,
}

impl ChangeLog {
  /// Every session change taken, in order.
  fn list(&self) -> Vec<SessionChange> {
    self
      .taken
      .lock()
      .unwrap()
      .iter()
      .flat_map(|event| event.sessions.clone())
      .collect()
  }
}

impl AssetChangeSubscriber for ChangeLog {
  fn take_change(&self, _: &AssetMountState, event: &AssetChangeEvent) {
    self.taken.lock().unwrap().push(event.clone());
  }
}

/// A deferred derivation held until the test lets it go: it says when it starts, then waits.
struct HeldDerivation {
  pool: Arc<rayon::ThreadPool>,
  started: Receiver<()>,
  release: Sender<()>,
  calls: Arc<AtomicU32>,
}

impl HeldDerivation {
  /// A session deriving `"derived"`, read from nothing, on a pool of one thread, held as above.
  fn session() -> (Session<&'static str>, Self) {
    let pool: Arc<rayon::ThreadPool> = Arc::new(
      rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("a test pool"),
    );
    let (started_sender, started) = mpsc::channel::<()>();
    let (release, released) = mpsc::channel::<()>();
    let (started_sender, released) = (Mutex::new(started_sender), Mutex::new(released));
    let calls: Arc<AtomicU32> = Arc::default();
    let counted: Arc<AtomicU32> = Arc::clone(&calls);
    let session: Session<&str> = Session::new("test").with_deferred_derivation(Arc::clone(&pool), move |_, _| {
      counted.fetch_add(1, Ordering::SeqCst);
      started_sender.lock().unwrap().send(()).unwrap();
      released.lock().unwrap().recv().unwrap();

      Ok(SessionDerived::new("derived", XrayInputRecord::new()))
    });

    (
      session,
      Self {
        pool,
        started,
        release,
        calls,
      },
    )
  }

  /// Waits for the derivation to start reading.
  fn await_start(&self) {
    self
      .started
      .recv_timeout(Duration::from_secs(10))
      .expect("the derivation started");
  }

  /// Lets the derivation land and waits until its pool has nothing left to run.
  fn land(&self) {
    self.release.send(()).unwrap();
    self.pool.install(|| ());
  }
}

#[test]
fn a_deferred_derivation_is_reported_at_once_and_published_once_it_lands() {
  let (assets, search) = opened_tree("deferred", &["configs/system.ltx"]);
  let (session, held) = HeldDerivation::session();
  let log: ChangeLog = ChangeLog::default();

  assets.add_follower(session.clone());
  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "opened", &["configs\\system.ltx"]);

  opened.parts.get_or_init::<CountedPart>(|| Arc::new(1));
  write(&assets, &search, "configs\\system.ltx");
  held.await_start();

  let first: XrayRevision = assets.get_vfs().get_revision();

  assert!(
    Arc::ptr_eq(&session.get().unwrap().unwrap(), &opened),
    "the commit does not wait for it"
  );

  // A commit while it reads starts no second derivation, and is followed once it lands.
  write(&assets, &search, "configs\\system.ltx");
  held.land();

  let next: Arc<SessionSnapshot<&str>> = session.get().unwrap().unwrap();
  let deriving: SessionChange = SessionChange {
    session_id: opened.session_id,
    revision: XrayRevision::MOUNTED,
    root: SessionRootChange::Deriving,
    stale_parts: Vec::new(),
    is_view_renewed: false,
  };

  assert_eq!(held.calls.load(Ordering::SeqCst), 1);
  assert_eq!(**next, "derived");
  assert_eq!(next.revision, assets.get_vfs().get_revision());
  assert_eq!(
    log.list(),
    vec![
      deriving.clone(),
      deriving,
      SessionChange {
        session_id: opened.session_id,
        revision: first,
        root: SessionRootChange::Derived,
        stale_parts: vec!["counted"],
        is_view_renewed: true,
      },
      SessionChange {
        session_id: opened.session_id,
        revision: next.revision,
        root: SessionRootChange::Kept,
        stale_parts: Vec::new(),
        is_view_renewed: false,
      },
    ]
  );
}

#[test]
fn a_deferred_derivation_gives_way_to_a_close_or_an_open_published_while_it_reads() {
  for is_closed in [true, false] {
    let (assets, search) = opened_tree("deferred_lost", &["configs/system.ltx"]);
    let (session, held) = HeldDerivation::session();
    let log: ChangeLog = ChangeLog::default();

    assets.add_follower(session.clone());
    assets.add_subscriber(log.clone());

    let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "opened", &["configs\\system.ltx"]);

    write(&assets, &search, "configs\\system.ltx");
    held.await_start();

    if is_closed {
      session.close(&[opened.session_id]).unwrap();
    } else {
      open_reading(&session, &search, "newer", &[]);
    }

    held.land();

    assert_eq!(
      session.get().unwrap().map(|current| **current),
      (!is_closed).then_some("newer")
    );
    assert!(
      log
        .list()
        .iter()
        .all(|change| change.root == SessionRootChange::Deriving),
      "nothing derived is announced: {:?}",
      log.list()
    );
  }
}

#[test]
fn a_subscriber_takes_each_commits_changes_once_its_sessions_moved_before_it_returns() {
  let (assets, search) = opened_tree("subscribed", &["configs/system.ltx", "configs/game.ltx"]);
  let session: Session<&str> = Session::new("followed");
  let taken: Arc<Mutex<Vec<(AssetChangeEvent, XrayRevision)>>> = Arc::default();

  assets.add_follower(session.clone());
  assets.add_subscriber(RecordingSubscriber {
    session: session.clone(),
    taken: Arc::clone(&taken),
  });

  let opened: Arc<SessionSnapshot<&str>> = open_reading(&session, &search, "followed", &["configs\\system.ltx"]);

  write(&assets, &search, "configs\\game.ltx");

  let taken: Vec<(AssetChangeEvent, XrayRevision)> = taken.lock().unwrap().clone();
  let revision: XrayRevision = assets.get_vfs().get_revision();

  assert_eq!(taken.len(), 1);
  assert_eq!(taken[0].1, revision, "the session moved before the subscriber heard");
  assert_eq!(taken[0].0.revision, revision);
  assert_eq!(
    taken[0].0.sessions,
    vec![SessionChange {
      session_id: opened.session_id,
      revision,
      root: SessionRootChange::Kept,
      stale_parts: Vec::new(),
      is_view_renewed: false,
    }]
  );
}

#[test]
fn a_commit_moves_every_handle_of_a_registry_each_by_what_it_read() {
  let (assets, search) = opened_tree("registry", &["configs/system.ltx", "configs/game.ltx"]);
  let sessions: Sessions<&str> = Sessions::new("test", &SessionScopes::default())
    .with_derivation(|_, _| Ok(SessionDerived::new("derived", XrayInputRecord::new())));
  let open = |value: &'static str, path: &str| -> SessionId {
    let inputs: XrayInputRecord = XrayInputRecord::new();

    read(&search.clone().with_recording(&inputs), path);

    let id: SessionId = SessionId::generate();

    sessions.begin_open(id).unwrap();
    sessions
      .commit_open_with(id, value, search.get_vfs().get_revision(), inputs, SessionParts::new())
      .unwrap();

    id
  };
  let (untouched, touched): (SessionId, SessionId) =
    (open("system", "configs\\system.ltx"), open("game", "configs\\game.ltx"));

  assets.add_follower(sessions.clone());
  write(&assets, &search, "configs\\game.ltx");

  let revision: XrayRevision = assets.get_vfs().get_revision();

  assert_eq!(**sessions.require(untouched).unwrap(), "system", "its value is kept");
  assert_eq!(sessions.require(untouched).unwrap().revision, revision);
  assert_eq!(
    **sessions.require(touched).unwrap(),
    "derived",
    "its value is read again"
  );
  assert_eq!(sessions.require(touched).unwrap().revision, revision);
}
