use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_renderer_core::{ShaderAddressSpace, ShaderLayoutVerifier};

use crate::contract::render_debug_view::RenderDebugView;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::light_view::LightView;
use crate::scene::level::level_light_shadows::LevelLightShadows;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;
use crate::shader::generated_shaders::list_generated_shaders;
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

/// The lamps read their shadows in texels of the atlas the renderer allocates their tiles in.
#[test]
fn the_lamps_read_the_shadow_atlas_at_its_allocated_size() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_float_constant(&library, "common/local_lights", "ATLAS_TEXEL"),
    1.0 / LevelLightShadows::ATLAS_SIZE as f32
  );
}

/// A reload stales the modules that compose from what changed and no others; a changed fallback stales the modules
/// that import the module it replaces.
#[test]
fn a_reload_stales_only_the_modules_composing_from_a_change() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let quad: Vec<&str> = library.list_dependents(&["common/quad"]);

  assert_eq!(
    quad,
    vec![
      "common/quad",
      "frame/flare",
      "frame/particles",
      "frame/thunder",
      "static/impostor"
    ]
  );

  let traced: Vec<&str> = library.list_dependents(&["common/scene_trace_none"]);

  assert!(traced.contains(&"common/scene_trace"));
  assert!(traced.contains(&"common/scene_trace_none"));
  assert!(!traced.contains(&"frame/present"));
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

/// The library's table is kept by hand: a file left out of it is never validated or reloaded, and fails only when a GPU
/// pass composes it.
#[test]
fn embeds_every_shader_file_and_every_generated_module() {
  let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders");
  let mut files: BTreeSet<String> = BTreeSet::new();

  list_shader_files(&root, &root, &mut files);

  let library: ShaderLibrary = ShaderLibrary::default();
  let embedded: BTreeSet<String> = library.list_modules().map(str::to_owned).collect();

  let unlisted: Vec<&String> = files.difference(&embedded).collect();
  let missing: Vec<&String> = embedded.difference(&files).collect();

  assert!(
    unlisted.is_empty() && missing.is_empty(),
    "not embedded: {unlisted:?}; embedded without a file: {missing:?}"
  );

  for (name, _) in list_generated_shaders().expect("the passes sharing a module agree on its bindings") {
    assert!(embedded.contains(name), "'{name}' is generated but not embedded");
  }
}

/// Every `.wgsl` file under `directory`, named as an `#import` names it: its path under `root`, without the extension.
fn list_shader_files(root: &Path, directory: &Path, files: &mut BTreeSet<String>) {
  for entry in fs::read_dir(directory).expect("the shaders directory to be read") {
    let path: PathBuf = entry.expect("a shaders directory entry to be read").path();

    if path.is_dir() {
      list_shader_files(root, &path, files);
    } else if path.extension().is_some_and(|extension| extension == "wgsl") {
      let name: PathBuf = path
        .strip_prefix(root)
        .expect("a file under the root")
        .with_extension("");

      files.insert(name.to_string_lossy().replace('\\', "/"));
    }
  }
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

/// naga lays the camera and the page's backdrop out where the derive does, and takes each as a uniform.
#[test]
fn the_camera_and_the_backdrop_are_laid_out_as_naga_lays_them() {
  ShaderLayoutVerifier::verify::<CameraUniform>(ShaderAddressSpace::Uniform).unwrap();
  ShaderLayoutVerifier::verify::<BackdropUniform>(ShaderAddressSpace::Uniform).unwrap();
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
/// The value a module declares a float constant as.
fn get_float_constant(library: &ShaderLibrary, name: &str, constant: &str) -> f32 {
  let source: String = library.compose(name).expect("composes");
  let module: naga::Module = naga::front::wgsl::parse_str(&source).expect("parses");
  let (_, declared) = module
    .constants
    .iter()
    .find(|(_, declared)| declared.name.as_deref() == Some(constant))
    .unwrap_or_else(|| panic!("'{name}' declares no {constant}"));

  match module.global_expressions[declared.init] {
    naga::Expression::Literal(naga::Literal::F32(value)) => value,
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}

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
