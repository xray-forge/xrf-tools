use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_backend_availability::RenderBackendAvailability;

/// Whether the renderer can draw with each backend on this machine, in `RenderBackend::ALL`'s order. Each probe starts
/// an instance and asks for an adapter, some tens of milliseconds, so it runs off the render thread and on demand.
pub fn probe_render_backends() -> Vec<RenderBackendAvailability> {
  RenderBackend::ALL.into_iter().map(GpuContext::probe).collect()
}
