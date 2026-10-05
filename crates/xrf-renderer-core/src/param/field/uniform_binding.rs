use std::marker::PhantomData;

use crate::alloc::UploadSlice;
use crate::param::field::uniform_field::UniformField;
use crate::param::pass_binding::PassBinding;
use crate::shader::ShaderType;

/// A value pushed to an upload ring this frame, bound as a uniform at its offset.
pub struct UniformBinding<'a, T: ShaderType> {
  buffer: &'a wgpu::Buffer,
  slice: UploadSlice,
  value: PhantomData<T>,
}

impl<'a, T: ShaderType> UniformBinding<'a, T> {
  /// The value at `slice` of the ring's buffer.
  pub fn new(buffer: &'a wgpu::Buffer, slice: UploadSlice) -> Self {
    Self {
      buffer,
      slice,
      value: PhantomData,
    }
  }
}

impl<T: ShaderType> UniformField for UniformBinding<'_, T> {
  type Value = T;

  fn get_binding(&self) -> PassBinding<'_> {
    PassBinding::BufferRange {
      buffer: self.buffer,
      size: T::SIZE,
    }
  }

  fn get_dynamic_offset(&self) -> u32 {
    self.slice.offset
  }
}
