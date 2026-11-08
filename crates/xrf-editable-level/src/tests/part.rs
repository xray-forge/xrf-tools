use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::tests::fixtures::{GUID, binary_part_bytes, chunk, guid_bytes, spawn_part_text};
use crate::{EditableBinaryPart, EditableLevelPart, EditablePartObject, EditableTextPart};

#[test]
fn reads_a_text_parts_objects_with_their_own_sections() -> XrfResult {
  let text: String = spawn_part_text(&GUID, 2)?;
  let part: EditableTextPart = EditableTextPart::read_from_bytes(text.as_bytes())?;
  let objects: Vec<EditablePartObject> = part.get_objects()?;

  assert_eq!(part.get_guid()?, GUID);
  assert_eq!(objects.len(), 2);
  assert_eq!(objects[0].get_name()?, "lx8_restrictor");
  assert_eq!(objects[0].get_class_id()?, 6);
  assert_eq!(objects[0].get_transform()?.scale, Vector3d::ONE);
  assert!(objects[0].get_child("attached_0").is_some());
  assert!(objects[0].get_child("spawndata").is_none());
  assert_eq!(
    objects[1].get_child("spawndata").and_then(|data| data.get("000024")),
    Some("\"[logic]\r\ncfg = scripts\\labx8\\box.ltx\r\n\r\n[story_object]\r\nstory_id = box\"")
  );
  assert_eq!(part.write_to_bytes()?, text.as_bytes());

  Ok(())
}

#[test]
fn takes_only_sections_named_after_a_counted_object() -> XrfResult {
  let mut part: EditableTextPart = EditableTextPart::read_from_bytes(spawn_part_text(&GUID, 2)?.as_bytes())?;

  // Neither is an object's: a padded index, and an index past the count.
  part.ini.section_mut("object_01").set("name", "padded");
  part.ini.section_mut("object_2_spawndata").set("000001", "1");

  let objects: Vec<EditablePartObject> = part.get_objects()?;

  assert_eq!(objects.len(), 2);
  assert_eq!(objects[0].children.len(), 1);
  assert_eq!(objects[1].children.len(), 1);
  assert_eq!(objects[1].get_name()?, "lx8_box");

  Ok(())
}

#[test]
fn refuses_objects_its_header_counts_but_does_not_hold() -> XrfResult {
  let part: EditableTextPart = EditableTextPart::read_from_bytes(spawn_part_text(&GUID, 3)?.as_bytes())?;

  assert!(part.get_objects().is_err());

  Ok(())
}

#[test]
fn a_binary_part_comes_back_in_the_order_its_file_stored_it() -> XrfResult {
  // The tool data stored before the GUID, and a chunk the editor never writes between them.
  let bytes: Vec<u8> = binary_part_bytes(&[
    chunk(0x8000 + 13, b"nodes"),
    chunk(0x0042, b"x"),
    chunk(0x7000, &guid_bytes(&GUID)?),
  ])?;
  let part: EditableBinaryPart = EditableBinaryPart::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(part.guid, GUID);
  assert_eq!(part.class_id, 13);
  assert_eq!(part.data, b"nodes");
  assert_eq!(part.layout.kept, vec![chunk(0x0042, b"x")]);
  assert_eq!(part.write_to_bytes::<XRayByteOrder>()?, bytes);

  Ok(())
}

#[test]
fn refuses_a_binary_part_the_editor_would_skip() -> XrfResult {
  let guid: Vec<u8> = guid_bytes(&GUID)?;

  for (chunks, reason) in [
    (vec![chunk(0x8000 + 13, b"nodes")], "no GUID"),
    (vec![chunk(0x7000, &guid)], "no tool data"),
    (vec![chunk(0x7000, &guid[..8]), chunk(0x8000, b"")], "a short GUID"),
  ] {
    assert!(
      EditableBinaryPart::read_from_bytes::<XRayByteOrder>(binary_part_bytes(&chunks)?).is_err(),
      "a part with {reason} was read"
    );
  }

  Ok(())
}

#[test]
fn tells_text_and_binary_parts_apart_by_their_first_byte() -> XrfResult {
  let text: EditableLevelPart =
    EditableLevelPart::read_from_bytes::<XRayByteOrder>(spawn_part_text(&GUID, 2)?.into_bytes())?;
  let binary: EditableLevelPart = EditableLevelPart::read_from_bytes::<XRayByteOrder>(binary_part_bytes(&[
    chunk(0x7000, &guid_bytes(&GUID)?),
    chunk(0x8000 + 4, b""),
  ])?)?;

  assert!(matches!(text, EditableLevelPart::Text(_)));
  assert!(matches!(binary, EditableLevelPart::Binary(_)));
  assert_eq!(text.get_guid()?, GUID);
  assert_eq!(binary.get_guid()?, GUID);

  Ok(())
}
