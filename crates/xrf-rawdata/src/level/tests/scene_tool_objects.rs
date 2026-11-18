use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableBuildParams, EditableCustomObject, EditableGroupTool, EditableLevelCamera, EditableLevelFile,
  EditableLevelOptions, EditableLevelPart, EditableLevelScene, EditableObjectPart, EditableObjectTransform,
  EditableSceneObject, EditableSceneTool, EditableTextPart, EditableToolObject,
};
use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniSection};
use xrf_job::JobHandle;
use xrf_test_utils::GeneratedResources;

use crate::level::build::level_build_exporter::LevelBuildExporter;
use crate::level::build::level_build_request::LevelBuildRequest;
use crate::level::level_scene_tool_objects::LevelSceneToolObjects;
use crate::level::tests::fixtures::GUID;

fn scene_object(name: &str) -> EditableSceneObject {
  EditableSceneObject {
    object: EditableCustomObject::new(name, EditableObjectTransform::default()),
    reference: String::from("statics\\stol\\stol_4"),
    flags: 0,
  }
}

/// The group tool holding one group, `name`, of `members`.
fn group_tool(name: &str, members: &[EditableSceneObject]) -> EditableSceneTool {
  let mut ini: IniFile = IniFile::default();

  GUID.write_to_section(ini.section_mut("guid"));
  ini.section_mut("main").set("objects_count", "1");
  ini.section_mut("main").set("version", "0");

  let group: &mut IniSection = ini.section_mut("object_0");

  group.set("clsid", EditableGroupTool::CLASS_ID.to_string());
  group.set("name", name);
  group.set(EditableGroupTool::MEMBER_COUNT_KEY, members.len().to_string());

  for (index, member) in members.iter().enumerate() {
    let section: String = format!("object_0_ingroup_{index}");

    ini
      .section_mut(&section)
      .set("clsid", EditableSceneObject::CLASS_ID.to_string());
    member.write_to_ini(&mut ini, &section);
  }

  EditableSceneTool {
    name: String::from(EditableGroupTool::TOOL_NAME),
    parts: vec![EditableLevelPart::Text(EditableTextPart { ini })],
  }
}

fn scene(tools: Vec<EditableSceneTool>) -> EditableLevelScene {
  let mut level: EditableLevelFile = EditableLevelFile::default();

  level.set_guid(&GUID);

  EditableLevelScene { level, tools }
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

#[test]
fn reads_the_groups_members_apart_from_the_tools_own_objects_and_merges_them_first() -> XrfResult {
  let scene: EditableLevelScene = scene(vec![
    group_tool("lab_group", &[scene_object("grouped")]),
    EditableObjectPart::new(GUID, vec![scene_object("own_0"), scene_object("own_1")]).to_scene_tool()?,
  ]);
  let read: LevelSceneToolObjects<EditableSceneObject> = LevelSceneToolObjects::read(&scene, GUID)?;

  assert_eq!(
    read
      .members
      .iter()
      .map(|member| (member.group.as_str(), member.object.object.name.as_str()))
      .collect::<Vec<_>>(),
    [("lab_group", "grouped")]
  );
  assert_eq!(read.part.objects.len(), 2, "the tool's own objects apart");
  assert_eq!(
    read
      .into_part()
      .objects
      .iter()
      .map(|object| object.object.name.as_str())
      .collect::<Vec<_>>(),
    ["grouped", "own_0", "own_1"],
    "the members first, as the editor loads the group tool first"
  );

  Ok(())
}

#[test]
fn a_tool_the_scene_lacks_is_an_empty_part_holding_its_members() -> XrfResult {
  let read: LevelSceneToolObjects<EditableSceneObject> =
    LevelSceneToolObjects::read(&scene(vec![group_tool("lab_group", &[scene_object("grouped")])]), GUID)?;

  assert_eq!(read.part, EditableObjectPart::new(GUID, Vec::new()));
  assert_eq!(read.members.len(), 1);

  Ok(())
}
