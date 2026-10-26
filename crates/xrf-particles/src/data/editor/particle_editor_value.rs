use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::data::editor::particle_editor_domain::ParticleEditorDomain;
use crate::data::editor::particle_editor_value_kind::ParticleEditorValueKind;

/// One property's value, as `EParticleAction` keeps it in its `floats`, `vectors`, `domains`, `bools` and `ints` maps.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum ParticleEditorValue {
  Float(f32),
  Vector(Vector3d),
  Domain(ParticleEditorDomain),
  Bool(bool),
  Int(i32),
}

impl ParticleEditorValue {
  pub fn get_kind(&self) -> ParticleEditorValueKind {
    match self {
      Self::Float(_) => ParticleEditorValueKind::Float,
      Self::Vector(_) => ParticleEditorValueKind::Vector,
      Self::Domain(_) => ParticleEditorValueKind::Domain,
      Self::Bool(_) => ParticleEditorValueKind::Bool,
      Self::Int(_) => ParticleEditorValueKind::Int,
    }
  }

  /// Reads a value of the kind this one is, replacing it: the kind is the schema's, never the file's.
  ///
  /// # Errors
  ///
  /// Returns an error when the reader ends before the value does.
  pub fn read_same_kind<T: ByteOrder, D: ChunkDataSource>(&self, reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(match self {
      Self::Float(_) => Self::Float(reader.read_f32::<T>()?),
      Self::Vector(_) => Self::Vector(reader.read_xr::<T, _>()?),
      Self::Domain(_) => Self::Domain(reader.read_xr::<T, _>()?),
      // `PBool` keeps a `BOOL`, saved as its low byte: any byte but zero is true.
      Self::Bool(_) => Self::Bool(reader.read_u8()? != 0),
      Self::Int(_) => Self::Int(reader.read_i32::<T>()?),
    })
  }

  /// Writes the value as `EParticleAction::Save` does.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer fails.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    match self {
      Self::Float(value) => writer.write_f32::<T>(*value)?,
      Self::Vector(value) => writer.write_xr::<T, _>(value)?,
      Self::Domain(value) => writer.write_xr::<T, _>(value)?,
      Self::Bool(value) => writer.write_u8(u8::from(*value))?,
      Self::Int(value) => writer.write_i32::<T>(*value)?,
    }

    Ok(())
  }
}
