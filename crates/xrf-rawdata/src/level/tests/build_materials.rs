//! A build of a scene placing objects needs the game materials its spawn list's ladders and breakables are cut by.

use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableBuildParams, EditableCustomObject, EditableLevelCamera, EditableLevelFile, EditableLevelOptions,
  EditableLevelScene, EditableObjectPart, EditableObjectTransform, EditableSceneObject,
};
use xrf_error::XrfResult;
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

/// The ladders and breakables a build cuts out of placed objects by game material open the spawn list, so a scene
/// placing objects is not built without the game materials, and nothing is written.
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
  .write_to_path::<XRayByteOrder, _>(&source)?;

  let error: String = LevelBuildExporter::build_to_directory::<XRayByteOrder>(
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
