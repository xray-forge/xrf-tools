use std::sync::Arc;

use xrf_renderer_core::PipelineCache;

use crate::context::gpu_context::GpuContext;
use crate::host::render_cache_store::RenderCacheStore;

/// The pipeline cache a GPU's passes are made through, opened over the driver's pipelines kept from the last run on its
/// adapter and driver, where its backend keeps them.
pub(crate) struct KeptPipelines {
  pub cache: Arc<PipelineCache>,
  /// The name the driver's pipelines are kept under, none where its backend keeps none.
  pub key: Option<String>,
  /// How many bytes of them were kept.
  pub kept: usize,
}

impl KeptPipelines {
  /// Opens the cache for `context`'s device over what `store` kept for its adapter and driver; an empty one where
  /// nothing keeps them.
  pub(crate) fn open(context: &GpuContext, store: Option<&dyn RenderCacheStore>) -> Self {
    let key: Option<String> = store.and(wgpu::util::pipeline_cache_key(&context.adapter.get_info()));
    let cached: Option<Vec<u8>> = store
      .zip(key.as_deref())
      .and_then(|(store, key)| store.read_cached(key));

    Self {
      cache: Arc::new(PipelineCache::with_driver_cache(&context.device, cached.as_deref())),
      kept: cached.map_or(0, |cached| cached.len()),
      key,
    }
  }
}
