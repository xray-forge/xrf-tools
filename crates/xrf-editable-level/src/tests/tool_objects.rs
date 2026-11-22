use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_file::EditableLevelFile;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_object_transform::EditableObjectTransform;
use crate::object::editable_scene_object::EditableSceneObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_level_part::EditableLevelPart;
use crate::part::editable_object_part::EditableObjectPart;
use crate::part::editable_text_part::EditableTextPart;
use crate::scene::editable_group_tool::EditableGroupTool;
use crate::scene::editable_level_scene::EditableLevelScene;
use crate::scene::editable_scene_tool::EditableSceneTool;
use crate::scene::editable_tool_objects::EditableToolObjects;
use crate::tests::fixtures::GUID;

fn scene_object(name: &str) -> EditableSceneObject {
  EditableSceneObject {
    object: EditableCustomObject::new(name, EditableObjectTransform::default()),
    reference: String::from("statics\\stol\\stol_4"),
    flags: 0,
  }
}

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

#[test]
fn reads_the_groups_members_apart_from_the_tools_own_objects_and_merges_them_first() -> XrfResult {
  let scene: EditableLevelScene = scene(vec![
    group_tool("lab_group", &[scene_object("grouped")]),
    EditableObjectPart::new(GUID, vec![scene_object("own_0"), scene_object("own_1")]).to_scene_tool()?,
  ]);
  let read: EditableToolObjects<EditableSceneObject> = EditableToolObjects::read(&scene, GUID)?;

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
fn keeps_the_groups_members_where_the_tools_own_part_does_not_read() -> XrfResult {
  // A part holding no `main` section, which no tool's part reads without.
  let unreadable: EditableSceneTool = EditableSceneTool {
    name: String::from(EditableSceneObject::TOOL_NAME),
    parts: vec![EditableLevelPart::Text(EditableTextPart {
      ini: IniFile::default(),
    })],
  };
  let scene: EditableLevelScene = scene(vec![group_tool("lab_group", &[scene_object("grouped")]), unreadable]);

  assert!(EditableToolObjects::<EditableSceneObject>::read(&scene, GUID).is_err());

  let (read, error) = EditableToolObjects::<EditableSceneObject>::read_keeping_members(&scene, GUID)?;

  assert!(error.is_some(), "the own part's error kept beside the members");
  assert_eq!(read.part, EditableObjectPart::new(GUID, Vec::new()));
  assert_eq!(
    read
      .iter_indexed()
      .map(|(index, object)| (index, object.object.name.as_str()))
      .collect::<Vec<_>>(),
    [(0, "grouped")],
    "the members indexed as when the part reads"
  );

  Ok(())
}

#[test]
fn a_tool_the_scene_lacks_is_an_empty_part_holding_its_members() -> XrfResult {
  let read: EditableToolObjects<EditableSceneObject> =
    EditableToolObjects::read(&scene(vec![group_tool("lab_group", &[scene_object("grouped")])]), GUID)?;

  assert_eq!(read.part, EditableObjectPart::new(GUID, Vec::new()));
  assert_eq!(read.members.len(), 1);

  Ok(())
}
