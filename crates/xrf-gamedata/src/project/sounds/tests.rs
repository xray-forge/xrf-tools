//! Verifies sound references declared in a project's configs.
//!
//! The project addresses its configs logically, so the check must read them through the project. A filesystem read of a
//! logical path resolves nothing, and every config would be skipped as unreadable while the check still reported success.

use std::fs;
use std::path::PathBuf;

use xrf_gamemtl::GameMtlFile;
use xrf_test_utils::GeneratedResources;

use super::sounds_verification_result::GamedataSoundsVerificationResult;
use super::sounds_verifier::SoundsVerifier;
use crate::project::gamedata_check::GamedataCheck;
use crate::project::gamedata_check_result::GamedataCheckResult;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_read_options::GamedataProjectReadOptions;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::tests::fixtures::{engine_game_materials, game_materials_bytes};

fn open_project(name: &str, config: &str) -> (PathBuf, GamedataProject) {
  open_project_with_materials(name, config, None, &[])
}

/// A project of one weapon config and one sound, with the game materials and the further files, each by its path under
/// the root, given.
fn open_project_with_materials(
  name: &str,
  config: &str,
  game_materials: Option<&GameMtlFile>,
  files: &[(&str, &str)],
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

  for (path, content) in files {
    let path: PathBuf = root.join(path);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
  }

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    ..Default::default()
  })
  .unwrap();

  (root, project)
}

fn verify(project: &GamedataProject) -> GamedataSoundsVerificationResult {
  SoundsVerifier::new(project, &GamedataProjectVerifyOptions::default())
    .verify()
    .expect("sounds verified")
}

#[test]
fn resolves_a_sound_reference_declared_in_a_config() {
  let (root, project) = open_project("resolved", "[wpn_ak74]\nsnd_shoot = weapons\\ak74_shot\n");
  let message: String = verify(&project).get_summary();

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

  let (root, project) = open_project_with_materials("pairs", "", Some(&library), &[]);
  let result: GamedataSoundsVerificationResult = verify(&project);
  let message: String = result.get_summary();

  assert!(message.contains("1/2 sound references valid"), "{message}");
  assert_eq!(
    result.get_findings()[0].message(),
    "Unknown sound reference: step sounds of 2 pairs = weapons\\missing"
  );

  fs::remove_dir_all(root).unwrap();
}

#[test]
fn reads_xml_references_as_the_engine_does_through_an_include_and_a_comment_banner() {
  // The part has no root, and the banner's `--` is what a strict reader refuses: both read as the engine reads them.
  let (root, project) = open_project_with_materials(
    "xml",
    "",
    None,
    &[
      (
        "configs/gameplay/character_desc.xml",
        "<?xml version=\"1.0\" encoding=\"windows-1251\"?>\r\n<!-- -- characters -- -->\r\n<characters>\r\n\
         #include \"gameplay\\character_sounds.xml\"\r\n</characters>\r\n",
      ),
      (
        "configs/gameplay/character_sounds.xml",
        "<sound>weapons\\ak74_shot</sound>\r\n<sound>weapons\\missing</sound>\r\n",
      ),
    ],
  );

  let result: GamedataSoundsVerificationResult = verify(&project);
  let message: String = result.get_summary();

  let references: Vec<&str> = result
    .get_findings()
    .iter()
    .map(|finding| finding.message())
    .filter(|message| message.contains("sound reference"))
    .collect();

  assert!(message.contains("1/2 sound references valid"), "{message}");
  assert_eq!(references, ["Unknown sound reference: <sound> = weapons\\missing"]);

  fs::remove_dir_all(root).unwrap();
}

#[test]
fn reports_a_config_reference_with_no_sound_file() {
  let (root, project) = open_project("missing", "[wpn_ak74]\nsnd_shoot = weapons\\missing\n");
  let message: String = verify(&project).get_summary();

  assert!(
    message.contains("0/1 sound references valid"),
    "the missing reference is reported rather than skipped: {message}"
  );

  fs::remove_dir_all(root).unwrap();
}
