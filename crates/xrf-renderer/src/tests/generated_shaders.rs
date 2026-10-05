use std::path::PathBuf;

use xrf_renderer_core::GeneratedShaderFile;

use crate::shader::generated_shaders::list_generated_shaders;

/// Each generated module is what its Rust declarations write: a stale one is rewritten and fails this once.
#[test]
fn every_generated_module_is_up_to_date() {
  let shaders: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders");
  let stale: Vec<&str> = list_generated_shaders()
    .expect("the passes sharing a module agree on its bindings")
    .into_iter()
    .filter(|(name, wgsl)| {
      GeneratedShaderFile::new(shaders.join(format!("{name}.wgsl")))
        .sync(wgsl)
        .unwrap_or_else(|error| panic!("'{name}': {error}"))
    })
    .map(|(name, _)| name)
    .collect();

  assert!(stale.is_empty(), "rewritten, commit them: {stale:?}");
}
