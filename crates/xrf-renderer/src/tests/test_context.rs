use xrf_test_utils::gpu::skip_without_gpu;

use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::tests::test_bundle::checkout_dxc;

/// A headless context as the application starts one: D3D12 compiling with the checkout's DXC, else Vulkan. None where
/// the machine has no adapter, the test skipped saying it had no GPU `what` it needed one for, or failed where the run
/// requires one.
pub(crate) fn create_test_context(what: &str) -> Option<GpuContext> {
  match GpuContext::create_headless(RenderBackend::D3d12, checkout_dxc().as_deref())
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan, None))
  {
    Ok(context) => Some(context),
    Err(error) => {
      skip_without_gpu(format_args!("no GPU {what} ({error})"));

      None
    }
  }
}
