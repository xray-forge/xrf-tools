//! Pins that formatting configs commits its rewrites, so what the app retains and the sessions reading them follow.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionScopes, Sessions};
use xrf_ltx::{LtxFormatOptions, LtxProjectFormatResult};
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayAsset, XrayInputRecord, XrayMountMode, XrayRevision, XrayRoots, XraySearch, XrayTransaction};

use crate::format::ConfigsFormat;
use crate::project::{ConfigsProjects, OpenedConfigsProject};

const SYSTEM_LTX: &str = "configs\\system.ltx";
const GAME_LTX: &str = "configs\\game.ltx";

/// Reads a config through `search`, counting the parses that ran.
fn read(search: &XraySearch, logical_path: &str, parses: &AtomicU32) -> Arc<Vec<u8>> {
  let asset: XrayAsset = search
    .find(logical_path)
    .expect("the lookup succeeds")
    .expect("the config is there");

  search
    .read_parsed(&asset, |bytes| {
      parses.fetch_add(1, Ordering::Relaxed);

      Ok(bytes)
    })
    .expect("the config reads")
}

#[test]
fn a_format_commits_its_rewrites_so_retained_configs_and_sessions_follow() {
  let root: PathBuf = GeneratedResources::path("configs_format/committed");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(root.join("configs")).expect("the tree is creatable");
  fs::write(root.join("configs/system.ltx"), "[a]\nkey=value\n").expect("config written");
  fs::write(root.join("configs/game.ltx"), "[b]\r\nkey = value\r\n").expect("config written");

  let roots: XrayRoots = XrayRoots::one(root.clone(), XrayMountMode::Directory);
  let assets: AssetMountState = AssetMountState::new();
  let search: XraySearch = assets.search(&roots).expect("the tree mounts");
  let parses: AtomicU32 = AtomicU32::new(0);
  let sessions: Sessions<&str> = Sessions::new("test", &SessionScopes::default()).with_derivation(&assets, |_, _| {
    Ok(SessionDerived::new("derived", XrayInputRecord::new()))
  });
  let inputs: XrayInputRecord = XrayInputRecord::new();
  let id: SessionId = SessionId::generate();

  read(&search.clone().with_recording(&inputs), SYSTEM_LTX, &parses);
  read(&search, GAME_LTX, &parses);
  sessions
    .begin_fixture_open(id)
    .unwrap()
    .commit_fixture(SessionDerived::new("opened", inputs), XrayRevision::MOUNTED)
    .unwrap();

  let result: LtxProjectFormatResult =
    ConfigsFormat::format(&assets, &roots, None, LtxFormatOptions::default()).expect("the format runs");
  let after: XraySearch = assets.search(&roots).expect("the tree is found");
  let followed = sessions.require(id).unwrap();

  assert_eq!((result.total_files, result.invalid_files), (2, 1));
  assert_eq!(
    fs::read_to_string(root.join("configs/system.ltx")).unwrap(),
    "[a]\r\nkey = value\r\n"
  );
  assert_ne!(
    assets.get_vfs().get_revision(),
    XrayRevision::MOUNTED,
    "the format moved the revision"
  );
  assert_eq!(*read(&after, SYSTEM_LTX, &parses), b"[a]\r\nkey = value\r\n");
  assert_eq!(*read(&after, GAME_LTX, &parses), b"[b]\r\nkey = value\r\n");
  assert_eq!(
    parses.load(Ordering::Relaxed),
    3,
    "the rewritten config is parsed again and the untouched one is served"
  );
  assert_eq!(**followed, "derived");
  assert_eq!(followed.revision, assets.get_vfs().get_revision());
}

#[test]
fn an_open_project_lists_a_config_committed_beside_it_once_read_again() {
  let root: PathBuf = GeneratedResources::path("configs_follow/created");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(root.join("configs")).expect("the tree is creatable");
  fs::write(root.join("configs/system.ltx"), "[a]\nkey = value\n").expect("config written");
  fs::write(root.join("configs/game.ltx"), "[b]\nkey = value\n").expect("config written");

  let roots: XrayRoots = XrayRoots::one(root.clone(), XrayMountMode::Directory);
  let assets: AssetMountState = AssetMountState::new();
  // Derived inline rather than on a pool, so the commit below has read the project again by the time it returns.
  let projects: ConfigsProjects = ConfigsProjects::new("configs project", &SessionScopes::default())
    .with_derivation(&assets, OpenedConfigsProject::derive);
  let search: XraySearch = assets.search(&roots).expect("the tree mounts");
  let id: SessionId = SessionId::generate();
  let read = OpenedConfigsProject::read(&search, roots.clone(), None, false).expect("it opens");

  assert_eq!(read.value.descriptor.inventory.files.len(), 2);

  projects
    .begin_fixture_open(id)
    .unwrap()
    .commit_fixture(read, search.get_vfs().get_revision())
    .unwrap();
  assets
    .commit(
      &XrayTransaction::new(&search)
        .with_write("configs\\weather.ltx", "[c]\nkey = value\n")
        .unwrap(),
    )
    .expect("the config commits");

  let followed = projects.require(id).unwrap();

  assert_eq!(followed.revision, assets.get_vfs().get_revision(), "the session moved");
  assert_eq!(followed.descriptor.inventory.files.len(), 3);
}
