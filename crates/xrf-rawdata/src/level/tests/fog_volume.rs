use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{
  EditableCustomObject, EditableFogVolumeKind, EditableFogVolumeObject, EditableObjectPart, EditableObjectTransform,
  EditableShape, EditableTextPart,
};
use xrf_error::XrfResult;
use xrf_level::LevelFogVolFile;
use xrf_math::Vector3d;

use crate::level::tests::GUID;
use crate::{LevelFogVolumeConversion, LevelObjectNames};

fn volume(kind: EditableFogVolumeKind, group: u32, position: [f32; 3], rotation: [f32; 3]) -> EditableFogVolumeObject {
  EditableFogVolumeObject {
    object: EditableCustomObject::new(
      "authored",
      EditableObjectTransform {
        position: Vector3d::from(position),
        rotation: Vector3d::from(rotation),
        scale: Vector3d::new(10.0, 4.0, 6.0),
      },
    ),
    shape: EditableShape::new_unit_box(),
    kind,
    group,
  }
}

fn emitter(profile: &str, group: u32, position: [f32; 3]) -> EditableFogVolumeObject {
  volume(
    EditableFogVolumeKind::Emitter {
      profile: profile.to_owned(),
    },
    group,
    position,
    [0.0, 1.25, 0.0],
  )
}

fn occlusion(group: u32, position: [f32; 3]) -> EditableFogVolumeObject {
  volume(EditableFogVolumeKind::Occlusion, group, position, [0.3, -0.7, 0.1])
}

fn part(volumes: Vec<EditableFogVolumeObject>) -> EditableObjectPart<EditableFogVolumeObject> {
  EditableObjectPart::new(GUID, volumes)
}

fn bytes_of(file: &LevelFogVolFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

#[test]
fn fog_comes_back_to_the_same_file_through_the_parts_text() -> XrfResult {
  // Groups export by ascending number, the ungrouped last, and each emitter before its occlusion volumes.
  let file: LevelFogVolFile = LevelFogVolumeConversion::export(&part(vec![
    occlusion(7, [1.0, 2.0, 3.0]),
    emitter("environment\\fog\\swamp.ltx", 7, [0.0, 0.0, 0.0]),
    emitter(
      "environment\\fog\\default.ltx",
      EditableFogVolumeObject::UNGROUPED,
      [-5.0, 1.0, 9.5],
    ),
    emitter("environment\\fog\\cave.ltx", 2, [40.0, -3.0, 12.25]),
    occlusion(7, [4.0, 5.0, 6.0]),
  ]))?;
  let decompiled = LevelFogVolumeConversion::decompile(&file, GUID, &mut LevelObjectNames::default())?;
  let text: Vec<u8> = decompiled.to_text_part()?.write_to_bytes()?;
  let read: EditableObjectPart<EditableFogVolumeObject> =
    EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(&text)?)?;
  let groups: Vec<(u32, Option<&str>)> = read
    .objects
    .iter()
    .map(|volume| (volume.group, volume.get_profile()))
    .collect();

  assert_eq!(
    file
      .volumes
      .iter()
      .map(|body| body.profile.as_str())
      .collect::<Vec<_>>(),
    [
      "environment\\fog\\cave.ltx",
      "environment\\fog\\swamp.ltx",
      "environment\\fog\\default.ltx"
    ]
  );
  assert_eq!(file.volumes[1].obstacles.len(), 2);
  assert_eq!(
    groups,
    [
      (1, Some("environment\\fog\\cave.ltx")),
      (2, Some("environment\\fog\\swamp.ltx")),
      (2, None),
      (2, None),
      (3, Some("environment\\fog\\default.ltx"))
    ]
  );
  assert_eq!(read.objects[0].object.name, "fog_volume_0000");
  assert_eq!(bytes_of(&LevelFogVolumeConversion::export(&read)?)?, bytes_of(&file)?);

  Ok(())
}

#[test]
fn a_scene_of_no_fog_exports_a_file_of_no_body() -> XrfResult {
  let file: LevelFogVolFile = LevelFogVolumeConversion::export(&part(Vec::new()))?;

  assert_eq!(bytes_of(&file)?, [3, 0, 0, 0, 0, 0]);
  assert!(
    LevelFogVolumeConversion::decompile(&file, GUID, &mut LevelObjectNames::default())?
      .objects
      .is_empty()
  );

  Ok(())
}

#[test]
fn refuses_a_group_the_build_fails_on() {
  for (volumes, reason) in [
    (vec![occlusion(1, [0.0; 3])], "no emitter"),
    (
      vec![
        emitter("environment\\fog\\default.ltx", 1, [0.0; 3]),
        emitter("environment\\fog\\cave.ltx", 1, [1.0; 3]),
      ],
      "two emitters",
    ),
  ] {
    assert!(
      LevelFogVolumeConversion::export(&part(volumes)).is_err(),
      "a group of {reason} was exported"
    );
  }
}

#[test]
fn refuses_a_profile_line_the_export_does_not_end_so() -> XrfResult {
  let mut file: LevelFogVolFile =
    LevelFogVolumeConversion::export(&part(vec![emitter("environment\\fog\\default.ltx", 1, [0.0; 3])]))?;

  file.volumes[0].terminator = String::from("\n");

  assert!(LevelFogVolumeConversion::decompile(&file, GUID, &mut LevelObjectNames::default()).is_err());

  Ok(())
}
