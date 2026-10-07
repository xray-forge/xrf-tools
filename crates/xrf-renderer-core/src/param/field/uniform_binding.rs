use std::marker::PhantomData;

use crate::alloc::UploadSlice;
use crate::param::field::uniform_field::UniformField;
use crate::param::pass_binding::PassBinding;
use crate::param::pass_resources::PassResources;
use crate::shader::ShaderType;

/// A value pushed to the graph runtime's upload ring this frame, bound as a uniform at its offset.
pub struct UniformBinding<T: ShaderType> {
  slice: UploadSlice,
  value: PhantomData<fn() -> T>,
}

impl<T: ShaderType> UniformBinding<T> {
  /// The value at `slice` of the ring's buffer, which is resolved when the pass binds it, after the ring is flushed.
  pub fn new(slice: UploadSlice) -> Self {
    Self {
      slice,
      value: PhantomData,
    }
  }
}

impl<T: ShaderType> Clone for UniformBinding<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T: ShaderType> Copy for UniformBinding<T> {}

impl<T: ShaderType> UniformField for UniformBinding<T> {
  type Value = T;

  fn get_binding<'r>(&'r self, resources: &dyn PassResources<'r>) -> PassBinding<'r> {
    PassBinding::BufferRange {
      buffer: resources.get_upload_buffer(),
      offset: 0,
      size: T::SIZE,
    }
  }

  fn get_dynamic_offset(&self) -> u32 {
    self.slice.offset
  }
}
