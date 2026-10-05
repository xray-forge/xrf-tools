use crate::shader::shader_declarations::ShaderDeclarations;

/// A type a shader and Rust share: its WGSL name, and its alignment and size by WGSL's host-shareable layout rules.
/// Implemented for the scalars, glam's vectors and matrices whose layout WGSL shares, arrays, and every
/// `#[derive(ShaderStruct)]`.
pub trait ShaderType {
  const ALIGN: u64;
  const SIZE: u64;
  /// Whether it is a struct with a declaration of its own.
  const IS_STRUCT: bool = false;

  fn get_wgsl_name() -> String;

  /// Adds the declarations it needs, a struct's own and its members', to those a shader is given.
  fn declare(_declarations: &mut ShaderDeclarations) {}
}
