//! Pins what an open configs project answers: formatting commits its rewrites so what the app retains and the sessions
//! reading them follow, a commit is read again, and a declaration search compares names as the project's dialect does.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionScopes, Sessions};
use xrf_ltx::{LtxFormatOptions, LtxProjectFormatResult};
use xrf_ltx_inspect::LtxDeclarationQuery;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayAsset, XrayInputRecord, XrayMountMode, XrayRevision, XrayRoots, XraySearch, XrayTransaction};

use crate::format::ConfigsFormat;
use crate::project::{ConfigsDeclarationSearch, ConfigsProjects, OpenedConfigsProject};

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

  let roots: XrayRoots = XrayRoots::one(root, XrayMountMode::Directory);
  let assets: AssetMountState = AssetMountState::new();
  // Derived inline rather than on a pool, so the commit below has read the project again by the time it returns.
  let projects: ConfigsProjects = ConfigsProjects::new("configs project", &SessionScopes::default())
    .with_derivation(&assets, OpenedConfigsProject::derive);
  let search: XraySearch = assets.search(&roots).expect("the tree mounts");
  let id: SessionId = SessionId::generate();
  let read = OpenedConfigsProject::read(&search, roots, None, false).expect("it opens");

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

#[test]
fn a_declaration_search_compares_sections_as_the_dialect_does_and_caps_its_answer() {
  let root: PathBuf = GeneratedResources::path("configs_find/declarations");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(root.join("configs")).expect("the tree is creatable");

  // More statements writing `cost` than one answer carries, so the cut and the count both show.
  let many: String = (0..ConfigsDeclarationSearch::MAXIMUM_DECLARATIONS + 20)
    .map(|index| format!("[wpn_{index}]\ncost = {index}\n"))
    .collect();

  fs::write(root.join("configs/system.ltx"), format!("[wpn_ak74]\ncost = 1\n{many}")).expect("config written");
  fs::write(root.join("configs/mod_system_a.ltx"), "![WPN_AK74]\ncost = 2\n").expect("config written");

  let roots: XrayRoots = XrayRoots::one(root, XrayMountMode::Directory);
  let search: XraySearch = AssetMountState::new().search(&roots).expect("the tree mounts");

  let find = |is_dltx: bool, query: LtxDeclarationQuery| {
    OpenedConfigsProject::read(&search, roots.clone(), None, is_dltx)
      .expect("it opens")
      .value
      .find_declarations(query)
      .expect("the search runs")
  };
  let section = |name: &str| LtxDeclarationQuery {
    section: Some(String::from(name)),
    ..LtxDeclarationQuery::default()
  };

  // The page never says how names compare: DLTX lowercases section names, so both headers name one section there.
  let dltx: ConfigsDeclarationSearch = find(true, section("wpn_ak74"));
  let standard: ConfigsDeclarationSearch = find(false, section("wpn_ak74"));

  assert_eq!(dltx.total, 2);
  assert!(dltx.search.query.is_section_case_folded);
  assert_eq!(standard.total, 1);
  assert!(!standard.search.query.is_section_case_folded);

  let capped: ConfigsDeclarationSearch = find(
    false,
    LtxDeclarationQuery {
      field: Some(String::from("cost")),
      ..LtxDeclarationQuery::default()
    },
  );

  assert_eq!(capped.total, ConfigsDeclarationSearch::MAXIMUM_DECLARATIONS + 22);
  assert_eq!(
    capped.search.declarations.len(),
    ConfigsDeclarationSearch::MAXIMUM_DECLARATIONS
  );
  assert_eq!(
    capped.search.declarations[0].path, "configs\\mod_system_a.ltx",
    "the first ones in project order"
  );
}
