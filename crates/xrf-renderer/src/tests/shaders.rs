use crate::contract::render_debug_view::RenderDebugView;
use crate::pass::light_view::LightView;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;
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

/// The cull tells a batch's class by its index as Rust lays batches out, a layout's classes in `StaticClass` order,
/// and finds a surface pass's draw list after them as Rust does.
#[test]
fn the_cull_counts_classes_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "static/cull", "CLASS_COUNT"),
    StaticClass::ALL.len() as u32
  );
  assert_eq!(
    get_constant(&library, "static/cull", "WATER_CLASS"),
    StaticClass::Water.get_index() as u32
  );
  assert_eq!(
    get_constant(&library, "static/cull", "SURFACE_PASS_COUNT"),
    StaticSurfacePass::COUNT as u32
  );
}

/// The present tells a debug view by its index as Rust numbers them, FSR's last and from the first it names on.
#[test]
fn the_present_numbers_debug_views_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/present", "VIEW_HEIGHTS"),
    RenderDebugView::Heights.get_index()
  );

  for (name, view) in [
    ("VIEW_LUMINANCE", RenderDebugView::Luminance),
    ("VIEW_ALBEDO_RANGE", RenderDebugView::AlbedoRange),
    ("VIEW_OVERDRAW", RenderDebugView::Overdraw),
    ("VIEW_CASCADES", RenderDebugView::Cascades),
    ("VIEW_LAMPS", RenderDebugView::LampsPerCluster),
    ("VIEW_SAMPLED_MIP", RenderDebugView::SampledMip),
  ] {
    assert_eq!(
      get_constant(&library, "frame/present", name),
      view.get_index(),
      "{name}"
    );
  }

  for (name, view) in [
    ("LIGHT_VIEW_CASCADES", LightView::Cascades),
    ("LIGHT_VIEW_LAMPS", LightView::LampsPerCluster),
    ("LIGHT_VIEW_OVERDRAW", LightView::Overdraw),
  ] {
    assert_eq!(get_constant(&library, "frame/sun", name), view as u32, "{name}");
  }
  assert_eq!(
    get_constant(&library, "frame/present", "VIEW_FSR_FIRST"),
    RenderDebugView::FsrMasks.get_index()
  );
  assert!(RenderDebugView::FsrFrameShare.get_index() > RenderDebugView::FsrMasks.get_index());
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

/// A `u32` constant's value in a module.
fn get_constant(library: &ShaderLibrary, name: &str, constant: &str) -> u32 {
  let source: String = library.compose(name).expect("composes");
  let module: naga::Module = naga::front::wgsl::parse_str(&source).expect("parses");
  let (_, declared) = module
    .constants
    .iter()
    .find(|(_, declared)| declared.name.as_deref() == Some(constant))
    .unwrap_or_else(|| panic!("'{name}' declares no {constant}"));

  match module.global_expressions[declared.init] {
    naga::Expression::Literal(naga::Literal::U32(value)) => value,
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}
