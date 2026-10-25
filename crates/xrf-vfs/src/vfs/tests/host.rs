//! The host's snapshot rules: reads never wait on a mount, a search asked for inside another mounts beside it, a
//! failed source stays settled except when a person opens its roots afresh, and a retained file changed on disk is
//! applied when its roots are looked at.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use xrf_test_utils::GeneratedResources;

use crate::{
  XrayAsset, XrayCachePolicy, XrayChangeKind, XrayChangeSet, XrayMountMode, XrayRoots, XraySearch, XrayVfs, XrayVfsHost,
};

/// An installation whose `fsgame.ltx` declares archives and loose files, holding the files it is given.
fn install(name: &str, files: &[&str]) -> PathBuf {
  let root: PathBuf = GeneratedResources::create_dir(&format!("xray_vfs_host/{name}")).expect("install root");

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

/// Parses a config through `search`, counting the parses that ran.
fn parse(search: &XraySearch, logical_path: &str, parses: &AtomicUsize) -> String {
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

// A retained loose file changed on disk behind every reader is read again once its roots are looked at, one that did
// not change keeps its retained value, and nothing changed publishes nothing.
#[test]
fn applies_a_retained_file_changed_on_disk_and_keeps_an_unchanged_one() {
  let root: PathBuf = install(
    "changed_on_disk",
    &["gamedata/configs/system.ltx", "gamedata/configs/game.ltx"],
  );
  let host: XrayVfsHost = XrayVfsHost::new(XrayVfs::new().with_cache_policy(XrayCachePolicy::configs()));
  let parses: AtomicUsize = AtomicUsize::new(0);
  let search: XraySearch = host.search_fresh(&roots(&root)).expect("the roots mount");

  parse(&search, "configs\\system.ltx", &parses);
  parse(&search, "configs\\game.ltx", &parses);

  let unchanged: Arc<XrayVfs> = host.get_vfs();

  assert!(
    host
      .apply_changed_on_disk(&search)
      .expect("the files are looked at")
      .is_empty()
  );
  assert!(
    Arc::ptr_eq(&unchanged, &host.get_vfs()),
    "nothing changed publishes nothing"
  );

  fs::write(root.join("gamedata/configs/system.ltx"), b"edited behind the app").expect("file rewritten");

  let changes: XrayChangeSet = host.apply_changed_on_disk(&search).expect("the change applies");
  let after: XraySearch = host.search(&roots(&root)).expect("the roots are found");

  assert_eq!(changes.get_changes().len(), 1);
  assert_eq!(changes.get_changes()[0].get_kind(), XrayChangeKind::Modified);
  assert_eq!(changes.get_changes()[0].get_path().as_str(), "configs\\system.ltx");
  assert_eq!(after.get_vfs().get_revision(), changes.get_revision());
  assert_eq!(parse(&after, "configs\\system.ltx", &parses), "edited behind the app");
  assert_eq!(parse(&after, "configs\\game.ltx", &parses), "payload");
  assert_eq!(
    parses.load(Ordering::Relaxed),
    3,
    "the changed file is parsed again and the unchanged one is served"
  );
  assert!(
    host.apply_changed_on_disk(&after).expect("looked at again").is_empty(),
    "a file read again is not changed again"
  );
}

/// Writes a volume storing one uncompressed entry, the least a planned archive mount opens.
fn write_volume(path: &Path, name: &str, payload: &[u8]) {
  let mut descriptors: Vec<u8> = Vec::new();
  let header: u16 = 16 + u16::try_from(name.len()).expect("a short name");
  let size: u32 = u32::try_from(payload.len()).expect("a short payload");

  descriptors.extend_from_slice(&header.to_le_bytes());
  descriptors.extend_from_slice(&size.to_le_bytes());
  descriptors.extend_from_slice(&size.to_le_bytes());
  descriptors.extend_from_slice(&0u32.to_le_bytes());
  descriptors.extend_from_slice(name.as_bytes());

  // The payload sits in a data chunk ahead of the table, straight after that chunk's own id and size.
  descriptors.extend_from_slice(&8u32.to_le_bytes());

  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&0u32.to_le_bytes());
  bytes.extend_from_slice(&size.to_le_bytes());
  bytes.extend_from_slice(payload);
  bytes.extend_from_slice(&1u32.to_le_bytes());
  bytes.extend_from_slice(&u32::try_from(descriptors.len()).expect("a short table").to_le_bytes());
  bytes.extend_from_slice(&descriptors);

  fs::create_dir_all(path.parent().expect("a volume sits in a directory")).expect("directory created");
  fs::write(path, bytes).expect("volume written");
}

/// Whether a search answers a path with a loose copy.
fn answers_loose(search: &XraySearch, logical_path: &str) -> bool {
  search
    .find(logical_path)
    .expect("the lookup succeeds")
    .expect("the path is there")
    .to_physical_path()
    .is_some()
}

// An installation's loose override beats its archived copy whichever of its sources a person opened first: opening its
// `db` folder alone mounts the volumes ahead of the installation's gamedata, and the installation then reuses that
// mount, so its step must search in its own order rather than in mount order.
#[test]
fn searches_an_installation_in_its_own_order_after_its_archives_were_opened_alone() {
  let root: PathBuf = install("plan_order", &["gamedata/configs/system.ltx"]);

  write_volume(&root.join("db/configs.db0"), "configs\\system.ltx", b"archived");

  let host: XrayVfsHost = XrayVfsHost::default();
  let archives: XraySearch = host
    .search(&XrayRoots::one(root.join("db"), XrayMountMode::Auto))
    .expect("the archives mount");

  assert!(!answers_loose(&archives, "configs\\system.ltx"));

  let installation: XraySearch = host.search(&roots(&root)).expect("the installation mounts");

  assert_eq!(
    host.get_vfs().get_mounts().len(),
    2,
    "the installation reuses the archive mount opened first"
  );
  assert!(answers_loose(&installation, "configs\\system.ltx"));

  let fresh: XrayVfsHost = XrayVfsHost::default();

  fresh.search(&roots(&root)).expect("the installation mounts");

  assert!(answers_loose(
    &fresh
      .search(&XrayRoots::one(root.join("db"), XrayMountMode::Auto))
      .and_then(|_| fresh.search(&roots(&root)))
      .expect("both orders mount"),
    "configs\\system.ltx"
  ));
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
