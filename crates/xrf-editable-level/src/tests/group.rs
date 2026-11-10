use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat};

use crate::tests::fixtures::GUID;
use crate::{
  EditableGroupTool, EditableLevelPart, EditableSceneObject, EditableSceneTool, EditableTextPart, EditableToolObject,
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
    section.set("name", format!("group_{index}"));
    section.set("ingroup_count", classes.len().to_string());
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
  let members: Vec<EditableSceneObject> = EditableGroupTool::read_members(&tool)?;

  assert_eq!(
    members
      .iter()
      .map(|member| (member.object.name.as_str(), member.object.transform.position.x))
      .collect::<Vec<_>>(),
    [("member_0", 0.0), ("member_1", 1.0), ("member_0", 0.0)],
    "the members of the class, at their own places, the second group's in the second file"
  );

  Ok(())
}
