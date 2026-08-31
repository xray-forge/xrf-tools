//! Covers the always-run coverage check, over installations whose declared sources partly fail to open.
//!
//! Arranged through real plans rather than a stub source, because what is under test is the record mounting keeps when
//! it tolerates a source it cannot open — and only mounting can produce that record.

use std::fs;
use std::path::PathBuf;

use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayLookupScope, XrayMountPlan, XraySourceKind, XrayVfs};

use crate::{
  GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions,
  GamedataVerificationReport, GamedataVerificationStatus, GamedataVerificationType,
};

/// An installation shaped like a real one: gamedata declared beside aliases that resolve inside it, and writable state
/// directories that a copied install simply does not have yet.
const FSGAME: &str = "\
;abbreviation   = recurs| notif | root             | add
$app_data_root$ = true  | false | $fs_root$        | appdata\\
$game_data$     = true  | true  | $fs_root$        | gamedata\\
$game_meshes$   = true  | true  | $game_data$      | meshes\\
$game_textures$ = true  | true  | $game_data$      | textures\\
$level$         = false | false | $game_data$      | levels\\
$logs$          = true  | false | $app_data_root$  | logs\\
$screenshots$   = true  | false | $app_data_root$  | screenshots\\
";

fn resource(name: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("gamedata_coverage/{name}"));

  let _ = fs::remove_dir_all(&root);

  root
}

/// A gamedata tree holding the one config that makes it a project.
fn gamedata(name: &str) -> PathBuf {
  let root: PathBuf = resource(name);

  fs::create_dir_all(root.join("configs")).expect("configs directory");
  fs::write(root.join("configs").join("system.ltx"), "[section]\r\nvalue = 1\r\n").expect("system.ltx");

  root
}

/// A project over `root` plus one archive source per absent path, so each is planned and then fails to open.
fn project_missing_sources(root: &PathBuf, absent: &[(&str, PathBuf)]) -> GamedataProject {
  let mut plan: XrayMountPlan = XrayMountPlan::root(root).expect("the gamedata root is planned");

  for (origin, path) in absent {
    plan = plan
      .with_kind(path, "", origin, XraySourceKind::Archive)
      .expect("the absent archive is planned");
  }

  let vfs: XrayVfs = XrayVfs::from_plan(&plan).expect("the readable mount still opens");

  GamedataProject {
    ltx_project: LtxProject::open_at_scope_opt(
      root.join("configs"),
      vfs,
      XrayLookupScope::all().with_prefix("configs").expect("a valid prefix"),
      LtxProjectOptions::default(),
    )
    .expect("a project over the readable mount assembles"),
    root: root.clone(),
    scope: XrayLookupScope::all(),
  }
}

fn options(checks: Vec<GamedataVerificationType>) -> GamedataProjectVerifyOptions {
  GamedataProjectVerifyOptions {
    checks,
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  }
}

/// The three things a reader needs: the alias to find the declaration, the path to find the source, the reason to act.
#[test]
fn reports_every_unopened_source_with_its_alias_and_reason() {
  let root: PathBuf = gamedata("unopened");
  let volumes: PathBuf = root.join("db");
  let patches: PathBuf = root.join("patches");

  let result = project_missing_sources(
    &root,
    &[("$arch_dir$", volumes.clone()), ("$arch_dir_patches$", patches.clone())],
  )
  .verify_coverage(&options(Vec::new()))
  .expect("the coverage check completes");

  assert_eq!(result.get_status(), GamedataVerificationStatus::Incomplete);
  assert_eq!(
    result.get_failure_message(),
    "2 declared source(s) could not be opened, so no result covers them"
  );

  let findings: Vec<(Option<&str>, &str)> = result
    .get_findings()
    .iter()
    .map(|finding| (finding.subject(), finding.rule_id().as_str()))
    .collect();

  assert_eq!(
    findings,
    vec![
      (
        Some(xrf_utils::to_portable_path_string(&volumes).as_str()),
        "coverage.skipped-mount"
      ),
      (
        Some(xrf_utils::to_portable_path_string(&patches).as_str()),
        "coverage.skipped-mount"
      ),
    ]
  );

  assert!(
    result.get_findings()[0]
      .message()
      .starts_with("Declared source '$arch_dir$'"),
    "the alias names the declaration to repair, got: {}",
    result.get_findings()[0].message()
  );

  fs::remove_dir_all(root).expect("cleanup");
}

/// Full coverage is not a passing verdict, or a run whose selected checks judged nothing would read as success.
#[test]
fn adds_nothing_to_the_verdict_when_every_declared_source_opened() {
  let root: PathBuf = gamedata("complete");

  let result = project_missing_sources(&root, &[])
    .verify_coverage(&options(Vec::new()))
    .expect("the coverage check completes");

  assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
  assert_eq!(result.get_failure_message(), "Every declared source opened");
  assert!(result.get_findings().is_empty());

  fs::remove_dir_all(root).expect("cleanup");
}

/// The point of the issue: passing checks over part of an installation must not report that the whole one passed.
#[test]
fn marks_a_run_incomplete_whose_selected_check_passed() {
  let root: PathBuf = gamedata("incomplete_run");
  let project: GamedataProject = project_missing_sources(&root, &[("$arch_dir$", root.join("db"))]);

  let report: GamedataVerificationReport = project
    .verify(&options(vec![GamedataVerificationType::Ltx]))
    .expect("verification completes");

  assert_eq!(
    report.get_status(),
    GamedataVerificationStatus::Incomplete,
    "a verification that did not inspect all intended game data cannot claim that it passed"
  );

  let checks: Vec<(GamedataVerificationType, GamedataVerificationStatus)> = report
    .get_checks()
    .iter()
    .map(|check| (check.get_verification_type(), check.get_status()))
    .collect();

  assert_eq!(
    checks,
    vec![
      (
        GamedataVerificationType::Coverage,
        GamedataVerificationStatus::Incomplete
      ),
      (
        GamedataVerificationType::Collisions,
        GamedataVerificationStatus::Skipped
      ),
      (GamedataVerificationType::Ltx, GamedataVerificationStatus::Passed),
    ],
    "coverage is judged beside the selection rather than inside it"
  );

  assert_eq!(
    report.get_failure_messages(),
    vec![String::from(
      "1 declared source(s) could not be opened, so no result covers them"
    )]
  );

  fs::remove_dir_all(root).expect("cleanup");
}

/// An `fsgame.ltx` alias omitted while planning is not the same fact, and must not cost a healthy project its verdict.
///
/// A real installation declares some thirty-five aliases and mounts around ten: the rest resolve inside the gamedata
/// root already mounted, or name writable state a copied install has not created yet. Recording those as lost coverage
/// would leave every project permanently incomplete, which is the same as reporting nothing at all.
#[test]
fn does_not_treat_a_declared_alias_omitted_while_planning_as_lost_coverage() {
  let root: PathBuf = resource("planned_omissions");
  let configs: PathBuf = root.join("gamedata").join("configs");

  fs::create_dir_all(&configs).expect("configs directory");
  fs::create_dir_all(root.join("gamedata").join("meshes")).expect("meshes directory");
  fs::write(root.join("fsgame.ltx"), FSGAME).expect("fsgame");
  fs::write(configs.join("system.ltx"), "[section]\r\nvalue = 1\r\n").expect("system.ltx");

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .expect("the installation opens");

  assert!(
    project.skipped_mounts().is_empty(),
    "aliases inside gamedata and absent writable state are correctly omitted, not skipped: {:?}",
    project.skipped_mounts()
  );

  let report: GamedataVerificationReport = project
    .verify(&options(vec![GamedataVerificationType::Ltx]))
    .expect("verification completes");

  assert_eq!(report.get_status(), GamedataVerificationStatus::Passed);
  assert_eq!(report.get_checks()[0].get_summary(), "Every declared source opened");

  fs::remove_dir_all(root).expect("cleanup");
}
