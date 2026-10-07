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

    validate(name, &source, naga::valid::Capabilities::all());
  }
}

/// A module composed for a device without ray queries, its traced imports in their fallbacks, validates without them.
#[test]
fn every_module_composes_without_ray_queries() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let features: wgpu::Features = wgpu::Features::all().difference(wgpu::Features::EXPERIMENTAL_RAY_QUERY);
  let mut fallen_back: usize = 0;

  for name in library.list_modules() {
    let source: String = library
      .compose_for(name, features)
      .unwrap_or_else(|error| panic!("'{name}': {error}"));

    if source != library.compose(name).expect("composes") {
      fallen_back += 1;
      validate(
        name,
        &source,
        naga::valid::Capabilities::all().difference(naga::valid::Capabilities::RAY_QUERY),
      );
    }
  }

  assert!(fallen_back > 0, "No module imports a fallback");
}

fn validate(name: &str, source: &str, capabilities: naga::valid::Capabilities) {
  let module: naga::Module = naga::front::wgsl::parse_str(source)
    .unwrap_or_else(|error| panic!("'{name}' does not parse:\n{}", error.emit_to_string(source)));

  naga::valid::Validator::new(naga::valid::ValidationFlags::all(), capabilities)
    .validate(&module)
    .unwrap_or_else(|error| panic!("'{name}' does not validate:\n{}", error.emit_to_string(source)));
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

/// The static draws lay a surface out as large as it is written.
#[test]
fn a_static_surface_is_as_large_as_its_shaders_read() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_struct_size(&library, "static/gbuffer", "Surface"),
    size_of::<StaticSurface>() as u32
  );
}
