use std::path::{Path, PathBuf};

use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_backend_availability::RenderBackendAvailability;
use crate::contract::render_gpu::RenderGpu;
use crate::tests::test_bundle::checkout_dxc;

/// A GPU no machine has.
const MISSING: RenderGpu = RenderGpu {
  vendor: 0xffff,
  device: 0xffff,
};

/// A missing GPU falls back to the automatic one, which the probe lists first, and every listed GPU starts. Skipped on
/// machines without a hardware GPU, such as CI's.
#[test]
fn draws_on_the_chosen_gpu_and_falls_back_to_the_automatic_one() {
  let Some(availability) = [RenderBackend::D3d12, RenderBackend::Vulkan]
    .into_iter()
    .map(GpuContext::probe)
    .find(|it| !it.gpus.is_empty())
  else {
    eprintln!("Skipped: no hardware GPU");

    return;
  };
  let RenderBackendAvailability { backend, gpus, .. } = availability;
  let dxc: Option<PathBuf> = checkout_dxc();
  let libraries: (Option<&Path>, Option<&Path>) = (dxc.as_deref(), None);

  let fallback: GpuContext = GpuContext::create(backend, Some(MISSING), libraries).expect("the automatic GPU starts");

  assert_eq!(
    fallback.gpu, gpus[0].gpu,
    "a missing GPU falls back to the automatic one"
  );

  for listed in &gpus {
    let context: GpuContext = GpuContext::create(backend, Some(listed.gpu), libraries).expect("a listed GPU starts");

    assert_eq!(context.gpu, listed.gpu, "{} draws on itself", listed.name);
  }
}
