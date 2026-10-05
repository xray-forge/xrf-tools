use crate::shader::shader_member::ShaderMember;
use crate::shader::shader_type::ShaderType;

/// A struct a shader and Rust share, declared once in Rust: `#[derive(ShaderStruct)]` lays it out by WGSL's rules,
/// asserts at compile time that Rust laid it out the same, and writes its WGSL declaration.
pub trait ShaderStruct: ShaderType {
  /// Its members in order, padding left out.
  const MEMBERS: &'static [ShaderMember];

  /// Its WGSL declaration.
  fn get_wgsl_declaration() -> String {
    let members: String = Self::MEMBERS
      .iter()
      .map(|member| format!("  {}: {},\n", member.name, (member.get_wgsl_name)()))
      .collect();

    format!("struct {} {{\n{members}}}\n", Self::get_wgsl_name())
  }
}
