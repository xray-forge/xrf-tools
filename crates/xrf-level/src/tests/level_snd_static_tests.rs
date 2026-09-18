use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::snd_static::level_snd_static_file::LevelSndStaticFile;
use crate::snd_static::level_snd_static_sound::SndStaticSound;
use crate::tests::fixtures::{chunk, floats, string};

/// One static sound record, wrapped in the chunk its payload sits in.
fn sound_bytes(name: &str, active: (u32, u32)) -> XrfResult<Vec<u8>> {
  let mut payload: Vec<u8> = string(name);

  payload.extend(floats(&[1.0, 2.0, 3.0, 0.8, 1.0]));
  payload.extend_from_slice(&active.0.to_le_bytes());
  payload.extend_from_slice(&active.1.to_le_bytes());

  for value in [1000u32, 2000, 3000, 4000] {
    payload.extend_from_slice(&value.to_le_bytes());
  }

  chunk(SndStaticSound::DATA_CHUNK_ID, &payload)
}

/// A sound list: one numbered chunk per record, each holding one payload chunk.
fn file_bytes(sounds: &[(&str, (u32, u32))]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = Vec::new();

  for (index, (name, active)) in sounds.iter().enumerate() {
    bytes.extend(chunk(index as u32, &sound_bytes(name, *active)?)?);
  }

  Ok(bytes)
}

#[test]
fn a_static_sound_reads_the_file_it_plays_and_where_it_sits() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[("ambient\\wind_trees_1", (0, 0))])?;
  let file: LevelSndStaticFile = LevelSndStaticFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.sounds.len(), 1);
  assert_eq!(file.sounds[0].sound, "ambient\\wind_trees_1");
  assert_eq!(file.sounds[0].volume, 0.8);
  assert_eq!(file.sounds[0].position.x, 1.0);

  Ok(())
}

#[test]
fn a_window_of_two_zeroes_is_no_window_rather_than_one_of_no_length() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[("ambient\\wind_trees_1", (0, 0)), ("ambient\\house_wind1", (6, 20))])?;
  let file: LevelSndStaticFile = LevelSndStaticFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert!(!file.sounds[0].active.is_bounded());
  assert!(file.sounds[1].active.is_bounded());
  assert_eq!(file.get_scheduled_count(), 1);

  Ok(())
}

#[test]
fn a_sound_list_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[("ambient\\wind_trees_1", (0, 0)), ("ambient\\house_wind1", (6, 20))])?;
  let file: LevelSndStaticFile = LevelSndStaticFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}
