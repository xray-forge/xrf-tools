use crate::pass::water_uniform::WaterUniform;
use crate::shader::shader_library::ShaderLibrary;

/// Every module composes and validates, so a shader error fails a test on a machine without a GPU.
#[test]
fn every_module_composes_and_validates() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let modules: Vec<&str> = library.list_modules().collect();

  assert!(!modules.is_empty());

  for name in modules {
    let source: String = library
      .compose(name)
      .unwrap_or_else(|error| panic!("'{name}': {error}"));
    let module: naga::Module = naga::front::wgsl::parse_str(&source)
      .unwrap_or_else(|error| panic!("'{name}' does not parse:\n{}", error.emit_to_string(&source)));

    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
      .validate(&module)
      .unwrap_or_else(|error| panic!("'{name}' does not validate:\n{}", error.emit_to_string(&source)));
  }
}

/// The bytes a module's global of a name takes, as the shader lays it out.
fn get_global_size(library: &ShaderLibrary, name: &str, global: &str) -> u32 {
  let source: String = library.compose(name).expect("composes");
  let module: naga::Module = naga::front::wgsl::parse_str(&source).expect("parses");
  let mut layouter: naga::proc::Layouter = naga::proc::Layouter::default();

  layouter.update(module.to_ctx()).expect("lays out");

  let (_, variable) = module
    .global_variables
    .iter()
    .find(|(_, variable)| variable.name.as_deref() == Some(global))
    .unwrap_or_else(|| panic!("'{name}' has no '{global}'"));

  layouter[variable.ty].size
}

/// Both shaders reading the water's uniform lay it out as large as it is written.
#[test]
fn the_water_uniform_is_as_large_as_both_its_shaders_read() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let size: u32 = size_of::<WaterUniform>() as u32;

  assert_eq!(get_global_size(&library, "static/water", "water"), size);
  assert_eq!(get_global_size(&library, "frame/water_blur", "water"), size);
}
