//! Verifies sound references declared in a project's configs.
//!
//! The project addresses its configs logically, so the check must read them through the project. A filesystem read of a
//! logical path resolves nothing, and every config would be skipped as unreadable while the check still reported success.

use std::fs;
use std::path::PathBuf;

use xrf_gamemtl::GameMtlFile;
use xrf_test_utils::GeneratedResources;

use super::sounds_verification_result::GamedataSoundsVerificationResult;
use crate::project::tests::fixtures::{engine_game_materials, game_materials_bytes};
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions};

fn open_project(name: &str, config: &str) -> (PathBuf, GamedataProject) {
  open_project_with_materials(name, config, None)
}

fn open_project_with_materials(
  name: &str,
  config: &str,
  game_materials: Option<&GameMtlFile>,
) -> (PathBuf, GamedataProject) {
  let root: PathBuf = GeneratedResources::create_dir(&format!("sounds/{name}")).unwrap();
  let configs: PathBuf = root.join("configs");

  fs::create_dir_all(configs.join("weapons")).unwrap();
  fs::create_dir_all(root.join("sounds").join("weapons")).unwrap();
  fs::write(configs.join("system.ltx"), "").unwrap();
  fs::write(configs.join("weapons").join("wpn.ltx"), config).unwrap();
  fs::write(root.join("sounds").join("weapons").join("ak74_shot.ogg"), "").unwrap();

  if let Some(game_materials) = game_materials {
    fs::write(root.join(GameMtlFile::FILE_NAME), game_materials_bytes(game_materials)).unwrap();
  }

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
fn resolves_each_sound_material_pairs_list_once() {
  let mut library: GameMtlFile = engine_game_materials();

  library.pairs[0].step_sounds = String::from("weapons\\ak74_shot, weapons\\missing");
  library.pairs.push(library.pairs[0].clone());

  let (root, project) = open_project_with_materials("pairs", "", Some(&library));
  let result: GamedataSoundsVerificationResult = verify(&project);
  let message: String = result.get_failure_message();

  assert!(message.contains("1/2 sound references valid"), "{message}");
  assert_eq!(
    result.get_findings()[0].message(),
    "Unknown sound reference: step sounds of 2 pairs = weapons\\missing"
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
