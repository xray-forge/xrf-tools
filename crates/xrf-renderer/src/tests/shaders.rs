use crate::shader::shader_library::ShaderLibrary;

/// Every module composes and validates, so a shader error fails a test on a machine without a GPU.
#[test]
fn every_module_composes_and_validates() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let modules: Vec<&str> = library.list_modules().collect();

  assert!(!modules.is_empty());

  for name in modules {
    let source: String = library
      .compose(name, &[])
      .unwrap_or_else(|error| panic!("'{name}': {error}"));
    let module: naga::Module = naga::front::wgsl::parse_str(&source)
      .unwrap_or_else(|error| panic!("'{name}' does not parse:\n{}", error.emit_to_string(&source)));

    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
      .validate(&module)
      .unwrap_or_else(|error| panic!("'{name}' does not validate:\n{}", error.emit_to_string(&source)));
  }
}
