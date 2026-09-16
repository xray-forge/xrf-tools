use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::shader_compiler::shader_compiler_shader::ShaderCompilerShader;

/// The compiler shader library, `shaders_xrlc.xr`.
///
/// `Shader_xrLC_LIB::Load` (`utils/Shader_xrLC.h`) reads it as a plain array of fixed records and asserts that the
/// file divides by the record size, so it is not chunked at all - the only `.xr` in the trees that is not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderCompilerFile {
  pub shaders: Vec<ShaderCompilerShader>,
}

impl ShaderCompilerFile {
  /// Reads a compiler shader library from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a compiler shader library this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Compiler shader library was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a compiler shader library from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a compiler shader library this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived library arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a compiler shader library this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes do not divide into whole records, which is the assert the engine makes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let size: u64 = reader.read_bytes_remain();

    if !size.is_multiple_of(ShaderCompilerShader::SERIALIZED_SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected compiler shader library of {size} bytes, expected whole {}-byte records",
        ShaderCompilerShader::SERIALIZED_SIZE
      )));
    }

    let count: u64 = size / ShaderCompilerShader::SERIALIZED_SIZE;
    let mut shaders: Vec<ShaderCompilerShader> =
      reader.new_bounded_vec(count, ShaderCompilerShader::SERIALIZED_SIZE, "compiler shaders")?;

    for _ in 0..count {
      let (name, name_trailing) = ShaderCompilerShader::read_name(&reader.read_bytes(ShaderCompilerShader::NAME_SIZE)?)?;

      shaders.push(ShaderCompilerShader {
        name,
        name_trailing,
        flags: reader.read_u32::<T>()?,
        vertex_translucency: reader.read_f32::<T>()?,
        vertex_ambient: reader.read_f32::<T>()?,
        lightmap_density: reader.read_f32::<T>()?,
      });
    }

    reader.assert_read("Expect all data to be read from compiler shader library")?;

    Ok(Self { shaders })
  }

  /// Writes the library back as the plain array `Shader_xrLC_LIB::Save` writes.
  ///
  /// # Errors
  ///
  /// Returns an error when a name does not fit its fixed field, or the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for shader in &self.shaders {
      writer.buffer.extend_from_slice(&shader.write_name()?);
      writer.write_u32::<T>(shader.flags)?;
      writer.write_f32::<T>(shader.vertex_translucency)?;
      writer.write_f32::<T>(shader.vertex_ambient)?;
      writer.write_f32::<T>(shader.lightmap_density)?;
    }

    Ok(())
  }

  /// The shader a name resolves to, matched the way `Shader_xrLC_LIB::GetID` matches it.
  pub fn find_shader(&self, name: &str) -> Option<&ShaderCompilerShader> {
    self.shaders.iter().find(|shader| shader.name.eq_ignore_ascii_case(name))
  }
}
