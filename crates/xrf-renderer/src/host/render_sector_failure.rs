use xrf_renderer_core::ProxyHandle;

use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;

/// A sector the scene could not take in, told back to the world that posted it.
#[derive(Clone, Debug)]
pub struct RenderSectorFailure {
  pub handle: ProxyHandle<StaticSectorProxy>,
  pub reason: String,
}
