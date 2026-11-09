use xrf_error::XrfResult;
use xrf_ini::IniSection;
use xrf_math::Vector3d;

use crate::tests::fixtures::{GUID, way_part_text};
use crate::{EditableLevelValue, EditableObjectPart, EditableTextPart, EditableWayLink, EditableWayObject};

fn read_ways(text: &str) -> XrfResult<EditableObjectPart<EditableWayObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_way_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = way_part_text(&GUID, &[])?;
  let part: EditableObjectPart<EditableWayObject> = read_ways(&text)?;
  let way: &EditableWayObject = &part.objects[0];

  assert_eq!(way.object.name, "lx8_patrol_walk");
  assert_eq!(way.kind, EditableWayObject::PATROL_PATH);
  assert_eq!(way.points[0].position, Vector3d::new(-91.75, -24.3125, 83.125));
  assert!(way.points[0].selected && !way.points[1].selected);
  assert_eq!(way.points[1].flags, 0x8000_0000, "printed through `%d`");
  assert_eq!(
    way.points[0].links,
    [
      EditableWayLink {
        to: 1,
        probability: 1.0
      },
      EditableWayLink {
        to: 2,
        probability: 0.5
      }
    ]
  );
  assert_eq!(
    way.points[1].links,
    [EditableWayLink {
      to: 0,
      probability: 1.0
    }]
  );
  assert!(way.points[2].links.is_empty());
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn appends_links_in_the_order_their_keys_sort() -> XrfResult {
  // Ten links of one point: the padding sorts `   9` before `  10`, so the order is the links' numbers.
  let links: Vec<(String, String)> = (0..11)
    .map(|link| {
      (
        format!("link_wp_   0_{link:4}"),
        format!("1.000000, {:.6}", link as f32 / 10.0),
      )
    })
    .collect();
  let fields: Vec<(&str, &str)> = links
    .iter()
    .map(|(key, value)| (key.as_str(), value.as_str()))
    .collect();
  let part = read_ways(&way_part_text(&GUID, &fields)?)?;
  let probabilities: Vec<f32> = part.objects[0].points[0]
    .links
    .iter()
    .map(|link| link.probability)
    .collect();

  assert_eq!(
    probabilities,
    (0..11).map(|link| link as f32 / 10.0).collect::<Vec<_>>()
  );

  Ok(())
}

#[test]
fn refuses_a_way_the_editor_does_not_load() -> XrfResult {
  for (field, reason) in [
    (("version", "18"), "another version"),
    (("name", ""), "an empty name"),
    (("link_wp_   2_   0", "3.000000, 1.000000"), "a link to a missing point"),
    (("link_wp_   0_   0", "1.000000"), "a link of one float"),
  ] {
    assert!(
      read_ways(&way_part_text(&GUID, &[field])?).is_err(),
      "a way of {reason} was read"
    );
  }

  Ok(())
}

#[test]
fn reads_a_flag_as_the_editor_does() -> XrfResult {
  let mut section: IniSection = IniSection::new("flags");

  for (key, value) in [
    ("a", "On"),
    ("b", "YES"),
    ("c", "true"),
    ("d", "1"),
    ("e", "off"),
    ("f", "2"),
  ] {
    section.set(key, value);
  }

  let read: Vec<bool> = ["a", "b", "c", "d", "e", "f"]
    .iter()
    .map(|key| EditableLevelValue::read_bool(&section, key))
    .collect::<XrfResult<Vec<bool>>>()?;

  assert_eq!(read, [true, true, true, true, false, false]);
  assert_eq!(EditableLevelValue::format_bool(true), "on");

  Ok(())
}
