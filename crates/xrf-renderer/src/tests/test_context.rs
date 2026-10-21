use xrf_test_utils::gpu::skip_without_gpu;

use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::tests::test_bundle::checkout_dxc;

/// A headless context as the application starts one: the first of the platform's backends that starts, D3D12 compiling
/// with the checkout's DXC. None where the machine has no adapter, the test skipped saying it had no GPU `what` it
/// needed one for, or failed where the run requires one.
pub(crate) fn create_test_context(what: &str) -> Option<GpuContext> {
  let mut problems: Vec<String> = Vec::new();

  for backend in RenderBackend::ALL.iter().copied() {
    match GpuContext::create_headless(backend, checkout_dxc().as_deref()) {
      Ok(context) => return Some(context),
      Err(error) => problems.push(format!("{}: {error}", backend.get_label())),
    }
  }

  skip_without_gpu(format_args!("no GPU {what} ({})", problems.join("; ")));

  None
}
