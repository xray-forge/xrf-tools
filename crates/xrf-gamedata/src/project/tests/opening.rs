//! Covers what a project accepts as openable, across the mount modes.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayMountMode;

use crate::{GamedataProject, GamedataProjectReadOptions};

/// Declares gamedata the way a real installation does, so configs resolve one level below the path opened.
const FSGAME: &str = "\
;abbreviation   = recurs| notif | root      | add
$game_data$     = true  | true  | $fs_root$ | gamedata\\
";

fn options(root: PathBuf) -> GamedataProjectReadOptions {
  GamedataProjectReadOptions {
    root,
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  }
}

/// Builds an installation whose `gamedata\configs` holds a minimal `system.ltx`.
fn install(name: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("gamedata_opening/{name}"));
  let configs: PathBuf = root.join("gamedata").join("configs");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&configs).expect("configs directory");
  fs::write(root.join("fsgame.ltx"), FSGAME).expect("fsgame written");
  fs::write(configs.join("system.ltx"), "[section]\nvalue = 1\n").expect("system.ltx written");

  root
}

#[test]
fn opens_an_installation_whose_configs_are_declared_by_fsgame() {
  let root: PathBuf = install("declared");

  let project: GamedataProject =
    GamedataProject::open_with_mode(XrayMountMode::Installation, &options(root.clone())).expect("installation opens");

  assert_eq!(project.root(), root, "output names the game directory, not a mount");
  assert_eq!(
    project
      .ltx_project
      .system_ltx()
      .expect("system.ltx reads")
      .ltx
      .get_from("section", "value"),
    Some("1"),
    "configs resolve through the declared mount"
  );

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn auto_opens_an_installation_that_declares_itself() {
  let root: PathBuf = install("auto");

  let project: GamedataProject = GamedataProject::open(&options(root.clone())).expect("auto opens the installation");

  assert_eq!(project.collisions().len(), 0, "one mount, nothing shadowed");
  assert_eq!(
    project.ltx_project.system_ltx_path().expect("scoped path").as_str(),
    "configs\\system.ltx",
    "the config project is scoped to configs inside the game tree"
  );

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn opens_when_an_ignore_hides_the_root_config() {
  // Identity is decided before inspection filters apply: checks that do not read configs must still run.
  let root: PathBuf = install("ignored_configs");
  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    ignored: vec![String::from("configs")],
    output: xrf_output::OutputOptions::default(),
    root: root.clone(),
    ..Default::default()
  })
  .expect("the unfiltered project identity is valid");

  assert!(
    project.entries().is_empty(),
    "the filtered project still hides all config assets from checks"
  );

  // A prefix that hides something else is none of the gate's business.
  let project = GamedataProject::open(&GamedataProjectReadOptions {
    ignored: vec![String::from("textures\\wip")],
    output: xrf_output::OutputOptions::default(),
    root: root.clone(),
    ..Default::default()
  });

  assert!(project.is_ok(), "an unrelated ignored prefix still opens");

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn refuses_a_directory_that_resolves_no_system_ltx() {
  let root: PathBuf = build_absolute_generated_test_resource_path("gamedata_opening/empty");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(root.join("configs")).expect("configs directory");

  let error = GamedataProject::open(&options(root.clone())).expect_err("nothing resolves system.ltx");

  assert!(
    error.to_string().contains("configs\\system.ltx"),
    "the refusal names what failed to resolve, got: {error}"
  );

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn directory_mode_ignores_a_declaration_and_reads_the_path_itself() {
  // `Directory` is what keeps a command named at a gamedata tree from widening into the game around it, so an installation
  // root opened this way must fail: its own directory holds no configs.
  let root: PathBuf = install("ignored");

  let error = GamedataProject::open_with_mode(XrayMountMode::Directory, &options(root.clone()))
    .expect_err("the installation root itself holds no configs");

  assert!(
    error.to_string().contains("configs\\system.ltx"),
    "the refusal names what failed to resolve, got: {error}"
  );

  fs::remove_dir_all(root).expect("cleanup");
}
