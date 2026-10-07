use crate::graph::{GraphBuffer, GraphBufferAccess};
use crate::param::pass_binding::PassBinding;
use crate::param::pass_resources::PassResources;
use crate::shader::ShaderType;

/// A parameter field bound as a storage buffer: an array of `Element`, or one, read or read and written.
pub trait StorageField {
  type Element: ShaderType;
  const IS_WRITABLE: bool;
  /// Whether WGSL reads it as `array<Element>` rather than as one `Element`.
  const IS_ARRAY: bool = true;

  fn get_buffer(&self) -> GraphBuffer;

  /// The bytes of the buffer bound, as offset and size; the whole buffer where none.
  fn get_range(&self) -> Option<(u64, u64)> {
    None
  }

  fn get_access(&self) -> GraphBufferAccess {
    if Self::IS_WRITABLE {
      GraphBufferAccess::StorageReadWrite
    } else {
      GraphBufferAccess::StorageRead
    }
  }

  fn get_binding<'r>(&self, resources: &dyn PassResources<'r>) -> PassBinding<'r> {
    let buffer: &'r wgpu::Buffer = resources.get_buffer(self.get_buffer());

    match self.get_range() {
      Some((offset, size)) => PassBinding::BufferRange { buffer, offset, size },
      None => PassBinding::Buffer(buffer),
    }
  }
}
