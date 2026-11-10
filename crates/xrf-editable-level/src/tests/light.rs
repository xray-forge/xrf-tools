use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};
use xrf_math::Vector3d;

use crate::tests::fixtures::GUID;
use crate::{EditableLightObject, EditableObjectPart, EditableTextPart, EditableToolObject};

/// A light tool's part as the CoP SDK saves `labx8`'s: `[main]` with the tool's own sun direction and last control
/// index, `[lcontrols]`, and one fuzzy light shaped as `light_galogen_1m_ambient_0018` (at values `%f` prints
/// exactly), with its fields replaced or added.
fn light_part_text(fields: &[(&str, &str)]) -> XrfResult<String> {
  let mut file: IniFile = IniFile::default();

  GUID.write_to_section(file.section_mut("guid"));

  for (key, value) in [("$hemi", "1"), ("$static", "0"), ("$sun", "2")] {
    file.section_mut("lcontrols").set(key, value);
  }

  for (key, value) in [
    ("flags", "0"),
    ("lcontrol_last_idx", "3"),
    ("objects_count", "1"),
    ("sun_shadow_dir", "-0.437500, 5.093750"),
    ("version", "0"),
  ] {
    file.section_mut("main").set(key, value);
  }

  file.section_mut("modif").set("name", "");
  file.section_mut("modif").set("time", "0");

  let section: &mut IniSection = file.section_mut("object_0");

  for (key, value) in [
    ("anim_ref_name", ""),
    ("attenuation0", "0.500000"),
    ("attenuation1", "0.375000"),
    ("attenuation2", "0.125000"),
    ("brightness", "0.375000"),
    ("clsid", "3"),
    ("co_flags", "0"),
    ("color", "1.000000, 0.750000, 0.500000, 0.000000"),
    ("cone", "0.375000"),
    ("fallof_texture", ""),
    ("fuzzy_box_dim", "0.125000, 0.125000, 0.125000"),
    ("fuzzy_point_0", "0.000000, 0.000000, 0.000000"),
    ("fuzzy_point_1", "0.000488, 0.000244, -0.093750"),
    ("fuzzy_point_count", "2"),
    ("fuzzy_shape_type", "0"),
    ("fuzzy_sphere_radius", "0.125000"),
    ("light_control", "1"),
    ("light_flags", "17"),
    ("name", "light_galogen_1m_ambient_0018"),
    ("position", "-78.125000, -12.312500, 73.500000"),
    ("range", "1.500000"),
    ("rotation", "0.000000, 0.000000, 0.000000"),
    ("scale", "1.000000, 1.000000, 1.000000"),
    ("type", "1"),
    ("use_in_d3d", "on"),
    ("version", "17"),
    ("virtual_size", "0.000000"),
  ]
  .into_iter()
  .chain(fields.iter().copied())
  {
    section.set(key, value);
  }

  file.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

fn read_lights(text: &str) -> XrfResult<EditableObjectPart<EditableLightObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_light_part_typed_and_writes_it_back_with_the_tools_own_data() -> XrfResult {
  let text: String = light_part_text(&[])?;
  let part: EditableObjectPart<EditableLightObject> = read_lights(&text)?;
  let light: &EditableLightObject = &part.objects[0];

  assert_eq!(
    light.flags,
    EditableLightObject::AFFECT_STATIC,
    "the fuzzy bit stands as `fuzzy`"
  );
  assert_eq!(
    light.get_saved_flags(),
    EditableLightObject::AFFECT_STATIC | EditableLightObject::POINT_FUZZY
  );
  assert_eq!(light.color, [1.0, 0.75, 0.5, 0.0]);
  assert_eq!(light.control, 1);
  assert!(light.use_in_d3d);
  assert_eq!(
    light.fuzzy.as_ref().map(|fuzzy| fuzzy.points.clone()),
    Some(vec![Vector3d::ZERO, Vector3d::new(0.000_488, 0.000_244, -0.093_75)])
  );
  assert_eq!(part.tool_data.get("lcontrols", "$sun"), Some("2"));
  assert_eq!(part.tool_data.get("main", "lcontrol_last_idx"), Some("3"));
  assert_eq!(
    part.tool_data.get("main", "version"),
    None,
    "the header's own keys stay the header's"
  );
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn reads_the_tools_controls_and_sun_direction_and_none_from_a_tool_without_them() -> XrfResult {
  let lights: EditableObjectPart<EditableLightObject> = read_lights(&light_part_text(&[])?)?;
  let controls = EditableLightObject::read_controls(&lights.tool_data)?;

  assert_eq!(
    (
      controls.len(),
      controls.get(&EditableLightObject::SUN_CONTROL).map(String::as_str)
    ),
    (3, Some("$sun"))
  );
  assert!(EditableLightObject::read_sun_direction(&lights.tool_data)?.is_some());

  let mut fresh: IniFile = IniFile::default();

  assert!(EditableLightObject::read_controls(&fresh)?.is_empty());
  assert_eq!(EditableLightObject::read_sun_direction(&fresh)?, None);

  EditableLightObject::write_new_tool_data(&mut fresh);

  // Written as `%f` prints it, so within its six decimals.
  let (read, default) = (
    EditableLightObject::read_sun_direction(&fresh)?.expect("a new tool's sun direction"),
    EditableLightObject::get_default_sun_direction(),
  );

  assert!((read.x - default.x).abs() < 1e-6 && (read.y - default.y).abs() < 1e-6);

  fresh
    .section_mut(EditableLightObject::CONTROLS_SECTION)
    .set("$broken", "x");

  assert!(EditableLightObject::read_controls(&fresh).is_err());

  Ok(())
}

#[test]
fn reads_no_fuzzy_points_for_a_light_not_flagged_fuzzy() -> XrfResult {
  let part: EditableObjectPart<EditableLightObject> = read_lights(&light_part_text(&[("light_flags", "2")])?)?;
  let written: String = String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?).into_owned();

  assert_eq!(part.objects[0].fuzzy, None);
  assert!(!written.contains("fuzzy"), "{written}");

  Ok(())
}

#[test]
fn refuses_a_light_the_editor_does_not_load() -> XrfResult {
  for (field, reason) in [
    (("version", "16"), "another version"),
    (("color", "1.0, 0.5, 0.25"), "a colour of three channels"),
    (("fuzzy_point_count", "3"), "a fuzzy point it does not hold"),
  ] {
    assert!(
      read_lights(&light_part_text(&[field])?).is_err(),
      "a light of {reason} was read"
    );
  }

  Ok(())
}
