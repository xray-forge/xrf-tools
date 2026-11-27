use xrf_error::XrfResult;

use crate::context::gpu_context::GpuContext;
use crate::host::render_bundle::RenderBundle;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::support::test_workers::create_workers;
use crate::thread::gpu::gpu_sources::GpuSources;
use crate::thread::gpu::gpu_state::GpuState;
use crate::thread::gpu::kept_pipelines::KeptPipelines;

/// Builds every pass on `context` from `shaders` and `bundle` as the renderer's start does, with no driver pipelines
/// kept.
pub(crate) fn build_test_gpu(
  context: GpuContext,
  shaders: &ShaderLibrary,
  bundle: &dyn RenderBundle,
) -> XrfResult<GpuState> {
  let kept: KeptPipelines = KeptPipelines::open(&context, None);

  GpuState::new(
    context,
    kept,
    GpuSources {
      shaders,
      workers: &create_workers(),
      bundle,
      store: None,
    },
  )
}
