//! What a verification sweep reports about how far it has got.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use xrf_job::{JobHandle, JobProgress, RecordingSink};
use xrf_ltx::{LtxFormatOptions, LtxVerifyOptions};
use xrf_test_utils::GeneratedResources;

use crate::project::gamedata_project_verify::PHASE_CHECKS;
use crate::{GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationType};

/// Opens a gamedata root holding only `configs/system.ltx`, which every check can run over.
fn open(name: &str) -> (GamedataProject, PathBuf) {
  let root: PathBuf = GeneratedResources::path(&format!("gamedata_progress/{name}"));

  fs::create_dir_all(root.join("configs")).expect("configs directory");
  fs::write(root.join("configs/system.ltx"), "[system]\nversion = 1\n").expect("system config");

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    ..Default::default()
  })
  .expect("project opens");

  (project, root)
}

/// Every snapshot a sweep over `checks` reported, one per unit.
fn verify(project: &GamedataProject, checks: Vec<GamedataVerificationType>) -> Vec<JobProgress> {
  let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());

  project
    .verify(&GamedataProjectVerifyOptions {
      job: JobHandle::with_interval(sink.clone(), Duration::ZERO),
      checks,
      ..Default::default()
    })
    .expect("selected checks run");

  sink.take_reported()
}

/// The checks level's `(completed, total)` in every snapshot that shows it.
fn checks_counts(reported: &[JobProgress]) -> Vec<(u64, Option<u64>)> {
  reported
    .iter()
    .filter_map(|progress| progress.levels.first())
    .filter(|level| level.id == PHASE_CHECKS)
    .map(|level| (level.completed, level.total))
    .collect()
}

#[test]
fn a_sweep_counts_each_selected_check_once() {
  let (project, root) = open("counts");
  let without_ltx: Vec<GamedataVerificationType> = GamedataVerificationType::get_all()
    .into_iter()
    .filter(|check| *check != GamedataVerificationType::Ltx)
    .collect();

  // The LTX and environment checks enter levels of their own; their exits used to count against the checks level on
  // top of the check itself, so a full sweep passed its total (issue 0154).
  for (selection, expected) in [
    (
      GamedataVerificationType::get_all(),
      GamedataVerificationType::ALL.len() as u64,
    ),
    (without_ltx, GamedataVerificationType::ALL.len() as u64 - 1),
  ] {
    let reported: Vec<JobProgress> = verify(&project, selection);
    let counts: Vec<(u64, Option<u64>)> = checks_counts(&reported);

    assert!(
      counts.iter().all(|(completed, total)| Some(*completed) <= *total),
      "a count passed its total: {counts:?}"
    );
    assert_eq!(counts.last(), Some(&(expected, Some(expected))));
    // Checks running together report nothing of their own: one stack would show them nested in each other.
    assert!(
      reported.iter().all(|progress| progress.levels.len() == 1),
      "a check running beside others reported a level: {:?}",
      reported.iter().map(|progress| progress.levels.len()).max()
    );
  }

  fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn a_check_running_alone_reports_its_own_levels_beneath_the_checks_level() {
  let (project, root) = open("alone");
  let reported: Vec<JobProgress> = verify(&project, vec![GamedataVerificationType::Ltx]);
  let nested: Vec<Vec<&str>> = reported
    .iter()
    .filter(|progress| progress.levels.len() > 1)
    .map(|progress| progress.levels.iter().map(|level| level.id.as_str()).collect())
    .collect();

  assert!(
    nested.contains(&vec![PHASE_CHECKS, LtxFormatOptions::PHASE_CHECK]),
    "{nested:?}"
  );
  assert!(
    nested.contains(&vec![PHASE_CHECKS, LtxVerifyOptions::PHASE_VERIFY]),
    "{nested:?}"
  );
  assert_eq!(checks_counts(&reported).last(), Some(&(1, Some(1))));

  fs::remove_dir_all(root).expect("cleanup");
}
