use xrf_error::XrfResult;
use xrf_ini::IniSection;
use xrf_math::Vector3d;

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::spawn::editable_spawn_point_object::EditableSpawnPointObject;
use crate::object::way::editable_way_link::EditableWayLink;
use crate::object::way::editable_way_object::EditableWayObject;
use crate::object::way::editable_way_point::EditableWayPoint;
use crate::part::editable_level_part::EditableLevelPart;
use crate::part::editable_object_part::EditableObjectPart;
use crate::part::editable_text_part::EditableTextPart;
use crate::scene::editable_scene_tool::EditableSceneTool;
use crate::tests::fixtures::{GUID, way_part_text};

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
fn becomes_the_scene_tool_its_part_is_saved_under() -> XrfResult {
  let part: EditableObjectPart<EditableWayObject> = read_ways(&way_part_text(&GUID, &[])?)?;
  let tool: EditableSceneTool = part.to_scene_tool()?;

  assert_eq!(tool.name, "way");
  assert_eq!(tool.get_file_name(0), "way.part");
  assert_eq!(tool.parts, [EditableLevelPart::Text(part.to_text_part()?)]);
  assert_eq!(EditableObjectPart::read_from_scene_tool(&tool)?, part);
  assert!(EditableObjectPart::<EditableSpawnPointObject>::read_from_scene_tool(&tool).is_err());

  Ok(())
}

#[test]
fn appends_links_in_the_order_their_keys_sort() -> XrfResult {
  // The padding sorts `   9` before `  10`, so the order is the links' numbers.
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
    (("link_wp_   2_   0", "2.000000, 1.000000"), "a link to its own point"),
    // Point 2 exists and the link to point 0 is valid: only the five digits refuse it.
    (
      ("link_wp_00002_   0", "0.000000, 1.000000"),
      "a point number past four digits",
    ),
    (("link_wp_   2_   x", "1.000000, 1.000000"), "no link number"),
  ] {
    assert!(
      read_ways(&way_part_text(&GUID, &[field])?).is_err(),
      "a way of {reason} was read"
    );
  }

  Ok(())
}

#[test]
fn checks_that_every_link_leads_to_another_point_of_the_way() -> XrfResult {
  let mut way: EditableWayObject = read_ways(&way_part_text(&GUID, &[])?)?.objects.remove(0);

  way.check_links()?;

  for to in [0, 3] {
    let mut linked: EditableWayObject = way.clone();

    linked.points[0].links.push(EditableWayLink { to, probability: 1.0 });

    assert!(linked.check_links().is_err(), "a link to point {to} is refused");
  }

  way.points[2].links.push(EditableWayLink {
    to: 1,
    probability: 1.0,
  });
  way.check_links()?;

  // Point 10000's key would take five digits, which the editor's `%4d` reads short.
  let mut long: EditableWayObject = way.clone();
  let last: EditableWayPoint = long.points[0].clone();

  long.points.resize(10_001, last);
  long.points[10_000].links = vec![EditableWayLink {
    to: 1,
    probability: 1.0,
  }];

  assert!(long.check_links().is_err(), "a link from point 10000 is refused");

  long.points[10_000].links.clear();
  long.points[9_999].links = vec![EditableWayLink {
    to: 1,
    probability: 1.0,
  }];
  long.check_links()?;

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
