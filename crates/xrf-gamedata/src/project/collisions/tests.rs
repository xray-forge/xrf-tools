//! Covers the always-run collisions check, over a source built to collide.
//!
//! A loose case-only pair cannot exist in one directory on a case-insensitive filesystem, which is where this
//! workspace's tests run, so the collision is arranged through the source seam [`XrayAssetSource`] exists for rather
//! than through a platform-gated fixture that would never execute in CI.

use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_vfs::{
  XrayAssetContainer, XrayAssetSource, XrayCollisionSite, XrayLogicalPath, XrayLookupScope, XrayPathCollision,
  XraySourceKind, XrayVfs,
};

use crate::{
  GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationReport,
  GamedataVerificationStatus, GamedataVerificationType,
};

/// A mount that holds nothing and reports the collisions it was built with.
///
/// Only [`XrayAssetSource::get_collisions`] carries data: the check reads the recorded collisions and never the entries
/// around them, so entries would say nothing about it either way.
#[derive(Debug)]
struct CollidingSource {
  collisions: Vec<XrayPathCollision>,
  root: PathBuf,
}

impl CollidingSource {
  fn new(collisions: Vec<XrayPathCollision>) -> Self {
    Self {
      collisions,
      root: PathBuf::from("gamedata"),
    }
  }
}

impl XrayAssetSource for CollidingSource {
  fn get_label(&self) -> &str {
    "colliding"
  }

  fn get_kind(&self) -> XraySourceKind {
    XraySourceKind::Directory
  }

  fn is_writable(&self) -> bool {
    false
  }

  fn get_root_path(&self) -> &Path {
    &self.root
  }

  fn locate(&self, _path: &str) -> Option<XrayAssetContainer> {
    None
  }

  fn read(&self, path: &str) -> XrfResult<Vec<u8>> {
    Err(XrfError::new_not_found_error(format!("No entry '{path}'")))
  }

  fn write(&self, _path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_not_implemented_error("A test source is read-only"))
  }

  fn create(&self, _path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_not_implemented_error("A test source is read-only"))
  }

  fn list_entries<'a>(&'a self, _prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a> {
    Box::new(std::iter::empty())
  }

  fn get_size(&self, _path: &str) -> Option<u64> {
    None
  }

  fn get_collisions(&self) -> &[XrayPathCollision] {
    &self.collisions
  }
}

/// One loose file made unreachable by another under the same engine identity.
fn collision(logical_path: &str, kept: &str, unreachable: &str) -> XrayPathCollision {
  XrayPathCollision {
    logical_path: XrayLogicalPath::new(logical_path).expect("a valid logical path"),
    kept: XrayCollisionSite::Loose(PathBuf::from("gamedata").join(kept)),
    unreachable: XrayCollisionSite::Loose(PathBuf::from("gamedata").join(unreachable)),
  }
}

/// A project whose single mount holds nothing but records `collisions`.
fn project(collisions: Vec<XrayPathCollision>) -> GamedataProject {
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount("", Box::new(CollidingSource::new(collisions)))
    .expect("the test source mounts at the logical root");

  GamedataProject {
    ltx_project: LtxProject::open_at_scope_opt(
      PathBuf::new(),
      vfs,
      XrayLookupScope::all(),
      LtxProjectOptions::default(),
    )
    .expect("a project over an empty mount assembles"),
    root: PathBuf::new(),
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

/// The finding a person acts on: which identity is claimed, and which of the two files to remove.
#[test]
fn reports_both_sites_of_every_unreachable_file() {
  let result = project(vec![
    collision("textures\\a.dds", "textures\\a.dds", "textures\\A.dds"),
    collision("meshes\\b.ogf", "meshes\\B.OGF", "meshes\\b.ogf"),
  ])
  .verify_collisions(&options(Vec::new()))
  .expect("the collisions check completes");

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_eq!(
    result.get_failure_message(),
    "2 file(s) cannot be reached, another file claims their path"
  );

  let findings: Vec<(Option<&str>, &str, &str)> = result
    .get_findings()
    .iter()
    .map(|finding| (finding.subject(), finding.rule_id().as_str(), finding.message()))
    .collect();

  assert_eq!(
    findings,
    vec![
      (
        Some("textures/a.dds"),
        "collisions.unreachable",
        "File 'gamedata/textures/A.dds' cannot be reached, 'gamedata/textures/a.dds' claims this path",
      ),
      (
        Some("meshes/b.ogf"),
        "collisions.unreachable",
        "File 'gamedata/meshes/b.ogf' cannot be reached, 'gamedata/meshes/B.OGF' claims this path",
      ),
    ]
  );
}

/// A reachable index is not a passing verdict, or a run whose selected checks judged nothing would read as success.
#[test]
fn adds_nothing_to_the_verdict_when_every_file_is_reachable() {
  let result = project(Vec::new())
    .verify_collisions(&options(Vec::new()))
    .expect("the collisions check completes");

  assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
  assert_eq!(result.get_failure_message(), "No unreachable files");
  assert!(result.get_findings().is_empty());
}

/// The point of the issue: a narrow selection must not hide a file the game cannot load.
#[test]
fn fails_a_run_whose_selected_check_is_unrelated() {
  let project: GamedataProject = project(vec![collision("textures\\a.dds", "textures\\a.dds", "textures\\A.dds")]);

  let report: GamedataVerificationReport = project
    .verify(&options(vec![GamedataVerificationType::Ltx]))
    .expect("verification completes");

  assert_eq!(
    report.get_status(),
    GamedataVerificationStatus::Failed,
    "an unreachable file is a failing project, whichever kinds were selected"
  );

  let checks: Vec<(GamedataVerificationType, GamedataVerificationStatus)> = report
    .get_checks()
    .iter()
    .map(|check| (check.get_verification_type(), check.get_status()))
    .collect();

  assert_eq!(
    checks,
    vec![
      (GamedataVerificationType::Coverage, GamedataVerificationStatus::Skipped),
      (GamedataVerificationType::Collisions, GamedataVerificationStatus::Failed),
      (GamedataVerificationType::Ltx, GamedataVerificationStatus::Passed),
    ],
    "the collisions check runs before the selection rather than inside it"
  );

  assert_eq!(
    report.get_failure_messages(),
    vec![String::from(
      "1 file(s) cannot be reached, another file claims their path"
    )]
  );

  let shared = report.to_report();

  assert_eq!(shared.checks()[1].id().as_str(), "collisions");
  assert_eq!(
    shared.checks()[1].findings()[0].rule_id().as_str(),
    "collisions.unreachable"
  );
  assert_eq!(shared.status(), GamedataVerificationStatus::Failed);
}

/// Selecting nothing is still a usage error, and the always-run check does not paper over it.
#[test]
fn refuses_a_run_with_no_selected_checks() {
  assert!(project(Vec::new()).verify(&options(Vec::new())).is_err());
}

/// A clean project keeps reporting the check, so a consumer can tell "nothing collided" from "never looked".
#[test]
fn reports_the_collisions_check_on_a_clean_run() {
  let report: GamedataVerificationReport = project(Vec::new())
    .verify(&options(vec![GamedataVerificationType::Scripts]))
    .expect("verification completes");

  assert_eq!(report.get_status(), GamedataVerificationStatus::Passed);
  assert_eq!(
    report.get_checks()[1].get_verification_type(),
    GamedataVerificationType::Collisions
  );
  assert_eq!(report.get_checks()[1].get_summary(), "No unreachable files");
}
