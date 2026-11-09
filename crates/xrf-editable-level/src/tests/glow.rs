use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};
use xrf_math::Vector3d;

use crate::tests::fixtures::GUID;
use crate::{EditableGlowObject, EditableObjectPart, EditableSpawnPointObject, EditableTextPart};

/// A glow tool's part as the CoP SDK saves one: `[main]` keeping flags and the tool's version under
/// `glow_tool_version`, and one glow as GSC's `labx8` holds `glow_galogen_0009` (at a position `%f` prints exactly),
/// with its fields replaced or added.
fn glow_part_text(fields: &[(&str, &str)]) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();

  GUID.write_to_section(file.section_mut("guid"));
  file.section_mut("main").set("flags", "0");
  file.section_mut("main").set("glow_tool_version", "0");
  file.section_mut("main").set("objects_count", "1");
  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  let section: &mut IniSection = file.section_mut("object_0");

  for (key, value) in [
    ("clsid", "1"),
    ("co_flags", "0"),
    ("flags", "0"),
    ("name", "glow_galogen_0009"),
    ("position", "-78.125000, -12.156250, 73.250000"),
    ("radius", "1.000000"),
    ("rotation", "0.000000, 0.000000, 3.141593"),
    ("scale", "1.000000, 1.000000, 1.000000"),
    ("shader_name", "effects\\glow"),
    ("texture_name", "glow\\glow_04"),
    ("version", "18"),
  ]
  .into_iter()
  .chain(fields.iter().copied())
  {
    section.set(key, value);
  }

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

fn read_glows(text: &str) -> XrfResult<EditableObjectPart<EditableGlowObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_glow_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = glow_part_text(&[])?;
  let part: EditableObjectPart<EditableGlowObject> = read_glows(&text)?;
  let glow: &EditableGlowObject = &part.objects[0];

  assert_eq!(part.flags, Some(0));
  assert_eq!(glow.object.name, "glow_galogen_0009");
  assert_eq!(glow.radius, 1.0);
  assert_eq!(
    (glow.shader.as_str(), glow.texture.as_str()),
    ("effects\\glow", "glow\\glow_04")
  );
  assert_eq!(glow.object.transform.scale, Vector3d::new(1.0, 1.0, 1.0));
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn counts_the_objects_of_a_part_whatever_key_its_version_is_under() -> XrfResult {
  let part: EditableTextPart = EditableTextPart::read_from_bytes(glow_part_text(&[])?.as_bytes())?;

  assert_eq!(part.get_objects()?.len(), 1);
  assert!(
    EditableObjectPart::<EditableSpawnPointObject>::read_from_text_part(&part).is_err(),
    "a tool reading `version` refuses a part that holds none"
  );

  Ok(())
}

#[test]
fn refuses_a_glow_the_editor_does_not_load() -> XrfResult {
  for (field, reason) in [
    (("version", "17"), "another version"),
    (("radius", "wide"), "a radius of no number"),
  ] {
    assert!(
      read_glows(&glow_part_text(&[field])?).is_err(),
      "a glow of {reason} was read"
    );
  }

  Ok(())
}
