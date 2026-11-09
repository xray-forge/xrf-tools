use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};

use crate::field::chunk_field::ChunkField;
use crate::field::chunk_field_kind::ChunkFieldKind;
use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::field::stream::chunk_field_source::ChunkFieldSource;

/// Reads values from fields, as a `NET_Packet` with an ini stream attached reads one field per primitive; the reverse
/// of a [`crate::ChunkFieldRecorder`]. Its position counts the bytes the values take, as the recorder's does. A raw
/// read of no bytes reads nothing, any other is refused.
pub struct ChunkFieldReplay<'s, S: ChunkFieldSource + ?Sized> {
  source: &'s mut S,
  position: usize,
}

impl<'s, S: ChunkFieldSource + ?Sized> ChunkFieldReplay<'s, S> {
  pub const fn new(source: &'s mut S) -> Self {
    Self { source, position: 0 }
  }

  /// The next field, which must be of `kind`, unwrapped by `take`.
  fn read<V>(&mut self, kind: ChunkFieldKind, take: impl FnOnce(ChunkField) -> Option<V>) -> XrfResult<V> {
    let field: ChunkField = self.source.read_field(kind)?;
    let answered: ChunkFieldKind = field.get_kind();

    self.position += field.measure_bytes()?;

    take(field)
      .ok_or_else(|| XrfError::new_parsing_error(format!("A {answered:?} field stands where a {kind:?} is read")))
  }
}

impl<S: ChunkFieldSource + ?Sized> ChunkFieldReader for ChunkFieldReplay<'_, S> {
  fn is_field_form(&self) -> bool {
    true
  }

  fn read_u8(&mut self) -> XrfResult<u8> {
    self.read(ChunkFieldKind::U8, |field| match field {
      ChunkField::U8(value) => Some(value),
      _ => None,
    })
  }

  fn read_u16<T: ByteOrder>(&mut self) -> XrfResult<u16> {
    self.read(ChunkFieldKind::U16, |field| match field {
      ChunkField::U16(value) => Some(value),
      _ => None,
    })
  }

  fn read_u32<T: ByteOrder>(&mut self) -> XrfResult<u32> {
    self.read(ChunkFieldKind::U32, |field| match field {
      ChunkField::U32(value) => Some(value),
      _ => None,
    })
  }

  fn read_u64<T: ByteOrder>(&mut self) -> XrfResult<u64> {
    self.read(ChunkFieldKind::U64, |field| match field {
      ChunkField::U64(value) => Some(value),
      _ => None,
    })
  }

  fn read_f32<T: ByteOrder>(&mut self) -> XrfResult<f32> {
    self.read(ChunkFieldKind::F32, |field| match field {
      ChunkField::F32(value) => Some(value),
      _ => None,
    })
  }

  fn read_vector3<T: ByteOrder>(&mut self) -> XrfResult<[f32; 3]> {
    self.read(ChunkFieldKind::Vector3, |field| match field {
      ChunkField::Vector3(value) => Some(value),
      _ => None,
    })
  }

  fn read_w1251_string(&mut self) -> XrfResult<String> {
    self.read(ChunkFieldKind::String, |field| match field {
      ChunkField::String(value) => Some(value),
      _ => None,
    })
  }

  fn read_bytes(&mut self, count: usize) -> XrfResult<Vec<u8>> {
    if count == 0 {
      return Ok(Vec::new());
    }

    Err(XrfError::new_parsing_error(format!(
      "Cannot read {count} raw bytes from fields, which hold none"
    )))
  }

  fn get_position(&self) -> usize {
    self.position
  }

  fn is_ended(&self) -> bool {
    self.source.is_ended()
  }
}
