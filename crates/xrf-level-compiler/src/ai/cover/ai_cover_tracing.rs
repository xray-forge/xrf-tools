use std::path::Path;

use crate::ai::cover::ai_ray_model::AiRayModel;
use crate::detail::detail_gpu::DetailGpu;
use crate::detail::detail_light_backend::DetailLightBackend;

/// How the cover pass traces its corner nodes' rays: against what, on which backend, and for the GPU backend with which
/// DXC and on which GPU.
#[derive(Clone, Copy)]
pub(crate) struct AiCoverTracing<'a> {
  pub(crate) rays: AiRayModel<'a>,
  pub(crate) backend: DetailLightBackend,
  #[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read by the GPU backend alone"))]
  pub(crate) dxc: Option<&'a Path>,
  #[cfg_attr(not(feature = "gpu"), expect(dead_code, reason = "read by the GPU backend alone"))]
  pub(crate) gpu: Option<DetailGpu>,
}
