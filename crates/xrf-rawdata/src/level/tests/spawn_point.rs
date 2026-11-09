use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{EditableObjectPart, EditableSpawnPointKind, EditableSpawnPointObject, EditableTextPart};
use xrf_error::XrfResult;
use xrf_level::{LevelEnvModFile, LevelEnvModifier, LevelGameRPoint};
use xrf_math::Vector3d;

use crate::level::tests::GUID;
use crate::{LevelObjectNames, LevelSpawnPointConversion};

fn respawn_point(team: u8, profile: &str) -> LevelGameRPoint {
  LevelGameRPoint {
    position: Vector3d::new(12.25, 1.0, 11.625),
    rotation: Vector3d::new(0.0, -3.125, 0.0),
    team,
    kind: if profile.is_empty() { 0 } else { 2 },
    game_types: 4,
    profile: profile.to_owned(),
  }
}

/// A modifier whose colours are `u32_3f` of these bytes.
fn modifier(fog: u8, sky: u8) -> LevelEnvModifier {
  let channel = |byte: u8| (1.0_f32 / 255.0) * f32::from(byte);

  LevelEnvModifier {
    position: Vector3d::new(-4.5, 2.0, 7.25),
    radius: 10.0,
    power: 1.0,
    far_plane: 300.0,
    fog_color: Vector3d::new(channel(fog), channel(fog), channel(fog)),
    fog_density: 0.75,
    ambient: Vector3d::ZERO,
    sky_color: Vector3d::new(channel(sky), channel(0), channel(255)),
    hemi_color: Vector3d::new(1.0, 1.0, 1.0),
    use_flags: Some(63),
  }
}

fn environment(modifiers: Vec<LevelEnvModifier>) -> LevelEnvModFile {
  LevelEnvModFile {
    version: EditableSpawnPointObject::VERSION,
    modifiers,
  }
}

fn bytes_of(file: &LevelEnvModFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

fn decompile(
  respawn_points: &[LevelGameRPoint],
  environment: &LevelEnvModFile,
) -> XrfResult<EditableObjectPart<EditableSpawnPointObject>> {
  LevelSpawnPointConversion::decompile(respawn_points, environment, GUID, &mut LevelObjectNames::default())
}

#[test]
fn spawn_points_come_back_to_the_same_files_through_the_parts_text() -> XrfResult {
  let respawn_points: Vec<LevelGameRPoint> = vec![respawn_point(2, ""), respawn_point(0, "mp_medkit")];
  let environment: LevelEnvModFile = environment(vec![modifier(128, 64), modifier(17, 254)]);
  let part = decompile(&respawn_points, &environment)?;
  let text: Vec<u8> = part.to_text_part()?.write_to_bytes()?;
  let read: EditableObjectPart<EditableSpawnPointObject> =
    EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(&text)?)?;
  let names: Vec<&str> = read.objects.iter().map(|it| it.object.name.as_str()).collect();

  assert_eq!(read, part);
  assert_eq!(names, ["rpoint_0000", "rpoint_0001", "env_mod_0000", "env_mod_0001"]);
  assert!(matches!(
    read.objects[2].kind,
    EditableSpawnPointKind::EnvironmentModifier {
      fog_color: 0x0080_8080,
      sky_color: 0x0040_00FF,
      ..
    }
  ));
  assert_eq!(LevelSpawnPointConversion::export_respawn_points(&read), respawn_points);
  assert_eq!(
    bytes_of(&LevelSpawnPointConversion::export_environment_modifiers(&read))?,
    bytes_of(&environment)?
  );

  Ok(())
}

#[test]
fn every_byte_of_a_colour_comes_back_through_u32_3f() -> XrfResult {
  let environment: LevelEnvModFile = environment((0..=u8::MAX).map(|byte| modifier(byte, byte)).collect());
  let part = decompile(&[], &environment)?;

  assert_eq!(
    LevelSpawnPointConversion::export_environment_modifiers(&part),
    environment
  );

  Ok(())
}

#[test]
fn refuses_a_colour_no_packed_word_exports_to() {
  let mut odd: LevelEnvModifier = modifier(128, 64);

  odd.fog_color.y = 0.5;

  let error: String = decompile(&[], &environment(vec![odd])).unwrap_err().to_string();

  assert!(error.contains("Environment modifier 0's fog colour"), "{error}");
}

#[test]
fn a_scene_of_no_modifier_exports_a_file_of_its_version_alone() -> XrfResult {
  let part = decompile(&[respawn_point(1, "")], &environment(Vec::new()))?;

  assert_eq!(
    bytes_of(&LevelSpawnPointConversion::export_environment_modifiers(&part))?,
    bytes_of(&environment(Vec::new()))?
  );

  Ok(())
}
