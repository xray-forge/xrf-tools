use std::marker::PhantomData;

use crate::graph::GraphBuffer;
use crate::param::field::storage_field::StorageField;
use crate::shader::ShaderType;

/// A graph buffer a pass reads as an array of `T`, whole or a range of its bytes.
pub struct StorageArray<T: ShaderType> {
  buffer: GraphBuffer,
  range: Option<(u64, u64)>,
  element: PhantomData<T>,
}

impl<T: ShaderType> StorageArray<T> {
  pub fn new(buffer: GraphBuffer) -> Self {
    Self {
      buffer,
      range: None,
      element: PhantomData,
    }
  }

  /// `size` bytes of the buffer from `offset`, which is a multiple of the device's storage offset alignment.
  pub fn new_range(buffer: GraphBuffer, offset: u64, size: u64) -> Self {
    Self {
      buffer,
      range: Some((offset, size)),
      element: PhantomData,
    }
  }
}

impl<T: ShaderType> Clone for StorageArray<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ShaderType> Copy for StorageArray<T> {}

impl<T: ShaderType> StorageField for StorageArray<T> {
  type Element = T;
  const IS_WRITABLE: bool = false;

  fn get_buffer(&self) -> GraphBuffer {
    self.buffer
  }

  fn get_range(&self) -> Option<(u64, u64)> {
    self.range
  }
}
