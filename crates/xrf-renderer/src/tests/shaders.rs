use crate::pass::water_uniform::WaterUniform;
use crate::scene::static_scene::static_surface::StaticSurface;
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

/// The bytes a struct of a name takes in a module, as the shader lays it out.
fn get_struct_size(library: &ShaderLibrary, name: &str, type_name: &str) -> u32 {
  let source: String = library.compose(name).expect("composes");
  let module: naga::Module = naga::front::wgsl::parse_str(&source).expect("parses");
  let mut layouter: naga::proc::Layouter = naga::proc::Layouter::default();

  layouter.update(module.to_ctx()).expect("lays out");

  let (handle, _) = module
    .types
    .iter()
    .find(|(_, it)| it.name.as_deref() == Some(type_name))
    .unwrap_or_else(|| panic!("'{name}' has no '{type_name}'"));

  layouter[handle].size
}

/// Both shaders reading the water's uniform lay it out as large as it is written.
#[test]
fn the_water_uniform_is_as_large_as_both_its_shaders_read() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let size: u32 = size_of::<WaterUniform>() as u32;

  assert_eq!(get_struct_size(&library, "static/water", "Water"), size);
  assert_eq!(get_struct_size(&library, "frame/water_blur", "Water"), size);
}

/// The static draws lay a surface out as large as it is written.
#[test]
fn a_static_surface_is_as_large_as_its_shaders_read() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_struct_size(&library, "static/gbuffer", "Surface"),
    size_of::<StaticSurface>() as u32
  );
}
