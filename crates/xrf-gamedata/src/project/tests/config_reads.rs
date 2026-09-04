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
fn every_check_that_wants_system_ltx_shares_one_resolution() {
  let root: PathBuf = install("shared");
  let project: GamedataProject = open(root.clone());

  // Assembly reads each config once to learn its includes, and resolves nothing yet.
  let after_open: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  assert_eq!(after_open.reads, 2);
  assert_eq!(after_open.parses, 2);
  assert_eq!(after_open.resolutions, 0);

  // Animations runs three verifiers that each want `system.ltx`; weapons wants it a fourth time.
  verify(
    &project,
    vec![GamedataVerificationType::Animations, GamedataVerificationType::Weapons],
  );

  let after_checks: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  assert_eq!(
    after_checks.resolutions, 1,
    "four asking checks, one resolution between them"
  );
  assert_eq!(after_checks.reads, 2, "nothing was read a second time");
  assert_eq!(after_checks.parses, 2);

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn the_ltx_check_parses_each_config_once_and_reads_its_bytes_for_formatting() {
  let root: PathBuf = install("ltx_check");
  let project: GamedataProject = open(root.clone());

  verify(&project, vec![GamedataVerificationType::Ltx]);

  let counters: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  // Two files, parsed once each: verification reads what assembly already held, and the formatting pass renders from
  // the same documents. It does read each file's bytes a second time, which is not redundant - judging formatting means
  // comparing the bytes as authored against the canonical rendering, and only the bytes answer the first half.
  assert_eq!(counters.resolutions, 1);
  assert_eq!(counters.parses, 2, "one parse per config");
  assert_eq!(
    counters.reads, 4,
    "one content read and one formatting-comparison read per config"
  );

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn a_whole_sweep_parses_each_config_once() {
  let root: PathBuf = install("whole_sweep");
  let project: GamedataProject = open(root.clone());

  verify(
    &project,
    vec![
      GamedataVerificationType::Animations,
      GamedataVerificationType::Ltx,
      GamedataVerificationType::Sounds,
      GamedataVerificationType::Weapons,
      GamedataVerificationType::Weathers,
    ],
  );

  let counters: LtxReadCountersSnapshot = project.ltx_project.get_read_counters();

  // The invariant, across every check that touches configs: two files in the tree, parsed twice in total, whatever
  // asked for them. Resolutions count roots, and only `system.ltx` is one here. The extra reads are the formatting
  // pass comparing bytes, one per file, and nothing else re-reads.
  assert_eq!(counters.parses, 2);
  assert_eq!(counters.resolutions, 1);
  assert_eq!(counters.reads, 4);

  fs::remove_dir_all(root).expect("cleanup");
}
