use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::shader_compiler::shader_compiler_file::ShaderCompilerFile;
use crate::shader_compiler::shader_compiler_shader::ShaderCompilerShader;

/// One record, with whatever the editor left in the name field past the terminator.
fn shader(name: &str, trailing: &[u8], flags: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = name.as_bytes().to_vec();

  bytes.push(0);
  bytes.extend_from_slice(trailing);
  bytes.resize(ShaderCompilerShader::NAME_SIZE, 0);

  bytes.extend_from_slice(&flags.to_le_bytes());
  bytes.extend_from_slice(&0.5f32.to_le_bytes());
  bytes.extend_from_slice(&0.0f32.to_le_bytes());
  bytes.extend_from_slice(&1.0f32.to_le_bytes());

  bytes
}

/// A whole library out of the records it is given.
fn library(shaders: &[Vec<u8>]) -> Vec<u8> {
  shaders.concat()
}

#[test]
fn reads_every_record_of_a_plain_array() -> XrfResult {
  let file: ShaderCompilerFile = ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(library(&[
    shader("default", &[], 0x3F),
    shader("def_vertex", &[], 0x07),
  ]))?;

  assert_eq!(file.shaders.len(), 2);
  assert_eq!(file.shaders[0].name, "default");
  assert_eq!(file.shaders[1].lightmap_density, 1.0);

  Ok(())
}

#[test]
fn names_the_flags_a_shader_sets() -> XrfResult {
  let file: ShaderCompilerFile =
    ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(library(&[shader("default", &[], 0x07)]))?;

  assert_eq!(
    file.shaders[0].get_named_flags(),
    vec!["collision", "rendering", "optimize UV"]
  );

  Ok(())
}

#[test]
fn keeps_what_the_editor_left_past_a_name_so_a_library_rewrites_exactly() -> XrfResult {
  // `Shader_xrLC` writes its whole `char Name[128]` out of a struct it never zeroes, so vanilla's own first record
  // still carries a `0` from a name that used to be longer. Zero-padding on write would change every shipped file.
  let original: Vec<u8> = library(&[shader("default", b"0", 0x3F)]);
  let file: ShaderCompilerFile = ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

  assert_eq!(file.shaders[0].name, "default");
  assert_eq!(file.shaders[0].name_trailing[1], b'0');

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}

#[test]
fn finds_a_shader_the_way_the_compiler_matches_a_name() -> XrfResult {
  let file: ShaderCompilerFile =
    ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(library(&[shader("def_shadow", &[], 0x3F)]))?;

  assert!(file.find_shader("DEF_SHADOW").is_some());
  assert!(file.find_shader("missing").is_none());

  Ok(())
}

#[test]
fn refuses_a_library_that_does_not_divide_into_whole_records() -> XrfResult {
  // The engine asserts the same thing before reading anything, so a file that fails here is one it would refuse.
  let mut bytes: Vec<u8> = library(&[shader("default", &[], 0x3F)]);

  bytes.truncate(bytes.len() - 4);

  assert!(ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn refuses_a_name_field_with_no_terminator() -> XrfResult {
  let mut bytes: Vec<u8> = library(&[shader("default", &[], 0x3F)]);

  bytes[..ShaderCompilerShader::NAME_SIZE].fill(b'x');

  assert!(ShaderCompilerFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}
