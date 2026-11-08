use std::f32::consts::FRAC_PI_2;

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{
  EditableLevelGuid, EditableLevelValue, EditableObjectPart, EditableObjectTransform, EditableParticleObject,
  EditableSoundSourceObject, EditableTextPart,
};
use xrf_error::XrfResult;
use xrf_level::{
  LevelPsStaticFile, LevelPsStaticPlacement, LevelSndStaticFile, LevelSndStaticSound, LevelSndStaticWindow,
};
use xrf_math::{Matrix4x4, Vector2d, Vector3d};

use crate::{LevelObjectNames, LevelParticleConversion, LevelSoundSourceConversion};

const GUID: EditableLevelGuid = EditableLevelGuid { g0: 7, g1: 11 };

/// A placement the editor exported from an object whose saved text held these values.
fn placement(effect: &str, position: Vector3d, rotation: Vector3d) -> LevelPsStaticPlacement {
  LevelPsStaticPlacement {
    game_types: Some(EditableParticleObject::ALL_GAME_TYPES),
    effect: effect.to_owned(),
    transform: EditableObjectTransform {
      position,
      rotation,
      scale: Vector3d::ONE,
    }
    .to_matrix(),
  }
}

fn bytes_of(file: &LevelPsStaticFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

fn decompile(file: &LevelPsStaticFile) -> XrfResult<EditableObjectPart<EditableParticleObject>> {
  LevelParticleConversion::decompile(file, GUID, &mut LevelObjectNames::default())
}

#[test]
fn particles_come_back_to_the_same_file_through_the_parts_text() -> XrfResult {
  // Angles on every axis: decomposed, they land units in the last place off the saved ones until rounded.
  let file: LevelPsStaticFile = LevelPsStaticFile {
    version: LevelPsStaticFile::CURRENT_VERSION,
    placements: vec![
      placement(
        "industrial_particles\\exhaust_isparenie_blue_down",
        Vector3d::new(-76.429_19, -25.461_156, 70.090_61),
        Vector3d::new(1.221_731, 0.0, 0.0),
      ),
      placement(
        "industrial_particles\\exhaust_isparenie_blue_down",
        Vector3d::new(12.5, 3.25, -40.0),
        Vector3d::new(0.312_457, -2.104_829, 0.751_002),
      ),
      placement("ambient\\fog_dust", Vector3d::ZERO, Vector3d::new(-0.9, 0.0, 2.5)),
    ],
  };
  let part: EditableObjectPart<EditableParticleObject> = decompile(&file)?;
  let text: Vec<u8> = part.to_text_part()?.write_to_bytes()?;
  let read: EditableObjectPart<EditableParticleObject> =
    EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(&text)?)?;
  let names: Vec<&str> = part.objects.iter().map(|it| it.object.name.as_str()).collect();

  assert_eq!(read, part, "the part reads back as it was decompiled");
  assert_eq!(
    names,
    [
      "exhaust_isparenie_blue_down_0000",
      "exhaust_isparenie_blue_down_0001",
      "fog_dust_0000"
    ]
  );
  assert_eq!(
    bytes_of(&LevelParticleConversion::export(&read).unwrap())?,
    bytes_of(&file)?
  );

  Ok(())
}

#[test]
fn a_particle_pointing_straight_down_comes_back_within_float_precision() -> XrfResult {
  // At a right-angle pitch `getHPB` takes no bank, so the angles saved are not the ones decomposed: the transform is
  // the same one, its bits a few units in the last place off. No vanilla level plants such a particle.
  let file: LevelPsStaticFile = LevelPsStaticFile {
    version: LevelPsStaticFile::CURRENT_VERSION,
    placements: vec![placement(
      "ambient\\fog_dust",
      Vector3d::new(1.0, 2.0, 3.0),
      Vector3d::new(EditableLevelValue::round_as_saved(-FRAC_PI_2), 0.0, 2.5),
    )],
  };
  let exported: LevelPsStaticFile = LevelParticleConversion::export(&decompile(&file)?).unwrap();
  let largest: f32 = exported.placements[0]
    .transform
    .values
    .iter()
    .zip(&file.placements[0].transform.values)
    .map(|(ours, shipped)| (ours - shipped).abs())
    .fold(0.0, f32::max);

  assert!(largest <= 1e-5, "{largest}");

  Ok(())
}

#[test]
fn a_version_0_file_plays_its_particles_in_every_game_mode() -> XrfResult {
  let mut effect: LevelPsStaticPlacement = placement("ambient\\fog_dust", Vector3d::ZERO, Vector3d::ZERO);

  effect.game_types = None;

  let part = decompile(&LevelPsStaticFile {
    version: 0,
    placements: vec![effect],
  })?;
  let exported: LevelPsStaticFile = LevelParticleConversion::export(&part).unwrap();

  assert_eq!(part.objects[0].game_types, EditableParticleObject::ALL_GAME_TYPES);
  assert_eq!(exported.version, LevelPsStaticFile::CURRENT_VERSION);
  assert_eq!(
    exported.placements[0].game_types,
    Some(EditableParticleObject::ALL_GAME_TYPES)
  );

  Ok(())
}

#[test]
fn a_part_of_no_particles_exports_no_file() -> XrfResult {
  let part = decompile(&LevelPsStaticFile {
    version: LevelPsStaticFile::CURRENT_VERSION,
    placements: Vec::new(),
  })?;

  assert!(LevelParticleConversion::export(&part).is_none());

  Ok(())
}

fn sound(name: &str, active: (u32, u32), play: (u32, u32), pause: (u32, u32)) -> LevelSndStaticSound {
  let window = |(from, to): (u32, u32)| LevelSndStaticWindow { from, to };

  LevelSndStaticSound {
    sound: name.to_owned(),
    position: Vector3d::new(-71.280_304, -3.343_151, 64.624_26),
    volume: 0.8,
    frequency: 1.0,
    active: window(active),
    play: window(play),
    pause: window(pause),
  }
}

fn sound_bytes_of(file: &LevelSndStaticFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

#[test]
fn sounds_come_back_to_the_same_file_through_the_parts_text() -> XrfResult {
  // A play of 511 ms saves as 0.511001 seconds: 0.511 exports as 510.
  let file: LevelSndStaticFile = LevelSndStaticFile {
    sounds: vec![
      sound("ambient\\air_2", (0, 0), (0, 0), (0, 0)),
      sound("ambient\\air_2", (21_600_000, 79_200_000), (511, 5_500), (300, 1_700)),
      sound("ambient\\underground\\steam_2", (0, 0), (0, 0), (10_000, 30_000)),
    ],
  };
  let part = LevelSoundSourceConversion::decompile(&file, GUID, &mut LevelObjectNames::default())?;
  let text: Vec<u8> = part.to_text_part()?.write_to_bytes()?;
  let read: EditableObjectPart<EditableSoundSourceObject> =
    EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(&text)?)?;
  let names: Vec<&str> = part.objects.iter().map(|it| it.object.name.as_str()).collect();

  assert_eq!(read, part, "the part reads back as it was decompiled");
  assert_eq!(names, ["air_2_0000", "air_2_0001", "steam_2_0000"]);
  assert_eq!(part.objects[1].active_time, Vector2d::new(21_600.0, 79_200.0));
  assert_eq!(part.objects[1].play_time.x, 0.511_001);
  assert_eq!(part.objects[1].flags, EditableSoundSourceObject::LOOPED);
  assert_eq!(
    sound_bytes_of(&LevelSoundSourceConversion::export(&read).unwrap())?,
    sound_bytes_of(&file)?
  );

  Ok(())
}

#[test]
fn every_millisecond_of_the_first_minutes_has_saved_seconds_exporting_to_it() -> XrfResult {
  let milliseconds: Vec<u32> = (0..=600_000).step_by(7).chain(0..=2_000).collect();
  let file: LevelSndStaticFile = LevelSndStaticFile {
    sounds: milliseconds
      .iter()
      .map(|&it| sound("ambient\\air_2", (0, 0), (it, it), (0, 0)))
      .collect(),
  };
  let part = LevelSoundSourceConversion::decompile(&file, GUID, &mut LevelObjectNames::default())?;

  assert_eq!(LevelSoundSourceConversion::export(&part).unwrap(), file);

  Ok(())
}

#[test]
fn a_part_of_no_sounds_exports_no_file() -> XrfResult {
  let part = LevelSoundSourceConversion::decompile(
    &LevelSndStaticFile { sounds: Vec::new() },
    GUID,
    &mut LevelObjectNames::default(),
  )?;

  assert!(LevelSoundSourceConversion::export(&part).is_none());

  Ok(())
}

#[test]
fn refuses_a_window_no_saved_seconds_export_to() {
  // Near midnight an `f32` of seconds steps by 1/128, so its products by 1000 skip most milliseconds: the nearest
  // seconds export to 86,399,992 and 86,400,000.
  let error: String = LevelSoundSourceConversion::decompile(
    &LevelSndStaticFile {
      sounds: vec![sound("ambient\\air_2", (0, 86_399_999), (0, 0), (0, 0))],
    },
    GUID,
    &mut LevelObjectNames::default(),
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("Static sound 0 'ambient\\air_2'"), "{error}");
  assert!(error.contains("active window of 0..86399999 ms"), "{error}");
}

#[test]
fn refuses_a_particle_no_object_stands_at() {
  let mut effect: LevelPsStaticPlacement = placement("ambient\\fog_dust", Vector3d::ZERO, Vector3d::ZERO);

  effect.transform = Matrix4x4::from_scale(&Vector3d::new(1.0, 0.0, 1.0));

  let error: String = decompile(&LevelPsStaticFile {
    version: LevelPsStaticFile::CURRENT_VERSION,
    placements: vec![effect],
  })
  .unwrap_err()
  .to_string();

  assert!(error.contains("Static particle 0 'ambient\\fog_dust'"), "{error}");
}
