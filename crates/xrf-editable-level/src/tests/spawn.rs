use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::tests::fixtures::{GUID, spawn_point_part_text};
use crate::{
  EditableObjectPart, EditableShapeElement, EditableSpawnPointKind, EditableSpawnPointObject, EditableTextPart,
};

fn read_spawn_points(text: &str) -> XrfResult<EditableObjectPart<EditableSpawnPointObject>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

#[test]
fn reads_a_spawn_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = spawn_point_part_text(&GUID, &[])?;
  let part: EditableObjectPart<EditableSpawnPointObject> = read_spawn_points(&text)?;
  let [respawn, environment, entity] = part.objects.as_slice() else {
    panic!("three spawn points");
  };

  assert_eq!(part.flags, Some(0), "the spawn tool keeps flags");
  assert_eq!(
    respawn.kind,
    EditableSpawnPointKind::RespawnPoint {
      team: 2,
      kind: 0,
      game_types: 4,
      profile: String::new()
    }
  );
  assert!(matches!(
    environment.kind,
    EditableSpawnPointKind::EnvironmentModifier {
      fog_color: 0x0080_8080,
      flags: 63,
      ..
    }
  ));

  let EditableSpawnPointKind::Entity { section, flags, data } = &entity.kind else {
    panic!("an entity");
  };
  let attached = entity.attached.as_ref().expect("a shape attached");

  assert_eq!((section.as_str(), *flags), ("space_restrictor", 0));
  assert_eq!(
    data.get("name"),
    None,
    "the section is kept apart from the packet's fields"
  );
  assert_eq!(
    attached.shape.elements,
    [EditableShapeElement::Sphere {
      center: Vector3d::ZERO,
      radius: 2.5
    }]
  );
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn refuses_a_spawn_part_keeping_no_flags_as_the_spawn_tool_requires_them() -> XrfResult {
  let mut part: EditableTextPart = EditableTextPart::read_from_bytes(spawn_point_part_text(&GUID, &[])?.as_bytes())?;

  part.ini.section_mut("main").remove("flags");

  assert!(EditableObjectPart::<EditableSpawnPointObject>::read_from_text_part(&part).is_err());
  assert_eq!(
    EditableObjectPart::<EditableSpawnPointObject>::new(GUID, Vec::new()).flags,
    Some(0)
  );

  Ok(())
}

#[test]
fn takes_an_older_spawn_point_as_the_editor_does() -> XrfResult {
  // Version 0x16 keeps no respawn profile, and version 0x15 no modifier flags, which then mix in every value.
  let part = read_spawn_points(&spawn_point_part_text(
    &GUID,
    &[
      ("object_0", "version", "22"),
      ("object_0", "rp_profile", "mp_item"),
      ("object_1", "version", "21"),
    ],
  )?)?;

  assert!(matches!(&part.objects[0].kind, EditableSpawnPointKind::RespawnPoint { profile, .. } if profile.is_empty()));
  assert!(matches!(
    part.objects[1].kind,
    EditableSpawnPointKind::EnvironmentModifier { flags: u16::MAX, .. }
  ));

  Ok(())
}

#[test]
fn refuses_a_spawn_point_the_editor_does_not_load() -> XrfResult {
  for (field, reason) in [
    (("object_0", "version", "19"), "a version older than 0x14"),
    (
      ("object_0", "version", "20"),
      "a respawn point of version 0x14's single game mode",
    ),
    (("object_0", "type", "3"), "an unknown type"),
    (("object_2", "attached_count", "2"), "two attachments"),
    (
      ("object_2_attached_0", "clsid", "6"),
      "an attachment other than a shape",
    ),
    (("object_1", "type", "2"), "an entity of no spawn data"),
  ] {
    assert!(
      read_spawn_points(&spawn_point_part_text(&GUID, &[field])?).is_err(),
      "a spawn point of {reason} was read"
    );
  }

  Ok(())
}
