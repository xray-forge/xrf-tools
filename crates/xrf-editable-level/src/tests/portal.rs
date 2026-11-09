use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};
use xrf_math::Vector3d;

use crate::tests::fixtures::GUID;
use crate::{EditableObjectPart, EditablePortalObject, EditableTextPart};

/// A portal tool's part as the CoP SDK saves `labx8`'s: `[main]` keeping flags, and one portal of `labx8`'s shape (at
/// corners `%f` prints exactly), closing on its first corner as GSC's do, with its fields replaced or added.
fn portal_part_text(fields: &[(&str, &str)]) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();

  GUID.write_to_section(file.section_mut("guid"));
  file.section_mut("main").set("flags", "0");
  file.section_mut("main").set("objects_count", "1");
  file.section_mut("main").set("version", "0");
  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  let section: &mut IniSection = file.section_mut("object_0");

  for (key, value) in [
    ("clsid", "9"),
    ("co_flags", "0"),
    ("name", "portal_0000"),
    ("position", "0.000000, 0.000000, 0.000000"),
    ("rotation", "0.000000, 0.000000, 0.000000"),
    ("scale", "1.000000, 1.000000, 1.000000"),
    ("sector_back", "sector_0009"),
    ("sector_front", "sector_0011"),
    ("version", "16"),
    ("vert_count", "4"),
    ("vertex_0000", "-74.250000, -15.937500, 69.750000"),
    ("vertex_0001", "-75.875000, -15.937500, 69.750000"),
    ("vertex_0002", "-75.875000, -13.281250, 69.750000"),
    ("vertex_0003", "-74.250000, -15.937500, 69.750000"),
  ]
  .into_iter()
  .chain(fields.iter().copied())
  {
    section.set(key, value);
  }

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

fn read_portals(text: &str) -> XrfResult<EditableObjectPart<EditablePortalObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_portal_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = portal_part_text(&[])?;
  let part: EditableObjectPart<EditablePortalObject> = read_portals(&text)?;
  let portal: &EditablePortalObject = &part.objects[0];

  assert_eq!(
    (portal.sector_front.as_str(), portal.sector_back.as_str()),
    ("sector_0011", "sector_0009")
  );
  assert_eq!(portal.vertices.len(), 4);
  assert_eq!(portal.vertices[2], Vector3d::new(-75.875, -13.281_25, 69.75));
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn refuses_a_portal_the_editor_does_not_load() -> XrfResult {
  for (field, reason) in [
    (("version", "15"), "another version"),
    (("vert_count", "2"), "two corners"),
    (("vert_count", "5"), "a corner it does not hold"),
  ] {
    assert!(
      read_portals(&portal_part_text(&[field])?).is_err(),
      "a portal of {reason} was read"
    );
  }

  Ok(())
}
