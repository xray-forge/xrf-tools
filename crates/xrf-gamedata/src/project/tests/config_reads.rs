//! How many times a verification sweep reads the config tree.

use std::fs;
use std::path::PathBuf;

use xrf_ltx::LtxReadCountersSnapshot;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationType};

/// Declares gamedata the way an installation does, so configs resolve one level below the path opened.
const FSGAME: &str = "\
;abbreviation   = recurs| notif | root      | add
$game_data$     = true  | true  | $fs_root$ | gamedata\\
";

/// An entry point that includes one file, so a resolution is more than one parse.
const SYSTEM: &str = "#include \"weapons.ltx\"\n\n[system]\nversion = 1\n";

const WEAPONS: &str = "[wpn_base]\ncost = 1000\n";

/// Builds an installation holding a two-file config tree.
fn install(name: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("gamedata_config_reads/{name}"));
  let configs: PathBuf = root.join("gamedata").join("configs");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&configs).expect("configs directory");
  fs::write(root.join("fsgame.ltx"), FSGAME).expect("fsgame written");
  fs::write(configs.join("system.ltx"), SYSTEM).expect("system.ltx written");
  fs::write(configs.join("weapons.ltx"), WEAPONS).expect("weapons.ltx written");

  root
}

fn open(root: PathBuf) -> GamedataProject {
  GamedataProject::open(&GamedataProjectReadOptions {
    root,
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .expect("installation opens")
}

fn verify(project: &GamedataProject, checks: Vec<GamedataVerificationType>) {
  project
    .verify(&GamedataProjectVerifyOptions {
      checks,
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .expect("selected checks run");
}

#[test]
fn each_check_that_wants_system_ltx_resolves_the_whole_include_tree_again() {
  let root: PathBuf = install("repeated");
  let project: GamedataProject = open(root.clone());

  // Assembly first: one read and one include-only parse per config, and nothing resolved yet.
  let after_open: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  assert_eq!(after_open.include_scans, 2);
  assert_eq!(after_open.resolutions, 0);
  assert_eq!(after_open.parses, 0);

  // Animations runs three verifiers that each want `system.ltx`; weapons wants it once more.
  verify(
    &project,
    vec![GamedataVerificationType::Animations, GamedataVerificationType::Weapons],
  );

  let after_checks: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  assert_eq!(
    after_checks.resolutions, 4,
    "hud item, hud motion collisions, player hud, and weapons each resolve system.ltx"
  );
  assert_eq!(
    after_checks.parses, 8,
    "both configs of the tree are parsed once per resolution"
  );

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn the_ltx_check_walks_the_project_for_formatting_and_again_for_verification() {
  let root: PathBuf = install("ltx_check");
  let project: GamedataProject = open(root.clone());

  verify(&project, vec![GamedataVerificationType::Ltx]);

  let counters: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  // The check verifies entry points and separately checks every file's formatting, so `system.ltx` is resolved once
  // while `weapons.ltx` is read again on its own for the format pass.
  assert_eq!(counters.resolutions, 1);
  assert!(
    counters.reads > counters.include_scans,
    "the format pass reads beyond assembly, got {counters:?}"
  );

  fs::remove_dir_all(root).expect("cleanup");
}
