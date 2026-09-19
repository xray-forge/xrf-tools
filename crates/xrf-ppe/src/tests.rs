use std::io::Write;

use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;

use crate::ppe_color::PpeColor;
use crate::ppe_file::PpeFile;

/// One interpolating key, as `st_Key::Load_2` lays it out: value, time, shape, then seven quantised parameters.
fn key_bytes(value: f32, time: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&value.to_le_bytes());
  bytes.extend_from_slice(&time.to_le_bytes());
  bytes.push(0);
  bytes.extend_from_slice(&[0u8; 7 * 2]);

  bytes
}

/// An envelope: the two behaviours, the key count, then the keys.
fn envelope_bytes(keys: &[Vec<u8>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = vec![1, 1];

  bytes.extend_from_slice(&(keys.len() as u16).to_le_bytes());

  for key in keys {
    bytes.extend_from_slice(key);
  }

  bytes
}

/// A colour parameter: the unread base, then one envelope per channel.
fn color_bytes(keys: &[Vec<u8>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&0.5f32.to_le_bytes());
  bytes.extend(envelope_bytes(keys));
  bytes.extend(envelope_bytes(&[]));
  bytes.extend(envelope_bytes(&[]));

  bytes
}

/// A whole effect, flat, exactly as `BasicPostProcessAnimator::Load` reads one.
fn effect_bytes(version: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&version.to_le_bytes());

  // Base colour carries the only keyed channel; the other two colours are empty throughout.
  bytes.extend(color_bytes(&[key_bytes(0.0, 0.0), key_bytes(1.0, 3.0)]));
  bytes.extend(color_bytes(&[]));
  bytes.extend(color_bytes(&[]));

  // Gray value is keyed and runs longer than the colour; the six remaining scalars are empty.
  bytes.extend(envelope_bytes(&[key_bytes(0.0, 0.5), key_bytes(0.8, 5.5)]));

  for _ in 1..PpeFile::VALUE_COUNT {
    bytes.extend(envelope_bytes(&[]));
  }

  if version >= PpeFile::COLOR_MAP_VERSION {
    bytes.extend(envelope_bytes(&[key_bytes(1.0, 0.0)]));
    bytes.extend_from_slice(b"grad\\grad_psi\0");
  }

  bytes
}

#[test]
fn an_effect_reads_as_the_version_and_the_parameters_it_is() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;

  assert_eq!(effect.version, 2);
  assert_eq!(effect.colors.len(), PpeFile::COLOR_COUNT);
  assert_eq!(effect.values.len(), PpeFile::VALUE_COUNT);
  assert_eq!(effect.colors[0].red.keys.len(), 2);
  assert!(effect.colors[0].green.keys.is_empty());
  assert_eq!(effect.values[0].keys.len(), 2);

  Ok(())
}

#[test]
fn a_version_one_effect_carries_no_color_map_at_all() -> XrfResult {
  // Absent, rather than present and empty: the engine never reads the parameter for such a file.
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(1))?;

  assert_eq!(effect.version, 1);
  assert_eq!(effect.color_map, None);

  Ok(())
}

#[test]
fn a_version_two_effect_carries_the_grading_and_the_texture_it_names() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;
  let color_map = effect.color_map.expect("a version 2 effect carries a color map");

  assert_eq!(color_map.texture, "grad\\grad_psi");
  assert_eq!(color_map.influence.keys.len(), 1);
  assert!(color_map.is_used());

  Ok(())
}

#[test]
fn an_effect_is_written_back_byte_for_byte() -> XrfResult {
  for version in PpeFile::SUPPORTED_VERSIONS {
    let original: Vec<u8> = effect_bytes(version);
    let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    effect.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, original, "version {version} did not round trip");
  }

  Ok(())
}

#[test]
fn a_version_kept_rather_than_raised_is_what_a_write_emits() -> XrfResult {
  // The engine's own save always writes 2, which would append parameters a version 1 author never stored.
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(1))?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  effect.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer.len(), effect_bytes(1).len());
  assert_ne!(effect.version, PpeFile::CURRENT_VERSION);

  Ok(())
}

#[test]
fn a_version_nothing_here_reads_is_refused_by_name() {
  let mut bytes: Vec<u8> = 7u32.to_le_bytes().to_vec();

  bytes.extend(effect_bytes(2).into_iter().skip(4));

  let error: String = PpeFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("an unknown version is refused")
    .to_string();

  assert!(error.contains('7'), "'{error}' names the version it refused");
}

#[test]
fn a_color_map_a_version_cannot_hold_is_refused_rather_than_written() -> XrfResult {
  let mut effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;

  effect.version = 1;

  assert!(effect.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_err());

  effect.color_map = None;

  assert!(effect.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_ok());

  Ok(())
}

#[test]
fn an_effect_missing_a_parameter_is_refused_rather_than_written_short() -> XrfResult {
  let mut effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;

  effect.values.pop();

  assert!(effect.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_err());

  Ok(())
}

#[test]
fn a_payload_the_parameters_do_not_account_for_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = effect_bytes(2);

  bytes.extend_from_slice(&[0, 0, 0, 0]);

  assert!(PpeFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn an_effect_runs_as_long_as_its_longest_parameter() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;

  // The base colour spans 3 s and the gray value 5, so the effect is 5 - not 5.5, which is where its last key sits.
  assert_eq!(effect.get_length_seconds(), 5.0);
  assert_eq!(effect.get_keys_count(), 5);
  assert!(effect.is_keyed());

  Ok(())
}

#[test]
fn an_effect_keyed_on_nothing_runs_for_no_time() -> XrfResult {
  let mut effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;

  effect.colors = (0..PpeFile::COLOR_COUNT)
    .map(|_| PpeColor {
      base: 0.0,
      red: AnimationEnvelope {
        behavior: (1, 1),
        keys: Vec::new(),
      },
      green: AnimationEnvelope {
        behavior: (1, 1),
        keys: Vec::new(),
      },
      blue: AnimationEnvelope {
        behavior: (1, 1),
        keys: Vec::new(),
      },
    })
    .collect();
  effect.values = (0..PpeFile::VALUE_COUNT)
    .map(|_| AnimationEnvelope {
      behavior: (1, 1),
      keys: Vec::new(),
    })
    .collect();
  effect.color_map = None;
  effect.version = 1;

  assert_eq!(effect.get_length_seconds(), 0.0);
  assert_eq!(effect.get_keys_count(), 0);
  assert!(!effect.is_keyed());

  Ok(())
}

#[test]
fn a_colour_counts_the_keys_of_all_three_channels_rather_than_the_reds() -> XrfResult {
  // `CPostProcessColor::get_keys_count` answers with the red channel alone, which is an editor convenience.
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(effect_bytes(2))?;

  assert_eq!(effect.colors[0].get_keys_count(), 2);
  assert_eq!(effect.colors[0].get_length_seconds(), 3.0);

  Ok(())
}

#[test]
fn an_effect_reads_the_same_off_a_reader_as_off_its_bytes() -> XrfResult {
  let bytes: Vec<u8> = effect_bytes(2);
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_all(&bytes)?;

  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes.clone())?;

  assert_eq!(
    PpeFile::read::<XRayByteOrder, _>(&mut reader)?,
    PpeFile::read_from_bytes::<XRayByteOrder>(bytes)?
  );

  Ok(())
}
