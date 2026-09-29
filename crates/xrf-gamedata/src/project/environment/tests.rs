use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_engine_target::XrayEngine;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use super::verify_environment_result::GamedataEnvironmentVerificationResult;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

/// A game tree whose environment configs define what its keyframes name, and whose textures hold its skies.
struct EnvironmentTree {
  root: PathBuf,
}

impl EnvironmentTree {
  fn new() -> Self {
    let unique: u64 = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("gamedata_environment/fixture-{unique}"));

    let _ = fs::remove_dir_all(&root);

    Self { root }
      .with("configs/system.ltx", "")
      .with(
        "configs/environment/suns.ltx",
        "[sun]\nblend_down_time = 60\nblend_rise_time = 60\nflares = off\ngradient = off\nsun = off\n",
      )
      .with("configs/environment/thunderbolt_collections.ltx", "")
      .with("configs/environment/thunderbolts.ltx", "")
      .with(
        "configs/environment/environment.ltx",
        "[environment]\naltitude = 20\ndelta_longitude = 30\nfog_color = 0.1\nmin_dist_factor = 0.9\n\
         second_propability = 0.5\nsky_color = 0.1\nsun_color = 0.9\ntilt = 17\n",
      )
      .with(
        "configs/environment/weathers/day.ltx",
        &(keyframe("00:00:00") + &keyframe("12:00:00")),
      )
      .with("textures/sky/sky_cube.dds", "")
      .with("textures/sky/sky_cube#small.dds", "")
      .with("textures/sky/clouds.dds", "")
  }

  fn with(self, relative: &str, contents: &str) -> Self {
    let path: PathBuf = self.root.join(relative);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();

    self
  }

  fn without(self, relative: &str) -> Self {
    fs::remove_file(self.root.join(relative)).unwrap();

    self
  }

  fn verify(&self, engine: XrayEngine) -> GamedataEnvironmentVerificationResult {
    GamedataProject::open(&GamedataProjectReadOptions {
      engine,
      root: self.root.clone(),
      ..Default::default()
    })
    .unwrap()
    .verify_environment(&GamedataProjectVerifyOptions::default())
    .unwrap()
  }
}

impl Drop for EnvironmentTree {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn keyframe(time: &str) -> String {
  format!(
    "[{time}]
ambient_color = 0, 0, 0
clouds_color = 0, 0, 0, 1
clouds_texture = sky\\clouds
far_plane = 500
fog_color = 0, 0, 0
fog_density = 0.25
fog_distance = 500
hemisphere_color = 0, 0, 0, 1
rain_color = 1, 1, 1
rain_density = 0
sky_color = 1, 1, 1
sky_texture = sky\\sky_cube
sun = sun
sun_altitude = 0
sun_color = 0, 0, 0
sun_longitude = -10
thunderbolt_collection =
wind_direction = 0
wind_velocity = 0

"
  )
}

/// Each finding's rule and message, which is what the report carries.
fn reported(result: &GamedataEnvironmentVerificationResult) -> Vec<(String, String)> {
  result
    .get_findings()
    .iter()
    .map(|finding| (finding.rule_id().to_string(), finding.message().to_owned()))
    .collect()
}

#[test]
fn passes_a_tree_the_engine_loads_whole() {
  let result: GamedataEnvironmentVerificationResult = EnvironmentTree::new().verify(XrayEngine::Vanilla);

  assert_eq!(reported(&result), Vec::new());
  assert_eq!(result.get_failure_message(), "5/5 environment configs valid");
}

#[test]
fn reports_what_the_engine_refuses_under_its_rule() {
  let broken: String = keyframe("00:00:00").replace("far_plane = 500\n", "") + &keyframe("12:00:00");
  let result = EnvironmentTree::new()
    .with("configs/environment/weathers/day.ltx", &broken)
    .verify(XrayEngine::Vanilla);

  assert_eq!(
    reported(&result),
    vec![(
      String::from("environment.engine"),
      String::from("Weather [00:00:00] is missing required field [far_plane]")
    )]
  );
  assert_eq!(result.get_failure_message(), "4/5 environment configs valid");
  assert!(
    result.get_findings()[0]
      .subject()
      .is_some_and(|subject| subject.ends_with("configs/environment/weathers/day.ltx"))
  );
}

#[test]
fn reports_a_sky_the_game_does_not_hold() {
  let result = EnvironmentTree::new()
    .without("textures/sky/sky_cube#small.dds")
    .verify(XrayEngine::Vanilla);

  assert_eq!(
    reported(&result),
    vec![
      (
        String::from("environment.asset"),
        String::from("Weather [00:00:00] references missing sky texture [sky\\sky_cube#small]")
      ),
      (
        String::from("environment.asset"),
        String::from("Weather [12:00:00] references missing sky texture [sky\\sky_cube#small]")
      ),
    ]
  );
}

// Anomaly's cycles name no clouds at all, which draws none rather than a missing texture.
#[test]
fn takes_an_empty_clouds_texture_for_none() {
  let clear: String =
    (keyframe("00:00:00") + &keyframe("12:00:00")).replace("clouds_texture = sky\\clouds", "clouds_texture =");
  let result = EnvironmentTree::new()
    .with("configs/environment/weathers/day.ltx", &clear)
    .without("textures/sky/clouds.dds")
    .verify(XrayEngine::Vanilla);

  assert_eq!(reported(&result), Vec::new());
}

#[test]
fn reads_the_configs_as_the_engine_the_tree_is_for() {
  let result = EnvironmentTree::new().verify(XrayEngine::Extended);

  assert_eq!(
    reported(&result),
    vec![(
      String::from("environment.engine"),
      String::from("There is no environment\\sun_positions.ltx, which the engine stands the sun by")
    )]
  );
}

#[test]
fn stops_reading_when_cancelled() {
  let tree: EnvironmentTree = EnvironmentTree::new();
  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: tree.root.clone(),
    ..Default::default()
  })
  .unwrap();
  let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions::default();

  options.job.cancel();

  assert!(project.verify_environment(&options).is_err());
}
