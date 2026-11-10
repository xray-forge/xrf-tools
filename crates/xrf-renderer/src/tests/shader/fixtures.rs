//! Reading what a shader module declares: its constants' values and the module itself as naga parses it, for tests
//! holding WGSL to Rust or reading the bounds a shader works within.

use crate::shader::shader_library::ShaderLibrary;

/// A `u32` constant's value in a module.
pub(crate) fn get_constant(library: &ShaderLibrary, name: &str, constant: &str) -> u32 {
  let module: naga::Module = parse(library, name);

  match module.global_expressions[get_declared(&module, name, constant).init] {
    naga::Expression::Literal(naga::Literal::U32(value)) => value,
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}

/// An `i32` constant's value in a module.
pub(crate) fn get_signed_constant(library: &ShaderLibrary, name: &str, constant: &str) -> i32 {
  let module: naga::Module = parse(library, name);

  match module.global_expressions[get_declared(&module, name, constant).init] {
    naga::Expression::Literal(naga::Literal::I32(value)) => value,
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}

/// The value a module declares a float constant as.
pub(crate) fn get_float_constant(library: &ShaderLibrary, name: &str, constant: &str) -> f32 {
  let module: naga::Module = parse(library, name);

  read_float(&module, name, get_declared(&module, name, constant).init)
}

/// The components a module declares a float vector constant with.
pub(crate) fn get_vector_constant(library: &ShaderLibrary, name: &str, constant: &str) -> Vec<f32> {
  let module: naga::Module = parse(library, name);

  match module.global_expressions[get_declared(&module, name, constant).init] {
    naga::Expression::Compose { ref components, .. } => components
      .iter()
      .map(|component| read_float(&module, name, *component))
      .collect(),
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}

/// A module composed from the library and parsed by naga.
pub(crate) fn parse(library: &ShaderLibrary, name: &str) -> naga::Module {
  let source: String = library.compose(name).expect("composes");

  naga::front::wgsl::parse_str(&source).expect("parses")
}

fn get_declared<'m>(module: &'m naga::Module, name: &str, constant: &str) -> &'m naga::Constant {
  module
    .constants
    .iter()
    .map(|(_, declared)| declared)
    .find(|declared| declared.name.as_deref() == Some(constant))
    .unwrap_or_else(|| panic!("'{name}' declares no {constant}"))
}

fn read_float(module: &naga::Module, name: &str, expression: naga::Handle<naga::Expression>) -> f32 {
  match module.global_expressions[expression] {
    naga::Expression::Literal(naga::Literal::F32(value)) => value,
    ref other => panic!("'{name}' holds {other:?} where a float stands"),
  }
}
