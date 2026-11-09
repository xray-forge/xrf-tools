use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{EditableObjectPart, EditableSoundSourceObject, EditableTextPart};
use xrf_error::XrfResult;
use xrf_level::{LevelSndStaticFile, LevelSndStaticSound, LevelSndStaticWindow};
use xrf_math::{Vector2d, Vector3d};

use crate::level::tests::GUID;
use crate::{LevelDecompileContext, LevelSoundSourceConversion};

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

fn bytes_of(file: &LevelSndStaticFile) -> XrfResult<Vec<u8>> {
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
  let part = LevelSoundSourceConversion::decompile(&file, &mut LevelDecompileContext::new(GUID))?;
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
    bytes_of(&LevelSoundSourceConversion::export(&read).unwrap())?,
    bytes_of(&file)?
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
  let part = LevelSoundSourceConversion::decompile(&file, &mut LevelDecompileContext::new(GUID))?;

  assert_eq!(LevelSoundSourceConversion::export(&part).unwrap(), file);

  Ok(())
}

#[test]
fn a_part_of_no_sounds_exports_no_file() -> XrfResult {
  let part = LevelSoundSourceConversion::decompile(
    &LevelSndStaticFile { sounds: Vec::new() },
    &mut LevelDecompileContext::new(GUID),
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
    &mut LevelDecompileContext::new(GUID),
  )
  .unwrap_err()
  .to_string();

  assert!(error.contains("Static sound 0 'ambient\\air_2'"), "{error}");
  assert!(error.contains("active window of 0..86399999 ms"), "{error}");
}
