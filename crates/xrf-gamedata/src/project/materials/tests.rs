//! Verifies the game material library and the names resolved against it, over projects written to disk.

use std::fs;
use std::path::PathBuf;

use xrf_gamemtl::GameMtlFile;
use xrf_test_utils::GeneratedResources;

use crate::project::materials::verify_materials_result::GamedataMaterialsVerificationResult;
use crate::project::tests::fixtures::{engine_game_materials, game_materials_bytes};
use crate::{
  GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions,
  GamedataVerificationStatus,
};

fn verify(name: &str, library: Option<&GameMtlFile>, config: &str) -> GamedataMaterialsVerificationResult {
  let root: PathBuf = GeneratedResources::create_dir(&format!("materials/{name}")).unwrap();

  fs::create_dir_all(root.join("configs")).unwrap();
  fs::create_dir_all(root.join("textures").join("wm")).unwrap();
  fs::write(root.join("configs").join("system.ltx"), config).unwrap();
  fs::write(root.join("textures").join("wm").join("wm_hit.dds"), [1]).unwrap();

  if let Some(library) = library {
    fs::write(root.join(GameMtlFile::FILE_NAME), game_materials_bytes(library)).unwrap();
  }

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();

  let result: GamedataMaterialsVerificationResult = project
    .verify_materials(&GamedataProjectVerifyOptions::default())
    .expect("materials verified");

  fs::remove_dir_all(root).unwrap();

  result
}

fn messages(result: &GamedataMaterialsVerificationResult) -> Vec<String> {
  result
    .get_findings()
    .iter()
    .map(|finding| format!("{}: {}", finding.rule_id(), finding.message()))
    .collect()
}

#[test]
fn passes_a_library_the_engine_loads_and_configs_that_resolve_against_it() {
  let mut library: GameMtlFile = engine_game_materials();

  library.pairs[0].collide_marks = String::from("wm\\wm_hit");

  let result: GamedataMaterialsVerificationResult =
    verify("passing", Some(&library), "[stalker]\nmaterial = Materials\\Earth\n");

  assert_eq!(
    result.get_status(),
    GamedataVerificationStatus::Passed,
    "{:?}",
    messages(&result)
  );
  assert_eq!(
    result.get_failure_message(),
    "7 materials, 1 pairs; 9/9 material references valid"
  );
}

#[test]
fn fails_without_a_library() {
  let result: GamedataMaterialsVerificationResult = verify("missing", None, "");

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_eq!(result.get_failure_message(), "Game material library unreadable");
  assert_eq!(result.get_findings()[0].rule_id().to_string(), "materials.library");
}

#[test]
fn reports_what_the_engine_looks_up_by_name_and_does_not_find_as_it_needs() {
  let mut library: GameMtlFile = engine_game_materials();

  library.materials.retain(|material| material.name != "objects\\knife");
  library.materials[1].flags = 0;

  assert_eq!(
    messages(&verify("engine", Some(&library), "")),
    [
      "materials.engine: The engine asserts material 'default_object' is dynamic, and the library defines it as static",
      "materials.engine: The engine looks up material 'objects\\knife', which the library does not define",
    ]
  );
}

#[test]
fn reports_a_pair_naming_an_undefined_material_and_lists_past_the_engine_limit() {
  let mut library: GameMtlFile = engine_game_materials();

  library.pairs[0].material_b = 42;
  library.pairs[0].step_sounds = String::from("a,b,c,d,e,f,g");
  library.pairs[0].collide_marks = String::from("wm\\wm_hit,wm\\missing");

  assert_eq!(
    messages(&verify("pairs", Some(&library), "")),
    [
      "materials.list: Pair materials\\earth - 42 lists 7 step sounds, more than the 6 the engine loads",
      "materials.mark: Wallmark texture 'wm\\missing' named by 1 pairs is missing",
      "materials.pair: Pair materials\\earth - 42 names material 42, which the library does not define",
    ]
  );
}

#[test]
fn reports_a_config_naming_an_undefined_material() {
  let result: GamedataMaterialsVerificationResult = verify(
    "config",
    Some(&engine_game_materials()),
    "[m_dog]\nmaterial = creatures\\medium ; a comment is not part of the name\n",
  );

  assert_eq!(
    messages(&result),
    ["materials.reference: Unknown game material: [m_dog] material = creatures\\medium"]
  );
  assert_eq!(
    result.get_failure_message(),
    "7 materials, 1 pairs; 7/8 material references valid"
  );
}
