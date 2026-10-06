use crate::param::pass_binding::PassBinding;
use crate::shader::ShaderType;

/// A field `#[uniform]` binds: a value read at a dynamic offset into a buffer.
pub trait UniformField {
  type Value: ShaderType;

  fn get_binding(&self) -> PassBinding<'_>;

  fn get_dynamic_offset(&self) -> u32;
}
