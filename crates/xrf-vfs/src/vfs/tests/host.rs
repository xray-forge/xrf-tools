//! The host's snapshot rules: reads never wait on a mount, a search asked for inside another mounts beside it, and a
//! failed source stays settled except when a person opens its roots afresh.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{XrayMountMode, XrayRoots, XraySearch, XrayVfsHost};

/// An installation whose `fsgame.ltx` declares archives and loose files, holding the files it is given.
fn install(name: &str, files: &[&str]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_vfs_host/{name}"));

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("install root");
  fs::write(
    root.join("fsgame.ltx"),
    "$arch_dir$  = false | false | $fs_root$ | db\\\n$game_data$ = true  | true  | $fs_root$ | gamedata\\\n",
  )
  .expect("fsgame written");

  for file in files {
    write(&root, file);
  }

  root
}

fn write(root: &Path, file: &str) {
  let path: PathBuf = root.join(file);

  fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("directory created");
  fs::write(&path, b"payload").expect("file written");
}

fn roots(root: &Path) -> XrayRoots {
  XrayRoots::one(root.to_path_buf(), XrayMountMode::Installation)
}

fn holds(search: &XraySearch, logical_path: &str) -> bool {
  search.find(logical_path).is_ok_and(|found| found.is_some())
}

// A search holds no lock, so a search asked for while another is held mounts beside it rather than waiting on it, and
// what it mounts reaches the next search rather than the held one.
#[test]
fn mounts_a_search_asked_for_while_another_is_held() {
  let first: PathBuf = install("nested_first", &["gamedata/configs/system.ltx"]);
  let second: PathBuf = install("nested_second", &["gamedata/configs/game.ltx"]);
  let host: XrayVfsHost = XrayVfsHost::default();

  let outer: XraySearch = host.search(&roots(&first)).expect("the outer roots mount");
  let inner: XraySearch = host.search(&roots(&second)).expect("the inner roots mount");

  assert!(holds(&inner, "configs\\game.ltx"));
  assert!(holds(&outer, "configs\\system.ltx"));
  assert!(
    !holds(&outer, "configs\\game.ltx"),
    "the held search keeps the snapshot it was made over"
  );
  assert!(holds(
    &host
      .search(&roots(&second))
      .expect("the second roots are found mounted"),
    "configs\\game.ltx"
  ));
}

// Reads of mounted roots go on while other roots mount, each over whichever snapshot was newest when it began.
#[test]
fn reads_mounted_roots_while_others_mount() {
  let mounted: PathBuf = install("concurrent_mounted", &["gamedata/configs/system.ltx"]);
  let others: Vec<PathBuf> = (0..4)
    .map(|index| install(&format!("concurrent_other_{index}"), &["gamedata/configs/game.ltx"]))
    .collect();
  let host: XrayVfsHost = XrayVfsHost::default();

  host.search(&roots(&mounted)).expect("the roots mount");

  thread::scope(|scope| {
    let readers: Vec<thread::ScopedJoinHandle<bool>> = (0..4)
      .map(|_| {
        scope.spawn(|| {
          (0..50).all(|_| {
            holds(
              &host.search(&roots(&mounted)).expect("the mounted roots are found"),
              "configs\\system.ltx",
            )
          })
        })
      })
      .collect();
    let mounters: Vec<thread::ScopedJoinHandle<bool>> = others
      .iter()
      .map(|other| {
        scope.spawn(|| {
          holds(
            &host.search(&roots(other)).expect("the other roots mount"),
            "configs\\game.ltx",
          )
        })
      })
      .collect();

    for handle in readers.into_iter().chain(mounters) {
      assert!(handle.join().expect("no thread panics"));
    }
  });

  for other in &others {
    assert!(holds(
      &host.search(&roots(other)).expect("every mount was published"),
      "configs\\game.ltx"
    ));
  }
}

// A volume that failed to open stays settled for every read, and is tried again, with the roots planned again, when a
// person opens them.
#[test]
fn tries_the_roots_again_when_they_are_opened_afresh() {
  let root: PathBuf = install("fresh", &["db/textures.db0"]);
  let host: XrayVfsHost = XrayVfsHost::default();

  assert_eq!(
    host
      .search(&roots(&root))
      .expect("the roots mount")
      .list_skipped_sources()
      .len(),
    1
  );

  // The volume goes away and the loose tree arrives, as when a mod manager finishes installing.
  fs::remove_file(root.join("db/textures.db0")).expect("volume removed");
  write(&root, "gamedata/configs/system.ltx");

  let settled: XraySearch = host.search(&roots(&root)).expect("the roots are found");

  assert_eq!(
    settled.list_skipped_sources().len(),
    1,
    "a plain read keeps what it settled"
  );
  assert!(!holds(&settled, "configs\\system.ltx"));

  let fresh: XraySearch = host.search_fresh(&roots(&root)).expect("the roots mount afresh");

  assert!(fresh.list_skipped_sources().is_empty());
  assert!(holds(&fresh, "configs\\system.ltx"));
}

// Opening one set of roots afresh tries its own failures again and leaves another's settled: that one's volume is not
// opened again, so it keeps the reason it first failed with even once the file behind it is gone.
#[test]
fn leaves_other_roots_failures_settled_when_one_is_opened_afresh() {
  let opened: PathBuf = install("fresh_opened", &["db/textures.db0"]);
  let other: PathBuf = install("fresh_other", &["db/textures.db0"]);
  let host: XrayVfsHost = XrayVfsHost::default();
  let reasons = |search: XraySearch| -> Vec<String> {
    search
      .list_skipped_sources()
      .iter()
      .map(|skipped| skipped.reason.clone())
      .collect()
  };

  host.search(&roots(&opened)).expect("the roots mount");

  let first: Vec<String> = reasons(host.search(&roots(&other)).expect("the other roots mount"));

  fs::remove_file(other.join("db/textures.db0")).expect("volume removed");
  host.search_fresh(&roots(&opened)).expect("the roots mount afresh");

  assert_eq!(first.len(), 1);
  assert_eq!(
    reasons(host.search(&roots(&other)).expect("the other roots are found")),
    first
  );
}

// A root that is not there is no failure to open, so opening it afresh publishes no new snapshot for it.
#[test]
fn opening_a_missing_root_afresh_publishes_nothing() {
  let present: PathBuf = install("missing_present", &["gamedata/configs/system.ltx"]);
  let host: XrayVfsHost = XrayVfsHost::default();
  let spec: XrayRoots = XrayRoots::new([
    crate::XrayRoot::new(present.clone(), XrayMountMode::Installation),
    crate::XrayRoot::new(present.with_file_name("missing_absent"), XrayMountMode::Directory),
  ]);

  host.search(&spec).expect("the roots mount");

  let before = host.get_vfs();

  host.search_fresh(&spec).expect("the roots are found afresh");

  assert!(std::sync::Arc::ptr_eq(&before, &host.get_vfs()));
}
