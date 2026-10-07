use crate::shader::shader_type::ShaderType;

/// A `u32` shaders change atomically, as an element of a storage array: `atomic<u32>` in WGSL, a plain `u32` in Rust.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderAtomicU32(pub u32);

impl ShaderType for ShaderAtomicU32 {
  const ALIGN: u64 = 4;
  const SIZE: u64 = 4;

  fn get_wgsl_name() -> String {
    String::from("atomic<u32>")
  }
}
