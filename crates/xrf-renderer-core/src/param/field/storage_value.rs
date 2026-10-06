use std::marker::PhantomData;

use crate::graph::GraphBuffer;
use crate::param::field::storage_field::StorageField;
use crate::shader::ShaderType;

/// A graph buffer a pass reads as one `T` from its head, the rest of the buffer past it unread.
pub struct StorageValue<T: ShaderType> {
  buffer: GraphBuffer,
  value: PhantomData<T>,
}

impl<T: ShaderType> StorageValue<T> {
  pub fn new(buffer: GraphBuffer) -> Self {
    Self {
      buffer,
      value: PhantomData,
    }
  }
}

impl<T: ShaderType> Clone for StorageValue<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ShaderType> Copy for StorageValue<T> {}

impl<T: ShaderType> StorageField for StorageValue<T> {
  type Element = T;
  const IS_WRITABLE: bool = false;
  const IS_ARRAY: bool = false;

  fn get_buffer(&self) -> GraphBuffer {
    self.buffer
  }
}
