use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::light_anim::light_anim_file::LightAnimFile;
use crate::light_anim::light_anim_item::LightAnimItem;

/// Frames a payload as one X-Ray chunk.
fn chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// One animation, as the library stores it.
fn item(name: &str, fps: f32, frames: u32, keys: &[(u32, u32)]) -> XrfResult<Vec<u8>> {
  let mut common: Vec<u8> = name.as_bytes().to_vec();

  common.push(0);
  common.extend_from_slice(&fps.to_le_bytes());
  common.extend_from_slice(&frames.to_le_bytes());

  let mut bytes: Vec<u8> = chunk(LightAnimItem::COMMON_CHUNK_ID, &common)?;
  let mut payload: Vec<u8> = (keys.len() as u32).to_le_bytes().to_vec();

  for (frame, color) in keys {
    payload.extend_from_slice(&frame.to_le_bytes());
    payload.extend_from_slice(&color.to_le_bytes());
  }

  bytes.extend(chunk(LightAnimItem::KEYS_CHUNK_ID, &payload)?);

  Ok(bytes)
}

/// A whole library holding the animations it is given.
fn library(version: u16, items: &[Vec<u8>]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LightAnimFile::VERSION_CHUNK_ID, &version.to_le_bytes())?;
  let mut payload: Vec<u8> = Vec::new();

  for (index, item) in items.iter().enumerate() {
    payload.extend(chunk(index as u32, item)?);
  }

  bytes.extend(chunk(LightAnimFile::ITEMS_CHUNK_ID, &payload)?);

  Ok(bytes)
}

/// A library holding one animation of two keys.
fn sample() -> XrfResult<Vec<u8>> {
  library(
    1,
    &[item("campfire", 15.0, 30, &[(0, 0xFF20_1000), (15, 0xFF80_4020)])?],
  )
}

#[test]
fn reads_an_animation_and_its_keys() -> XrfResult {
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(sample()?)?;

  assert_eq!(file.items.len(), 1);
  assert_eq!(file.items[0].name, "campfire");
  assert_eq!(file.items[0].frame_count, 30);
  assert_eq!(file.items[0].keys.len(), 2);
  assert_eq!(file.get_keys_count(), 2);

  Ok(())
}

#[test]
fn takes_a_duration_from_the_frames_over_the_rate() -> XrfResult {
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(sample()?)?;

  assert_eq!(file.items[0].get_duration_seconds(), Some(2.0));

  Ok(())
}

#[test]
fn an_animation_that_never_advances_has_no_duration_rather_than_an_infinite_one() -> XrfResult {
  // Dividing by a rate of zero would answer infinity, which reads as a measurement rather than as its absence.
  let file: LightAnimFile =
    LightAnimFile::read_from_bytes::<XRayByteOrder>(library(1, &[item("still", 0.0, 1, &[(0, 0)])?])?)?;

  assert_eq!(file.items[0].get_duration_seconds(), None);

  Ok(())
}

#[test]
fn splits_a_key_into_the_channels_the_engine_reads() -> XrfResult {
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(sample()?)?;
  let key = file.items[0].keys[1];

  assert_eq!(key.get_alpha(), 0xFF);
  assert_eq!(key.get_red(), 0x80);
  assert_eq!(key.get_green(), 0x40);
  assert_eq!(key.get_blue(), 0x20);

  Ok(())
}

#[test]
fn reports_a_version_zero_library_as_channel_swapped() -> XrfResult {
  // `ELightAnimLibrary::Load` swaps every key of a version 0 file on load, so the stored colours are not the ones
  // the engine ends up using and a reader that said nothing would mislead.
  let file: LightAnimFile =
    LightAnimFile::read_from_bytes::<XRayByteOrder>(library(0, &[item("old", 15.0, 1, &[(0, 0)])?])?)?;

  assert!(file.is_bgr());

  Ok(())
}

#[test]
fn a_library_without_a_version_chunk_reads_as_the_oldest_one() -> XrfResult {
  // `find_chunk` leaves the version at its initial zero when the chunk is absent, which is the swapped reading.
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend(chunk(
    LightAnimFile::ITEMS_CHUNK_ID,
    &chunk(0, &item("old", 15.0, 1, &[(0, 0)])?)?,
  )?);

  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.version, LightAnimFile::BGR_VERSION);
  assert!(file.is_bgr());

  Ok(())
}

#[test]
fn writes_a_library_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = sample()?;
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}
