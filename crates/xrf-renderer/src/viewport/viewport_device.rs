use crate::contract::render_backend::RenderBackend;
use crate::contract::render_dlss_availability::RenderDlssAvailability;

/// What a viewport's report says of the device it draws on, the same for every viewport of a frame.
#[derive(Clone, Copy, Debug)]
pub struct ViewportDevice<'a> {
  pub backend: RenderBackend,
  pub adapter: &'a str,
  /// Whether it traces rays, by hardware ray query.
  pub has_ray_queries: bool,
  pub dlss: RenderDlssAvailability,
  pub ray_reconstruction: RenderDlssAvailability,
}
