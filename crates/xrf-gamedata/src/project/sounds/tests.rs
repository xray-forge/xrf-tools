//! Verifies sound references declared in a project's configs.
//!
//! The project addresses its configs logically, so the check must read them through the project. A filesystem read of a
//! logical path resolves nothing, and every config would be skipped as unreadable while the check still reported success.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use super::sounds_verification_result::GamedataSoundsVerificationResult;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions};

fn open_project(name: &str, config: &str) -> (PathBuf, GamedataProject) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("sounds/{name}"));
  let configs: PathBuf = root.join("configs");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(configs.join("weapons")).unwrap();
  fs::create_dir_all(root.join("sounds").join("weapons")).unwrap();
  fs::write(configs.join("system.ltx"), "").unwrap();
  fs::write(configs.join("weapons").join("wpn.ltx"), config).unwrap();
  fs::write(root.join("sounds").join("weapons").join("ak74_shot.ogg"), "").unwrap();

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();

  (root, project)
}

fn verify(project: &GamedataProject) -> GamedataSoundsVerificationResult {
  project
    .verify_sounds(&GamedataProjectVerifyOptions::default())
    .expect("sounds verified")
}

#[test]
fn resolves_a_sound_reference_declared_in_a_config() {
  let (root, project) = open_project("resolved", "[wpn_ak74]\nsnd_shoot = weapons\\ak74_shot\n");
  let message: String = verify(&project).get_failure_message();

  assert!(
    message.contains("1/1 sound references valid"),
    "the config reference is read and resolved: {message}"
  );

  fs::remove_dir_all(root).unwrap();
}

#[test]
fn reports_a_config_reference_with_no_sound_file() {
  let (root, project) = open_project("missing", "[wpn_ak74]\nsnd_shoot = weapons\\missing\n");
  let message: String = verify(&project).get_failure_message();

  assert!(
    message.contains("0/1 sound references valid"),
    "the missing reference is reported rather than skipped: {message}"
  );

  fs::remove_dir_all(root).unwrap();
}
