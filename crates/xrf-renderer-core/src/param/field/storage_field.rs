use crate::graph::{GraphBuffer, GraphBufferAccess};
use crate::shader::ShaderType;

/// A field `#[storage]` binds: a graph buffer read, or read and written, as an array of `Element`.
pub trait StorageField {
  type Element: ShaderType;
  const IS_WRITABLE: bool;

  fn get_buffer(&self) -> GraphBuffer;

  fn get_access(&self) -> GraphBufferAccess {
    if Self::IS_WRITABLE {
      GraphBufferAccess::StorageReadWrite
    } else {
      GraphBufferAccess::StorageRead
    }
  }
}
