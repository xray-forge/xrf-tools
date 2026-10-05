use glam::{Mat4, Vec4};
use xrf_renderer_core::{ProxyHandle, Span};

use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_row::StaticRow;

/// One spawned object standing as a model: its place, its bone matrices where its model is skinned, and a row a slot of
/// its model.
#[derive(Debug)]
pub struct StaticObjectProxy {
  /// The object, by its index among the level's spawned objects.
  pub object: u32,
  pub model: ProxyHandle<StaticModelProxy>,
  pub place: Span,
  /// Its bone matrices, this frame's and the last's, three rows a bone, and how many bones.
  pub bones: Option<(Span, u32)>,
  pub rows: Vec<ProxyHandle<StaticRow>>,
  /// Entries of the visible list it may need a batch.
  pub capacities: [u32; StaticBatch::COUNT],
  pub transform: Mat4,
  /// Its bounding sphere in renderer space, as its model stands in its place.
  pub sphere: Vec4,
}
