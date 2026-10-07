use xrf_renderer::{RenderModelSkeleton, StaticModelPlace, StaticModelProxy, StaticObjectProxy};
use xrf_renderer_core::ProxyHandle;

/// A spawned model streamed into a scene: its handle there, every place an object stands as it with the object's handle
/// while it stands, and the skeleton its objects are posed by where it is skinned.
pub struct StreamedModel {
  pub handle: ProxyHandle<StaticModelProxy>,
  pub places: Vec<(StaticModelPlace, Option<ProxyHandle<StaticObjectProxy>>)>,
  pub skeleton: Option<RenderModelSkeleton>,
}
