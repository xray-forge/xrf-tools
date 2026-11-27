use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::queue::render_workers::RenderWorkers;

/// What a GPU's passes are built from: the shaders, the host's pool, the files the renderer ships with, and where the
/// driver's pipelines are kept between runs, none where nothing keeps them.
#[derive(Clone, Copy)]
pub(crate) struct GpuSources<'a> {
  pub shaders: &'a ShaderLibrary,
  pub workers: &'a RenderWorkers,
  pub bundle: &'a dyn RenderBundle,
  pub store: Option<&'a dyn RenderCacheStore>,
}
