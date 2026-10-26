use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::light_anim_file::LightAnimFile;
use crate::light_anim_item::LightAnimItem;
use crate::light_anim_key::LightAnimKey;

/// Frames a payload as one X-Ray chunk.
fn chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// One animation, as the library stores it.
fn item(name: &str, fps: f32, frames: u32, keys: &[(i32, u32)]) -> XrfResult<Vec<u8>> {
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
fn reads_a_key_before_frame_zero_as_the_engine_does() -> XrfResult {
  // `svet_01` in every shipped library keys frame -1, which an unsigned reading turns into the last frame there is.
  let original: Vec<u8> = library(
    1,
    &[item(
      "svet_01",
      30.0,
      30,
      &[(-1, 0x00E9_CE72), (0, 0x00FF_C066), (29, 0x00FF_C066)],
    )?],
  )?;
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  assert_eq!(file.items[0].keys[0].frame, -1);
  assert_eq!(
    file.items[0]
      .to_loaded_keys(false)
      .iter()
      .map(|key| key.frame)
      .collect::<Vec<i32>>(),
    [-1, 0, 29]
  );

  Ok(())
}

#[test]
fn loads_keys_by_frame_keeping_the_last_colour_of_a_repeated_one() -> XrfResult {
  // `CLAItem::Keys` is a map, so a frame listed twice holds what was read last and the keys come out by frame.
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(library(
    1,
    &[item(
      "repeated",
      15.0,
      10,
      &[(5, 0x0000_0005), (0, 0x0000_0000), (5, 0x0000_0006)],
    )?],
  )?)?;

  assert_eq!(
    file.items[0].to_loaded_keys(false),
    [LightAnimKey { frame: 0, color: 0 }, LightAnimKey { frame: 5, color: 6 },]
  );

  Ok(())
}

#[test]
fn loads_a_bgr_library_with_red_and_blue_exchanged_and_alpha_kept() -> XrfResult {
  let file: LightAnimFile =
    LightAnimFile::read_from_bytes::<XRayByteOrder>(library(0, &[item("old", 15.0, 1, &[(0, 0x80_102030)])?])?)?;
  let loaded: Vec<LightAnimKey> = file.items[0].to_loaded_keys(file.is_bgr());

  assert_eq!(loaded[0].color, 0x80_302010);
  assert_eq!(
    (loaded[0].get_alpha(), loaded[0].get_red(), loaded[0].get_blue()),
    (0x80, 0x30, 0x10)
  );
  assert_eq!(
    file.items[0].keys[0].color, 0x80_102030,
    "the stored key is kept as stored"
  );

  Ok(())
}

#[test]
fn finds_an_animation_by_its_exact_name_only() -> XrfResult {
  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(sample()?)?;

  assert_eq!(file.find_item("campfire").map(|item| item.frame_count), Some(30));
  assert!(file.find_item("Campfire").is_none(), "names compare with their case");
  assert!(file.find_item("").is_none(), "an empty name names no animation");

  Ok(())
}

#[test]
fn stops_reading_animations_at_the_first_number_the_list_lacks() -> XrfResult {
  // `ELightAnimLibrary::Load` opens chunk 0, 1, 2 and on, so one numbered past a gap never reaches the game.
  let mut payload: Vec<u8> = chunk(0, &item("first", 15.0, 1, &[(0, 0)])?)?;

  payload.extend(chunk(2, &item("past_the_gap", 15.0, 1, &[(0, 0)])?)?);

  let mut bytes: Vec<u8> = chunk(LightAnimFile::VERSION_CHUNK_ID, &1u16.to_le_bytes())?;

  bytes.extend(chunk(LightAnimFile::ITEMS_CHUNK_ID, &payload)?);

  let file: LightAnimFile = LightAnimFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(
    file.items.iter().map(|item| item.name.as_str()).collect::<Vec<&str>>(),
    ["first"]
  );

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
