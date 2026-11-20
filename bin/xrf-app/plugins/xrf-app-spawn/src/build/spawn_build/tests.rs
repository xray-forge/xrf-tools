use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::json;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_job::{ExecutionRequest, JobHandle, JobOutcome};
use xrf_ltx::LtxStandardDialect;
use xrf_spawn_builder::{SpawnRootOptions, SpawnRootRequest};
use xrf_test_utils::GeneratedResources;

use crate::build::SpawnBuild;

// Lists `labx8` alone and its built files are empty, so a build reads the list and refuses the level's files.
fn request(case: &str) -> SpawnRootRequest {
  let root: PathBuf = GeneratedResources::create_dir(&format!("spawn/build/{case}")).expect("scratch directory");
  let (gamedata, levels) = (root.join("gamedata"), root.join("levels"));

  fs::create_dir_all(gamedata.join("configs")).expect("configs directory");
  fs::write(
    gamedata.join("configs/game_levels.ltx"),
    "[levels]\nlevel150\n\n[level150]\nname = labx8\noffset = 0, 0, 0\nid = 150\n",
  )
  .expect("level list");
  fs::create_dir_all(levels.join("labx8")).expect("level directory");

  for file in ["level.ai", "level.spawn", "level.game"] {
    fs::write(levels.join("labx8").join(file), b"").expect("level file");
  }

  SpawnRootRequest {
    gamedata,
    levels_root: Some(levels),
    levels: Some(vec![String::from("labx8")]),
    destination: Some(root.join("spawn")),
    name: String::from("qa_levels"),
    options: SpawnRootOptions {
      start: Some(String::from("labx8")),
      is_separator_strict: false,
    },
    is_forced: false,
  }
}

#[test]
fn the_page_asks_for_the_build_the_cli_runs() {
  let request: SpawnRootRequest = serde_json::from_value(json!({
    "gamedata": "D:\\qa\\gamedata",
    "levelsRoot": "D:\\qa\\gamedata\\levels",
    "levels": ["zaton", "labx8"],
    "destination": "D:\\qa\\spawn",
    "name": "qa_levels",
    "options": { "start": "labx8", "isSeparatorStrict": false },
    "isForced": false
  }))
  .expect("the page's request");

  assert_eq!(request.get_levels_root(), Path::new("D:\\qa\\gamedata\\levels"));
  assert_eq!(
    request.get_spawn_path(),
    Path::new("D:\\qa\\spawn").join("qa_levels.spawn")
  );
  assert_eq!(request.options.start.as_deref(), Some("labx8"));
  assert_eq!(request.levels, Some(vec![String::from("zaton"), String::from("labx8")]));
}

#[test]
fn a_build_cancelled_before_it_writes_ends_cancelled_with_nothing_written() {
  let request: SpawnRootRequest = request("cancelled");
  let job: JobHandle = JobHandle::inert();

  job.cancel();

  let result = SpawnBuild::build(&request, &LtxStandardDialect, &job).expect("a cancelled build");

  assert_eq!(result.outcome, JobOutcome::Cancelled);
  assert!(result.spawn.is_none());
  assert!(!request.get_spawn_path().exists());
}

#[test]
fn a_refused_build_fails_with_its_reason_and_releases_its_spawn() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto).expect("execution pool");
  let id: Uuid = Uuid::new_v4();
  let run = |request: SpawnRootRequest, id: Uuid| {
    tauri::async_runtime::block_on(registry.run(
      &execution,
      SpawnBuild::start(&request, id, Channel::new(|_| Ok(()))),
      move |job| SpawnBuild::build(&request, &LtxStandardDialect, &job),
      |result| result.outcome,
    ))
  };

  let refused: SpawnRootRequest = request("refused");
  let spawn: PathBuf = refused.get_spawn_path();
  let error: String = run(refused, id).expect_err("a refused build");

  assert!(error.contains("Level labx8:"), "Unexpected error: {error}");
  assert!(!spawn.exists());

  let listed = registry
    .list()
    .into_iter()
    .find(|job| job.id == id)
    .expect("retained job");

  assert_eq!(serde_json::to_value(listed.conclusion).unwrap(), json!("failed"));

  // A level the game does not list is refused by name, and the lease of the first run is gone.
  let unlisted: SpawnRootRequest = SpawnRootRequest {
    levels: Some(vec![String::from("jupiter")]),
    ..request("unlisted")
  };
  let error: String = run(unlisted, Uuid::new_v4()).expect_err("an unlisted level");

  assert!(error.contains("Level jupiter is not one"), "Unexpected error: {error}");
}

#[test]
fn one_build_runs_at_a_time() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let (first, second) = (request("lease-first"), request("lease-second"));
  let (_job, registration) = registry
    .register_fixture(SpawnBuild::start(&first, Uuid::new_v4(), Channel::new(|_| Ok(()))))
    .expect("the first build registers");

  assert!(
    registry
      .register_fixture(SpawnBuild::start(&second, Uuid::new_v4(), Channel::new(|_| Ok(()))))
      .is_err()
  );

  drop(registration);

  assert!(
    registry
      .register_fixture(SpawnBuild::start(&second, Uuid::new_v4(), Channel::new(|_| Ok(()))))
      .is_ok()
  );
}
