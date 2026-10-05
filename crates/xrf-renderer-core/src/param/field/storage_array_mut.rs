use std::marker::PhantomData;

use crate::graph::GraphBuffer;
use crate::param::field::storage_field::StorageField;
use crate::shader::ShaderType;

/// A graph buffer a pass reads and writes as an array of `T`.
pub struct StorageArrayMut<T: ShaderType> {
  buffer: GraphBuffer,
  element: PhantomData<T>,
}

impl<T: ShaderType> StorageArrayMut<T> {
  pub fn new(buffer: GraphBuffer) -> Self {
    Self {
      buffer,
      element: PhantomData,
    }
  }
}

impl<T: ShaderType> StorageField for StorageArrayMut<T> {
  type Element = T;
  const IS_WRITABLE: bool = true;

  fn get_buffer(&self) -> GraphBuffer {
    self.buffer
  }
}
