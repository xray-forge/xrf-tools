use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::efd_file::EfdFile;
use crate::efd_pattern::EfdPattern;
use crate::efd_variable::EfdVariable;

/// A whole evaluation function, laid out exactly as `vfLoadEF` reads one.
fn function_bytes(variables: &[(u32, u32)], patterns: &[&[u32]], parameters: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&1u32.to_le_bytes());
  bytes.extend_from_slice(&1u32.to_le_bytes());
  bytes.extend_from_slice(&(variables.len() as u32).to_le_bytes());

  for (range, _) in variables {
    bytes.extend_from_slice(&range.to_le_bytes());
  }

  for (_, kind) in variables {
    bytes.extend_from_slice(&kind.to_le_bytes());
  }

  bytes.extend_from_slice(&76u32.to_le_bytes());
  bytes.extend_from_slice(&0.0f32.to_le_bytes());
  bytes.extend_from_slice(&1.0f32.to_le_bytes());
  bytes.extend_from_slice(&(patterns.len() as u32).to_le_bytes());

  for pattern in patterns {
    bytes.extend_from_slice(&(pattern.len() as u32).to_le_bytes());

    for index in *pattern {
      bytes.extend_from_slice(&index.to_le_bytes());
    }
  }

  for index in 0..parameters {
    bytes.extend_from_slice(&(index as f32).to_le_bytes());
  }

  bytes
}

#[test]
fn a_function_reads_as_the_variables_patterns_and_weights_it_is() -> XrfResult {
  // Two inputs of three and five buckets over one pattern reading both: fifteen weights.
  let function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 15))?;

  assert_eq!(function.builder_version, EfdFile::CURRENT_VERSION);
  assert_eq!(function.function_type, 76);
  assert_eq!(
    function.variables,
    vec![EfdVariable { range: 3, kind: 10 }, EfdVariable { range: 5, kind: 11 },]
  );
  assert_eq!(function.patterns, vec![EfdPattern { variables: vec![0, 1] }]);
  assert_eq!(function.parameters.len(), 15);

  Ok(())
}

#[test]
fn a_function_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 15);
  let function: EfdFile = EfdFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  function.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn the_weight_block_is_sized_by_a_number_the_file_never_stores() -> XrfResult {
  // Two patterns: one over both inputs and one over the first alone, so 15 + 3.
  let function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1], &[0]], 18))?;

  assert_eq!(function.parameters.len(), 18);
  assert_eq!(function.patterns[0].get_complexity(&function.variables), Some(15));
  assert_eq!(function.patterns[1].get_complexity(&function.variables), Some(3));

  Ok(())
}

#[test]
fn a_weight_block_the_patterns_do_not_account_for_is_refused() {
  // One weight short of the fifteen the pattern claims.
  assert!(EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 14)).is_err());
}

#[test]
fn a_pattern_naming_an_input_the_function_does_not_declare_is_refused() {
  let error: String = EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10)], &[&[0, 4]], 3))
    .expect_err("an index past the variable list is refused")
    .to_string();

  assert!(error.contains("outside the 1"), "Unexpected error: {error}");
}

#[test]
fn a_builder_version_the_engine_refuses_is_refused_here_too() {
  let mut bytes: Vec<u8> = function_bytes(&[(3, 10)], &[&[0]], 3);

  bytes[0..4].copy_from_slice(&2u32.to_le_bytes());

  assert!(EfdFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());
}

#[test]
fn a_weight_block_disagreeing_with_the_patterns_is_refused_rather_than_written_short() -> XrfResult {
  let mut function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 15))?;

  function.parameters.pop();

  assert!(function.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_err());

  Ok(())
}
