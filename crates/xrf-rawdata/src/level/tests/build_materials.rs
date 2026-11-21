use std::path::PathBuf;

use xrf_chunk::ChunkWriter;
use xrf_editable_level::{
  EditableBuildParams, EditableCustomObject, EditableLevelCamera, EditableLevelFile, EditableLevelOptions,
  EditableLevelScene, EditableObjectPart, EditableObjectTransform, EditableSceneObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_job::JobHandle;
use xrf_test_utils::GeneratedResources;

use crate::level::build::level_build_exporter::LevelBuildExporter;
use crate::level::build::level_build_request::LevelBuildRequest;
use crate::level::tests::fixtures::GUID;

fn scene_object(name: &str) -> EditableSceneObject {
  EditableSceneObject {
    object: EditableCustomObject::new(name, EditableObjectTransform::default()),
    reference: String::from("statics\\stol\\stol_4"),
    flags: 0,
  }
}

// A build cuts ladders and breakables out of placed objects by game material.
#[test]
fn refuses_to_build_a_scene_placing_objects_without_the_game_materials() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "no_materials"))?;
  let source: PathBuf = root.join("scenes").join("test_level.level");
  let destination: PathBuf = root.join("level");

  EditableLevelScene {
    level: EditableLevelFile::new(
      &GUID,
      &EditableLevelOptions::new("test_level", EditableLevelOptions::SINGLE_PLAYER),
      &EditableBuildParams::DEFAULT,
      &EditableLevelCamera::default(),
      "test",
      0,
    ),
    tools: vec![EditableObjectPart::new(GUID, vec![scene_object("own_0")]).to_scene_tool()?],
  }
  .write_to_path(&source)?;

  let error: String = LevelBuildExporter::build_to_directory(
    &LevelBuildRequest {
      source,
      destination: destination.clone(),
      ..LevelBuildRequest::default()
    },
    &JobHandle::inert(),
  )
  .expect_err("a scene of objects is refused without game materials")
  .to_string();

  assert!(error.contains("name the game materials"), "{error}");
  assert!(!destination.exists(), "a refusal writes nothing");

  Ok(())
}

#[test]
fn hands_the_compilers_files_to_the_check_before_writing_any() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "checked"))?;
  let source: PathBuf = root.join("scenes").join("test_level.level");
  let game_materials: PathBuf = root.join("gamemtl.xr");
  let destination: PathBuf = root.join("level");
  let mut writer: ChunkWriter = ChunkWriter::new();

  GameMtlFile::new(Vec::new(), Vec::new()).write(&mut writer)?;
  std::fs::write(&game_materials, writer.buffer)?;
  std::fs::create_dir_all(root.join("objects"))?;

  EditableLevelScene {
    level: EditableLevelFile::new(
      &GUID,
      &EditableLevelOptions::new("test_level", EditableLevelOptions::SINGLE_PLAYER),
      &EditableBuildParams::DEFAULT,
      &EditableLevelCamera::default(),
      "test",
      0,
    ),
    tools: Vec::new(),
  }
  .write_to_path(&source)?;

  let mut checked: Vec<&str> = Vec::new();
  let error: String = LevelBuildExporter::build_to_directory_checked(
    &LevelBuildRequest {
      source,
      destination: destination.clone(),
      game_materials: Some(game_materials),
      objects: vec![root.join("objects")],
      ..LevelBuildRequest::default()
    },
    &JobHandle::inert(),
    |export| {
      checked = export.get_file_names();

      Err(XrfError::new_invalid_error("refused by the check"))
    },
  )
  .expect_err("the check refuses the build")
  .to_string();

  assert!(error.contains("refused by the check"), "{error}");
  // A scene of nothing makes no details, AI map, occluders or impostors.
  assert_eq!(checked, ["build.prj", "level.ltx"]);
  assert!(!destination.exists(), "a refused check writes nothing");

  Ok(())
}
