use xrf_chunk::ChunkLayout;
use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat};

use crate::tests::fixtures::GUID;
use crate::{
  EditableBinaryPart, EditableCustomObject, EditableGroupMember, EditableGroupTool, EditableLevelPart,
  EditableObjectTransform, EditableSceneObject, EditableSceneTool, EditableTextPart, EditableToolObject,
};

/// One group part as GSC's `labx8` saves them: groups (class 0) up to `group`, the last of a member per class of
/// `classes`, each member's fields a scene object's at its own place in the level, the second carrying a section named
/// after it.
fn group_part(group: usize, classes: &[i32]) -> XrfResult<EditableLevelPart> {
  let mut file: IniFile = IniFile::default();
  let name: String = format!("object_{group}");

  GUID.write_to_section(file.section_mut("guid"));
  file.section_mut("main").set("objects_count", (group + 1).to_string());
  file.section_mut("main").set("version", "0");
  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  for index in 0..=group {
    let section = file.section_mut(&format!("object_{index}"));

    section.set("clsid", "0");
    EditableCustomObject::new(format!("group_{index}"), EditableObjectTransform::default()).write_to_section(section);
    section.set(
      EditableGroupTool::MEMBER_COUNT_KEY,
      if index == group { classes.len() } else { 0 }.to_string(),
    );
  }

  for (member, class) in classes.iter().enumerate() {
    let section = file.section_mut(&format!("{name}_ingroup_{member}"));

    for (key, value) in [
      ("clsid", class.to_string()),
      ("co_flags", String::from("0")),
      ("flags", String::from("0")),
      ("name", format!("member_{member}")),
      ("position", format!("{member}.000000, 2.000000, 3.000000")),
      ("reference_name", String::from("statics\\stol\\stol_4")),
      ("rotation", String::from("0.000000, 0.000000, 0.000000")),
      ("scale", String::from("1.000000, 1.000000, 1.000000")),
      ("version", String::from("18")),
    ] {
      section.set(key, value);
    }

    if member == 1 {
      file
        .section_mut(&format!("{name}_ingroup_{member}_spawndata"))
        .set("000001", "1");
    }
  }

  Ok(EditableLevelPart::Text(EditableTextPart::read_from_bytes(
    file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)?.as_bytes(),
  )?))
}

#[test]
fn reads_each_groups_members_of_a_class_in_place_across_its_files() -> XrfResult {
  let class: i32 = EditableSceneObject::CLASS_ID;
  let tool: EditableSceneTool = EditableSceneTool {
    name: String::from(EditableGroupTool::TOOL_NAME),
    parts: vec![group_part(0, &[class, class, 6])?, group_part(1, &[class])?],
  };
  let members: Vec<EditableGroupMember<EditableSceneObject>> = EditableGroupTool::read_grouped_members(&tool)?;

  assert_eq!(
    members
      .iter()
      .map(|member| (
        member.object.object.name.as_str(),
        member.object.object.transform.position.x
      ))
      .collect::<Vec<_>>(),
    [("member_0", 0.0), ("member_1", 1.0), ("member_0", 0.0)],
    "the members of the class, at their own places, the second group's in the second file"
  );

  Ok(())
}

/// The SDK saves sections sorted as strings, `ingroup_0, ingroup_1, ingroup_10, ingroup_11, ingroup_2`; the editor loads
/// them by number.
#[test]
fn reads_a_groups_members_in_the_editors_numeric_order() -> XrfResult {
  let tool: EditableSceneTool = EditableSceneTool {
    name: String::from(EditableGroupTool::TOOL_NAME),
    parts: vec![group_part(0, &[EditableSceneObject::CLASS_ID; 12])?],
  };
  let members: Vec<EditableGroupMember<EditableSceneObject>> = EditableGroupTool::read_grouped_members(&tool)?;

  assert_eq!(
    members
      .iter()
      .map(|member| member.object.object.name.clone())
      .collect::<Vec<_>>(),
    (0..12).map(|index| format!("member_{index}")).collect::<Vec<_>>()
  );

  Ok(())
}

/// A member section the count leaves out, or a counted member with no section, is refused with a reason rather than
/// read or dropped.
#[test]
fn refuses_a_member_past_the_count_and_a_counted_member_missing() -> XrfResult {
  for count in ["1", "3"] {
    let EditableLevelPart::Text(mut part) = group_part(0, &[EditableSceneObject::CLASS_ID; 2])? else {
      unreachable!("the fixture is a text part");
    };

    part
      .ini
      .section_mut("object_0")
      .set(EditableGroupTool::MEMBER_COUNT_KEY, count);

    let tool: EditableSceneTool = EditableSceneTool {
      name: String::from(EditableGroupTool::TOOL_NAME),
      parts: vec![EditableLevelPart::Text(part)],
    };

    assert!(
      EditableGroupTool::read_grouped_members::<EditableSceneObject>(&tool).is_err(),
      "a count of {count} for two member sections is refused"
    );
  }

  Ok(())
}

#[test]
fn reads_each_member_with_the_name_of_its_group() -> XrfResult {
  let tool: EditableSceneTool = EditableSceneTool {
    name: String::from(EditableGroupTool::TOOL_NAME),
    parts: vec![
      group_part(1, &[EditableSceneObject::CLASS_ID, 6])?,
      group_part(0, &[EditableSceneObject::CLASS_ID])?,
    ],
  };
  let members: Vec<EditableGroupMember<EditableSceneObject>> = EditableGroupTool::read_grouped_members(&tool)?;

  assert_eq!(
    members
      .iter()
      .map(|member| (member.group.as_str(), member.object.object.name.as_str()))
      .collect::<Vec<_>>(),
    [("group_1", "member_0"), ("group_0", "member_0")],
    "each member named with the group holding it, in the order the files hold them"
  );

  Ok(())
}

#[test]
fn reads_every_group_across_its_files_and_refuses_a_binary_part() -> XrfResult {
  let tool: EditableSceneTool = EditableSceneTool {
    name: String::from(EditableGroupTool::TOOL_NAME),
    parts: vec![group_part(0, &[EditableSceneObject::CLASS_ID])?, group_part(1, &[])?],
  };

  assert_eq!(
    EditableGroupTool::read_groups(&tool)?
      .iter()
      .map(|group| group.name.as_str())
      .collect::<Vec<_>>(),
    ["group_0", "group_0", "group_1"],
    "every group of each file, in the order the files hold them"
  );

  let binary: EditableSceneTool = EditableSceneTool {
    name: String::from(EditableGroupTool::TOOL_NAME),
    parts: vec![EditableLevelPart::Binary(EditableBinaryPart {
      guid: GUID,
      class_id: 0,
      data: Vec::new(),
      layout: ChunkLayout::default(),
    })],
  };

  assert!(EditableGroupTool::read_groups(&binary).is_err());
  assert!(EditableGroupTool::read_grouped_members::<EditableSceneObject>(&binary).is_err());

  Ok(())
}
