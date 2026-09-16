use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::level::level_env_mod_file::LevelEnvModFile;
use crate::level::level_env_modifier::EnvModifier;
use crate::level::tests::fixtures::{chunk, floats};

/// One environment modifier, at the width its version gives it.
fn modifier_bytes(flags: Option<u16>) -> Vec<u8> {
  let mut bytes: Vec<u8> = floats(&[
    10.0, 2.0, -5.0, // position
    30.0, 0.5, 250.0, // radius, power, far plane
    0.4, 0.4, 0.5,  // fog colour
    0.02, // fog density
    0.1, 0.1, 0.1, // ambient
    0.3, 0.4, 0.6, // sky
    0.2, 0.2, 0.3, // hemi
  ]);

  if let Some(flags) = flags {
    bytes.extend_from_slice(&flags.to_le_bytes());
  }

  bytes
}

/// A modifier list: a version chunk, then one numbered chunk per record.
fn file_bytes(version: u32, modifiers: &[Option<u16>]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LevelEnvModFile::VERSION_CHUNK_ID, &version.to_le_bytes())?;

  for (index, flags) in modifiers.iter().enumerate() {
    bytes.extend(chunk(index as u32 + 1, &modifier_bytes(*flags))?);
  }

  Ok(bytes)
}

#[test]
fn an_environment_modifier_carries_the_flag_word_its_version_admits() -> XrfResult {
  let file: LevelEnvModFile = LevelEnvModFile::read_from_bytes::<XRayByteOrder>(file_bytes(0x0017, &[Some(0b110)])?)?;

  assert_eq!(file.version, 0x0017);
  assert_eq!(file.modifiers.len(), 1);
  assert_eq!(file.modifiers[0].radius, 30.0);
  assert_eq!(
    file.modifiers[0].get_used_parameters(),
    vec!["fog colour", "fog density"]
  );

  Ok(())
}

#[test]
fn a_modifier_below_the_flag_version_mixes_in_everything() -> XrfResult {
  // `use_flags.one()` runs before the read, so a file too old to carry the word uses every value.
  let file: LevelEnvModFile = LevelEnvModFile::read_from_bytes::<XRayByteOrder>(file_bytes(0x0015, &[None])?)?;

  assert_eq!(file.modifiers[0].use_flags, None);
  assert_eq!(file.modifiers[0].get_used_parameters().len(), 6);

  Ok(())
}

#[test]
fn a_file_carrying_only_a_version_carries_no_modifiers_rather_than_failing() -> XrfResult {
  // Which is what most shipped levels are: a version chunk and nothing else.
  let file: LevelEnvModFile = LevelEnvModFile::read_from_bytes::<XRayByteOrder>(file_bytes(0x0017, &[])?)?;

  assert!(file.modifiers.is_empty());

  Ok(())
}

#[test]
fn a_modifier_list_is_written_back_byte_for_byte() -> XrfResult {
  for (version, modifiers) in [(0x0017u32, vec![Some(0b110)]), (0x0015, vec![None]), (0x0017, vec![])] {
    let bytes: Vec<u8> = file_bytes(version, &modifiers)?;
    let file: LevelEnvModFile = LevelEnvModFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    file.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, bytes, "version {version:#06x} did not round trip");
  }

  Ok(())
}

#[test]
fn a_modifier_is_the_width_its_version_declares() {
  assert_eq!(EnvModifier::SERIALIZED_SIZE, 76);
}
