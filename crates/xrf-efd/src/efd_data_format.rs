use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

/// How wide the real numbers of an evaluation function are stored: its result range and every weight.
///
/// The SDK's builder writes either: `DOUBLE_DATA_FORMAT` and `FLOAT_DATA_FORMAT` in IX-Ray's copy of its source,
/// `utils/efc/fitter.cpp`, which narrows each value with a plain cast when asked to convert. The engine reads only the
/// float layout: `vfLoadEF` never looks at the field, so a double file loads there as garbage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EfdDataFormat {
  /// Eight-byte reals, as the builder fits them.
  Double,
  /// Four-byte reals, the layout every shipped function is in and the only one the engine reads.
  Float,
}

impl EfdDataFormat {
  /// The format a header's `u32` names.
  ///
  /// # Errors
  ///
  /// Returns an error for a value the builder never writes.
  pub fn from_value(value: u32) -> XrfResult<Self> {
    match value {
      0 => Ok(Self::Double),
      1 => Ok(Self::Float),
      _ => Err(XrfError::new_not_implemented_error(format!(
        "Unexpected evaluation function data format {value}, the builder writes only 0 (double) and 1 (float)"
      ))),
    }
  }

  /// The `u32` a header stores for this format.
  pub const fn to_value(self) -> u32 {
    match self {
      Self::Double => 0,
      Self::Float => 1,
    }
  }

  /// Whether the engine reads a function stored in this format as what it is.
  pub const fn is_engine_readable(self) -> bool {
    matches!(self, Self::Float)
  }

  /// Bytes one real occupies in this format.
  pub const fn get_real_size(self) -> u64 {
    match self {
      Self::Double => 8,
      Self::Float => 4,
    }
  }

  /// Reads one real at this format's width.
  pub(crate) fn read_real<T: ByteOrder, D: ChunkDataSource>(self, reader: &mut ChunkReader<D>) -> XrfResult<f64> {
    Ok(match self {
      Self::Double => reader.read_f64::<T>()?,
      Self::Float => f64::from(reader.read_f32::<T>()?),
    })
  }

  /// Writes one real at this format's width, narrowing it for the float layout as the builder's cast does.
  pub(crate) fn write_real<T: ByteOrder>(self, writer: &mut ChunkWriter, value: f64) -> XrfResult {
    match self {
      Self::Double => writer.write_f64::<T>(value)?,
      Self::Float => writer.write_f32::<T>(value as f32)?,
    }

    Ok(())
  }
}
