use crate::shader::shader_struct::ShaderStruct;

/// The WGSL declarations a shader is given, each once, a struct after the structs it holds.
#[derive(Default)]
pub struct ShaderDeclarations {
  names: Vec<String>,
  texts: Vec<String>,
}

impl ShaderDeclarations {
  pub fn new() -> Self {
    Self::default()
  }

  /// Adds a type and everything it holds.
  pub fn declare<T: crate::shader::shader_type::ShaderType>(&mut self) -> &mut Self {
    T::declare(self);
    self
  }

  /// Adds a struct's own declaration, once; its members' are added first by its `ShaderType::declare`.
  pub fn declare_struct<T: ShaderStruct>(&mut self) {
    let name: String = T::get_wgsl_name();

    if !self.names.contains(&name) {
      self.names.push(name);
      self.texts.push(T::get_wgsl_declaration());
    }
  }

  pub fn to_wgsl(&self) -> String {
    self.texts.join("\n")
  }
}
