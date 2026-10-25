use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::efd_data_format::EfdDataFormat;
use crate::efd_file::EfdFile;
use crate::efd_pattern::EfdPattern;
use crate::efd_variable::EfdVariable;

/// A whole float evaluation function, laid out exactly as `vfLoadEF` reads one, its weights counting up from zero.
fn function_bytes(variables: &[(u32, u32)], patterns: &[&[u32]], parameters: usize) -> Vec<u8> {
  function_bytes_in(EfdDataFormat::Float, variables, patterns, parameters)
}

/// The same layout in either of the builder's data formats, which decides how wide every real is written.
fn function_bytes_in(
  format: EfdDataFormat,
  variables: &[(u32, u32)],
  patterns: &[&[u32]],
  parameters: usize,
) -> Vec<u8> {
  let real = |value: f64| -> Vec<u8> {
    match format {
      EfdDataFormat::Double => value.to_le_bytes().to_vec(),
      EfdDataFormat::Float => (value as f32).to_le_bytes().to_vec(),
    }
  };
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&1u32.to_le_bytes());
  bytes.extend_from_slice(&format.to_value().to_le_bytes());
  bytes.extend_from_slice(&(variables.len() as u32).to_le_bytes());

  for (range, _) in variables {
    bytes.extend_from_slice(&range.to_le_bytes());
  }

  for (_, kind) in variables {
    bytes.extend_from_slice(&kind.to_le_bytes());
  }

  bytes.extend_from_slice(&76u32.to_le_bytes());
  bytes.extend(real(0.0));
  bytes.extend(real(100.0));
  bytes.extend_from_slice(&(patterns.len() as u32).to_le_bytes());

  for pattern in patterns {
    bytes.extend_from_slice(&(pattern.len() as u32).to_le_bytes());

    for index in *pattern {
      bytes.extend_from_slice(&index.to_le_bytes());
    }
  }

  for index in 0..parameters {
    bytes.extend(real(index as f64));
  }

  bytes
}

#[test]
fn a_function_reads_as_the_variables_patterns_and_weights_it_is() -> XrfResult {
  // Two inputs of three and five buckets over one pattern reading both: fifteen weights.
  let function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 15))?;

  assert_eq!(function.builder_version, EfdFile::CURRENT_VERSION);
  assert_eq!(function.data_format, EfdDataFormat::Float);
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
fn a_double_function_reads_and_writes_at_eight_bytes_a_real() -> XrfResult {
  // What the builder writes when it is not asked to convert: the result range and every weight as doubles.
  let bytes: Vec<u8> = function_bytes_in(EfdDataFormat::Double, &[(3, 10), (5, 11)], &[&[0, 1]], 15);
  let function: EfdFile = EfdFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(function.data_format, EfdDataFormat::Double);
  assert!(!function.data_format.is_engine_readable());
  assert_eq!(function.maximum_result, 100.0);
  assert_eq!(function.parameters.len(), 15);
  assert_eq!(function.parameters[14], 14.0);

  let mut writer: ChunkWriter = ChunkWriter::new();

  function.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_data_format_the_builder_never_writes_is_refused() {
  let mut bytes: Vec<u8> = function_bytes(&[(3, 10)], &[&[0]], 3);

  bytes[4..8].copy_from_slice(&2u32.to_le_bytes());

  let error: String = EfdFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("expect format 2 to be refused")
    .to_string();

  assert!(error.contains("data format 2"), "Unexpected error: {error}");
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

#[test]
fn a_pattern_reads_its_inputs_as_one_number_with_the_first_most_significant() -> XrfResult {
  // `dwfGetPatternIndex`: input 0 of range 3 then input 1 of range 5, so (2, 4) is 2 * 5 + 4.
  let function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 15))?;

  assert_eq!(function.patterns[0].get_index(&[2, 4], &function.variables), Some(14));
  assert_eq!(function.patterns[0].get_index(&[2, 1], &function.variables), Some(11));
  // Listing the inputs the other way round makes input 1 the significant one: 1 * 3 + 2.
  assert_eq!(
    EfdPattern { variables: vec![1, 0] }.get_index(&[2, 1], &function.variables),
    Some(5)
  );

  Ok(())
}

#[test]
fn a_function_evaluates_to_the_weights_its_patterns_select() -> XrfResult {
  // Weights count up from zero, so each pattern's pick is its offset plus its own index: 14 from the first and
  // 15 + 2 from the second, which reads input 0 alone.
  let function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1], &[0]], 18))?;

  assert_eq!(function.evaluate(&[2, 4])?, 14.0 + 17.0);
  assert_eq!(function.evaluate(&[0, 0])?, 0.0 + 15.0);

  Ok(())
}

#[test]
fn a_function_refuses_values_it_cannot_select_a_weight_for() -> XrfResult {
  let function: EfdFile =
    EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10), (5, 11)], &[&[0, 1]], 15))?;

  assert!(function.evaluate(&[2]).is_err(), "one value short");
  assert!(function.evaluate(&[3, 0]).is_err(), "past the first input's range");

  Ok(())
}

#[test]
fn a_result_is_bucketed_over_the_function_range_as_the_engine_rounds_it() -> XrfResult {
  // Over 0 to 100 into 21 buckets: a bucket is five units wide, and a value rounds to the nearest.
  let function: EfdFile = EfdFile::read_from_bytes::<XRayByteOrder>(function_bytes(&[(3, 10)], &[&[0]], 3))?;

  assert_eq!(function.get_discrete_value(-5.0, 21), 0, "at or below the minimum");
  assert_eq!(function.get_discrete_value(50.0, 21), 10);
  assert_eq!(function.get_discrete_value(52.4, 21), 10, "rounds down below the half");
  assert_eq!(function.get_discrete_value(52.5, 21), 11, "and up from it");
  assert_eq!(function.get_discrete_value(100.0, 21), 20, "at or above the maximum");
  assert_eq!(
    function.get_discrete_value(50.0, 0),
    0,
    "a range of nothing has only bucket zero"
  );

  Ok(())
}
