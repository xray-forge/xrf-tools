use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};

use crate::tests::fixtures::GUID;
use crate::{EditableObjectPart, EditableSceneObject, EditableTextPart};

/// A scene object tool's part as the CoP SDK saves `labx8`'s: its own `[appendrandom]`, `[main]` keeping flags, and one
/// placed library object (at a transform `%f` prints exactly), with its fields replaced or added.
fn scene_object_part_text(fields: &[(&str, &str)]) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();

  for (key, value) in [
    ("AppendRandomMaxRotation", "0.000000, 0.000000, 0.000000"),
    ("AppendRandomMaxScale", "1.000000, 1.000000, 1.000000"),
    ("AppendRandomMinRotation", "0.000000, 0.000000, 0.000000"),
    ("AppendRandomMinScale", "1.000000, 1.000000, 1.000000"),
    ("AppendRandomObjects_size", "0"),
  ] {
    file.section_mut("appendrandom").set(key, value);
  }

  GUID.write_to_section(file.section_mut("guid"));
  file.section_mut("main").set("flags", "0");
  file.section_mut("main").set("objects_count", "1");
  file.section_mut("main").set("version", "0");
  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  let section: &mut IniSection = file.section_mut("object_0");

  for (key, value) in [
    ("clsid", "2"),
    ("co_flags", "0"),
    ("flags", "0"),
    ("name", "statics\\stol\\stol_4_0000"),
    ("position", "-80.250000, -15.937500, 114.093750"),
    ("reference_name", "statics\\stol\\stol_4"),
    ("rotation", "0.000000, -2.875000, 0.000000"),
    ("scale", "1.000000, 1.000000, 1.000000"),
    ("version", "18"),
  ]
  .into_iter()
  .chain(fields.iter().copied())
  {
    section.set(key, value);
  }

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

fn read_scene_objects(text: &str) -> XrfResult<EditableObjectPart<EditableSceneObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_scene_object_part_typed_and_writes_it_back_with_the_tools_own_data() -> XrfResult {
  let text: String = scene_object_part_text(&[])?;
  let part: EditableObjectPart<EditableSceneObject> = read_scene_objects(&text)?;

  assert_eq!(part.objects[0].reference, "statics\\stol\\stol_4");
  assert_eq!(part.objects[0].object.transform.rotation.y, -2.875);
  assert_eq!(
    part.tool_data.get("appendrandom", "AppendRandomObjects_size"),
    Some("0")
  );
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

/// A new part holds the tool's `[appendrandom]` as a new tool saves it, which the editor's loader requires: GSC's own,
/// as `labx8`'s part holds it.
#[test]
fn writes_a_new_part_with_the_tools_own_data_as_a_new_tool_holds_it() -> XrfResult {
  let gsc: EditableObjectPart<EditableSceneObject> = read_scene_objects(&scene_object_part_text(&[])?)?;
  let new: EditableObjectPart<EditableSceneObject> = EditableObjectPart::new(GUID, Vec::new());

  assert_eq!(new.tool_data, gsc.tool_data);

  Ok(())
}

#[test]
fn refuses_a_scene_object_of_no_reference() -> XrfResult {
  assert!(read_scene_objects(&scene_object_part_text(&[("reference_name", "")])?).is_err());

  Ok(())
}
