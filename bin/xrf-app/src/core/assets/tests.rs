use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayProbe, XrayRoots};

use crate::core::assets::AssetMountState;

/// An installation whose `fsgame.ltx` declares archives and loose files, holding the files it is given.
fn install(name: &str, files: &[&str]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("asset_mount_state/{name}"));

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

fn holds(probe: &XrayProbe, logical_path: &str) -> bool {
  probe
    .find(logical_path)
    .is_ok_and(|resolution| resolution.get_asset().is_some())
}

// A consumer runs over a snapshot and holds no lock, so a probe it asks for itself mounts beside it rather than
// waiting on it, and what it mounts reaches the next probe rather than the running one.
#[test]
fn mounts_a_probe_asked_for_inside_another_without_waiting_on_it() {
  let first: PathBuf = install("nested_first", &["gamedata/configs/system.ltx"]);
  let second: PathBuf = install("nested_second", &["gamedata/configs/game.ltx"]);
  let state: AssetMountState = AssetMountState::new();

  let (is_inner_found, is_outer_found): (bool, bool) = state
    .with_probe(&roots(&first), |outer| {
      let is_inner_found: bool = state
        .with_probe(&roots(&second), |inner| holds(inner, "configs\\game.ltx"))
        .expect("the inner roots mount");

      (is_inner_found, holds(outer, "configs\\system.ltx"))
    })
    .expect("the outer roots mount");

  assert!(is_inner_found && is_outer_found);
  assert!(
    state
      .with_probe(&roots(&second), |probe| holds(probe, "configs\\game.ltx"))
      .expect("the second roots are found mounted")
  );
}

// Reads of mounted roots go on while other roots mount, each over whichever world was newest when it began.
#[test]
fn reads_mounted_roots_while_others_mount() {
  let mounted: PathBuf = install("concurrent_mounted", &["gamedata/configs/system.ltx"]);
  let others: Vec<PathBuf> = (0..4)
    .map(|index| install(&format!("concurrent_other_{index}"), &["gamedata/configs/game.ltx"]))
    .collect();
  let state: AssetMountState = AssetMountState::new();

  state.with_probe(&roots(&mounted), |_| ()).expect("the roots mount");

  thread::scope(|scope| {
    let readers: Vec<thread::ScopedJoinHandle<bool>> = (0..4)
      .map(|_| {
        scope.spawn(|| {
          (0..50).all(|_| {
            state
              .with_probe(&roots(&mounted), |probe| holds(probe, "configs\\system.ltx"))
              .expect("the mounted roots are found")
          })
        })
      })
      .collect();
    let mounters: Vec<thread::ScopedJoinHandle<bool>> = others
      .iter()
      .map(|other| {
        scope.spawn(|| {
          state
            .with_probe(&roots(other), |probe| holds(probe, "configs\\game.ltx"))
            .expect("the other roots mount")
        })
      })
      .collect();

    for handle in readers.into_iter().chain(mounters) {
      assert!(handle.join().expect("no thread panics"));
    }
  });

  for other in &others {
    assert!(
      state
        .with_probe(&roots(other), |probe| holds(probe, "configs\\game.ltx"))
        .expect("every mount was published")
    );
  }
}

// A volume that failed to open stays settled for every read, and is tried again, with the roots planned again, when a
// person opens them.
#[test]
fn tries_the_roots_again_when_they_are_opened_afresh() {
  let root: PathBuf = install("fresh", &["db/textures.db0"]);
  let state: AssetMountState = AssetMountState::new();
  let skipped = |probe: &XrayProbe| probe.list_skipped_sources().len();

  assert_eq!(state.with_probe(&roots(&root), skipped).expect("the roots mount"), 1);

  // The volume goes away and the loose tree arrives, as when a mod manager finishes installing.
  fs::remove_file(root.join("db/textures.db0")).expect("volume removed");
  write(&root, "gamedata/configs/system.ltx");

  assert_eq!(
    state.with_probe(&roots(&root), skipped).expect("the roots are found"),
    1,
    "a plain read keeps what it settled"
  );
  assert!(
    !state
      .with_probe(&roots(&root), |probe| holds(probe, "configs\\system.ltx"))
      .expect("the roots are found")
  );

  let (skipped, is_found): (usize, bool) = state
    .with_fresh_probe(&roots(&root), |probe| {
      (skipped(probe), holds(probe, "configs\\system.ltx"))
    })
    .expect("the roots mount afresh");

  assert_eq!(skipped, 0);
  assert!(is_found);
}

// Opening one set of roots afresh tries its own failures again and leaves another's settled: that one's volume is not
// opened again, so it keeps the reason it first failed with even once the file behind it is gone.
#[test]
fn leaves_other_roots_failures_settled_when_one_is_opened_afresh() {
  let opened: PathBuf = install("fresh_opened", &["db/textures.db0"]);
  let other: PathBuf = install("fresh_other", &["db/textures.db0"]);
  let state: AssetMountState = AssetMountState::new();
  let reasons = |probe: &XrayProbe| -> Vec<String> {
    probe
      .list_skipped_sources()
      .iter()
      .map(|skipped| skipped.reason.clone())
      .collect()
  };

  state.with_probe(&roots(&opened), |_| ()).expect("the roots mount");

  let first: Vec<String> = state
    .with_probe(&roots(&other), reasons)
    .expect("the other roots mount");

  fs::remove_file(other.join("db/textures.db0")).expect("volume removed");
  state
    .with_fresh_probe(&roots(&opened), |_| ())
    .expect("the roots mount afresh");

  assert_eq!(first.len(), 1);
  assert_eq!(
    state
      .with_probe(&roots(&other), reasons)
      .expect("the other roots are found"),
    first
  );
}
