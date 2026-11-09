use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};

use crate::tests::fixtures::GUID;
use crate::{EditableObjectPart, EditableSectorItem, EditableSectorObject, EditableTextPart};

/// A sector tool's part as the CoP SDK saves `labx8`'s: `[main]` keeping its flags printed through `%d`, and one
/// sector as `labx8`'s `sector_0000`, with its fields replaced or added.
fn sector_part_text(fields: &[(&str, &str)]) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();

  GUID.write_to_section(file.section_mut("guid"));
  file.section_mut("main").set("flags", "-2147483648");
  file.section_mut("main").set("objects_count", "1");
  file.section_mut("main").set("version", "0");
  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  let section: &mut IniSection = file.section_mut("object_0");

  for (key, value) in [
    ("change_map_to_idx", "0"),
    ("clsid", "8"),
    ("co_flags", "0"),
    ("default", "off"),
    ("item_mesh_name_0000", "labx8_sector_9_2"),
    ("item_mesh_name_0001", "sign_no_smoking"),
    ("item_object_name_0000", "scenes\\labx8\\labx8_sector_9"),
    ("item_object_name_0001", "statics\\sign\\sign_no_smoking_0000"),
    ("items_count", "2"),
    ("name", "sector_0000"),
    ("position", "0.000000, 0.000000, 0.000000"),
    ("rotation", "0.000000, 0.000000, 0.000000"),
    ("scale", "1.000000, 1.000000, 1.000000"),
    ("sector_color", "127, 127, 0, 0"),
    ("version", "18"),
  ]
  .into_iter()
  .chain(fields.iter().copied())
  {
    section.set(key, value);
  }

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

fn read_sectors(text: &str) -> XrfResult<EditableObjectPart<EditableSectorObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_sector_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = sector_part_text(&[])?;
  let part: EditableObjectPart<EditableSectorObject> = read_sectors(&text)?;
  let sector: &EditableSectorObject = &part.objects[0];

  assert_eq!(part.flags, Some(0x8000_0000), "printed through `%d`");
  assert_eq!(sector.color, [127, 127, 0, 0]);
  assert!(!sector.is_default);
  assert_eq!(
    sector.items[1],
    EditableSectorItem {
      object: String::from("statics\\sign\\sign_no_smoking_0000"),
      mesh: String::from("sign_no_smoking"),
    }
  );
  assert_eq!(sector.map_index, 0);
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn reads_a_version_17_sector_as_switching_to_no_map() -> XrfResult {
  let part: EditableObjectPart<EditableSectorObject> = read_sectors(&sector_part_text(&[("version", "17")])?)?;

  assert_eq!(part.objects[0].map_index, EditableSectorObject::NO_MAP);

  Ok(())
}

#[test]
fn refuses_a_sector_the_editor_does_not_load() -> XrfResult {
  for (field, reason) in [
    (("version", "16"), "an older version"),
    (("items_count", "0"), "no item"),
    (("items_count", "3"), "an item it does not hold"),
    (("sector_color", "127, 127, 0"), "a colour of three channels"),
  ] {
    assert!(
      read_sectors(&sector_part_text(&[field])?).is_err(),
      "a sector of {reason} was read"
    );
  }

  Ok(())
}
