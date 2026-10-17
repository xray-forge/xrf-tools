use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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
