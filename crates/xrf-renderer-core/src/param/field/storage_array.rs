use std::marker::PhantomData;

use crate::graph::GraphBuffer;
use crate::param::field::storage_field::StorageField;
use crate::shader::ShaderType;

/// A graph buffer a pass reads as an array of `T`.
pub struct StorageArray<T: ShaderType> {
  buffer: GraphBuffer,
  element: PhantomData<T>,
}

impl<T: ShaderType> StorageArray<T> {
  pub fn new(buffer: GraphBuffer) -> Self {
    Self {
      buffer,
      element: PhantomData,
    }
  }
}

impl<T: ShaderType> StorageField for StorageArray<T> {
  type Element = T;
  const IS_WRITABLE: bool = false;

  fn get_buffer(&self) -> GraphBuffer {
    self.buffer
  }
}
