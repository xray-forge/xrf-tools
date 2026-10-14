use xrf_renderer_core::ProxyHandle;

use crate::scene::static_scene::static_model_proxy::StaticModelProxy;

/// A spawned model the scene could not take in, told back to the world that posted it.
#[derive(Clone, Debug)]
pub struct RenderModelFailure {
  pub handle: ProxyHandle<StaticModelProxy>,
  pub reason: String,
}
