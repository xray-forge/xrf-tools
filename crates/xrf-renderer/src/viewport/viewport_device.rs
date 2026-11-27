use xrf_dlss::Dlss;

use crate::context::gpu_context::GpuContext;
use crate::context::gpu_requirements::can_trace;
use crate::contract::settings::gpu::render_backend::RenderBackend;
use crate::contract::settings::gpu::render_dlss_availability::RenderDlssAvailability;

/// What a viewport's report says of the device it draws on, the same for every viewport of a frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewportDevice<'a> {
  pub backend: RenderBackend,
  pub adapter: &'a str,
  /// Whether it traces rays, by hardware ray query.
  pub has_ray_queries: bool,
  pub dlss: RenderDlssAvailability,
  pub ray_reconstruction: RenderDlssAvailability,
}

impl<'a> ViewportDevice<'a> {
  /// The device `context` draws on, with `dlss` as DLSS's start came to: none where it has none, or where its start
  /// has not ended and the frame does not wait for it.
  pub(crate) fn of(context: &'a GpuContext, dlss: Option<&Dlss>) -> Self {
    let (dlss, ray_reconstruction): (RenderDlssAvailability, RenderDlssAvailability) = match (Dlss::IS_BUILT, dlss) {
      (false, _) => (RenderDlssAvailability::Unbuilt, RenderDlssAvailability::Unbuilt),
      (true, Some(dlss)) => (
        RenderDlssAvailability::Available,
        if dlss.has_ray_reconstruction() {
          RenderDlssAvailability::Available
        } else {
          RenderDlssAvailability::Unsupported
        },
      ),
      (true, None) => (RenderDlssAvailability::Unsupported, RenderDlssAvailability::Unsupported),
    };

    Self {
      backend: context.backend,
      adapter: context.adapter_name.as_str(),
      has_ray_queries: can_trace(&context.device),
      dlss,
      ray_reconstruction,
    }
  }
}
