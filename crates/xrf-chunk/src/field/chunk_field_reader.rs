use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::read_write::chunk_field_read_write::ChunkFieldReadWrite;
use crate::read_write::chunk_field_read_write_list::ChunkFieldReadWriteList;
use crate::read_write::chunk_field_read_write_optional::ChunkFieldReadWriteOptional;

/// What a [`ChunkFieldReadWrite`] value reads itself from, mirroring [`crate::ChunkFieldWriter`]: bytes through a
/// [`crate::ChunkReader`], fields through a [`crate::ChunkFieldReplay`] as `NET_Packet` reads its ini stream.
///
/// Method names match [`crate::ChunkReader`]'s, so a value's code reads the same over either.
pub trait ChunkFieldReader {
  /// Whether values come from fields rather than bytes, `NET_Packet::inistream != nullptr`.
  fn is_field_form(&self) -> bool;

  fn read_u8(&mut self) -> XrfResult<u8>;

  fn read_u16<T: ByteOrder>(&mut self) -> XrfResult<u16>;

  fn read_u32<T: ByteOrder>(&mut self) -> XrfResult<u32>;

  fn read_u64<T: ByteOrder>(&mut self) -> XrfResult<u64>;

  fn read_f32<T: ByteOrder>(&mut self) -> XrfResult<f32>;

  /// Three floats as one value, `r_vec3`.
  fn read_vector3<T: ByteOrder>(&mut self) -> XrfResult<[f32; 3]>;

  /// A null-terminated Windows-1251 string, `r_stringZ`.
  fn read_w1251_string(&mut self) -> XrfResult<String>;

  /// `count` bytes as they are, `r`.
  fn read_bytes(&mut self, count: usize) -> XrfResult<Vec<u8>>;

  /// The bytes read so far, `r_tell`.
  fn get_position(&self) -> usize;

  /// Whether nothing is left to read, `r_eof`.
  fn is_ended(&self) -> bool;

  /// An empty vector sized for `count` declared records of at least `min_record_size` bytes each.
  ///
  /// Fields have no byte budget, so the default reserves nothing; a count too large fails when the fields run out.
  ///
  /// # Errors
  ///
  /// Returns an error, for bytes, when `count` records cannot fit the bytes left.
  fn new_bounded_vec<V>(&self, count: u64, min_record_size: u64, what: &str) -> XrfResult<Vec<V>> {
    let _ = (count, min_record_size, what);

    Ok(Vec::new())
  }

  fn read_xr<T: ByteOrder, V: ChunkFieldReadWrite>(&mut self) -> XrfResult<V>
  where
    Self: Sized,
  {
    V::read_fields::<T, Self>(self)
  }

  fn read_xr_list<T: ByteOrder, V: ChunkFieldReadWriteList>(&mut self) -> XrfResult<Vec<V>>
  where
    Self: Sized,
  {
    V::read_field_list::<T, Self>(self)
  }

  fn read_xr_optional<T: ByteOrder, V: ChunkFieldReadWriteOptional>(&mut self) -> XrfResult<Option<V>>
  where
    Self: Sized,
  {
    V::read_field_optional::<T, Self>(self)
  }

  /// An `xr_vector<u16>` as `load_data` reads it: a `u32` count, then each.
  fn read_u16_vector<T: ByteOrder>(&mut self) -> XrfResult<Vec<u16>> {
    let count: u32 = self.read_u32::<T>()?;

    (0..count).map(|_| self.read_u16::<T>()).collect()
  }
}
